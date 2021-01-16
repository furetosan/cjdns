#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
use crate::cffi::{Iface_t, String_t};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct RTypes_IfWrapper_t {
    pub internal: *mut Iface_t,
    pub external: *mut Iface_t,
}

#[repr(C)]
pub struct RTypes_StrList_t {
    pub len: usize,
    pub items: *mut *mut String_t,
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum RTypes_CryptoAuth_State_t {
    /// New CryptoAuth session, has not sent or received anything
    Init = 0,

    /// Sent a hello message, waiting for reply
    SentHello = 1,

    /// Received a hello message, have not yet sent a reply
    ReceivedHello = 2,

    /// Received a hello message, sent a key message, waiting for the session to complete
    SentKey = 3,

    /// Sent a hello message, received a key message, may or may not have sent some data traffic
    /// but no data traffic has yet been received
    ReceivedKey = 4,

    /// Received data traffic, session is in run state
    Established = 100,
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct RTypes_CryptoStats_t {
    /// Number of packets which were lost
    pub lost_packets: u64,

    /// Number of packets which were received but could not be validated
    pub received_unexpected: u64,

    /// Number of packets which were received (since last session setup)
    pub received_packets: u64,

    /// Number of packets which were received that were duplicates
    pub duplicate_packets: u64,
}

#[allow(dead_code)]
#[repr(C)]
pub struct RTypes_ExportMe {
    a: RTypes_IfWrapper_t,
    b: RTypes_StrList_t,
    c: RTypes_CryptoAuth_State_t,
    d: RTypes_CryptoStats_t,
}