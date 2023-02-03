use std::convert::From;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use macaddr::{MacAddr6, MacAddr8};
use crate::address_family::{AF_APPLETALK, AF_ASH, AF_ATMSVC, AF_AX25, AF_BLUETOOTH, AF_BRIDGE, AF_DECnet, AF_ECONET, AF_INET, AF_INET6, AF_IPX, AF_IRDA, AF_IUCV, AF_KEY, AF_LLC, AF_MAX, AF_NETBEUI, AF_NETLINK, AF_NETROM, AF_PACKET, AF_PPPOX, AF_ROSE, AF_RXRPC, AF_SECURITY, AF_SNA, AF_UNIX, AF_UNSPEC, AF_WANPIPE, AF_X25};

pub const MAX_ADDR_LEN: usize = 32;

pub enum NetworkAddressFamily {
    Unspecified= AF_UNSPEC as isize,
    Unix = AF_UNIX as isize,
    Inet = AF_INET as isize,
    Ax25 = AF_AX25 as isize,
    Ipx = AF_IPX as isize,
    AppleTalk = AF_APPLETALK as isize,
    Netrom = AF_NETROM as isize,
    Bridge = AF_BRIDGE as isize,
    X25 = AF_X25 as isize,
    Inet6 = AF_INET6 as isize,
    Rose = AF_ROSE as isize,
    DECNet = AF_DECnet as isize,
    NetBeui = AF_NETBEUI as isize,
    Security = AF_SECURITY as isize,
    Key = AF_KEY as isize,
    Netlink = AF_NETLINK as isize,
    Packet = AF_PACKET as isize,
    Ash = AF_ASH as isize,
    Econet = AF_ECONET as isize,
    AtmSvc = AF_ATMSVC as isize,
    SNA = AF_SNA as isize,
    IrDA = AF_IRDA as isize,
    PPPoX = AF_PPPOX as isize,
    WanPipe = AF_WANPIPE as isize,
    Llc = AF_LLC as isize,
    Bluetooth = AF_BLUETOOTH as isize,
    Iucv = AF_IUCV as isize,
    Rxrpc = AF_RXRPC as isize,
    Max = AF_MAX as isize,
    Invalid,
}

impl From<u16> for NetworkAddressFamily {
    fn from(value: u16) -> Self {
        match value {
            AF_UNSPEC => Self::Unspecified,
            AF_UNIX => Self::Unix,
            AF_INET => Self::Inet,
            AF_AX25 => Self::Ax25,
            AF_IPX => Self::Ipx,
            AF_APPLETALK => Self::AppleTalk,
            AF_NETROM => Self::Netrom,
            AF_BRIDGE => Self::Bridge,
            AF_X25 => Self::X25,
            AF_INET6 => Self::Inet6,
            AF_ROSE => Self::Rose,
            AF_DECnet => Self::DECNet,
            AF_NETBEUI => Self::NetBeui,
            AF_SECURITY => Self::Security,
            AF_KEY => Self::Key,
            AF_NETLINK => Self::Netlink,
            AF_PACKET => Self::Packet,
            AF_ASH => Self::Ash,
            AF_ECONET => Self::Econet,
            AF_ATMSVC => Self::AtmSvc,
            AF_SNA => Self::SNA,
            AF_IRDA => Self::IrDA,
            AF_PPPOX => Self::PPPoX,
            AF_WANPIPE => Self::WanPipe,
            AF_LLC => Self::Llc,
            AF_BLUETOOTH => Self::Bluetooth,
            AF_IUCV => Self::Iucv,
            AF_RXRPC => Self::Rxrpc,
            AF_MAX => Self::Max,
            _ => Self::Invalid
        }
    }
}

#[derive(Default,Debug,Clone)]
pub struct NetworkAddress {
    pub bytes: [u8;MAX_ADDR_LEN],
    _family: u16,
    pub bitmask: u32,
    pub port: u16,
    pub flow_info: u32,
    pub scope_id: u32,
}

impl NetworkAddress {
    pub fn new(bytes: Option<&mut [u8]>, addr_family: Option<u16>, port: Option<u16>, bitmask: Option<u32>) -> Self {
        let mut out = Self::default();
        if bytes.is_some() {
            out.bytes.clone_from_slice(bytes.unwrap());
        }
        if addr_family.is_some() {
            out.addr_family = addr_family.unwrap();
        }
        if port.is_some() {
            out.port = port.unwrap();
        }
        if bitmask.is_some() {
            out.bitmask = bitmask.unwrap()
        }
        out
    }

    pub fn family(&self) -> NetworkAddressFamily {
        NetworkAddressFamily::from(self._family)
    }
}

impl From<u32> for NetworkAddress {
    fn from(value: u32) -> Self {
        let in_bytes: [u8;4] = value.to_le_bytes();
        let mut out = Self::default();
        out.bytes.clone_from_slice(&in_bytes);
        out
    }
}

impl From<u128> for NetworkAddress {
    fn from(value: u128) -> Self {
        let in_bytes: [u8;16] = value.to_be_bytes();
        let mut out = Self::default();
        out.bytes.clone_from_slice(&in_bytes);
        out
    }
}

impl From<Ipv4Addr> for NetworkAddress {
    fn from(value: Ipv4Addr) -> Self {
        let mut out = Self::default();
        out.bytes.clone_from_slice(&value.octets());
        out
    }
}

impl From<Ipv6Addr> for NetworkAddress {
    fn from(value: Ipv6Addr) -> Self {
        let mut out = Self::default();
        out.bytes.clone_from_slice(&value.octets());
        out
    }
}

impl From<MacAddr6> for NetworkAddress {
    fn from(value: MacAddr6) -> Self {
        let mut out = Self::default();
        out.bytes.clone_from_slice(value.as_bytes());
        out
    }
}

impl From<MacAddr8> for NetworkAddress {
    fn from(value: MacAddr8) -> Self {
        let mut out = Self::default();
        out.bytes.clone_from_slice(value.as_bytes());
        out
    }
}
