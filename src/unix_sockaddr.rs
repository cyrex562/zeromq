
use libc::{c_char, c_int, c_ulong, size_t, c_void};

#[allow(non_camel_case_types)]
#[derive(Default,Debug,Clone)]
pub struct sockaddr {
    pub sa_family: u16, // AF_XXX
    pub sa_data: [u8;14] // protocol-specific address
}

#[allow(non_camel_case_types)]
#[derive(Default,Debug,Clone)]
pub struct sockaddr_in {
    pub sin_family: i16, // AF_XXX
    pub sin_port: u16, // port number in network-byte order
    pub sin_addr: in_addr, // 32-bit IP address in network byte order
    pub sin_zero: [u8;8], /// null
}

#[allow(non_camel_case_types)]
#[derive(Default,Debug,Clone)]
pub struct in_addr {
    pub s_addr: c_ulong, // 32-bit IP in network byte order
}

#[allow(non_camel_case_types)]
#[derive(Default,Debug,Clone)]
pub struct hostent {
    pub h_name: *mut c_char, // official hostname
    pub h_aliases: *mut *mut c_char, // list of hostname aliases
    pub h_addrtype: c_int, // address famly, always set to AF_INET
    pub h_length: c_int, // always set to 4 (length of AF_INET
}

#[allow(non_camel_case_types)]
#[derive(Default,Debug,Clone)]
pub struct servent {
    pub s_name: *mut c_char, // name of the service
    pub s_aliases: *mut *mut c_char, // alias names, frequently NULL
    pub s_port: i32, // port numbers
    pub s_proto: *mut c_char // name of the protocol like TCP or UDP
}

#[allow(non_camel_case_types)]
pub union in6_addr_u {
    bytes: [u8;16],
    words: [u16;8]
}

#[allow(non_camel_case_types)]
#[derive(Default,Debug,Clone)]
pub struct in6_addr {
    u: in6_addr_u
}

#[allow(non_camel_case_types)]
#[derive(Default,Debug,Clone)]
pub struct sockaddr_in6 {
    sin6_family: u16,
    sin6_port: u16,
    sin6_flowinfo: c_ulong,
    sin6_addr: in6_addr,
    sin6_scope_id: c_ulong,
}

#[allow(non_camel_case_types)]
#[derive(Default,Debug,Clone)]
pub struct addrinfo {
    pub ai_flags: i32, // flags including AI_PASSIVE, AI_CANNONAME, AI_NUMERICHOST, found in netdb.h
    pub ai_family: i32, // protocol family including PF_UNSPEC and PF_INET found in sys/socket.h
    pub ai_socktype: i32, // socket type, including SOCK_STREAM, and SOCK_DGRAM found in sys/socket.h
    pub ai_protocol: i32, // protocol, including IPPROTO_TCP and IPPROTO_UDP, found in netinet/in.h
    pub ai_addrlen: size_t, // the length of the addr member
    pub ai_canonname: *mut c_char, // the canonical name for nodename
    pub ai_addr: *mut sockaddr, // binary socket address
    pub ai_next: *mut c_void, // pointer to the next addrinfo struct
}
