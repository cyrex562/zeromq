use std::ffi::CString;
use std::fmt::{Debug, Display};
use std::ptr::null_mut;

use libc::{ECONNREFUSED, EINVAL, EOPNOTSUPP};
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::ERROR_BUFFER_OVERFLOW;
#[cfg(target_os = "windows")]
use windows::Win32::NetworkManagement::IpHelper::{GAA_FLAG_SKIP_ANYCAST, GAA_FLAG_SKIP_DNS_SERVER, GAA_FLAG_SKIP_MULTICAST, GetAdaptersAddresses, IP_ADAPTER_ADDRESSES_LH, IP_ADAPTER_UNICAST_ADDRESS_LH};

use crate::address::ip_address::ZmqIpAddress;
use crate::defines::{AF_INET, AF_INET6, AI_NUMERICHOST, AI_PASSIVE, SOCK_STREAM, ZmqAddrInfo, ZmqSockAddr, ZmqSockAddrIn, ZmqSockAddrIn6};
#[cfg(target_os = "windows")]
use crate::defines::AF_UNSPEC;
use crate::defines::err::ZmqError;
use crate::defines::err::ZmqError::AddressError;
use crate::ip::ip_resolver_options::IpResolverOptions;
use crate::options::ZmqOptions;
use crate::utils::get_errno;
#[cfg(target_os = "windows")]
use crate::utils::sock_utils::sockaddr_to_zmq_sockaddr;

pub fn sockaddr_to_str(sa: &ZmqSockAddr) -> String {
    let mut out = String::new();
    out += format!("sockaddr {{ sa_family: {}", sa.sa_family).as_str();
    out += ", sa_data: [";
    for i in 0..14 {
        out += format!(" {:02x}", sa.sa_data[i]).as_str();
    }
    out += " ] }}";
    out
}

pub fn sockaddr_in_to_str(sa: &ZmqSockAddrIn) -> String {
    let mut out = String::new();
    out += format!("sockaddr_in {{ sin_family: {}", sa.sin_family).as_str();
    out += format!(", sin_port: {}", sa.sin_port).as_str();
    out += format!(", sin_addr: {} }}", sa.sin_addr).as_str();
    out
}

pub fn sockaddr_in6_to_str(sa: &ZmqSockAddrIn6) -> String {
    let mut out = String::new();
    out += format!("sockaddr_in6 {{ sin6_family: {}", sa.sin6_family).as_str();
    out += format!(", sin6_port: {}", sa.sin6_port).as_str();
    out += format!(", sin6_flowinfo: {}", sa.sin6_flowinfo).as_str();
    out += ", sin6_addr: [";
    for i in 0..16 {
        out += format!(" {:02x}", sa.sin6_addr[i]).as_str();
    }
    out += " ]";
    out += format!(", sin6_scope_id: {} }}", sa.sin6_scope_id).as_str();
    out
}


//noinspection RsFunctionNaming
pub fn IN_MULTICAST(a: u32) -> bool {
    (a & 0xf0000000) == 0xe0000000
}

pub fn in6_is_addr_multicast(a: &[u8]) -> bool {
    a[0] == 0xff
}

#[derive(Default, Debug, Clone)]
pub struct IpResolver {
    pub _options: IpResolverOptions,
}

impl IpResolver {
    pub fn new(opts_: &mut IpResolverOptions) -> Self {
        Self {
            _options: opts_.clone(),
        }
    }

    pub fn do_getaddrinfo(&mut self, node: &str, service: &str, hints: &ZmqAddrInfo, res: &mut Vec<ZmqAddrInfo>) -> Result<(), ZmqError> {
        // TODO: call platform-specific getaddrinfo
        // getaddrinfo(node.as_mut_ptr() as *mut c_char, service.as_mut_ptr() as *mut c_char, hints, res);\
        todo!();
    }

    pub fn do_freeaddrinfo(&mut self, res: &mut ZmqAddrInfo) -> Result<(), ZmqError> {
        // TODO call platform specific freeaddrinfo
        // freeaddrinfo(res);
        todo!();
    }

    pub fn do_if_nametoindex(ifname_: &str) -> Result<u32, ZmqError> {
        // TODO: call platform-specific if_nametoindex

        let mut idx = 0;
        #[cfg(not(target_os = "windows"))]
        {
            idx = unsafe { libc::if_nametoindex(ifname_.as_ptr() as *const libc::c_char) };
        }

        return Ok(idx);
    }

    pub fn resolve(&mut self, options: &ZmqOptions, ip_addr_: &mut ZmqIpAddress, name: &str) -> Result<(), ZmqError> {
        let mut addr: String = String::new();
        let mut port = 0u16;

        if self._options.get_expect_port() {
            if name.contains(":") == false {
                // bail!("invalid address: {}", name);
                return Err(AddressError("invalid address"));
            }

            let x = name.split(":").collect::<Vec<&str>>();
            let addr_str = x[0].to_string();
            let port_str = x[1].to_string();
            if port_str == "*" {
                if self._options.get_bindable() == true {
                    port = 0;
                } else {
                    // bail!("not bindable");
                    return Err(AddressError("invalid address"));
                }
            } else if port_str == "0" {
                port = 0;
            } else {
                port = port_str.parse::<u16>()?;
            }
        } else {
            addr = name.to_string();
            port = 0;
        }

        if self._options.get_allow_path() {
            let pos = addr.find("/");
            match pos {
                Some(sz) => {
                    addr = addr[0..sz].to_string();
                }
                None => {}
            }
        }

        let brackets_length = 2;
        if (addr.len() >= brackets_length) && addr.starts_with("[") && addr.ends_with("]") {
            addr = addr[1..addr.len() - 2].to_string();
        }

        let pos = addr.rfind("%");
        let mut zone_id = 0u32;
        if pos.is_some() {
            let if_str = addr[pos.unwrap()..].to_string();
            addr = addr[0..pos.unwrap()].to_string();
            let chars = if_str.chars().collect::<Vec<char>>();
            let x = &chars[0];
            if x.is_alphanumeric() {
                zone_id = Self::do_if_nametoindex(&if_str)?;
            } else {
                zone_id = if_str.parse::<u32>()?;
            }

            if zone_id == 0 {
                // bail!("invalid zone id");
                return Err(AddressError("invalid zone id"));
            }
        }

        let mut resolved = false;
        let mut addr_str = addr.clone();
        if self._options.get_bindable() && addr == "*" {
            *ip_addr_ = ZmqIpAddress::any(if self._options.get_ipv6() {
                AF_INET6
            } else {
                AF_INET
            });
            resolved = true;
        }

        if !resolved && self._options.get_allow_nic_name() {
            unsafe { self.resolve_nic_name(ip_addr_, &mut addr_str)?; }
            resolved = true;
        }

        if !resolved {
            self.resolve_getaddrinfo(ip_addr_, &mut addr_str)?;
            resolved = true;
        }

        ip_addr_.set_port(port);
        if ip_addr_.family() == AF_INET6 {
            ip_addr_.scope_id = zone_id;
        }

        Ok(())
    }

    pub fn resolve_getaddrinfo(
        &mut self,
        ip_addr_: &mut ZmqIpAddress,
        addr_: &str,
    ) -> Result<(), ZmqError> {
        // let mut res: ZmqAddrInfo = ZmqAddrInfo {
        //     ai_flags: 0,
        //     ai_family: 0,
        //     ai_socktype: 0,
        //     ai_protocol: 0,
        //     ai_addrlen: 0,
        //     ai_addr: null_mut(),
        //     ai_canonname: null_mut(),
        //     ai_next: null_mut(),
        // };
        let mut res: Vec<ZmqAddrInfo> = vec![];

        let mut req: ZmqAddrInfo = ZmqAddrInfo {
            ai_flags: 0,
            ai_family: 0,
            ai_socktype: 0,
            ai_protocol: 0,
            ai_addrlen: 0,
            ai_addr: null_mut(),
            ai_canonname: null_mut(),
            ai_next: null_mut(),
        };

        req.ai_family = if self._options.get_ipv6() {
            AF_INET6
        } else {
            AF_INET
        };
        req.ai_socktype = SOCK_STREAM;
        req.ai_flags = 0;
        if self._options.get_bindable() {
            req.ai_flags |= AI_PASSIVE;
        }

        if self._options.get_allow_dns() {
            req.ai_flags |= AI_NUMERICHOST;
        }

        #[cfg(feature = "ai_v4mapped")]
        if req.ai_family == AF_INET6 {
            req.ai_flags |= AI_V4MAPPED;
        }

        let mut call_res = self.do_getaddrinfo(addr_, "", &req, &mut res);
        #[cfg(feature = "ai_v4mapped")]
        if (rc == EAI_BADFLAGS && (req.ai_flags & AI_V4MAPPED)) {
            req.ai_flags &= !AI_V4MAPPED;
            rc = do_getaddrinfo(addr_, NULL, &req, &res);
        }
        #[cfg(target_os = "windows")]
        // && (rc == WSAHOST_NOT_FOUND)
        if (req.ai_family == AF_INET6) && (call_res.is_err()) {
            req.ai_family = AF_INET;
            call_res = self.do_getaddrinfo(addr_, "", &req, &mut res);
        }

        // todo
        // if rc != 0 {
        //     if rc == EAI_MEMORY {
        //         bail!("out of memory");
        //     } else if rc != 0 && self._options.get_bindable() {
        //         bail!("enodev");
        //     } else {
        //         bail!("einval")
        //     }
        // }

        // copy_bytes(res.ai_addr, 0, res.ai_addrlen, ip_addr_, 0, ip_addr_.sockaddr_len())?;
        unsafe {
            // ip_addr_.generic = ZmqSockAddr {
            //     sa_family: (*res[0].ai_addr).sa_family,
            //     sa_data: [0; 14],
            // };
            ip_addr_.address_family = (*res[0].ai_addr).sa_family as i32;
        }
        unsafe { ip_addr_.addr_bytes.clone_from_slice(&(*res[0].ai_addr).sa_data[0..14]); }

        self.do_freeaddrinfo(&mut res[0])?;

        Ok(())
    }

    #[cfg(target_os = "linux")]
    pub fn resolve_nic_name(&mut self, ip_addr_: &mut ZmqIpAddress, nic_: &str) -> Result<(), ZmqError> {
        let mut ifa: *mut libc::ifaddrs = null_mut();
        let mut rc = 0i32;
        let max_attempts = 10;
        let backoff_msec = 1;
        for i in 0..max_attempts {
            rc = unsafe { libc::getifaddrs(&mut ifa) };
            let errno = get_errno();
            if rc == 0 || rc < 0 && errno == ECONNREFUSED {
                break;
            }
            unsafe { libc::sleep(std::time::Duration::from_millis(backoff_msec.clone()).as_secs() as libc::c_uint) };
        }

        let errno = get_errno();
        if rc != 0 && errno == EINVAL || errno == EOPNOTSUPP {
            // bail!("enodev");
            return Err(AddressError("enodev"));
        }

        let mut found = false;
        let mut ifp = ifa;
        unsafe {
            while ifp != null_mut() {
                if (*ifp).ifa_addr == null_mut() {
                    continue;
                }

                let family = (*ifp).ifa_addr.as_mut().unwrap().sa_family.clone() as i32;
                let if_nic_name = CString::from_raw((*ifp).ifa_name).into_string().unwrap();
                if family == (if self._options.get_ipv6() { AF_INET6 } else { AF_INET }) && nic_ == if_nic_name {
                    let match_sockaddr = (*ifp).ifa_addr.as_mut().unwrap();
                    // (*ip_addr_).generic = sockaddr_to_zmq_sockaddr(match_sockaddr);
                    (*ip_addr_).address_family = (*match_sockaddr).sa_family.clone() as i32;

                    let mut sa_data_bytes = [0u8; 14];
                    for i in 0..14 {
                        sa_data_bytes[i] = (*match_sockaddr).sa_data[i] as u8;
                    }
                    (*ip_addr_).addr_bytes.clone_from_slice(&sa_data_bytes);

                    found = true;
                    break;
                }
                ifp = (*ifp).ifa_next;
            }
        }

        unsafe { libc::freeifaddrs(ifa) };

        if found == false {
            // bail!("enodev");
            return Err(AddressError("enodev"));
        }

        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub fn get_interface_name(&mut self, index_: u32, dest: &mut String) -> Result<(), ZmqError> {
        let result = self.if_indextoname(index_, dest.as_mut_ptr());
    }

    #[cfg(target_os = "windows")]
    pub fn resolve_nic_name(&mut self, options: &ZmqOptions, ip_addr_: &mut ZmqIpAddress, nic_: &mut String) -> Result<(), ZmqError> {
        let mut rc = 0i32;
        let mut found = false;
        let max_attempts = 10;
        let mut iterations = 0;
        let addresses: *mut IP_ADAPTER_ADDRESSES_LH = null_mut();
        let mut out_buf_len = 0u32;
        while rc == ERROR_BUFFER_OVERFLOW && iterations < max_attempts {
            rc = unsafe {
                GetAdaptersAddresses(
                    AF_UNSPEC as u32,
                    GAA_FLAG_SKIP_ANYCAST | GAA_FLAG_SKIP_MULTICAST | GAA_FLAG_SKIP_DNS_SERVER,
                    None,
                    Some(addresses),
                    &mut out_buf_len,
                ) as i32
            };
            iterations += 1;
        }

        let mut current_addresses = addresses;
        while current_addresses != null_mut() {
            let mut if_name = String::new();
            let mut if_friendly_name = String::new();
            let mut str_rc1 = self.get_interface_name((*current_addresses).Ipv6IfIndex, &mut if_name);
            let mut str_rc2 = (*current_addresses).FriendlyName;
            current_addresses = current_addresses.as_mut().unwrap().Next;
            if (str_rc1 == 0 && nic_ == if_name) || (str_rc2.0 == null_mut() && nic_ == if_friendly_name) {
                let mut if_addr: *mut IP_ADAPTER_UNICAST_ADDRESS_LH = null_mut();
                if_addr = current_addresses.as_mut().unwrap().FirstUnicastAddress;
                while if_addr != null_mut() {
                    let family = if_addr.as_mut().unwrap().Address.lpSockaddr.as_mut().unwrap().sa_family.clone().0;
                    if family == (if options.get_ipv6() { AF_INET6 } else { AF_INET }) as u16 {
                        // let match_sockaddr = if_addr.as_mut().unwrap().Address.lpSockaddr.as_mut().unwrap();
                        let msa = (*if_addr).Address.lpSockaddr;
                        let mut match_sockaddr: ZmqSockAddr = ZmqSockAddr {
                            sa_family: (*msa).sa_family.0,
                            sa_data: [0; 14],
                        };
                        for i in 0..14 {
                            match_sockaddr.sa_data[i] = (*msa).sa_data[i];
                        }
                        (*ip_addr_).generic = *match_sockaddr;
                        found = true;
                        break;
                    }
                    if_addr = if_addr.as_mut().unwrap().Next;
                }
            }
        }
    }
}
