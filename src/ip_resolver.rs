

use std::{io, os};
use std::ffi::{c_char, CString};
use std::fmt::{Debug, Display, Formatter};
use crate::utils::copy_bytes;
use anyhow::bail;
use libc::{addrinfo, getaddrinfo, in6_addr, sa_family_t, sockaddr, sockaddr_in, sockaddr_in6, AF_INET, AF_INET6, AI_NUMERICHOST, AI_PASSIVE, EAI_MEMORY, INADDR_ANY, SOCK_STREAM, ifaddrs, getifaddrs, ECONNREFUSED, EINVAL, EOPNOTSUPP, freeifaddrs, if_indextoname, freeaddrinfo, if_nametoindex};
use std::ptr::null_mut;
use std::thread::sleep;
use windows::Win32::Networking::WinSock::AF_INET6;
use windows::Win32::NetworkManagement::IpHelper::IP_ADAPTER_ADDRESSES_LH;


pub union ZmqIpAddress {
    pub generic: sockaddr,
    pub ipv4: sockaddr_in,
    pub ipv6: sockaddr_in6,
}

impl Clone for ZmqIpAddress {
    fn clone(&self) -> Self {
        let mut out = Self {
           generic: sockaddr{
               sa_family: self.generic.sa_family.clone(),
               sa_data: [0;14]
           }
        };

        unsafe {
            for i in 0..14 {
                out.generic.sa_data[i] = self.generic.sa_data[i.clone()].clone();
            }
        }

        unsafe {
            if out.generic.sa_family == AF_INET6 as u16 {
                out.ipv6.sin6_family = self.ipv6.sin6_family.clone();
                out.ipv6.sin6_port = self.ipv6.sin6_port.clone();
                out.ipv6.sin6_flowinfo = self.ipv6.sin6_flowinfo.clone();
                for i in 0..16 {
                    out.ipv6.sin6_addr.s6_addr[i] = self.ipv6.sin6_addr.s6_addr[i.clone()].clone();
                }
                out.ipv6.sin6_scope_id = self.ipv6.sin6_scope_id.clone();
            } else if out.generic.sa_family == AF_INET as u16 {
                out.ipv4.sin_family = self.ipv4.sin_family.clone();
                out.ipv4.sin_port = self.ipv4.sin_port.clone();
                out.ipv4.sin_addr.s_addr = self.ipv4.sin_addr.s_addr.clone();
            } else {}
        }

        out
    }
}

impl Debug for ZmqIpAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ip_addr_t {{ generic: {}, ipv4: {}, ipv6: {} }}", sockaddr_to_str(&self.generic), sockaddr_in_to_str(&self.ipv4), sockaddr_in6_to_str(&self.ipv6))
    }
}

pub fn sockaddr_to_str(sa: &sockaddr) -> String {
    let mut out = String::new();
    out += format!("sockaddr {{ sa_family: {}", sa.sa_family).as_str();
    out += ", sa_data: [";
    for i in 0 .. 14 {
        out += format!(" {:02x}", sa.sa_data[i]).as_str();
    }
    out += " ] }}";
    out
}

pub fn sockaddr_in_to_str(sa: &sockaddr_in) -> String {
    let mut out = String::new();
    out += format!("sockaddr_in {{ sin_family: {}", sa.sin_family).as_str();
    out += format!(", sin_port: {}", sa.sin_port).as_str();
    out += format!(", sin_addr: {} }}", sa.sin_addr.s_addr).as_str();
    out
}

pub fn sockaddr_in6_to_str(sa: &sockaddr_in6) -> String {
    let mut out = String::new();
    out += format!("sockaddr_in6 {{ sin6_family: {}", sa.sin6_family).as_str();
    out += format!(", sin6_port: {}", sa.sin6_port).as_str();
    out += format!(", sin6_flowinfo: {}", sa.sin6_flowinfo).as_str();
    out += ", sin6_addr: [";
    for i in 0 .. 16 {
        out += format!(" {:02x}", sa.sin6_addr.s6_addr[i]).as_str();
    }
    out += " ]";
    out += format!(", sin6_scope_id: {} }}", sa.sin6_scope_id).as_str();
    out
}


impl ZmqIpAddress {
    pub fn set_port(&mut self, port_: u16) {
        if self.family() == AF_INET6 {
            self.ipv6.sin6_port = port_.to_be();
        } else {
            self.ipv4.sin_port = port_.to_be();
        }
    }
}

impl Display for ZmqIpAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ip_addr_t {{ generic: {}, ipv4: {}, ipv6: {} }}", sockaddr_to_str(&self.generic), sockaddr_in_to_str(&self.ipv4), sockaddr_in6_to_str(&self.ipv6))
    }
}

pub const in6addr_any: in6_addr = in6_addr { s6_addr: [0; 16] };

impl Default for ZmqIpAddress {
    fn default() -> Self {
        let mut out = Self {
            generic: sockaddr {
                sa_family: 0,
                sa_data: [0; 14],
            },
        };

        unsafe {
            out.ipv6.sin6_family = 0;
            out.ipv6.sin6_port = 0;
            out.ipv6.sin6_flowinfo = 0;
            out.ipv6.sin6_addr.s6_addr = [0; 16];
        }

        out
    }
}

pub fn IN_MULTICAST(a: u32) -> bool {
    (a & 0xf0000000) == 0xe0000000
}

pub fn IN6_IS_ADDR_MULTICAST(a: *const u8) -> bool {
    unsafe { *a == 0xff }
}

impl ZmqIpAddress {
    pub fn family(&mut self) -> i32 {
        self.generic.sa_family.clone() as i32
    }

    pub fn is_multicast(&mut self) -> bool {
        if self.family() == AF_INET {
            return IN_MULTICAST(self.ipv4.sin_addr.s_addr.to_be());
        }
        return IN6_IS_ADDR_MULTICAST(&self.ipv6.sin6_addr.s6_addr as *const u8) != false;
    }

    pub fn port(&mut self) -> u16 {
        if self.family() == AF_INET6 {
            return self.ipv6.sin6_port.to_be();
        }
        self.ipv4.sin_port.to_be()
    }

    pub fn as_sockaddr(&mut self) -> *mut sockaddr {
        &mut self.generic
    }

    pub fn sockaddr_len(&mut self) -> usize {
        if self.family() == AF_INET6 {
            std::mem::size_of::<sockaddr_in6>()
        } else {
            std::mem::size_of::<sockaddr_in>()
        }
    }

    pub unsafe fn any(family_: i32) -> anyhow::Result<ZmqIpAddress> {
        let mut addr = ZmqIpAddress::default();
        if family_ == AF_INET {
            addr.ipv4.sin_family = AF_INET as sa_family_t;
            addr.ipv4.sin_addr.s_addr = INADDR_ANY.to_be();
        } else if family_ == AF_INET6 {
            addr.ipv6.sin6_family = AF_INET6 as sa_family_t;
            copy_bytes(
                &in6addr_any.s6_addr,
                0,
                16,
                &mut addr.ipv6.sin6_addr.s6_addr,
                0,
                16,
            )?;
        } else {
            return Err(anyhow::anyhow!("invalid address family"));
        }
        Ok(addr)
    }
}

#[derive(Default, Debug, Clone)]
pub struct ip_resolver_options_t {
    pub _bindable_wanted: bool,
    pub _nic_name_allowed: bool,
    pub _ipv6_wanted: bool,
    pub _port_expected: bool,
    pub _dns_allowed: bool,
    pub _path_allowed: bool,
}

impl ip_resolver_options_t {
    pub fn new() -> Self {
        Self {
            _bindable_wanted: false,
            _nic_name_allowed: false,
            _ipv6_wanted: false,
            _port_expected: false,
            _dns_allowed: false,
            _path_allowed: false,
        }
    }

    pub fn bindable(&mut self, bindable_: bool) -> &Self {
        self._bindable_wanted = bindable_;
        self
    }

    pub fn allow_nic_name(&mut self, allow_: bool) -> &Self {
        self._nic_name_allowed = allow_;
        self
    }

    pub fn ipv6(&mut self, ipv6_: bool) -> &Self {
        self._ipv6_wanted = ipv6_;
        self
    }

    pub fn expect_port(&mut self, expect_: bool) -> &Self {
        self._port_expected = expect_;
        self
    }

    pub fn allow_dns(&mut self, allow_: bool) -> &Self {
        self._dns_allowed = allow_;
        self
    }

    pub fn allow_path(&mut self, allow_: bool) -> &Self {
        self._path_allowed = allow_;
        self
    }

    pub fn get_bindable(&mut self) -> bool {
        self._bindable_wanted.clone()
    }

    pub fn get_allow_nic_name(&mut self) -> bool {
        self._nic_name_allowed.clone()
    }

    pub fn get_ipv6(&mut self) -> bool {
        self._ipv6_wanted.clone()
    }

    pub fn get_expect_port(&mut self) -> bool {
        self._port_expected.clone()
    }

    pub fn get_allow_dns(&mut self) -> bool {
        self._dns_allowed.clone()
    }

    pub fn get_allow_path(&mut self) -> bool {
        self._path_allowed.clone()

    }
}

#[derive(Default, Debug, Clone)]
pub struct ip_resolver_t {
    pub _options: ip_resolver_options_t,
}

impl ip_resolver_t {
    pub fn new(opts_: &mut ip_resolver_options_t) -> Self {
        Self {
            _options: opts_.clone(),
        }
    }

    pub unsafe fn resolve(&mut self, ip_addr_: &mut ZmqIpAddress, name: &str) -> anyhow::Result<()> {
        let mut addr: String = String::new();
        let mut port = 0u16;

        if self._options.get_expect_port() {
            if name.contains(":") == false {
                bail!("invalid address: {}", name);
            }

            let x = name.split(":").collect::<Vec<&str>>();
            let addr_str = x[0].to_string();
            let port_str = x[1].to_string();
            if port_str == "*" {
                if self._options.get_bindable() == true {
                    port = 0;
                } else {
                    bail!("not bindable");
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
                zone_id = do_if_nametoindex(&if_str)?;
            } else {
                zone_id = if_str.parse::<u32>()?;
            }

            if zone_id == 0 {
                bail!("invalid zone id");
            }
        }

        let mut resolved = false;
        let addr_str = addr.clone();
        if self._options.get_bindable() && addr == "*" {
            *ip_addr_ = ZmqIpAddress::any(if self._options.get_ipv6() {
                AF_INET6
            } else {
                AF_INET
            })?;
            resolved = true;
        }

        if !resolved && self._options.get_allow_nic_name() {
            self.resolve_nic_name(ip_addr_, &addr_str)?;
            resolved = true;
        }

        if !resolved {
            self.resolve_getaddrinfo(ip_addr_, &addr_str)?;
            resolved = true;
        }

        ip_addr_.set_port(port);
        if ip_addr_.family() == AF_INET6 {
            ip_addr_.ipv6.sin6_scope_id = zone_id;
        }

        Ok(())
    }

    pub unsafe fn resolve_getaddrinfo(
        &mut self,
        ip_addr_: &mut ZmqIpAddress,
        addr_: &str,
    ) -> anyhow::Result<()> {
        let mut res: addrinfo = addrinfo {
            ai_flags: 0,
            ai_family: 0,
            ai_socktype: 0,
            ai_protocol: 0,
            ai_addrlen: 0,
            ai_addr: null_mut(),
            ai_canonname: null_mut(),
            ai_next: null_mut(),
        };

        let mut req: addrinfo = addrinfo {
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

        let rc = do_getaddrinfo(addr_, null_mut(), &req, &mut res);

        #[cfg(feature = "ai_v4mapped")]
        if (rc == EAI_BADFLAGS && (req.ai_flags & AI_V4MAPPED)) {
            req.ai_flags &= !AI_V4MAPPED;
            rc = do_getaddrinfo(addr_, NULL, &req, &res);
        }

        #[cfg(target_os = "windows")]
        if ((req.ai_family == AF_INET6) && (rc == WSAHOST_NOT_FOUND)) {
            req.ai_family = AF_INET;
            rc = do_getaddrinfo(addr_, NULL, &req, &res);
        }

        if rc != 0 {
            if rc == EAI_MEMORY {
                bail!("out of memory");
            } else if rc != 0 && self._options.get_bindable() {
                bail!("enodev");
            } else {
                bail!("einval")
            }
        }

        // copy_bytes(res.ai_addr, 0, res.ai_addrlen, ip_addr_, 0, ip_addr_.sockaddr_len())?;
        ip_addr_.generic = *res.ai_addr;

        do_freeaddrinfo(&mut res);

        Ok(())
    }

    #[cfg(target_os = "linux")]
    pub unsafe fn resolve_nic_name(&mut self, ip_addr_: &mut ZmqIpAddress, nic_: &str) -> anyhow::Result<()>
    {
        let mut ifa: *mut ifaddrs = null_mut();
        let mut rc = 0i32;
        let max_attempts = 10;
        let backoff_msec = 1;
        for i in 0 .. max_attempts {
            rc = getifaddrs(&mut ifa);
            let errno = io::Error::last_os_error().raw_os_error().unwrap();
            if rc == 0 || rc < 0 && errno == ECONNREFUSED {
                break;
            }
            sleep(std::time::Duration::from_millis(backoff_msec.clone()));
        }

        let errno = io::Error::last_os_error().raw_os_error().unwrap();
        if rc != 0 && errno == EINVAL || errno == EOPNOTSUPP {
            bail!("enodev");
        }

        let mut found = false;
        let mut ifp = ifa;
        while ifp != null_mut() {
            if (*ifp).ifa_addr == null_mut() {
                continue;
            }

            let family = (*ifp).ifa_addr.as_mut().unwrap().sa_family.clone() as i32;
            let if_nic_name = CString::from_raw((*ifp).ifa_name).into_string().unwrap();
            if family == (if self._options.get_ipv6() { AF_INET6} else {AF_INET}) && nic_ == if_nic_name {

               let match_sockaddr = (*ifp).ifa_addr.as_mut().unwrap();
                (*ip_addr_).generic = *match_sockaddr;

                found = true;
                break;
            }
            ifp = (*ifp).ifa_next;
        }

        freeifaddrs(ifa);

        if found == false {
            bail!("enodev");
        }

        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub fn get_interface_name(&mut self, index_: u32, dest: &mut String) -> anyhow::Result<()> {
        let result = if_indextoname(index_, dest.as_mut_ptr());
    }

    #[cfg(target_os = "windows")]
    pub fn resolve_nic_name(&mut self, ip_addr_: &mut ZmqIpAddress, nic_: &mut String) -> anyhow::Result<()>
    {
        let mut rc = 0i32;
        let mut found = false;
        let max_attempts = 10;
        let mut iterations = 0;
        let addresses: *mut IP_ADAPTER_ADDRESSES = null_mut();
        while rc == ERROR_BUFFER_OVERFLOW && iterations < max_attempts {
            rc = GetAdaptersAddresses(
                AF_UNSPEC,
                GAA_FLAG_SKIP_ANYCAST | GAA_FLAG_SKIP_MULTICAST | GAA_FLAG_SKIP_DNS_SERVER,
                null_mut(),
                addresses,
                &mut out_buf_len,
            );
            iterations += 1;
        }

        let current_addresses = addresses;
        while current_addresses != null_mut() {
            let mut if_name: *mut c_char = null_mut();
            let mut if_friendly_name: *mut c_char = null_mut();
            let mut str_rc1 = self.get_interface_name((*current_addresses).IfIndex, &mut if_name);
            let mut str_rc2 = current_addresses.FriendlyName;
            current_addresses = current_addresses.as_mut().unwrap().Next;
            if (str_rc1 == 0 && nic_ == if_name) || (str_rc2 == 0 && nic_ == if_friendly_name) {
                let mut if_addr: *mut IP_ADAPTER_UNICAST_ADDRESS = null_mut();
                if_addr = current_addresses.as_mut().unwrap().FirstUnicastAddress;
                while if_addr != null_mut() {
                    let family = if_addr.as_mut().unwrap().Address.lpSockaddr.as_mut().unwrap().sa_family.clone() as i32;
                    if family == (if self._options.get_ipv6() { AF_INET6} else {AF_INET}) {
                        let match_sockaddr = if_addr.as_mut().unwrap().Address.lpSockaddr.as_mut().unwrap();
                        (*ip_addr_).generic = *match_sockaddr;
                        found = true;
                        break;
                    }
                    if_addr = if_addr.as_mut().unwrap().Next;
                }
            }
        }
    }

    pub unsafe fn do_getaddrinfo(&mut self, node: &mut String, service: &mut String, hints: &mut addrinfo, res: *mut *mut addrinfo) -> anyhow::Result<()>
    {
        getaddrinfo(node.as_mut_ptr() as *mut c_char, service.as_mut_ptr() as *mut c_char, hints, res);
        Ok(())
    }

    pub unsafe fn do_freeaddrinfo(&mut self, res: *mut addrinfo) -> anyhow::Result<()> {
        freeaddrinfo(res);
        Ok(())
    }

    pub unsafe fn do_if_nametoindex(ifname_: &mut String) -> anyhow::Result<()>
    {
        if_nametoindex(ifname_.as_mut_ptr() as *mut c_char);
        Ok(())
    }
}
