#[derive(Debug, PartialEq, EnumString)]
enum protocol_name {
    inproc,
    tcp,
    udp,
    pgm,
    epgm,
    norm,
    ws,
    wss,
    ipc,
    tipc,
    vmci,
}

pub struct address_t {
    pub protocol: String,
    pub address: String,
    pub parent: *const ctx_t,
    pub tcp_addr: *mut tcp_address_t,
    pub udp_addr: *mut udp_address_t,
    #[cfg(feature = "ws")]
    pub ws_addr: *mut ws_address_t,
    #[cfg(feature = "wss")]
    pub wss_addr: *mut wss_address_t,
    #[cfg(feature = "ipc")]
    pub ipc_addr: *mut ipc_address_t,
    #[cfg(feature = "tipc")]
    pub tipc_addr: *mut tipc_address_t,
    #[cfg(feature = "vmci")]
    pub vmci_addr: *mut vmci_address_t,
}

impl address_t {
    pub fn new(protocol: &str, address: &str) {
        address_t {
            protocol: protocol.to_string(),
            address: address.to_string(),
            parent: std::ptr::null(),
            tcp_addr: std::ptr::null_mut(),
            udp_addr: std::ptr::null_mut(),
            #[cfg(feature = "ws")]
            ws_addr: std::ptr::null_mut(),
            #[cfg(feature = "wss")]
            wss_addr: std::ptr::null_mut(),
            #[cfg(feature = "ipc")]
            ipc_addr: std::ptr::null_mut(),
            #[cfg(feature = "tipc")]
            tipc_addr: std::ptr::null_mut(),
            #[cfg(feature = "vmci")]
            vmci_addr: std::ptr::null_mut(),
        }
    }
    pub fn to_string(&self, addr: &mut String) -> i32 {
        if self.protocol == protocol_name::tcp && self.tcp_addr.is_some() {
            self.tcp_addr.to_string(addr)
        } else if self.protocol == protocol_name::udp && self.udp_addr.is_some() {
            self.udp_addr.to_string(addr)
        } else if self.protocol == protocol_name::ws && self.ws_addr.is_some() {
            self.ws_addr.to_string(addr)
        } else if self.protocol == protocol_name::wss && self.wss_addr.is_some() {
            self.wss_addr.to_string(addr)
        } else if self.protocol == protocol_name::ipc && self.ipc_addr.is_some() {
            self.ipc_addr.to_string(addr)
        } else if self.protocol == protocol_name::tipc && self.tipc_addr.is_some() {
            self.tipc_addr.to_string(addr)
        } else if self.protocol == protocol_name::vmci && self.vmci_addr.is_some() {
            self.vmci_addr.to_string(addr)
        } else if self.protocol.is_ascii() == false && self.address.is_empty() == false {
            *addr = format!("{}://{}", self.protocol, self.address).to_string();
            0
        } else {
            -1
        }
    }
}

#[cfg(target_os = "windows")]
pub type zmq_socklen_t = u32;
#[cfg(not(target_os = "windows"))]
pub type zmq_socklen_t = socklen_t;

pub enum socket_end_t {
    socket_end_local,
    socket_end_remote,
}

pub fn get_socket_address(
    fd_: fd_t,
    socket_end: socket_end_t,
    ss: *mut sockaddr_storage,
) -> zmq_socklen_t {
    let mut sl: zmq_socklen_t = std::mem::size_of::<sockaddr_storage>() as zmq_socklen_t;
    if socket_end == socket_end_t::socket_end_local {
        unsafe {
            getsockname(fd_, ss as *mut sockaddr, &mut sl as *mut zmq_socklen_t);
        }
    } else {
        unsafe {
            getpeername(fd_, ss as *mut sockaddr, &mut sl as *mut zmq_socklen_t);
        }
    }
    sl
}

pub fn get_socket_name(fd: fd_t, socket_end: socket_end_t) -> String {
    let mut ss: sockaddr_storage;
    let mut sl: zmq_socklen_t = get_socket_address(fd_, socket_end, &mut ss);
    let mut address_string: String = "".to_string();
    let mut addr = unsafe { &ss as *const sockaddr_storage as *const sockaddr };
    address_string = addr.to_string();
    address_string
}
