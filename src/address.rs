use libc::sockaddr;

use crate::ctx::ZmqContext;
use crate::defines::{ZmqFd, ZmqSockaddrStorage};
use crate::ip_resolver::ZmqIpAddress;
use crate::tcp_address::ZmqTcpAddress;
use crate::udp_address::UdpAddress;

pub enum SocketEnd {
    SocketEndLocal,
    SocketEndRemote,
}

pub union AddressUnion {
    //pub dummy: *mut c_void,
    // pub tcp_addr: ZmqTcpAddress,
    // TCP Address

    // pub udp_addr: UdpAddress,
    // UDP Address

    // pub ws_addr: *mut ws_address_t,
    // pub wss_addr: *mut wss_address_t,
    // pub ipc_addr: *mut ipc_address_t,
    // pub tipc_addr: *mut tipc_address_t,
    // pub vmci_addr: *mut vmci_address_t,
}

impl std::fmt::Debug for AddressUnion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ZmqAddress<'a> {
    pub protocol: String,
    pub address: String,
    pub parent: &'a mut ZmqContext<'a>,
    // pub resolved: AddressUnion,
    // pub address: ZmqIpAddress,
    pub source_address: ZmqIpAddress,
    pub has_src_addr: bool,
    pub _bind_address: ZmqIpAddress,
    pub _bind_interface: i32,
    pub _target_address: ZmqIpAddress,
    pub _is_multicast: bool,
    pub _address: String,
    pub tcp_addr: ZmqTcpAddress,
    pub udp_addr: UdpAddress,
}

// impl Clone for ZmqAddress {
//     fn clone(&self) -> Self {
//         Self { protocol: self.protocol.clone(), address: self.address.clone(), parent: self.parent.clone(), resolved: self.resolved.clone() }
//     }
// }

// impl std::fmt::Debug for ZmqAddress
// {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("address_t").field("protocol", &self.protocol).field("address", &self.address).field("parent", &self.parent).field("resolved", &self.resolved).finish()
//     }
// }

// impl Default for ZmqAddress
// {
//     fn default() -> Self {
//         Self { resolved: AddressUnion{dummy: null_mut()}, ..Default::default() }
//     }
// }

impl ZmqAddress {
    pub fn new(protocol_: &mut String, address_: &mut String, parent_: &mut ZmqContext) -> Self {
        Self {
            protocol: (*protocol_).clone(),
            address: (*address_).clone(),
            parent: parent_,
            // resolved: AddressUnion{dummy: null_mut()},
            source_address: Default::default(),
            has_src_addr: false,
            _bind_address: Default::default(),
            _bind_interface: 0,
            _target_address: Default::default(),
            _is_multicast: false,
            _address: "".to_string(),
            tcp_addr: Default::default(),
            udp_addr: Default::default(),
        }
    }

    pub fn to_string(&mut self) -> String {
        todo!()
    }
}

// zmq::zmq_socklen_t zmq::get_socket_address (fd_t fd_,
//                                             socket_end_t socket_end_,
//                                             sockaddr_storage *ss_)
// {
//     zmq_socklen_t sl = static_cast<zmq_socklen_t> (sizeof (*ss_));
//
//     const int rc =
//       socket_end_ == socket_end_local
//         ? getsockname (fd_, reinterpret_cast<struct sockaddr *> (ss_), &sl)
//         : getpeername (fd_, reinterpret_cast<struct sockaddr *> (ss_), &sl);
//
//     return rc != 0 ? 0 : sl;
// }
pub fn get_socket_address(
    fd: ZmqFd,
    socket_end: SocketEnd,
    ss: &mut ZmqSockaddrStorage,
) -> ZmqSocklen {
    todo!()
}

// template <typename T>
// std::string get_socket_name (fd_t fd_, socket_end_t socket_end_)
// {
//     struct sockaddr_storage ss;
//     const zmq_socklen_t sl = get_socket_address (fd_, socket_end_, &ss);
//     if (sl == 0) {
//         return std::string ();
//     }
//
//     const T addr (reinterpret_cast<struct sockaddr *> (&ss), sl);
//     std::string address_string;
//     addr.to_string (address_string);
//     return address_string;
// }
// }
pub fn get_socket_name<T>(fd_: ZmqFd, socket_end_: SocketEnd) -> String {
    let mut ss = ZmqSockaddrStorage::default();
    let mut sl = get_socket_address(fd_, socket_end_, &mut ss);
    //  const T addr (reinterpret_cast<struct sockaddr *> (&ss), sl);
    // TODO figure out how to create instance of T from sl and ss

    let mut address_string = String::new();
    let addr = &ss as *mut sockaddr;
    addr.to_string(&mut address_string);
    address_string
}
