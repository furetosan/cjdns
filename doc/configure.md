Configuring cjdns
=================

In this document we are going to go over how to configure cjdns and what exactly each setting means. Note that this is a living document and this software is still in the alpha stages so things are subject to change.

Let's start from the top of the file. First off you may notice it's JSON, except this JSON has comments. Technically that's not valid but it's also not uncommon. Cjdns strips out the comments before parsing it so no worries.

Your Keys and Address
---------------------

The top part of the file specifies where the cjdns executable is, your encryption keys, and your cjdns IPv6 address.

````javascript
{
    // Private key:
    // Your confidentiality and data integrity depend on this key, keep it secret!
    "privateKey": "823e4EXAMPLEEXAMPLEEXAMPLEEXAMPLEEXAMPLEEXAMPLEEXAMPLEEXAMPLEc70",

    // This key corresponds to the public key and ipv6 address:
    "publicKey": "u2jf87mgqlxfzdnywp60z3tx6tkulvgh2nyc2jk1zc69zzt2s8u0.k",
    "ipv6": "fcff:a215:1e7b:a4e9:c00d:0813:93b3:7c87",
````

- `privateKey`: Your private key is part of the system that ensures all the data coming and going out of your computer is encrypted. You must protect your private key. Do not give it out to anyone.
- `publicKey`: The public key is what your computer gives to other computers to encrypt data with. This data can then only be decrypted with your private key, that way no one can access your information as it moves across the network.
- `ipv6`: This is your IP address on the cjdns network. It is unique to you and is created by securely hashing your public key.

Cjdns provides the publicKey and ipv6 fields for convenience, but does not actually require them. If for some reason you want to remove them from the configuration file, you can. At launch, the privateKey will be used to generate the corresponding publicKey, and the publicKey will be used to find the corresponding ipv6 address. Their relationships are deterministic, and passing only the most fundamental attribute avoids configuration errors.

Incoming Connections
--------------------

The `authorizedPasswords` section is the area where you can specify passwords to allow people to connect to you. Any system that presents a valid password will be allowed access.
**NOTE:** These passwords should be long and random. There is no reason to make them short, easily remembered words because they are only going to be used by cjdns.

````javascript
    // Anyone connecting and offering these passwords on connection will be allowed.
    //
    // WARNING: Currently there is no key derivation done on the password field,
    //          DO NOT USE A PASSWORD HERE use something which is truly random and
    //          cannot be guessed.
    // Including a username in the beginning of the password string is encouraged
    // to aid in remembering which users are who.
    //
    "authorizedPasswords":
    [
        // A unique string which is known to the client and server.
        {"password": "zxl6zgxpl4stnuybdt0xlg4tn2cdl5h", "user": "default-login"}

        // More passwords should look like this.
        // {"password": "10ru8br0mhk25ccpvubv0sqnl7kuc6s", "user": "my-second-peer"},
        // {"password": "y68jm490dztxn3d2gvuv09bz55wqmjj", "user": "my-third-peer"},
        // {"password": "bnpphnq205v8nf2ksrs1fknfr572xzc", "user": "my-fourth-peer"},

        // These are your connection credentials
        // for people connecting to you with your default password.
        // adding more passwords for different users is advisable
        // so that leaks can be isolated.
        //
        // "your.external.ip.goes.here:33808":{"login": "default-login", "password":"zxl6zgxpl4stnuybdt0xlg4tn2cdl5h","publicKey":"u2jf87mgqlxfzdnywp60z3tx6tkulvgh2nyc2jk1zc69zzt2s8u0.k"}
    ],
````
- `password`: This is the password that another system can give to your node and be allowed to connect. You would place it in the `password` section in the next part.
  + `password` blocks can be assigned a second `user` attribute. At runtime, when using the `peerStats` script, peers who have connected using a particular password will be associated with the appropriate `user` string.
- `your.external.ip.goes.here:33808` This section is what you would give to a friend so that they could connect with you. We suggest you add a `"name":"so-and-so"` section to it as well (don't forget the comma between sections!) so that it is easy to see who you have allowed access to. If it later turns out that you no longer wish to have this user connect to you then it is a simple matter to find them and delete or comment out the correct line.

Admin Interface
---------------

The `admin ` section defines the settings for the administrative interface of cjdns. Many of the scripts in `/contrib/` use this to interact with cjdns. You probably wont need to use anything in there unless you are helping to test something out.

````javascript
    // Settings for administering and extracting information from your router.
    // This interface provides functions which can be called through a TCP socket.
    "admin":
    {
        // Port to bind the admin RPC server to.
        "bind": "127.0.0.1:11234",

        // Password for admin RPC server.
        "password": "j6mukf2khplcgpbzz0kulb8hu0xq2v9"
    },
````

- `bind`: This tells cjdns what IP and port the admin interface should bind to. Since you don't want random people connecting to your admin interface, it is probably fine to leave it like this.
- `password`: This is the password that is needed in order to perform certain functions through the admin interface. If you wish to expose the admin interface to the network, then you should use a password like the one above. If you are binding only to a local address, then you can use `"NONE"` as a password. This is the new default behaviour so as to provide an easier default configuration to work with.

Connection Interface(s)
-----------------------

This specifies the settings for the connection interfaces to your node. Right now most people use the `UDPInterface` to connect to other cjdns peers over the internet or other traditional networks. You may also use `ETHInterface` to physically connect to another machine. Note that this is not a standard TCP/IP connection like you are used to.

````javascript
    // Interfaces to connect to the switch core.
    "interfaces":
    {
        // The interface which connects over UDP/IP based VPN tunnel.
        "UDPInterface":
        [
            {
                // Bind to this port.
                "bind": "0.0.0.0:33808",

                // Automatically connect to other nodes on the same LAN
                // This works by binding a second port and sending beacons
                // containing the main data port.
                // beacon is a number between 0 and 2:
                //   0 -> do not beacon nor connect to other nodes who beacon
                //   1 -> quiet mode, accept beacons from other nodes only
                //   2 -> send and accept beacons
                // beaconDevices is a list which can contain names of devices such
                // as eth0, as well as broadcast addresses to send to, such as
                // 192.168.101.255, or the pseudo-name "all".
                // in order to auto-peer, all cjdns nodes must use the same
                // beaconPort.
                "beacon": 2,
                "beaconDevices": [ "all" ],
                "beaconPort": 64512,

                // Nodes to connect to.
                "connectTo":
                {
                    // Add connection credentials here to join the network
                    // Ask somebody who is already connected.
                }
            }
        ]

        /*
        "ETHInterface":
        [
            {
                // Bind to this device (interface name, not MAC etc.)
                "bind": "eth0",

                // Auto-connect to other cjdns nodes on the same network.
                // Options:
                //
                // 0 -- Disabled.
                //
                // 1 -- Accept beacons, this will cause cjdns to accept incoming
                //      beacon messages and try connecting to the sender.
                //
                // 2 -- Accept and send beacons, this will cause cjdns to broadcast
                //      messages on the local network which contain a randomly
                //      generated per-session password, other nodes which have this
                //      set to 1 or 2 will hear the beacon messages and connect
                //      automatically.
                //
                "beacon": 2,

                // Node(s) to connect to manually.
                "connectTo":
                {
                    // Credentials for connecting look similar to UDP credentials
                    // except they begin with the mac address, for example:
                    // "01:02:03:04:05:06":{"password":"a","publicKey":"b"}
                }
            }
        ]
        */
    },
````

- `UDPInterface`:
    - `bind`: This tells cjdns what IP and port to use for listening to connections.
    - `beacon`: his controls peer auto-discovery. Set to 0 to disable auto-peering, 1 to use broadcast
    auto-peering passwords contained in "beacon" messages from other nodes, and 2 to both broadcast and accept beacons.
    - `beaconDevices`: A list of broadcast devices, including device names such as "eth0", address names and the
    pseudo-name "all" which means it will use broadcast addresses of all interfaces. For example:
    `[ "eth0", "wlan0", "192.168.300.255" ]`
    - `beaconPort`: The UDP port for sending beacon messages on, all nodes must have the same beaconPort to auto-peer.
    - `connectTo`: This is where you put the connection details for peers that you want to connect to. The format for this generally looks like this,
        "12.34.56.78:12345":
        {
            "password": "thisIsAnExampleOfAPassword",
            "publicKey": "z4s2EXAMPLEPUBLICKEYEXAMPLEPUBLICKEYEXAMPLEKEY4yjp0.k"
        },
    It is important to note that while some people may put additional fields in such as `node`, only `password` and `publicKey` are actually read by cjdns.
    - more recently generated configuration files will also feature a second block within the `UDPInterface` section, which is to be used to configure UDP peering over IPv6. Such blocks will look like this,
        "[2001:db8::2:1]:12345":
        {
            "password": "thisIsAnExampleOfAPassword",
            "publicKey": "z4s2EXAMPLEPUBLICKEYEXAMPLEPUBLICKEYEXAMPLEKEY4yjp0.k"
        },
- `ETHInterface`:
    - `bind`: This tells cjdns which device the ETHInterface should bind to. This may be different depending on your setup.
    - `connectTo`: The connectTo for the ETHInterface functions almost exactly like it does for the the UDPInterface, except instead of an IP address and a port at the beginning, it is a MAC address.
    - `beacon`: This controls peer auto-discovery. Set to 0 to disable auto-peering, 1 to use broadcast auto-peering passwords contained in "beacon" messages from other nodes, and 2 to both broadcast and accept beacons.
    - In earlier versions of cjdns, it was necessary to uncomment the ETHInterface if you want to use it, however, now it is uncommented by default.

Router
------

This is where you configure routing settings of your cjdns node.

````javascript
    // Configuration for the router.
    "router":
    {
        // DNS Seeds, these will be used to add peers automatically.
        // The first seed in the list is trusted to provide the snode.
        "dnsSeeds": [
            "seed.pns.cjdns.fr"
        ],

        // When publicPeer id is set, this node will post its public peering credentials
        // to its supernode. The specified peerID will be used to identify itself.
        // For PKT yielding this must be set to the registered peerID, otherwise
        // you can set it to anything. By *convention*, peerIDs that begin with "PUB_"
        // are shared publicly and those which do not are tested by the snode but not
        // shared, allowing you to use the snode's peer tester on an otherwise private
        // node. If you leave "id" commented, your peering credentials will remain
        // entirely private.
        //
        "publicPeer": {
            // "id": "PUB_XXX",

            // If you set the public peer, you may also hardcode the IPv4 address.
            // By default, cjdns will request its public IP address from its peers, but
            // in cases with non-standard routing, you may have a different IP address
            // for traffic initiated from outside. In this case, you must manually enter
            // the IP address. If the address is entered in the form of "x.x.x.x", then
            // the IP address will be used, but the port will be detected. If it is entered
            // as "0.0.0.0:xxx" then the port will be used, but the address will be detected
            // finally, if it is in the form of "x.x.x.x:xxx" then the address AND port will
            // be used.
            //
            // "ipv4": "1.2.3.4:56789",

            // If you have a public IPv6 address which cannot be detected, you may hard-code
            // it here. The same rules apply as IPv4 addresses: "xxxx:xxxx::" means use ip
            // but detect port. "[::]:xxx" means use port but detect ip, and
            // "[xxxx:xxxx::]:xxx" means use ip and port from configuration.
            //
            // "ipv6": "[1234:5678::]:9012",
        },

        // supernodes, if none are specified they'll be taken from your peers
        "supernodes": [
            //"6743gf5tw80ExampleExampleExampleExamplevlyb23zfnuzv0.k",
        ],

        // The interface which is used for connecting to the cjdns network.
        "interface":
        {
            // The type of interface (only TUNInterface is supported for now)
            "type": "TUNInterface"

            // The name of a persistent TUN device to use.
            // This for starting cjdroute as its own user.
            // *MOST USERS DON'T NEED THIS*
            //"tunDevice": "tun0"
        },
````

- `dnsSeeds`: If specified, cjdns will run a DNS TXT record lookup on these domains to get peers and an snode.
If unspecified, peers must be added manually.
- `publicPeer`
  - `id`: If specified, cjdns will create a "public" AuthorizedPassword, probe its peers to get
     its public IP address, and report that address and the password to the supernode.
  - `ipv4`: If specified and if `id` is specified, this will override the detected IPv4.
  - `ipv6`: If specified and if `id` is specified, this will override the detected IPv6.
- `supernodes`: If specified this will force a certain supernode, otherwise it's learned from the DNS seed or your peers.
- `interface`
  - `type`: This specifies the type of interface cjdns should use to connect to the network. Only TUNInterface is supported at the moment.
  - `tunDevice`: This specifies which TUN device cjdns should use to connect to the network. Most users do not need this.

IP Tunneling
------------

IP Tunneling will allow you to connect from the cjdns network to another outside network. This is still a work in-progress; although it does function, it requires a bit of manual configuration on both ends to make it useful.
````javascript
        // System for tunneling IPv4 and ICANN IPv6 through cjdn which will eventually be merged to master..
        // This is using the cjdns switch layer as a VPN carrier.
        "ipTunnel":
        {
            // Nodes allowed to connect to us.
            // When a node with the given public key connects, give them the
            // ip4 and/or ip6 addresses and prefix lengths listed.
            "allowedConnections":
            [
                // Give the client an address on 192.168.1.0/24, and an address
                // it thinks has all of IPv6 behind it.
                // ip4Prefix is the set of addresses which are routable from the tun
                // for example, if you're advertizing a VPN into a company network
                // which exists in 10.123.45.0/24 space, ip4Prefix should be 24
                // default is 32 for ipv4 and 128 for ipv6
                // so by default it will not install a route
                // ip4Alloc is the block of addresses which are allocated to the
                // for example if you want to issue 4 addresses to the client, those
                // being 192.168.123.0 to 192.168.123.3, you would set this to 30
                // default is 32 for ipv4 and 128 for ipv6 (1 address)
                // {
                //     "publicKey": "f64hfl7c4uxt6krmhPutTheRealAddressOfANodeHere7kfm5m0.k",
                //     "ip4Address": "192.168.1.24",
                //     "ip4Prefix": 0,
                //     "ip4Alloc": 32,
                //     "ip6Address": "2001:123:ab::10",
                //     "ip6Prefix": 0
                //     "ip6Alloc": 64,
                // },

                // It's ok to only specify one address and prefix/alloc are optional.
                // {
                //     "publicKey": "ydq8csdk8p8ThisIsJustAnExampleAddresstxuyqdf27hvn2z0.k",
                //     "ip4Address": "192.168.1.25",
                //     "ip4Prefix": 0,
                // }
            ],

            "outgoingConnections":
            [
                // Connect to one or more machines and ask them for IP addresses.
                // "6743gf5tw80ExampleExampleExampleExamplevlyb23zfnuzv0.k",
                // "pw9tfmr8pcrExampleExampleExampleExample8rhg1pgwpwf80.k",
                // "g91lxyxhq0kExampleExampleExampleExample6t0mknuhw75l0.k"
            ]
        }
    },
````

- `allowedConnections`: The pubkeys of nodes which we allow to connect to our node and what IP addresses
we will issue to them.
- `outgoingConnections`: Cjdns VPN exits to connect to.

Miscellaneous
-------------

This section contains the security section for configuring program options and a few other miscellaneous
things that don't fit in with a broader category elsewhere.
````javascript
    // Dropping permissions.
    "security":
    [
        // Set number of open files to zero, in Linux, this will succeed even if
        // files are already open and will not allow any files to be opened for the
        // duration of the program's operation.
        // Most security exploits require the use of files.
        "nofiles",

        // Change the user id to this user after starting up and getting resources.
        {"setuser": "nobody"}
     ],
}
````


