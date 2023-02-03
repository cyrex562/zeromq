use std::ptr::null_mut;
use crate::tcp_address::TcpAddress;

pub const inproc: String = String::from("inproc");
pub const tcp: String = String::from("tcp");
pub const udp: String = String::from("udp");
pub const pgm: String = String::from("pgm");
pub const epgm: String = String::from("epgm");
pub const norm: String = String::from("norm");
pub const ws: String = String::from("ws");
pub const wss: String = String::from("wss");
pub const ipc: String = String::from("ipc");
pub const tipc: String = String::from("tipc");
pub const vmci: String = String::from("vmci");

type ZmqSocklen = usize;

pub enum SocketEnd
{
    SocketEndLocal,
    SocketEndRemote
}

pub union AddressTResolved {
    pub dummy: *mut libc::c_void,
    pub tcp_addr: *mut TcpAddress,
    pub udp_addr: *mut UdpAddress,
    pub ws_addr: *mut WsAddress,
    pub wss_addr: *mut WssAddress,
    pub ipc_addr: *mut IpcAddress,
    pub tipc_addr: *mut TipcAddress,
    pub vmci_addr: *mut VmciAddress,
}

#[derive(Default,Debug,Clone)]
pub struct Address
{
    // const std::string protocol;
    pub protocol: String,
    // const std::string address;
    pub address: String,
    // ctx_t *const parent;
    pub parent: *mut ZmqContext,
    //  Protocol specific resolved address
    //  All members must be pointers to allow for consistent initialization
    pub resolved: AddressTResolved,
}

impl Address {
    // address_t (const std::string &protocol_,
    //     const std::string &address_,
    //     ctx_t *parent_);
    pub fn new(protocol_: &str, address: &str, parent: *mut ZmqContext) -> Self {
        Self {
            protocol: String::from(protocol_),
            address: String::from(address),
            parent,
            resolved: AddressTResolved{dummy: null_mut()}
        }
    }

    // ~address_t ();

    // int to_string (std::string &addr_) const;
    pub fn to_string(&self, addr_: &str) -> i32 {

    }
}


int Address::to_string (std::string &addr_) const
{
    if (protocol == protocol_name::tcp && resolved.tcp_addr)
        return resolved.tcp_addr->to_string (addr_);
    if (protocol == protocol_name::udp && resolved.udp_addr)
        return resolved.udp_addr->to_string (addr_);
// #ifdef ZMQ_HAVE_WS
    if (protocol == protocol_name::ws && resolved.ws_addr)
        return resolved.ws_addr->to_string (addr_);
// #endif
// #ifdef ZMQ_HAVE_WSS
    if (protocol == protocol_name::wss && resolved.ws_addr)
        return resolved.ws_addr->to_string (addr_);
// #endif
// #if defined ZMQ_HAVE_IPC
    if (protocol == protocol_name::ipc && resolved.ipc_addr)
        return resolved.ipc_addr->to_string (addr_);
// #endif
// #if defined ZMQ_HAVE_TIPC
    if (protocol == protocol_name::tipc && resolved.tipc_addr)
        return resolved.tipc_addr->to_string (addr_);
// #endif
// #if defined ZMQ_HAVE_VMCI
    if (protocol == protocol_name::vmci && resolved.vmci_addr)
        return resolved.vmci_addr->to_string (addr_);
// #endif

    if (!protocol.is_empty() && !address.empty ()) {
        std::stringstream s;
        s << protocol << "://" << address;
        addr_ = s.str ();
        return 0;
    }
    addr_.clear ();
    return -1;
}

ZmqSocklen get_socket_address (fd_t fd_,
                                            SocketEnd socket_end_,
                                            sockaddr_storage *ss_)
{
    ZmqSocklen sl = static_cast<ZmqSocklen> (sizeof (*ss_));

    const int rc =
      socket_end_ == SocketEndLocal
        ? getsockname (fd_, reinterpret_cast<struct sockaddr *> (ss_), &sl)
        : getpeername (fd_, reinterpret_cast<struct sockaddr *> (ss_), &sl);

    return rc != 0 ? 0 : sl;
}

pub fn get_socket_name<T>(fd_: fd_t, socket_end_: SocketEnd) -> String
{
    // struct sockaddr_storage ss;
    let mut ss: sockaddr_storage = sockaddr_storage{};
    let mut sl: ZmqSocklen = get_socket_address (fd_, socket_end_, &ss);
    if (sl == 0) {
        return String::empty();
    }

    // const T addr (reinterpret_cast<struct sockaddr *> (&ss), sl);
    let mut addr = Address::new();
    // std::string address_string;
    let mut address_string: String = String::new();
    addr.to_string(address_string);
    return address_string;
}
