use std::{collections::HashMap, net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, SocketAddrV6}, sync::Arc, time::Duration};
use anyhow::{anyhow, bail, Result, Context};
use cjdns_keys::{CJDNSPublicKey, PublicKey};
use parking_lot::Mutex;
use tokio::sync::mpsc;
use byteorder::{ReadBytesExt,WriteBytesExt,BE};

use cjdns::bytes::{dnsseed::{CjdnsPeer, CjdnsTxtRecord}, message::Message as RMessage};
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};

use crate::{cffi::{self, Control_LlAddr_Udp4_t, Control_LlAddr_Udp6_t, Control_LlAddr_t, PFChan_Node}, external::interface::iface::{Iface, IfacePvt}, interface::wire::message::Message, util::now_ms};

#[derive(Default)]
struct SeederState {
    tried_seeds: Vec<String>,
    current_peers: Vec<(PFChan_Node,u64)>,
    recommended_peers: HashMap<String,Vec<(CjdnsPeer,u64)>>,
    tried_peers: Vec<(CjdnsPeer,u64)>,
    last_get_peers: u64,
    last_dns_req: u64,

    recommended_snode: Option<CJDNSPublicKey>,
}

struct SeederInner {
    dns_seeds: Mutex<Vec<(String,bool)>>,
    snode_peers: Mutex<Vec<(CjdnsPeer,u64)>>,
    ifacep: IfacePvt,
    send_message: mpsc::Sender<Message>,
}
impl SeederInner {

    fn get_connect_peer(&self, st: &mut SeederState, now: u64, snode_peers_first: bool) -> Option<CjdnsPeer> {
        let mut candidate = None;
        let mut connected_count = 0;
        for (seeder, rps) in &st.recommended_peers {
            if snode_peers_first && !seeder.is_empty() {
                // Snode is identified as seeder ""
                continue;
            }
            for (rp, _) in rps.iter() {
                if st.current_peers.iter()
                    .find(|(cp,_)|cp.publicKey == rp.pubkey)
                    .is_some()
                {
                    connected_count += 1;
                } else if st.tried_peers.iter().find(|(p,_)|p.pubkey == rp.pubkey).is_some() {
                    // skip, recently tried
                } else if candidate.is_none() {
                    candidate = Some(rp);
                }
            }
        }
        let out = if st.tried_peers.iter().map(|(_,t)|*t).max().unwrap_or(0) + (1000 * 60) > now {
            None
        } else if let Some(cand) = candidate {
            // When getting peers from the snode, we shoot for 4
            // Otherwise we shoot for 2 only.
            if connected_count < if snode_peers_first { 4 } else { 2 } {
                Some(CjdnsPeer{
                    address: cand.address,
                    pubkey: cand.pubkey,
                    login: cand.login,
                    password: cand.password,
                    version: cand.version,
                })
            } else {
                None
            }
        } else {
            None
        };
        if snode_peers_first && out.is_none() {
            self.get_connect_peer(st, now, false)
        } else {
            out
        }
    }

    async fn cycle(self: &Arc<Self>, st: &mut SeederState) -> Result<()> {

        // Sync locks
        let seeds = {
            self.dns_seeds.lock().clone()
        };
        let snode_peers =
            std::mem::replace(&mut *self.snode_peers.lock(), Vec::new());

        log::debug!("Seeder cycle()");

        let now = now_ms();

        st.recommended_peers.entry(String::new()).or_default().extend(snode_peers.into_iter());
    
        // Remove dead peers that don't get withdrawn otherwise
        st.current_peers.retain(|(_,time)|time + (1000 * 60 * 20) > now);
        st.tried_peers.retain(|(_,time)|time + (1000 * 60 * 20) > now);
        for (_, rps) in &mut st.recommended_peers {
            rps.retain(|(_,t)|t + (1000 * 60 * 20) > now);
        }

        // Need to ask the core for peers?
        if st.last_get_peers + (1000 * 60 * 3) < now {
            log::debug!("Getting peers from ifc");
            let mut msg = Message::new(512);
            msg.write_u32::<BE>(cffi::PFChan_Pathfinder::PFChan_Pathfinder_PEERS as _)?;
            self.ifacep.send(msg)?;
            st.last_get_peers = now;
        }

        // Need a DNS req?
        if let Some((seed, trust)) = {
            if st.last_dns_req + (1000 * 60) > now {
                None
            } else if let Some(s) = seeds.iter()
                .find(|(s,_)|!st.tried_seeds.contains(s))
            {
                Some(s)
            } else {
                st.tried_seeds.clear();
                seeds.iter().next()
            }.cloned()
        } {
            log::debug!("Trying seed {seed}");
            let resolver =
                trust_dns_resolver::AsyncResolver::tokio(
                    ResolverConfig::default(),
                    ResolverOpts::default());

            let res = resolver.txt_lookup(&seed).await
                .with_context(||format!("Failed dns lookup for {seed}"))?;
            let txt = res.iter().next().ok_or_else(||anyhow!("No TXT records found"))?;
            let txt = txt.to_string();
            let ctr = CjdnsTxtRecord::decode(&txt)
                .with_context(||format!("Unable to decode seed TXT record {txt}"))?;

            if trust {
                if let Some(snode) = ctr.snode_pubkey {
                    st.recommended_snode = Some(CJDNSPublicKey::from(snode));
                }
            }
            let it = ctr.peers.into_iter().map(|p|(p,now));
            st.recommended_peers.entry(seed).or_default().extend(it);
            st.last_dns_req = now;
        }

        // Need to add a peer?
        if let Some(connect) = self.get_connect_peer(st, now, true) {
            log::debug!("Sending peer to connect {}", connect.address);
            let mut cp = cffi::PFChan_Pathfinder_ConnectPeer_t{
                ip: [0;16],
                pubkey: connect.pubkey,
                login: [0;16],
                password: [0;24],
                version: connect.version,
                port: connect.address.port(),
                _pad: 0,
            };

            let pl = connect.peering_line();
            let login = pl.login.as_bytes();
            cp.login[0..login.len()].copy_from_slice(login);
            let password = pl.password.as_bytes();
            cp.password[0..password.len()].copy_from_slice(password);

            match connect.address.ip() {
                std::net::IpAddr::V4(v4) => {
                    cp.ip[10] = 0xff;
                    cp.ip[11] = 0xff;
                    cp.ip[12..].copy_from_slice(&v4.octets());
                }
                std::net::IpAddr::V6(v6) => {
                    cp.ip = v6.octets();
                }
            }

            let mut msg = Message::new(512);
            msg.push(cp)?;
            msg.write_u32::<BE>(cffi::PFChan_Pathfinder::PFChan_Pathfinder_CONNECT_PEER as _)?;
            self.ifacep.send(msg)?;

            st.tried_peers.push((connect,now));
        }

        log::debug!("Seeder cycle complete");
        Ok(())
    }
    async fn parse_msg(&self, mut msg: Message, st: &mut SeederState) -> Result<()> {
        let t = msg.read_u32::<BE>()?;
        let gone = if t == cffi::PFChan_Core::PFChan_Core_PEER as u32 {
            false
        } else if t == cffi::PFChan_Core::PFChan_Core_PEER_GONE as u32 {
            true
        } else {
            bail!("Incoming message to Seeder with unhandled type {t}");
        };
        let peer: PFChan_Node = msg.pop()?;
        st.current_peers.retain(|(p,_)|p.publicKey != peer.publicKey);
        if !gone {
            st.current_peers.push((peer,now_ms()));
        }
        Ok(())
    }
    async fn run(
        self: Arc<Self>,
        mut recv_message: mpsc::Receiver<Message>,
        mut done: mpsc::Receiver<()>,
    ) {
        let mut st = SeederState::default();
        loop {
            tokio::select! {
                res = self.cycle(&mut st) => {
                    if let Err(e) = res {
                        log::info!("Seeder cycle error: {e}, delay 3 seconds");
                        tokio::time::sleep(Duration::from_secs(3)).await;
                    }
                }
                _ = tokio::time::sleep(Duration::from_secs(60)) => {
                    log::warn!("Seeder cycle() timed out after 60 seconds");
                }
            }
            let stop_time = tokio::time::Instant::now().checked_add(Duration::from_secs(10)).unwrap();
            loop {
                tokio::select! {
                    _ = done.recv() => {
                        log::info!("Seeder shutdown");
                        return;
                    }
                    msg = recv_message.recv() => {
                        if let Some(msg) = msg {
                            if let Err(e) = self.parse_msg(msg, &mut st).await {
                                log::info!("Error in parse_msg() -> {e}");
                            } else {
                                continue;
                            }
                        }
                    }
                    _ = tokio::time::sleep_until(stop_time) => {}
                }
                break;
            }
        }
    }
}

fn incoming_event(si: &Arc<SeederInner>, m: Message) -> Result<()> {
    si.send_message.try_send(m)?;
    Ok(())
}

struct MyPeeringPasswd {
    user_num: u16,
    passwd: u64,
    code: Vec<u8>,
}

#[derive(Default)]
struct MyPeeringInfo {
    passwd: Option<MyPeeringPasswd>,
    v4: Option<SocketAddrV4>,
    v6: Option<SocketAddrV6>,
}

pub struct Seeder {
    my_pubkey: PublicKey,
    inner: Arc<SeederInner>,
    mpi: Mutex<MyPeeringInfo>,
    _done: mpsc::Sender<()>,
}

impl Seeder {
    /// Whether we have an address from got_lladdr
    pub fn has_lladdr(&self) -> bool {
        let m = self.mpi.lock();
        m.v4.is_some() || m.v6.is_some()
    }
    fn got_lladdr4(&self, lla: &Control_LlAddr_Udp4_t) -> bool {
        let sa =
            SocketAddrV4::new(lla.addr.into(), u16::from_be(lla.port_be));
        let mut m = self.mpi.lock();
        if let Some(v4) = &mut m.v4 {
            if v4 == &sa {
                false
            } else {
                log::debug!("got_lladdr() change of IPv4: {} -> {}",
                    v4.to_string(), sa.to_string());
                true
            }
        } else {
            log::debug!("got_lladdr() got IPv4: {}", sa.to_string());
            m.v4 = Some(sa);
            true
        }
    }
    fn got_lladdr6(&self, lla: &Control_LlAddr_Udp6_t) -> bool {
        let sa = SocketAddrV6::new(
            lla.addr.into(), u16::from_be(lla.port_be), 0, 0);
        let mut m = self.mpi.lock();
        if let Some(v6) = &mut m.v6 {
            if v6 == &sa {
                false
            } else {
                log::debug!("got_lladdr() change of IPv6: {} -> {}",
                    v6.to_string(), sa.to_string());
                true
            }
        } else {
            log::debug!("got_lladdr() got IPv6: {}", sa.to_string());
            m.v6 = Some(sa);
            true
        }
    }
    /// Called when we got our public IP from a peer
    pub fn got_lladdr(&self, lla: Control_LlAddr_t) -> bool {
        if lla.magic != cffi::Control_LlAddr_REPLY_MAGIC {
            log::debug!("got_lladdr() with invalid magic {}", lla.magic);
            return false;
        }
        // unions are unsafe
        unsafe {
            match lla.addr.payload.type_ {
                cffi::Control_LlAddr_Udp4_TYPE => self.got_lladdr4(&lla.addr.udp4),
                cffi::Control_LlAddr_Udp6_TYPE => self.got_lladdr6(&lla.addr.udp6),
                cffi::Control_LlAddr_Other_TYPE => {
                    log::debug!("got_lladdr() with Other type: header: [{}]",
                        hex::encode(&lla.addr.other.sockaddrHeader));
                    false
                }
                other => {
                    log::debug!("got_lladdr() with invalid type {}", other);
                    false
                }
            }
        }
    }
    /// Make the peering credentials to post up to the snode
    pub fn mk_creds(&self) -> Result<Vec<u8>> {
        let m = self.mpi.lock();
        let passwd = if let Some(x) = &m.passwd {
            x
        } else {
            bail!("Missing passwd");
        };
        let mut addrs = Vec::new();
        if let Some(x) = &m.v4 {
            addrs.push(SocketAddr::V4(x.clone()));
        }
        if let Some(x) = &m.v6 {
            addrs.push(SocketAddr::V6(x.clone()));
        }
        if addrs.is_empty() {
            bail!("No IPv4 or IPv6 address");
        }
        let mut msg = RMessage::new();
        for address in addrs.into_iter() {
            cjdns::bytes::dnsseed::CjdnsPeer {
                address,
                pubkey: *self.my_pubkey.raw(),
                login: passwd.user_num,
                password: passwd.passwd.to_be_bytes(),
                version: cffi::Version_CURRENT_PROTOCOL,
            }.encode(&mut msg)?;
        }
        cjdns::bytes::dnsseed::PeerID{ id: passwd.code.clone() }.encode(&mut msg)?;
        Ok(msg.as_vec())
    }
    /// Called when we wish to become a public peer
    /// Returns: user, pass -> to include in authorized passwords
    pub fn public_peer(
        &self,
        user_num: u16,
        passwd: u64,
        code: Vec<u8>,
    ) -> (String, String) {
        let mut m = self.mpi.lock();
        m.passwd = Some(MyPeeringPasswd { user_num, passwd, code });
        let p = cjdns_bytes::dnsseed::CjdnsPeer {
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1,1,1,1)), 1),
            pubkey: [0_u8; 32],
            login: user_num,
            password: passwd.to_be_bytes(),
            version: cffi::Version_CURRENT_PROTOCOL,
        }.peering_line();
        (p.login, p.password)
    }
    /// Called when snode responds with new peers to add to our list
    pub fn got_peers(&self, peers_code: Vec<u8>) {
        let ctr = match CjdnsTxtRecord::decode_bin(&peers_code) {
            Err(e) => {
                log::warn!("Error decoding peering reply from snode: {e}");
                return;
            }
            Ok(x) => x,
        };
        let now = now_ms();
        self.inner.snode_peers.lock().extend(
            ctr.peers.into_iter().map(|p|(p,now))
        );
    }

    pub fn add_dns_seed(&self, seed: String, trust_snode: bool) {
        let mut ds = self.inner.dns_seeds.lock();
        for (s, t) in ds.iter_mut() {
            if &s[..] == &seed[..] {
                *t = trust_snode;
                return;
            }
        }
        ds.push((seed, trust_snode));
    }
    pub fn rm_dns_seed(&self, seed: String) -> bool {
        let mut ds = self.inner.dns_seeds.lock();
        let len0 = ds.len();
        ds.retain(|(s,_)|s != &seed);
        ds.len() < len0
    }
    pub fn list_dns_seeds(&self) -> Vec<(String,bool)> {
        self.inner.dns_seeds.lock().clone()
    }
    pub fn new(my_pubkey: CJDNSPublicKey) -> (Self, Iface) {
        let (_done, done_r) = mpsc::channel(1);
        let (send_message, recv_message) = mpsc::channel(512);
        let (mut iface, ifacep) = Iface::new(format!("Seeder()"));
        let inner = Arc::new(SeederInner{
            send_message,
            snode_peers: Default::default(),
            dns_seeds: Default::default(),
            ifacep,
        });
        iface.set_receiver_f(incoming_event, Arc::clone(&inner));
        tokio::task::spawn(Arc::clone(&inner).run(recv_message, done_r));
        (
            Self {
                my_pubkey,
                inner,
                _done,
                mpi: Default::default(),
            },
            iface,
        )
    }
}