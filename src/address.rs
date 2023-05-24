use std::ptr::null_mut;

use crate::context::ZmqContext;
use crate::fd::ZmqFileDesc;
use crate::ipc_address::IpcAddress;
use crate::platform_socket::{ZmqSockaddr, ZmqSockaddrStorage};
use crate::tcp_address::TcpAddress;
use crate::tipc_address::TipcAddress;
use crate::udp_address::UdpAddress;
use crate::vmci_address::VmciAddress;
use crate::ws_address::WsAddress;
use crate::wss_address::WssAddress;

pub const protocol_name_inproc: &str = "inproc";
pub const protocol_name_tcp: &str = "tcp";
pub const protocol_name_udp: &str = "udp";
pub const protocol_name_pgm: &str = "pgm";
pub const protocol_name_epgm: &str = "epgm";
pub const protocol_name_norm: &str = "norm";
pub const protocol_name_ws: &str = "ws";
pub const protocol_name_wss: &str = "wss";
pub const protocol_name_ipc: &str = "ipc";
pub const protocol_name_tipc: &str = "tipc";
pub const protocol_name_vmci: &str = "vmci";

pub enum SocketEnd {
    SocketEndLocal,
    SocketEndRemote,
}

// pub union ZmqAddressResolved {
//     pub dummy: *mut libc::c_void,
//     pub tcp_addr: *mut TcpAddress,
//     pub udp_addr: *mut UdpAddress,
//     pub ws_addr: *mut WsAddress,
//     pub wss_addr: *mut WssAddress,
//     pub ipc_addr: *mut IpcAddress,
//     pub tipc_addr: *mut TipcAddress,
//     pub vmci_addr: *mut VmciAddress,
// }

#[derive(Default, Debug, Clone)]
pub struct Address<'a, T> {
    // const std::string protocol;
    pub protocol: String,
    // const std::string address;
    pub address: String,
    // ctx_t *const parent;
    pub parent: ZmqContext,
    //  Protocol specific resolved address
    //  All members must be pointers to allow for consistent initialization
    pub resolved: T,
}

impl Address<T> {
    // address_t (const std::string &protocol_,
    //     const std::string &address_,
    //     ctx_t *parent_);
    pub fn new(protocol: &str, address: &str, parent: &ZmqContext) -> Self {
        Self {
            protocol: String::from(protocol),
            address: String::from(address),
            parent: parent.clone(),
            resolved: ZmqAddressResolved { dummy: null_mut() },
        }
    }

    pub fn from_sockaddr(sa: &mut ZmqSockaddr) -> Self {
        let mut addr = Self::default();
    }
}

impl ToString for Address<T> {
    fn to_string(&self) -> String {
        let mut s = String::new();
        match self.protocol {
            protocol_name_inproc => s.push_str("inproc://"),
            protocol_name_tcp => s.push_str("tcp://"),
            protocol_name_udp => s.push_str("udp://"),
            protocol_name_pgm => s.push_str("pgm://"),
            protocol_name_epgm => s.push_str("epgm://"),
            protocol_name_norm => s.push_str("norm://"),
            protocol_name_ws => s.push_str("ws://"),
            protocol_name_wss => s.push_str("wss://"),
            protocol_name_ipc => s.push_str("ipc://"),
            protocol_name_tipc => s.push_str("tipc://"),
            protocol_name_vmci => s.push_str("vmci://"),
            _ => {}
        }
        s.push_str(self.address.as_str());
        s
    }
}

// int Address::to_string (std::string &addr_) const
// {
//     if (protocol == protocol_name::tcp && resolved.tcp_addr)
//         return resolved.tcp_addr.to_string (addr_);
//     if (protocol == protocol_name::udp && resolved.udp_addr)
//         return resolved.udp_addr.to_string (addr_);
// // #ifdef ZMQ_HAVE_WS
//     if (protocol == protocol_name::ws && resolved.ws_addr)
//         return resolved.ws_addr.to_string (addr_);
// // #endif
// // #ifdef ZMQ_HAVE_WSS
//     if (protocol == protocol_name::wss && resolved.ws_addr)
//         return resolved.ws_addr.to_string (addr_);
// // #endif
// // #if defined ZMQ_HAVE_IPC
//     if (protocol == protocol_name::ipc && resolved.ipc_addr)
//         return resolved.ipc_addr.to_string (addr_);
// // #endif
// // #if defined ZMQ_HAVE_TIPC
//     if (protocol == protocol_name::tipc && resolved.tipc_addr)
//         return resolved.tipc_addr.to_string (addr_);
// // #endif
// // #if defined ZMQ_HAVE_VMCI
//     if (protocol == protocol_name::vmci && resolved.vmci_addr)
//         return resolved.vmci_addr.to_string (addr_);
// // #endif

//     if (!protocol.is_empty() && !address.empty ()) {
//         std::stringstream s;
//         s << protocol << "://" << address;
//         addr_ = s.str ();
//         return 0;
//     }
//     addr_.clear ();
//     return -1;
// }

pub fn get_socket_address(
    fd: ZmqFileDesc,
    socket_end: SocketEnd,
    ss: *mut ZmqSockaddrStorage,
) -> anyhow::Result<usize> {
    // // usize sl = static_cast<usize> (sizeof (*ss_));
    // let mut sl = mem::size_of::<ZmqSockaddrStorage>();

    // if socket_end == SocketEnd::SocketEndLocal {
    //     unsafe {libc::getsockname(fd, ss, sl)};
    // } else {
    //     unsafe {libc::getpeername(fd, ss, sl)}
    // }

    // Ok(sl as usize)
    todo!()
}

pub fn get_socket_name(fd: ZmqFileDesc, socket_end: SocketEnd) -> anyhow::Result<String> {
    // // struct sockaddr_storage ss;
    // let mut ss: ZmqSockaddrStorage = ZmqSockaddrStorage{};
    // let mut sl: usize = get_socket_address (fd, socket_end, &ss);
    // if (sl == 0) {
    //     return String::empty();
    // }

    // // const T addr (reinterpret_cast<struct sockaddr *> (&ss), sl);
    // let mut addr = Address::from_sockaddr(ss as ZmqSockaddr);
    // // std::string address_string;
    // let mut address_string: String = String::new();

    // return addr.to_string();;
    todo!()
}
