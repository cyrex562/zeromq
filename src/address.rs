use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use crate::defines::ZmqFileDesc;
use crate::platform_socket::ZmqSockaddrStorage;
use crate::transport::ZmqTransport;

#[derive(Default,Debug,Clone)]
pub struct ZmqAddress {
    pub protocol: ZmqTransport,
    pub family: i32,
    pub ip_addr: IpAddr,
    pub addr_sockaddr: SocketAddr,
    pub src_addr: SocketAddr,
    pub bind_addr: SocketAddr,
    pub tgt_addr: SocketAddr,
    pub bind_interface: i32,
    pub is_multicast: bool,
    pub has_src_addr: bool,
    pub addr_str: String,
    pub tipc_sockaddr: sockaddr_tipc,
    pub vm_sockaddr: sockaddr_vm,
    pub host: String,
    pub path: String,
}

pub enum ZmqSocketEnd {
    SocketEndLocal,
    SocketEndRemote,
}

#[cfg(target_os = "windows")]
pub struct sockaddr_tipc {}

#[cfg(target_os = "windows")]
pub struct sockaddr_vm {}
// struct sockaddr_vm {
//                sa_family_t    svm_family;    /* Address family: AF_VSOCK */
//                unsigned short svm_reserved1;
//                unsigned int   svm_port;      /* Port # in host byte order */
//                unsigned int   svm_cid;       /* Address in host byte order */
//                unsigned char  svm_zero[sizeof(struct sockaddr) -
//                                        sizeof(sa_family_t) -
//                                        sizeof(unsigned short) -
//                                        sizeof(unsigned int) -
//                                        sizeof(unsigned int)];
//            };

impl ZmqAddress {
    pub fn new(protocol: ZmqTransport,
               family: i32,
               bind_interface: i32,
               is_multicast: bool,
               ip_addr: Option<IpAddr>,
               addr_sockaddr: Option<SocketAddr>,
               addr_str: Option<&str>,
                bind_sockaddr: Option<SocketAddr>,
                src_sockaddr: Option<SocketAddr>,
                tgt_sockaddr: Option<SocketAddr>,
                tipc_sockaddr: Option<sockaddr_tipc>,
                vm_sockaddr: Option<sockaddr_vm>,
                host: Option<&str>,
                path: Option<&str>,
               ) -> Self {
        Self {
            protocol,
            ip_addr: ip_addr.unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
            addr_sockaddr: addr_sockaddr.unwrap_or(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0)),
            addr_str: addr_str.unwrap_or("").to_string(),
            tipc_sockaddr: tipc_sockaddr.unwrap_or(sockaddr_tipc{}).clone(),
            vm_sockaddr: vm_sockaddr.unwrap_or(sockaddr_vm{}).clone(),
            host: host.unwrap_or("").to_string(),
            path: path.unwrap_or("").to_string(),
            bind_addr: bind_sockaddr.unwrap_or(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0)).clone(),
            src_addr: src_sockaddr.unwrap_or(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0)).clone(),
            tgt_addr: tgt_sockaddr.unwrap_or(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0)).clone(),
            is_multicast,
            family,
            bind_interface,
            has_src_addr: src_sockaddr.is_some(),
        }
    }

    pub fn is_service(&self) -> bool {
     // if self.addr_tipc_sockaddr.addrtype == TIPC_ADDR_ID { return false }
        todo!()
    }

    //  This function sets up the address for UNIX domain transport.
    // int resolve (path_: &str);
    pub fn resolve(&mut self) -> anyhow::Result<()> {
        // TODO: Implement this
        // let path_len = path_.len();
        // if (path_len >= self.address.sun_path.len()) {
        //     errno = ENAMETOOLONG;
        //     return -1;
        // }
        // if (path_[0] == '@' && !path_[1]) {
        //     errno = EINVAL;
        //     return -1;
        // }
        //
        // address.sun_family = AF_UNIX;
        // copy_bytes(
        //     address.sun_path,
        //     0,
        //     path_.as_bytes(),
        //     0,
        //     (path_len + 1) as i32,
        // );
        // /* Abstract sockets start with 0 */
        // if (path_[0] == '@') {
        //     *address.sun_path = 0;
        // }
        //
        // _addrlen = (offsetof(sockaddr_un, sun_path) + path_len);
        // return 0;
        todo!()
    }

    // TODO: TCP
    // pub fn resolve(&mut self, name: &mut str, local: bool, ipv6: bool) -> i32 {
    //         // Test the ';' to know if we have a source address in name
    //         // const char *src_delimiter = strrchr (name, ';');
    //         let src_delimeter = name.find(';');
    //         if src_delimiter.is_some() {
    //             let src_name = &name[..src_delimeter.unwrap()];
    //             // const std::string src_name (name, src_delimiter - name);
    //
    //             let mut src_resolver_opts = IpResolverOptions {
    //                 bindable: true,
    //                 allow_dns: true,
    //                 allow_nic_name: true,
    //                 ipv6,
    //                 expect_port: true,
    //                 allow_path: false,
    //             };
    //
    //             // src_resolver_opts
    //             //   .bindable (true)
    //             //   //  Restrict hostname/service to literals to avoid any DNS
    //             //   //  lookups or service-name irregularity due to
    //             //   //  indeterminate socktype.
    //             //   .allow_dns (false)
    //             //   .allow_nic_name (true)
    //             //   .ipv6 (ipv6)
    //             //   .expect_port (true);
    //             let mut src_resolver = IpResolver::new(&src_resolver_opts);
    //             // TODO
    //             // let rc =  src_resolver.resolve (&mut self.src_addr.unwrap(), src_name);
    //             if rc != 0 {
    //                 return -1;
    //             }
    //             *name = src_delimiter[1..];
    //             self.has_src_addr = true;
    //         }
    //
    //         // IpResolverOptions resolver_opts;
    //         // resolver_opts.bindable (local_)
    //         //           .allow_dns (!local_)
    //         //           .allow_nic_name (local_)
    //         //           .ipv6 (ipv6)
    //         //           .expect_port (true);
    //         let resolver_opts = IpResolverOptions {
    //             bindable: false,
    //             allow_dns: !local,
    //             allow_nic_name: local,
    //             ipv6,
    //             expect_port: true,
    //             allow_path: false,
    //         };
    //
    //         // ip_resolver_t resolver (resolver_opts);
    //         let mut resolver = IpResolver::new(&resolver_opts);
    //
    //         // return resolver.resolve (&address, name);
    //         // TODO
    //         // return resolver.resolve(&mut self.addr, name);
    //     }

    // TODO: TIPC
    //  pub fn resolve(&mut self, name: &str) -> i32 {
    //         let mut type_ = 0;
    //         let mut lower = 0;
    //         let mut upper = 0;
    //         let mut ref_ = 0;
    //         let mut z = 1;
    //         let mut c = 0;
    //         let mut n = 0;
    //         // char eof;
    //         let mut eof = 0u8;
    //         // const char *domain;
    //         let mut domain: *mut c_char;
    //         let mut res = 0;
    //
    //
    //         // if (strncmp (name, "<*>", 3) == 0)
    //         if name.contains("<*>") {
    //             set_random();
    //             address.family = AF_TIPC;
    //             address.addrtype = TIPC_ADDR_ID;
    //             address.addr.id.node = 0;
    //             address.addr.id.ref_ = 0;
    //             address.scope = 0;
    //             return 0;
    //         }
    //
    //         // res = sscanf (name, "{%u,%u,%u}", &type, &lower, &upper);
    //         /* Fetch optional domain suffix. */
    //         // if ((domain = strchr (name, '@'))) {
    //         //     if (sscanf (domain, "@%u.%u.%u%c", &z, &c, &n, &eof) != 3)
    //         //         return EINVAL;
    //         // }
    //         if (res == 3) {
    //             if (type_ < TIPC_RESERVED_TYPES || upper < lower) {
    //                 return EINVAL;
    //             }
    //             address.family = AF_TIPC;
    //             address.addrtype = TIPC_ADDR_NAMESEQ;
    //             address.addr.nameseq.type_ = type_;
    //             address.addr.nameseq.lower = lower;
    //             address.addr.nameseq.upper = upper;
    //             address.scope = TIPC_ZONE_SCOPE;
    //             return 0;
    //         }
    //         if (res == 2 && type_ > TIPC_RESERVED_TYPES) {
    //             address.family = AF_TIPC;
    //             address.addrtype = TIPC_ADDR_NAME;
    //             address.addr.name.name.type_ = type_;
    //             address.addr.name.name.instance = lower;
    //             address.addr.name.domain = tipc_addr(z, c, n);
    //             address.scope = 0;
    //             return 0;
    //         } else if (res == 0) {
    //             res = sscanf(name, "<%u.%u.%u:%u>", &z, &c, &n, &ref_);
    //             if (res == 4) {
    //                 address.family = AF_TIPC;
    //                 address.addrtype = TIPC_ADDR_ID;
    //                 address.addr.id.node = tipc_addr(z, c, n);
    //                 address.addr.id.ref_ = ref_;
    //                 address.scope = 0;
    //                 return 0;
    //             }
    //         }
    //         return EINVAL;
    //     }

    // TODO UDP resolve
    //  pub fn resolve(&mut self, name: &str, bind: bool, ipv6: bool) -> anyhow::Result<()> {
    //         //  No IPv6 support yet
    //         let has_interface = false;
    //
    //         self.address = name.to_string();
    //
    //         //  If we have a semicolon then we should have an interface specifier in the
    //         //  URL
    //         // const char *src_delimiter = strrchr (name, ';');
    //         let src_delimiter = name.find(';');
    //         if src_delimiter.is_some() {
    //             let src_name = name[0..src_delimiter.unwrap()].to_string();
    //
    //             let mut src_resolver_opts: IpResolverOptions = Default::default();
    //
    //             src_resolver_opts.bindable(true);
    //                 //  Restrict hostname/service to literals to avoid any DNS
    //                 //  lookups or service-name irregularity due to
    //                 //  indeterminate socktype..allow_dns(false).allow_nic_name(true).ipv6(ipv6).expect_port(false);
    //
    //             let mut src_resolver = IpResolver::new(&src_resolver_opts);
    //
    //
    //
    //             if (self.bind_address.is_multicast()) {
    //                 //  It doesn't make sense to have a multicast address as a source
    //                 errno = EINVAL;
    //                 return -1;
    //             }
    //
    //             //  This is a hack because we need the interface index when binding
    //             //  multicast IPv6, we can't do it by address. Unfortunately for the
    //             //  time being we don't have a generic platform-independent function to
    //             //  resolve an interface index from an address, so we only support it
    //             //  when an actual interface name is provided.
    //             if (src_name == "*") {
    //                 self.bind_interface = 0;
    //             } else {
    // // #ifdef HAVE_IF_NAMETOINDEX
    //                 self.bind_interface = if_nametoindex(src_name.c_str());
    //                 if (self.bind_interface == 0) {
    //                     //  Error, probably not an interface name.
    //                     self.bind_interface = -1;
    //                 }
    // // #endif
    //             }
    //
    //             self.has_interface = true;
    //             *name = name[src_delimiter + 1..].to_string();
    //         }
    //
    //         let mut resolver_opts = IpResolverOptions::default();
    //
    //         resolver_opts.bindable(bind).allow_dns(!bind).allow_nic_name(bind).expect_port(true).ipv6(ipv6);
    //
    //         let mut resolver = IpResolver::new(resolver_opts);
    //
    //         let rc: i32 = resolver.resolve(&mut target_address, name);
    //         if (rc != 0) {
    //             return -1;
    //         }
    //
    //         self.is_multicast = target_address.is_multicast();
    //         let port = target_address.port();
    //
    //         if (self.has_interface) {
    //             //  If we have an interface specifier then the target address must be a
    //             //  multicast address
    //             if (!self.is_multicast) {
    //                 errno = EINVAL;
    //                 return -1;
    //             }
    //
    //             self.bind_address.set_port(port);
    //         } else {
    //             //  If we don't have an explicit interface specifier then the URL is
    //             //  ambiguous: if the target address is multicast then it's the
    //             //  destination address and the Bind address is ANY, if it's unicast
    //             //  then it's the Bind address when 'Bind' is true and the destination
    //             //  otherwise
    //             if (self.is_multicast || !bind) {
    //                 self.bind_address = if self.target_address.is_ipv4() {
    //                     IpAddr::from_str("0.0.0.0").unwrap()
    //                     // ip_addr_t::any (target_address.family ());
    //                 } else {
    //                     IpAddr::from_str("::").unwrap()
    //                 };
    //                 self.bind_address.set_port(port);
    //                 self.bind_interface = 0;
    //             } else {
    //                 //  If we were asked for a Bind socket and the address
    //                 //  provided was not multicast then it was really meant as
    //                 //  a Bind address and the target_address is useless.
    //                 self.bind_address = target_address;
    //             }
    //         }
    //
    //         if (bind_address.family() != target_address.family()) {
    //             errno = EINVAL;
    //             return -1;
    //         }
    //
    //         //  For IPv6 multicast we *must* have an interface index since we can't
    //         //  Bind by address.
    //         if (ipv6 && is_multicast && bind_interface < 0) {
    //             errno = ENODEV;
    //             return -1;
    //         }
    //
    //         return 0;
    //     }

    // TODO VMCI resolve
    //  pub fn resolve (path_: &str) -> i32
    //     {
    //         //  Find the ':' at end that separates address from the port number.
    //         // const char *delimiter = strrchr (path_, ':');
    //         // if (!delimiter) {
    //         //     errno = EINVAL;
    //         //     return -1;
    //         // }
    //         //
    //         // //  Separate the address/port.
    //         // std::string addr_str (path_, delimiter - path_);
    //         // std::string port_str (delimiter + 1);
    //         //
    //         // unsigned int cid = VMADDR_CID_ANY;
    //         // unsigned int port = VMADDR_PORT_ANY;
    //         //
    //         // if (!addr_str.length ()) {
    //         //     errno = EINVAL;
    //         //     return -1;
    //         // } else if (addr_str == "@") {
    //         //     cid = VMCISock_GetLocalCID ();
    //         //
    //         //     if (cid == VMADDR_CID_ANY) {
    //         //         errno = ENODEV;
    //         //         return -1;
    //         //     }
    //         // } else if (addr_str != "*" && addr_str != "-1") {
    //         //     const char *begin = addr_str;
    //         //     char *end = null_mut();
    //         //     unsigned long l = strtoul (begin, &end, 10);
    //         //
    //         //     if ((l == 0 && end == begin) || (l == ULONG_MAX && errno == ERANGE)
    //         //         || l > UINT_MAX) {
    //         //         errno = EINVAL;
    //         //         return -1;
    //         //     }
    //         //
    //         //     cid = static_cast<unsigned int> (l);
    //         // }
    //         //
    //         // if (!port_str.length ()) {
    //         //     errno = EINVAL;
    //         //     return -1;
    //         // } else if (port_str != "*" && port_str != "-1") {
    //         //     const char *begin = port_str;
    //         //     char *end = null_mut();
    //         //     unsigned long l = strtoul (begin, &end, 10);
    //         //
    //         //     if ((l == 0 && end == begin) || (l == ULONG_MAX && errno == ERANGE)
    //         //         || l > UINT_MAX) {
    //         //         errno = EINVAL;
    //         //         return -1;
    //         //     }
    //         //
    //         //     port = static_cast<unsigned int> (l);
    //         // }
    //         //
    //         // address.svm_family =
    //         //   static_cast<sa_family_t> (parent.get_vmci_socket_family ());
    //         // address.svm_cid = cid;
    //         // address.svm_port = port;
    //         todo!();
    //
    //         return 0;
    //     }

    // TODO: ws addr resolve
    //  pub fn resolve(name: &str, local_: bool, ipv6: bool) -> i32
    //     {
    //         //  find the host part, It's important to use str*r*chr to only get
    //         //  the latest colon since IPv6 addresses use colons as delemiters.
    //         // const char *delim = strrchr (name, ':');
    //         // if (delim == null_mut()) {
    //         //     errno = EINVAL;
    //         //     return -1;
    //         // }
    //         // _host = std::string (name, delim - name);
    //         let mut _host = String::from("");
    //         let name_only = name.split(":").first();
    //         if name_only.is_some() {
    //             _host = String::from(name_only.unwrap());
    //         } else {
    //             return -1;
    //         }
    //
    //         // find the path part, which is optional
    //         // TODO
    //         // delim = strrchr (name, '/');
    //         host_name: String;
    //         if (delim) {
    //             _path = std::string(delim);
    //             // remove the path, otherwise resolving the port will fail with wildcard
    //             host_name = std::string(name, delim - name);
    //         } else {
    //             _path = std::string("/");
    //             host_name = name;
    //         }
    //
    //         let resolver_opts: IpResolverOptions = IpResolverOptions {
    //             bindable: false,
    //             allow_nic_name: false,
    //             ipv6,
    //             expect_port: false,
    //             allow_dns: false,
    //             allow_path: false,
    //         };
    //         resolver_opts.bindable(local_)
    //             .allow_dns(!local_)
    //             .allow_nic_name(local_)
    //             .ipv6(ipv6)
    //             .allow_path(true)
    //             .expect_port(true);
    //
    //         let mut resolver = IpResolver::new(&resolver_opts);
    //
    //         return resolver.resolve(&mut address, host_name.c_str());
    //     }

    //  The opposite to resolve()
    pub fn to_string(&mut self) -> anyhow::Result<String> {
        todo!()
        // if (address.sun_family != AF_UNIX) {
        //     addr_.clear();
        //     return -1;
        // }
        //
        // let prefix = "ipc://";
        // // char buf[sizeof prefix + sizeof address.sun_path];
        // // char *pos = buf;
        // // memcpy (pos, prefix, sizeof prefix - 1);
        // let mut buf = String::new();
        // buf += prefix;
        // let mut buf_pos = prefix.len();
        // let src_pos = address.sun_path;
        //
        // if (!address.sun_path[0] && address.sun_path[1]) {
        //     // *pos+= 1 = '@';
        //     buf[buf_pos] = '@';
        //     src_pos += 1;
        //     buf_pos += 1;
        // }
        // // according to http://man7.org/linux/man-pages/man7/unix.7.html, NOTES
        // // section, address.sun_path might not always be null-terminated; therefore,
        // // we calculate the length based of addrlen
        // // TODO:
        // // let src_len =
        // // strnlen (src_pos, _addrlen - offsetof (sockaddr_un, sun_path)
        // //     - (src_pos - address.sun_path));
        // // copy_bytes (pos, src_pos, src_len);
        // // addr_.assign (buf, pos - buf + src_len);
        // return 0;
    }

    // TODO TCP address to string
    //  pub fn as_string(&mut self, addr_: &mut String) -> i32 {
    //         if self.addr.ip().is_ipv4() == false && self.addr.ip().is_ipv6() == false {
    //             // self.addr.clear();
    //             return -1;
    //         }
    //
    //         //  Not using service resolving because of
    //         //  https://github.com/zeromq/libzmq/commit/1824574f9b5a8ce786853320e3ea09fe1f822bc4
    //         // char hbuf[NI_MAXHOST];
    //         // let mut hbuf = String::new();
    //         // let mut rc = libc::getnameinfo(addr (), addrlen (), hbuf, mem::size_of::<hbuf>(), NULL,
    //         //                             0, NI_NUMERICHOST);
    //         // dns_lookup::getnameinfo()
    //         // if (rc != 0) {
    //         //     addr_.clear ();
    //         //     return rc;
    //         // }
    //         let mut hbuf = String::from(gethostname::gethostname().to_str().unwrap());
    //
    //
    //         if self.addr.ip().is_ipv6() {
    //             *addr_ = make_address_string(&hbuf, self.addr.port, ipv6_prefix,
    //                                          ipv6_suffix);
    //         } else {
    //             *addr_ = make_address_string(&hbuf, self.addr.port, ipv4_prefix,
    //                                          ipv4_suffix);
    //         }
    //         return 0;
    //     }

    // TODO TIPC to string
    //  pub fn to_string(addr_: &mut str) -> i32 {
    //         // TODO
    //         // if (address.family != AF_TIPC) {
    //         //     addr_.clear ();
    //         //     return -1;
    //         // }
    //         // std::stringstream s;
    //         // if (address.addrtype == TIPC_ADDR_NAMESEQ
    //         //     || address.addrtype == TIPC_ADDR_NAME) {
    //         //     s << "tipc://"
    //         //       << "{" << address.addr.nameseq.type;
    //         //     s << ", " << address.addr.nameseq.lower;
    //         //     s << ", " << address.addr.nameseq.upper << "}";
    //         //     addr_ = s.str ();
    //         // } else if (address.addrtype == TIPC_ADDR_ID || is_random ()) {
    //         //     s << "tipc://"
    //         //       << "<" << tipc_zone (address.addr.id.node);
    //         //     s << "." << tipc_cluster (address.addr.id.node);
    //         //     s << "." << tipc_node (address.addr.id.node);
    //         //     s << ":" << address.addr.id.ref << ">";
    //         //     addr_ = s.str ();
    //         // } else {
    //         //     addr_.clear ();
    //         //     return -1;
    //         // }
    //         // return 0;
    //         todo!()
    //     }

    // TODO UDP to string

    // TODO VMCI to string
    // pub fn to_string (&mut self, addr_: &str) -> i32
    //     {
    //         // if (address.svm_family != parent.get_vmci_socket_family ()) {
    //         //     addr_.clear ();
    //         //     return -1;
    //         // }
    //         //
    //         // std::stringstream s;
    //         //
    //         // s << "vmci://";
    //         //
    //         // if (address.svm_cid == VMADDR_CID_ANY) {
    //         //     s << "*";
    //         // } else {
    //         //     s << address.svm_cid;
    //         // }
    //         //
    //         // s << ":";
    //         //
    //         // if (address.svm_port == VMADDR_PORT_ANY) {
    //         //     s << "*";
    //         // } else {
    //         //     s << address.svm_port;
    //         // }
    //         //
    //         // addr_ = s.str ();
    //         // return 0;
    //         todo!()
    //     }

    // TODO
}

pub fn get_socket_address(
    fd: ZmqFileDesc,
    socket_end: ZmqSocketEnd,
    ss: &mut ZmqSockaddrStorage,
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

pub fn get_socket_name(fd: ZmqFileDesc, socket_end: ZmqSocketEnd) -> anyhow::Result<String> {
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