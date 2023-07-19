use crate::address_family::{AF_INET, AF_INET6, AF_UNSPEC};

#[cfg(target_os = "linux")]
use libc::{
    getaddrinfo, if_nametoindex, AI_NUMERICHOST, AI_PASSIVE, AI_V4MAPPED,
};
use libc::{c_void, c_uint, close, free, malloc, strcmp, EINVAL, ENODEV, ENOMEM, memcpy, c_char, sockaddr};
use std::ffi::{CStr, CString};
use std::mem;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::ptr::null_mut;
use std::str::FromStr;
use anyhow::{anyhow, bail};

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::ERROR_BUFFER_OVERFLOW;

#[cfg(target_os = "windows")]
use windows::Win32::Networking::WinSock::{
    freeaddrinfo, AI_NUMERICHOST, AI_PASSIVE, AI_V4MAPPED, SOCK_DGRAM, SOCK_STREAM,
    WSAHOST_NOT_FOUND,ADDRINFOA,getaddrinfo,SOCKADDR,SOCKADDR_IN,SOCKADDR_IN6
};
#[cfg(target_os="windows")]
use windows::Win32::NetworkManagement::IpHelper::{GAA_FLAG_SKIP_ANYCAST, GAA_FLAG_SKIP_DNS_SERVER, GAA_FLAG_SKIP_MULTICAST, GetAdaptersAddresses, if_nametoindex, IP_ADAPTER_ADDRESSES_LH};

use crate::zmq_addrinfo::ZmqAddrInfo;

pub struct IpResolverOptions {
    //
    //
    // bool _bindable_wanted;
    pub bindable: bool,
    // bool _nic_name_allowed;
    pub allow_nic_name: bool,
    // bool _ipv6_wanted;
    pub ipv6: bool,
    // bool _port_expected;
    pub expect_port: bool,
    // bool _dns_allowed;
    pub allow_dns: bool,
    // bool _path_allowed;
    pub allow_path: bool,
}

impl IpResolverOptions {
    // IpResolverOptions ();

    // IpResolverOptions &bindable (bool bindable_);
    // IpResolverOptions &allow_nic_name (bool allow_);
    // IpResolverOptions &ipv6 (bool ipv6);
    // IpResolverOptions &expect_port (bool expect_);
    // IpResolverOptions &allow_dns (bool allow_);
    // IpResolverOptions &allow_path (bool allow_);

    // bool bindable ();
    // bool allow_nic_name ();
    // bool ipv6 ();
    // bool expect_port ();
    // bool allow_dns ();
    // bool allow_path ();
}

pub struct IpResolver {
    pub options: IpResolverOptions,
}

impl IpResolver {
    pub fn new(opts: &IpResolverOptions) -> Self {
        Self {
            options: (*opts).clone(),
        }
    }

    pub fn resolve(&mut self, name: &str) -> anyhow::Result<Vec<SocketAddr>> {
        // let mut addr = String::new();
        let mut addr_str: &str = "";
        let mut port = 0u16;

        let mut out: Vec<SocketAddr> = Vec::new();

        if self.options.expect_port {
            //  We expect 'addr:port'. It's important to use str*r*chr to only get
            //  the latest colon since IPv6 addresses use colons as delemiters.
            // const char *delim = strrchr (name, ':');
            let delim = name.rfind(':');
            if delim.is_none() {
                bail!("EINVAL");
            }

            let addr = &name[..delim.unwrap()];
            let port_str = &name[delim.unwrap() + 1..];

            if port_str == "*" {
                if self._options.bindable() {
                    //  Resolve wildcard to 0 to allow autoselection of port
                    port = 0;
                } else {
                    bail!("EINVAL");
                }
            } else if port_str == "0" {
                //  Using "0" for a Bind address is equivalent to using "*". For a
                //  connectable address it could be used to connect to port 0.
                port = 0;
            } else {
                //  Parse the port number (0 is not a valid port).
                port = u16::from_str(port_str).unwrap();
                if port == 0 {
                    bail!("EINVAL");
                }
            }
        } else {
            addr_str = name;
            port = 0;
        }

        // Check if path is allowed in ip address, if allowed it must be truncated
        if self.options.allow_path {
            let pos = addr_str.rfind('/');
            if pos.is_some() {
                addr_str = &addr_str[pos.unwrap()..];
            }
        }

        //  Trim any square brackets surrounding the address. Used for
        //  IPv6 addresses to remove the confusion with the port
        //  delimiter.
        //  TODO Should we validate that the brackets are present if
        //  'addr' contains ':' ?
        if addr_str.contains('[') && addr_str.contains(']') {
            addr_str = addr_str.trim_matches(|c| c == '[' || c == ']');
        }

        //  Look for an interface name / zone_id in the address
        //  Reference: https://tools.ietf.org/html/rfc4007
        let mut pos = addr_str.rfind('%');

        let mut zone_id = 0u32;

        if pos.is_some() {
            let mut if_str = &addr_str[pos.unwrap() + 1..];
            if if_str.is_empty() {
                bail!("EINVAL");
            }
            addr_str = &addr_str[0..pos.unwrap()];
            if if_str[0].is_alphabetic {
                zone_id = do_if_nametoindex(if_str);
            } else {
                zone_id = u32::from(if_str);
            }

            if zone_id == 0 {
                bail!("EINVAL");
            }
        }

        let mut resolved = false;
        let mut addr_str = addr_str;

        if self.options.bindable && addr_str == "*" {
            // TODO: Return an ANY address

            resolved = true;
        }

        if !resolved && self.options.allow_nic_name() {
            //  Try to resolve the string as a NIC
            let mut addresses = self.resolve_nic_name(addr_str)?;
            if addresses.len() > 0 {
                resolved = true;
                for addr in addresses {
                    out.push(SocketAddr::new(addr, port));
                }
            }
        }

        if !resolved {
            let mut addresses = self.resolve_getaddrinfo(addr_str)?;
            if addresses.len() > 0 {
                resolved = true;
                for addr in addresses {
                    out.push(SocketAddr::new(addr, port));
                }
            }
        }
        Ok(out)
    }

    pub fn resolve_getaddrinfo(&mut self, addr_: &str) -> anyhow::Result<Vec<IpAddr>> {
        #[cfg(target_os="windows")]
        resolve_win(addr_)
    }

    //  On Solaris platform, network interface name can be queried by ioctl.
    #[cfg(target_os = "solaris")]
    pub fn resolve_nic_name(ip_addr_: &mut NetworkAddress, nic_: &str) -> i32 {
        //  Create a socket.
        let fd: i32 = open_socket(AF_INET, SOCK_DGRAM, 0);
        // errno_assert (fd != -1);

        //  Retrieve number of interfaces.
        let ifn: lifnum;
        ifn.lifn_family = AF_INET;
        ifn.lifn_flags = 0;
        let rc = ioctl(fd, SIOCGLIFNUM, &ifn);
        // errno_assert (rc != -1);

        //  Allocate memory to get interface names.
        let ifr_size = mem::size_of::<lifreq>() * ifn.lifn_count;
        // char *ifr =  malloc (ifr_size);
        let ifr = String::with_capacity(ifr_size);
        // alloc_assert (ifr);

        //  Retrieve interface names.
        let ifc: lifconf;
        ifc.lifc_family = AF_INET;
        ifc.lifc_flags = 0;
        ifc.lifc_len = ifr_size;
        ifc.lifc_buf = ifr;
        rc = ioctl(fd, SIOCGLIFCONF, &ifc);
        // errno_assert (rc != -1);

        //  Find the interface with the specified name and AF_INET family.
        let found: bool = false;
        lifreq * ifrp = ifc.lifc_req;
        // for (int n = 0; n <  (ifc.lifc_len / mem::size_of::<lifreq>()); n+= 1, ifrp+= 1)
        for n in 0..(ifc.lifc_len / mem::size_of::<lifreq>()) {
            if (!strcmp(nic_, ifrp.lifr_name)) {
                rc = ioctl(fd, SIOCGLIFADDR, ifrp);
                // errno_assert (rc != -1);
                if (ifrp.lifr_addr.ss_family == AF_INET) {
                    ip_addr_.ipv4 = &ifrp.lifr_addr;
                    found = true;
                    break;
                }
            }
            ifrp += 1;
        }

        //  Clean-up.
        free(ifr);
        close(fd);

        if (!found) {
          // errno = ENODEV;
            return -1;
        }
        return 0;
    }

    // #elif defined ZMQ_HAVE_AIX || defined ZMQ_HAVE_HPUX                            \
    //   || defined ZMQ_HAVE_ANDROID || defined ZMQ_HAVE_VXWORKS
    // #include <sys/ioctl.h>
    // #ifdef ZMQ_HAVE_VXWORKS
    // #include <ioLib.h>
    // #endif

    // int resolve_nic_name (ip_addr_t *ip_addr_, nic_: *const c_char)
    // {
    // // #if defined ZMQ_HAVE_AIX || defined ZMQ_HAVE_HPUX
    //     // IPv6 support not implemented for AIX or HP/UX.
    //     if (_options.ipv6 ()) {
    //       // errno = ENODEV;
    //         return -1;
    //     }
    // // #endif
    //
    //     //  Create a socket.
    //     const int sd =
    //       open_socket (_options.ipv6 () ? AF_INET6 : AF_INET, SOCK_DGRAM, 0);
    //     errno_assert (sd != -1);
    //
    //     struct ifreq ifr;
    //
    //     //  Copy interface name for ioctl get.
    //     strncpy (ifr.ifr_name, nic_, sizeof (ifr.ifr_name));
    //
    //     //  Fetch interface address.
    //     const int rc = ioctl (sd, SIOCGIFADDR, (caddr_t) &ifr, mem::size_of::<ifr>());
    //
    //     //  Clean up.
    //     close (sd);
    //
    //     if (rc == -1) {
    //       // errno = ENODEV;
    //         return -1;
    //     }
    //
    //     const int family = ifr.ifr_addr.sa_family;
    //     if (family == (_options.ipv6 () ? AF_INET6 : AF_INET)
    //         && !strcmp (nic_, ifr.ifr_name)) {
    //         memcpy (ip_addr_, &ifr.ifr_addr,
    //                 (family == AF_INET) ? sizeof (struct sockaddr_in)
    //                                     : sizeof (struct sockaddr_in6));
    //     } else {
    //       // errno = ENODEV;
    //         return -1;
    //     }
    //
    //     return 0;
    // }

    // #elif ((defined ZMQ_HAVE_LINUX || defined ZMQ_HAVE_FREEBSD                     \
    //         || defined ZMQ_HAVE_OSX || defined ZMQ_HAVE_OPENBSD                    \
    //         || defined ZMQ_HAVE_QNXNTO || defined ZMQ_HAVE_NETBSD                  \
    //         || defined ZMQ_HAVE_DRAGONFLY || defined ZMQ_HAVE_GNU)                 \
    //        && defined ZMQ_HAVE_IFADDRS)
    //
    // // #include <ifaddrs.h>
    //
    // //  On these platforms, network interface name can be queried
    // //  using getifaddrs function.
    // int resolve_nic_name (ip_addr_t *ip_addr_, nic_: *const c_char)
    // {
    //     //  Get the addresses.
    //     ifaddrs *ifa = NULL;
    //     int rc = 0;
    //     const int max_attempts = 10;
    //     const int backoff_msec = 1;
    //     for (int i = 0; i < max_attempts; i+= 1) {
    //         rc = getifaddrs (&ifa);
    //         if (rc == 0 || (rc < 0 && errno != ECONNREFUSED))
    //             break;
    //         usleep ((backoff_msec << i) * 1000);
    //     }
    //
    //     if (rc != 0 && ((errno == EINVAL) || (errno == EOPNOTSUPP))) {
    //         // Windows Subsystem for Linux compatibility
    //       // errno = ENODEV;
    //         return -1;
    //     }
    //     errno_assert (rc == 0);
    //     zmq_assert (ifa != NULL);
    //
    //     //  Find the corresponding network interface.
    //     bool found = false;
    //     for (const ifaddrs *ifp = ifa; ifp != NULL; ifp = ifp->ifa_next) {
    //         if (ifp->ifa_addr == NULL)
    //             continue;
    //
    //         const int family = ifp->ifa_addr->sa_family;
    //         if (family == (_options.ipv6 () ? AF_INET6 : AF_INET)
    //             && !strcmp (nic_, ifp->ifa_name)) {
    //             memcpy (ip_addr_, ifp->ifa_addr,
    //                     (family == AF_INET) ? sizeof (struct sockaddr_in)
    //                                         : sizeof (struct sockaddr_in6));
    //             found = true;
    //             break;
    //         }
    //     }
    //
    //     //  Clean-up;
    //     freeifaddrs (ifa);
    //
    //     if (!found) {
    //       // errno = ENODEV;
    //         return -1;
    //     }
    //     return 0;
    // }

    // #elif (defined ZMQ_HAVE_WINDOWS)
    //
    // // #include <netioapi.h>
    //
    // int get_interface_name (unsigned long index_,
    //                                             char **dest_) const
    // {
    // // #ifdef ZMQ_HAVE_WINDOWS_UWP
    //     char *buffer =  malloc (1024);
    // // #else
    //     char *buffer =  (malloc (IF_MAX_STRING_SIZE));
    // // #endif
    //     alloc_assert (buffer);
    //
    //     char *if_name_result = NULL;
    //
    // #if _WIN32_WINNT > _WIN32_WINNT_WINXP && !defined ZMQ_HAVE_WINDOWS_UWP
    //     if_name_result = if_indextoname (index_, buffer);
    // // #endif
    //
    //     if (if_name_result == NULL) {
    //         free (buffer);
    //         return -1;
    //     }
    //
    //     *dest_ = buffer;
    //     return 0;
    // }

    // int wchar_to_utf8 (const WCHAR *src_, char **dest_) const
    // {
    //     rc: i32;
    //     const int buffer_len =
    //       WideCharToMultiByte (CP_UTF8, 0, src_, -1, NULL, 0, NULL, 0);
    //
    //     char *buffer =  (malloc (buffer_len));
    //     alloc_assert (buffer);
    //
    //     rc =
    //       WideCharToMultiByte (CP_UTF8, 0, src_, -1, buffer, buffer_len, NULL, 0);
    //
    //     if (rc == 0) {
    //         free (buffer);
    //         return -1;
    //     }
    //
    //     *dest_ = buffer;
    //     return 0;
    // }

    #[cfg(target_os = "windows")]
    pub fn resolve_nic_name(&mut self, nic_name: &str) -> anyhow::Result<Vec<IpAddr>> {
        let max_attempts: i32 = 10;
        let mut iterations = 0;
        let mut addresses: *mut IP_ADAPTER_ADDRESSES_LH = null_mut();
        let mut out_buf_len = mem::size_of::<IP_ADAPTER_ADDRESSES_LH>() as u32;
        let mut dw_result: u32 = 0;

        let mut out: Vec<IpAddr> = vec![];

        loop {
            addresses = unsafe { malloc(out_buf_len as usize) as *mut IP_ADAPTER_ADDRESSES_LH };
            if addresses == null_mut() {
                return Err(anyhow!("Out of memory"));
            }

            unsafe {
                dw_result = GetAdaptersAddresses(
                    AF_UNSPEC as u32,
                    GAA_FLAG_SKIP_ANYCAST | GAA_FLAG_SKIP_MULTICAST | GAA_FLAG_SKIP_DNS_SERVER,
                    None,
                    Some(addresses),
                    &mut out_buf_len,
                );
            }
            if (dw_result == ERROR_BUFFER_OVERFLOW as u32) {
                unsafe { free(addresses as *mut c_void) };
                addresses = null_mut();
            } else {
                break;
            }
            iterations += 1;
            if !((dw_result == ERROR_BUFFER_OVERFLOW as u32) && (iterations < max_attempts)) {
                break;
            }
        }

        if (dw_result == 0) {
            // for (const IP_ADAPTER_ADDRESSES *current_addresses = addresses;
            //      current_addresses; current_addresses = current_addresses.Next)
            let mut current_address = addresses;
            while current_address != null_mut() {
                let mut if_name_raw = unsafe { CString::from_raw(current_address.AdapterName as *mut c_char) };
                let mut if_name = if_name_raw.into_string()?;

                let mut if_friendly_name_raw = unsafe { CString::from_raw(current_address.FriendlyName as *mut c_char) };
                let mut if_friendly_name = if_friendly_name_raw.into_string()?;

                if nic_name == if_name || nic_name == if_friendly_name {
                    let mut current_unicast_address = unsafe { (*current_address).FirstUnicastAddress };
                    while current_unicast_address != null_mut() {
                        let family = current_unicast_address.Address.lpSockaddr.sa_family as i32;
                        let mut sa: SocketAddr;
                        if family == AF_INET {
                            let sai4 = unsafe { (current_unicast_address.Address.lpSockaddr as *mut SOCKADDR_IN) };
                            let ip4_addr = unsafe {
                                Ipv4Addr::new(
                                    sai4.sin_addr.S_un.S_un_b.s_b1,
                                    sai4.sin_addr.S_un.S_un_b.s_b2,
                                    sai4.sin_addr.S_un.S_un_b.s_b3,
                                    sai4.sin_addr.S_un.S_un_b.s_b4,
                                )
                            };
                            let ip_addr = IpAddr::V4(ip4_addr);
                            out.push(ip_addr)
                        } else if family == AF_INET6 {
                            let sai6 = unsafe { (current_unicast_address.Address.lpSockaddr as *mut SOCKADDR_IN6) };
                            let ip6_addr = unsafe {
                                Ipv6Addr::new(
                                    sai6.sin6_addr.u.Word(0),
                                    sai6.sin6_addr.u.Word(1),
                                    sai6.sin6_addr.u.Word(2),
                                    sai6.sin6_addr.u.Word(3),
                                    sai6.sin6_addr.u.Word(4),
                                    sai6.sin6_addr.u.Word(5),
                                    sai6.sin6_addr.u.Word(6),
                                    sai6.sin6_addr.u.Word(7),
                                )
                            };
                            let ip_addr = IpAddr::V6(ip6_addr);
                            out.push(ip_addr);
                        } else {
                            // bail!("unsupported address family");
                        }
                        current_unicast_address = unsafe { (*current_unicast_address).Next };
                    }
                    break;
                }
                current_address = unsafe { (*current_address).Next };
            }


            unsafe { free(addresses as *mut c_void) };
        }

        Ok(out)
    }

    // #else

    //  On other platforms we assume there are no sane interface names.
    #[cfg(not(target_os = "windows"))]
    pub fn resolve_nic_name(ip_addr_: IpAddr, nic_: &str) -> i32 {
        // LIBZMQ_UNUSED (ip_addr_);
        // LIBZMQ_UNUSED (nic_);
        // errno = ENODEV;
        return -1;
    }

    // #endif

    pub fn do_getaddrinfo(
        node: &str,
        service_: &str,
        hints_: Option<&mut ZmqAddrInfo>,
    ) -> anyhow::Result<Vec<ZmqAddrInfo>> {
        // return unsafe { getaddrinfo(node_, service_, hints_, res_) };
        #[cfg(target_os = "windows")]
        do_getaddrinfo_win(node, service_, hints_)?
    }
}

#[cfg(target_os="windows")]
fn do_getaddrinfo_win(node: &str, service_: &str, hints_: Option<&mut ZmqAddrInfo>) -> Result<Vec<ZmqAddrInfo>, anyhow::Error> {
    let mut addrinfo_result: *mut *mut ADDRINFOA = null_mut();
    let res = unsafe{getaddrinfo(Some(node),
                                 Some(service_),
    hints_ as Option<*const ADDRINFOA>,
    addrinfo_result as *mut *mut ADDRINFOA)};
    if res != 0 {
        return Err(anyhow!("getaddrinfo failed"));
    }
    let mut out: Vec<ZmqAddrInfo> = Vec::new();

    let mut addrinfo = unsafe { *addrinfo_result };
    while addrinfo != null_mut() {

        let cn_raw = unsafe { CString::from_raw(*addrinfo.ai_canonname as *mut c_char) };
        let cn = cn_raw.into_string()?;

        let mut sa = SOCKADDR::default();
        sa.sa_family = addrinfo.ai_addr.sa_family;
        for i in 0 .. 14 {
            sa.sa_data[i] = addrinfo.ai_addr.sa_data[i];
        }

        let mut ai = ZmqAddrInfo {
            ai_flags: addrinfo.ai_flags,
            ai_family: addrinfo.ai_family,
            ai_socktype: addrinfo.ai_socktype,
            ai_protocol: addrinfo.ai_protocol,
            ai_addrlen: addrinfo.ai_addrlen,
            ai_canonname:cn,
            ai_addr: sa as sockaddr,
        };
        out.push(ai);


        addrinfo = unsafe { (*addrinfo).ai_next };
    }

    unsafe { freeaddrinfo(Some(addrinfo_result as *const ADDRINFOA)) };

    Ok(out)
}

#[cfg(target_os="windows")]
fn resolve_win(addr_: &str) -> Result<Vec<IpAddr>, anyhow::Error> {
    let mut out: Vec<IpAddr> = Vec::new();
        
    let addr_info: *mut *mut ADDRINFOA = null_mut();
    let iresult = unsafe { getaddrinfo(addr_, null_mut(), None, addr_info) };
    if iresult == 0 {
        let mut curr_addr_info = unsafe { *addr_info };
        while curr_addr_info != null_mut() {
            if curr_addr_info.ai_family == AF_INET {
                let sai4 = unsafe { (*curr_addr_info).ai_addr as *mut SOCKADDR_IN };
                let ip4_addr = unsafe {
                    Ipv4Addr::new(
                        sai4.sin_addr.S_un.S_un_b.s_b1,
                        sai4.sin_addr.S_un.S_un_b.s_b2,
                        sai4.sin_addr.S_un.S_un_b.s_b3,
                        sai4.sin_addr.S_un.S_un_b.s_b4,
                    )
                };
                out.push(IpAddr::V4(ip4_addr));
            } else if curr_addr_info.ai_family == AF_INET6 {
                let sai6 = unsafe { (*curr_addr_info).ai_addr as *mut SOCKADDR_IN6 };
                let ip6_addr = unsafe {
                    Ipv6Addr::new(
                        sai6.sin6_addr.u.Word(0),
                        sai6.sin6_addr.u.Word(1),
                        sai6.sin6_addr.u.Word(2),
                        sai6.sin6_addr.u.Word(3),
                        sai6.sin6_addr.u.Word(4),
                        sai6.sin6_addr.u.Word(5),
                        sai6.sin6_addr.u.Word(6),
                        sai6.sin6_addr.u.Word(7),
                    )
                };
                out.push(IpAddr::V6(ip6_addr));
            }
            curr_addr_info = unsafe { (*curr_addr_info).ai_next };
        }
    }
    Ok(out)
}

//  Construct an "ANY" address for the given family
// ip_addr_t ip_addr_t::any (family_: i32)
// {
//     ip_addr_t addr;
//
//     if (family_ == AF_INET) {
//         sockaddr_in *ip4_addr = &addr.ipv4;
//         memset (ip4_addr, 0, sizeof (*ip4_addr));
//         ip4_addr->sin_family = AF_INET;
//         ip4_addr->sin_addr.s_addr = htonl (INADDR_ANY);
//     } else if (family_ == AF_INET6) {
//         sockaddr_in6 *ip6_addr = &addr.ipv6;
//
//         memset (ip6_addr, 0, sizeof (*ip6_addr));
//         ip6_addr->sin6_family = AF_INET6;
// // #ifdef ZMQ_HAVE_VXWORKS
//         struct in6_addr newaddr = IN6ADDR_ANY_INIT;
//         memcpy (&ip6_addr->sin6_addr, &newaddr, mem::size_of::<in6_addr>());
// // #else
//         memcpy (&ip6_addr->sin6_addr, &in6addr_any, mem::size_of::<in6addr_any>());
// // #endif
//     } else {
//         assert (0 == "unsupported address family");
//     }
//
//     return addr;
// }

pub fn do_if_nametoindex(ifname_: &str) -> u32 {
    let arg_str = CString::new(ifname_).unwrap();
    let mut out = 0u32;
    unsafe {
        out = if_nametoindex(arg_str.as_ptr());
    };

    out
}
