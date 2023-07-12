use libc::{c_int,c_void,sockaddr,c_char};

// typedef struct addrinfo {
//   int             ai_flags;
//   int             ai_family;
//   int             ai_socktype;
//   int             ai_protocol;
//   size_t          ai_addrlen;
//   char            *ai_canonname;
//   struct sockaddr *ai_addr;
//   struct addrinfo *ai_next;
// } ADDRINFOA, *PADDRINFOA;
#[derive(Default, Debug, Clone)]
pub struct ZmqAddrInfo {
    pub ai_flags: c_int,
    pub ai_family: c_int,
    pub ai_socktype: c_int,
    pub ai_protocol: c_int,
    pub ai_addrlen: usize,
    pub ai_canonname: String,
    pub ai_addr: sockaddr,
    // pub ai_next: *mut c_void,
}