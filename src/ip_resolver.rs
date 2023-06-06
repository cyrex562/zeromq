use std::ffi::{CStr, CString};
use std::mem;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use std::ptr::null_mut;
use std::str::FromStr;
use libc::{AI_NUMERICHOST, AI_PASSIVE, AI_V4MAPPED, c_uint, close, EAI_BADFLAGS, EINVAL, ENODEV, ENOMEM, free, getaddrinfo, if_nametoindex, malloc, SOCK_STREAM, strcmp};
#[cfg(target_os = "windows")]
use windows::Win32::Networking::WinSock::{AI_NUMERICHOST, AI_PASSIVE, AI_V4MAPPED, freeaddrinfo, SOCK_DGRAM, SOCK_STREAM, WSAHOST_NOT_FOUND};
use windows::Win32::Networking::WinSock::ADDRINFOA;
use crate::address_family::{AF_INET, AF_INET6};
use crate::network_address::{NetworkAddress, NetworkAddressFamily};
use crate::unix_sockaddr::{addrinfo, sockaddr_in};

pub struct IpResolverOptions
{
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
    pub allow_path: bool
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

pub struct IpResolver
{
    // IpResolverOptions _options;
    pub options: IpResolverOptions,
}

impl IpResolver {
    //
    // ip_resolver_t (IpResolverOptions opts_);

    // virtual ~ip_resolver_t (){};

    // int resolve (ip_addr_t *ip_addr_, name: *const c_char);

  //
    //  Virtual functions that are overridden in tests
    // virtual int do_getaddrinfo (node_: *const c_char,
    //                             service_: *const c_char,
    //                             const struct addrinfo *hints_,
    //                             struct addrinfo **res_);

    // virtual void do_freeaddrinfo (struct addrinfo *res_);

    // virtual unsigned int do_if_nametoindex (ifname_: *const c_char);

  //
    // int resolve_nic_name (ip_addr_t *ip_addr_, nic_: *const c_char);

    // int resolve_getaddrinfo (ip_addr_t *ip_addr_, addr_: *const c_char);

// #if defined ZMQ_HAVE_WINDOWS
//     int get_interface_name (unsigned long index_, char **dest_) const;

//     int wchar_to_utf8 (const WCHAR *src_, char **dest_) const;

// #endif


    // IpResolver (IpResolverOptions opts_) :
    //     _options (opts_)
    // {
    // }
    pub fn new(opts: &IpResolverOptions) -> Self {
        Self {
            options: (*opts).clone(),
        }
    }

    pub fn resolve (&mut self, in_addr: &mut NetworkAddress, name: &str) -> i32
    {
        // let mut addr = String::new();
        let mut addr_str: &str = "";
        let mut port = 0u16;

        if self.options.expect_port {
            //  We expect 'addr:port'. It's important to use str*r*chr to only get
            //  the latest colon since IPv6 addresses use colons as delemiters.
            // const char *delim = strrchr (name, ':');
            let delim = name.find(':');

            if delim.is_none() {
                errno = EINVAL;
                return -1;
            }

            // addr = std::string (name, delim - name);
            let addr = &name[..delim.unwrap()];
            let port_str = &name[delim.unwrap() + 1..];

            if port_str == "*" {
                if self._options.bindable () {
                    //  Resolve wildcard to 0 to allow autoselection of port
                    port = 0;
                } else {
                    errno = EINVAL;
                    return -1;
                }
            } else if port_str == "0" {
                //  Using "0" for a bind address is equivalent to using "*". For a
                //  connectable address it could be used to connect to port 0.
                port = 0;
            } else {
                //  Parse the port number (0 is not a valid port).
                // port = static_cast<uint16_t> (atoi (port_str.c_str ()));
                port = u16::from_str(port_str).unwrap();
                if port == 0 {
                    errno = EINVAL;
                    return -1;
                }
            }
        } else {
            addr_str = name;
            port = 0;
        }

        // Check if path is allowed in ip address, if allowed it must be truncated
        if self.options.allow_path {
            let pos = addr_str.find ('/');
            if pos.is_some() {
                addr_str = &addr_str[..pos.unwrap()];
            }
        }

        //  Trim any square brackets surrounding the address. Used for
        //  IPv6 addresses to remove the confusion with the port
        //  delimiter.
        //  TODO Should we validate that the brackets are present if
        //  'addr' contains ':' ?
        let mut brackets_length: usize = 2;
        if addr_str.len() >= brackets_length && addr_str[0] == '['
            && addr_str[addr_str.len() - 1] == ']' {
            addr_str = &addr_str[1..addr_str.len()-brackets_length];
        }

        //  Look for an interface name / zone_id in the address
        //  Reference: https://tools.ietf.org/html/rfc4007
        let mut pos = addr_str.rfind ('%');
        let mut zone_id = 0u32;

        if pos.is_some() {
            let mut if_str = &addr_str[pos.unwrap() + 1..];
            if if_str.is_empty() {
                errno = EINVAL;
                return -1;
            }
            addr_str = &addr_str[0..pos.unwrap()];
            if if_str[0].is_alphabetic {
                zone_id = do_if_nametoindex (if_str);
            } else {
                zone_id = u32::from(if_str);
            }

            if zone_id == 0 {
                errno = EINVAL;
                return -1;
            }
        }

        let mut resolved = false;
        let mut addr_str = addr_str;

        if self.options.bindable && addr_str == "*" {
            //  Return an ANY address
            // *ip_addr = ip_addr_t::any (_options.ipv6 () ? AF_INET6 : AF_INET);

            if self.options.ipv6 {
                *in_addr = NetworkAddress::from(Ipv6Addr::UNSPECIFIED)
            } else {
                *in_addr = NetworkAddress::from(Ipv4Addr::UNSPECIFIED);
            }
        }

        if !resolved && self.options.allow_nic_name () {
            //  Try to resolve the string as a NIC name.
            let rc = resolve_nic_name (in_addr, addr_str);
            if rc == 0 {
                resolved = true;
            } else if errno != ENODEV {
                return rc;
            }
        }

        if !resolved {
            let rc = resolve_getaddrinfo (in_addr, addr_str);

            if rc != 0 {
                return rc;
            }
            resolved = true;
        }

        //  Store the port into the structure. We could get 'getaddrinfo' to do it
        //  for us but since we don't resolve service names it's a bit overkill and
        //  we'd still have to do it manually when the address is resolved by
        //  'resolve_nic_name'
        // in_addr. ->set_port (port);
        in_addr.port = port;

        // if (in_addr ->family () == AF_INET6)
        if in_addr.family() == NetworkAddressFamily::Inet6
        {
        // in_addr ->ipv6.sin6_scope_id = zone_id;
        in_addr.scope_id = zone_id;

        }

        // assert (resolved == true);
        return 0;
    }

    pub fn resolve_getaddrinfo (&mut self, ip_addr: &mut NetworkAddress,
                                                 addr_: &str) -> i32
    {
    // #if defined ZMQ_HAVE_OPENVMS && defined __ia64
    //     __addrinfo64 *res = NULL;
    //     __addrinfo64 req;
    // #else
    //     addrinfo *res = NULL;
    //     addrinfo req;
    // #endif

        // memset (&req, 0, mem::size_of::<req>());

        //  Choose IPv4 or IPv6 protocol family. Note that IPv6 allows for
        //  IPv4-in-IPv6 addresses.
        let mut req = addrinfo::default();
        req.ai_family = if self.options.ipv6 { AF_INET6 } else { AF_INET } as i32;

        //  Arbitrary, not used in the output, but avoids duplicate results.
        req.ai_socktype = SOCK_STREAM;

        req.ai_flags = 0;

        if (self._options.bindable ()) {
            req.ai_flags |= AI_PASSIVE;
        }

        if !_options.allow_dns () {
            req.ai_flags |= AI_NUMERICHOST;
        }

    // #if defined AI_V4MAPPED
        //  In this API we only require IPv4-mapped addresses when
        //  no native IPv6 interfaces are available (~AI_ALL).
        //  This saves an additional DNS roundtrip for IPv4 addresses.
        if req.ai_family == AF_INET6 {
            req.ai_flags |= AI_V4MAPPED;
        }
    // #endif

        //  Resolve the literal address. Some of the error info is lost in case
        //  of error, however, there's no way to report EAI errors via errno.
        // do_getaddrinfo (addr_, null_mut(), &req, &res);
        do_getaddrinfo (addr_, "");

        // Some OS do have AI_V4MAPPED defined but it is not supported in getaddrinfo()
        // returning EAI_BADFLAGS. Detect this and retry
        // if rc == EAI_BADFLAGS && (req.ai_flags & AI_V4MAPPED) != 0 {
        //     req.ai_flags &= !AI_V4MAPPED;
        //     rc = do_getaddrinfo (addr_, null_mut(), &req, &res);
        // }

        //  Resolve specific case on Windows platform when using IPv4 address
        //  with ZMQ_IPv6 socket option.
        #[cfg(target_os = "windows")]
        if (req.ai_family == AF_INET6) && (rc == WSAHOST_NOT_FOUND) {
            req.ai_family = AF_INET as i32;
            rc = do_getaddrinfo (addr_, null_mut(), &req, &res);
        }

        if rc {
            match rc {
                EAI_MEMORY =>
                    errno = ENOMEM,
                _ => {
                    if self.options.bindable {
                        errno = ENODEV;
                    } else {
                        errno = EINVAL;
                    }
                }
            }
            return -1;
        }

        //  Use the first result.
        // zmq_assert (res != NULL);
        // zmq_assert (static_cast<size_t> (res->ai_addrlen) <= sizeof (*ip_addr_));
        // memcpy (ip_addr_, res->ai_addr, res->ai_addrlen);
        // TODO:

        //  Cleanup getaddrinfo after copying the possibly referenced result.
        // do_freeaddrinfo (res);

        return 0;
    }

    // #ifdef ZMQ_HAVE_SOLARIS
    // #include <sys/sockio.h>

    //  On Solaris platform, network interface name can be queried by ioctl.
    #[cfg(target_os = "solaris")]
    pub fn resolve_nic_name (ip_addr_: &mut NetworkAddress, nic_: &str) -> i32
    {
        //  Create a socket.
        let fd: i32 = open_socket (AF_INET, SOCK_DGRAM, 0);
        // errno_assert (fd != -1);

        //  Retrieve number of interfaces.
        let ifn: lifnum;
        ifn.lifn_family = AF_INET;
        ifn.lifn_flags = 0;
        let rc = ioctl (fd, SIOCGLIFNUM, &ifn);
        // errno_assert (rc != -1);

        //  Allocate memory to get interface names.
        let ifr_size =  mem::size_of::<lifreq>() * ifn.lifn_count;
        // char *ifr =  malloc (ifr_size);
        let ifr = String::with_capacity(ifr_size);
        // alloc_assert (ifr);

        //  Retrieve interface names.
        let ifc:lifconf;
        ifc.lifc_family = AF_INET;
        ifc.lifc_flags = 0;
        ifc.lifc_len = ifr_size;
        ifc.lifc_buf = ifr;
        rc = ioctl (fd, SIOCGLIFCONF,  &ifc);
        // errno_assert (rc != -1);

        //  Find the interface with the specified name and AF_INET family.
        let found: bool = false;
        lifreq *ifrp = ifc.lifc_req;
        // for (int n = 0; n <  (ifc.lifc_len / mem::size_of::<lifreq>()); n+= 1, ifrp+= 1)
        for n in 0..(ifc.lifc_len / mem::size_of::<lifreq>())
        {
            if (!strcmp (nic_, ifrp.lifr_name)) {
                rc = ioctl (fd, SIOCGLIFADDR,  ifrp);
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
        free (ifr);
        close (fd);

        if (!found) {
            errno = ENODEV;
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
    //         errno = ENODEV;
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
    //         errno = ENODEV;
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
    //         errno = ENODEV;
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
    //         errno = ENODEV;
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
    //         errno = ENODEV;
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
    pub fn resolve_nic_name (ip_addr_: &NetworkAddress, nic_: &str)
    {
        rc: i32;
        let mut found = false;
        let max_attempts: i32 = 10;

        let mut iterations = 0;
        IP_ADAPTER_ADDRESSES *addresses;
        let mut out_buf_len = mem::size_of::<IP_ADAPTER_ADDRESSES>();

        loop {
            addresses =  (malloc (out_buf_len));
            // alloc_assert (addresses);

            rc =
              GetAdaptersAddresses (AF_UNSPEC,
                                    GAA_FLAG_SKIP_ANYCAST | GAA_FLAG_SKIP_MULTICAST
                                      | GAA_FLAG_SKIP_DNS_SERVER,
                                    null_mut(), addresses, &out_buf_len);
            if (rc == ERROR_BUFFER_OVERFLOW) {
                free (addresses);
                addresses = null_mut();
            } else {
                break;
            }
            iterations+= 1;
            if  !((rc == ERROR_BUFFER_OVERFLOW) && (iterations < max_attempts)) {break;}
        }

        if (rc == 0) {
            // for (const IP_ADAPTER_ADDRESSES *current_addresses = addresses;
            //      current_addresses; current_addresses = current_addresses.Next)
            for address in addresses
            {
                char *if_name = null_mut();
                char *if_friendly_name = null_mut();

                let str_rc1: i32 =
                  get_interface_name (current_addresses.IfIndex, &if_name);
                let str_rc2: i32 = wchar_to_utf8 (current_addresses.FriendlyName,
                                                   &if_friendly_name);

                //  Find a network adapter by its "name" or "friendly name"
                if (((str_rc1 == 0) && (!strcmp (nic_, if_name)))
                    || ((str_rc2 == 0) && (!strcmp (nic_, if_friendly_name)))) {
                    //  Iterate over all unicast addresses bound to the current network interface
                    // for (const IP_ADAPTER_UNICAST_ADDRESS *current_unicast_address =
                    //        current_addresses.FirstUnicastAddress;
                    //      current_unicast_address;
                    //      current_unicast_address = current_unicast_address.Next)
                    for current_unicast_address in current_addresses.FirstUnicastAddress
                    {
                        let family =
                          current_unicast_address.Address.lpSockaddr.sa_family;

                        if (family == (if self._options.ipv () { AF_INET6 } else { AF_INET }))
                        {
                            memcpy (
                              ip_addr_, current_unicast_address.Address.lpSockaddr,
                              if (family == AF_INET) { mem::size_of::<sockaddr_in>() } else {
                                  mem::sizeof::<sockaddr_in6>()});

                            found = true;
                            break;
                        }
                    }

                    if (found) {
                        break;
                    }
                }

                if (str_rc1 == 0) {
                    free(if_name);
                }
                if (str_rc2 == 0) {
                    free(if_friendly_name);
                }
            }

            free (addresses);
        }

        if (!found) {
            errno = ENODEV;
            return -1;
        }
        return 0;
    }

    // #else

    //  On other platforms we assume there are no sane interface names.
    #[cfg(not(target_os = "windows"))]
    pub fn resolve_nic_name (ip_addr_: ip_addr_t, nic_: &str) -> i32
    {
        // LIBZMQ_UNUSED (ip_addr_);
        // LIBZMQ_UNUSED (nic_);

        errno = ENODEV;
        return -1;
    }

    // #endif

    pub fn do_getaddrinfo (node_: &str, service_: &str, hints_: &addrinfo, &mut &mut res_: addrinfo) -> i32
    {
        // return unsafe { getaddrinfo(node_, service_, hints_, res_) };
        0
    }

    pub fn do_freeaddrinfo (res_: &addrinfo)
    {
        // unsafe { freeaddrinfo(Some(res_ as *ADDRINFOA)) };
        unimplemented!("do_freeaddrinfo")
    }

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
    unsafe { out = if_nametoindex(arg_str.as_ptr()); };

    out
}
