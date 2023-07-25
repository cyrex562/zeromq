use crate::utils::copy_bytes;
use anyhow::bail;
use libc::{
    addrinfo, getaddrinfo, in6_addr, sa_family_t, sockaddr, sockaddr_in, sockaddr_in6, AF_INET,
    AF_INET6, AI_NUMERICHOST, AI_PASSIVE, EAI_MEMORY, INADDR_ANY, SOCK_STREAM,
};
use std::ptr::null_mut;

pub union ip_addr_t {
    pub generic: sockaddr,
    pub ipv4: sockaddr_in,
    pub ipv6: sockaddr_in6,
}

pub const in6addr_any: in6_addr = in6_addr { s6_addr: [0; 16] };

impl Default for ip_addr_t {
    fn default() -> Self {
        Self {
            generic: sockaddr {
                sa_family: 0,
                sa_data: [0; 14],
            },
        }
    }
}

pub fn IN_MULTICAST(a: u32) -> bool {
    (a & 0xf0000000) == 0xe0000000
}

pub fn IN6_IS_ADDR_MULTICAST(a: *const u8) -> bool {
    unsafe { *a == 0xff }
}

impl ip_addr_t {
    pub fn family(&mut self) -> i32 {
        self.generic.sa_family as i32
    }

    pub fn is_multicast(&mut self) -> bool {
        if self.family() == AF_INET {
            return IN_MULTICAST(self.ipv4.sin_addr.s_addr.to_be());
        }
        return IN6_IS_ADDR_MULTICAST(&self.ipv6.sin6_addr.s6_addr as *const u8) != 0;
    }

    pub fn port(&mut self) -> u16 {
        if self.family() == AF_INET6 {
            self.ipv6.sin6_port.to_be()
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

    pub unsafe fn any(family_: i32) -> anyhow::Result<ip_addr_t> {
        let mut addr = ip_addr_t::default();
        if family_ == AF_INET {
            addr.ipv4.sin_family = AF_INET as sa_family_t;
            addr.ipv4.sin_addr.s_addr = INADDR_ANY.to_be();
        } else if (family_ == AF_INET6) {
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
        self._bindable_wanted
    }

    pub fn get_allow_nic_name(&mut self) -> bool {
        self._nic_name_allowed
    }

    pub fn get_ipv6(&mut self) -> bool {
        self._ipv6_wanted
    }

    pub fn get_expect_port(&mut self) -> bool {
        self._port_expected
    }

    pub fn get_allow_dns(&mut self) -> bool {
        self._dns_allowed
    }

    pub fn get_allow_path(&mut self) -> bool {
        self._path_allowed
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

    pub unsafe fn resolve(&mut self, ip_addr_: &mut ip_addr_t, name: &str) -> anyhow::Result<()> {
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
                Ok(sz) => {
                    addr = addr[0..sz].to_string();
                }
            }
        }

        let brackets_length = 2;
        if (addr.len() >= brackets_length && addr[0] == "[" && addr[addr.len() - 1] == "]") {
            addr = addr[1..addr.len() - 2].to_string();
        }

        let pos = addr.rfind("%");
        let mut zone_id = 0u32;
        if pos.is_some() {
            let if_str = addr[pos.unwrap()..].to_string();
            addr = addr[0..pos.unwrap()].to_string();
            let chars = if_str.chars().collect::<Vec<char>>();
            let x = chars[0];
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
            *ip_addr_ = ip_addr_t::any(if self._options.get_ipv6() {
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
        ip_addr_: &mut ip_addr_t,
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

        if rc == EAI_MEMORY {
            bail!("out of memory");
        } else if rc != 0 && self._options.get_bindable() {
            bail!("enodev");
        } else {
            bail!("einval")
        }
    }
}
