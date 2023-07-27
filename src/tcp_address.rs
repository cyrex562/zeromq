use crate::ip_resolver::{ip_addr_t, ip_resolver_options_t, ip_resolver_t};
use anyhow::bail;
use libc::{
    c_char, getnameinfo, sockaddr, sockaddr_in, sockaddr_in6, socklen_t, AF_INET, AF_INET6,
    NI_MAXHOST, NI_NUMERICHOST,
};
use std::ops::Index;
use windows::Win32::Networking::WinSock::sa_family_t;

#[derive(Default, Debug, Clone)]
pub struct tcp_address_t {
    pub _address: ip_addr_t,
    pub _source_address: ip_addr_t,
    pub _has_src_addr: bool,
}

impl tcp_address_t {
    pub fn new() -> Self {
        Self {
            _address: ip_addr_t::new(),
            _source_address: ip_addr_t::new(),
            _has_src_addr: false,
        }
    }

    pub fn new2(sa_: &sockaddr, sa_len_: socklen_t) -> Self {
        let mut out = Self {
            ..Default::default()
        };
        if sa_.sa_family == AF_INET && sa_len_ >= 4 {
            let sa_in = sa_ as *const sockaddr_in;
            out._address = ip_addr_t::new2(&sa_in.sin_addr, 4);
            out._source_address = ip_addr_t::new2(&sa_in.sin_addr, 4);
            out._has_src_addr = true;
        } else if sa_.sa_family == AF_INET6 && sa_len_ >= 16 {
            let sa_in6 = sa_ as *const sockaddr_in6;
            out._address = ip_addr_t::new2(&sa_in6.sin6_addr, 16);
            out._source_address = ip_addr_t::new2(&sa_in6.sin6_addr, 16);
            out._has_src_addr = true;
        }
        out
    }

    pub unsafe fn resolve(
        &mut self,
        name_: &mut String,
        local_: bool,
        ipv6_: bool,
    ) -> anyhow::Result<()> {
        let src_delimiter = name_.index(";");
        if src_delimiter.is_some() {
            let mut src_resolver_opts = ip_resolver_options_t::new();
            src_resolver_opts.bindable(true);
            src_resolver_opts.allow_dns(true);
            src_resolver_opts.ipv6(ipv6_);
            src_resolver_opts.expect_port(true);

            let mut src_resolver = ip_resolver_t::new(&mut src_resolver_opts);

            src_resolver.resolve(&mut self._address, name_)?;
            *name_ = name_[src_delimiter.unwrap() + 1..];
            self._has_src_addr = true;
        }

        let mut resolver_opts = ip_resolver_options_t::new();
        resolver_opts.bindable(local_);
        resolver_opts.allow_dns(true);
        resolver_opts.ipv6(ipv6_);
        resolver_opts.allow_nic_name(local_);
        resolver_opts.expect_port(true);

        let mut resolver: ip_resolver_t = ip_resolver_t::new(&mut resolver_opts);

        resolver.resolve(&mut self._address, name_)?;

        Ok(())
    }

    pub fn make_address_string(
        hbuf_: &str,
        port_: u16,
        ipv6_prefix_: &str,
        ipv6_suffix: &str,
    ) -> String {
        let max_port_string_length = 5;

        let mut buf = String::with_capacity(
            (NI_MAXHOST + ipv6_prefix_.len() + ipv6_suffix.len() + max_port_string_length) as usize,
        );
        buf += ipv6_prefix_;
        buf += hbuf_;
        buf += ipv6_suffix;
        buf += format!("{}", port_).as_str();
        return buf;
    }

    pub unsafe fn to_string(&mut self, addr_: &mut String) -> anyhow::Result<()> {
        if self._address.family() != AF_INET && self._address.family() != AF_INET6 {
            *addr_.clear();
            bail!("invalid address family")
        }

        let mut hbuf = String::with_capacity(NI_MAXHOST as usize);
        let mut rc = getnameinfo(
            self.addr(),
            self.addrlen(),
            hbuf.as_mut_ptr() as *mut c_char,
            NI_MAXHOST,
            std::ptr::null_mut(),
            0,
            NI_NUMERICHOST,
        );
        if rc != 0 {
            *addr_.clear();
            bail!("getnameinfo failed")
        }

        let ipv4_prefix: &'static str = "tcp://";
        let ipv4_suffix: &'static str = ":";
        let ipv6_prefix: &'static str = "tcp://[";
        let ipv6_suffix: &'static str = "]:";
        if self._address.family() == AF_INET6 {
            *addr_ = self.make_address_string(hbuf, self._address.port(), ipv6_prefix, ipv6_suffix);
        } else {
            *addr_ = self.make_address_string(hbuf, self._address.port(), ipv4_prefix, ipv4_suffix);
        }
        Ok(())
    }

    pub fn addr(&mut self) -> *mut sockaddr {
        self._address.as_sockaddr()
    }

    pub fn addrlen(&mut self) -> socklen_t {
        self._address.len()
    }

    pub fn src_addr(&mut self) -> *mut sockaddr {
        self._source_address.as_sockaddr()
    }

    pub fn src_addrlen(&mut self) -> socklen_t {
        self._source_address.len()
    }

    pub fn has_src_addr(&mut self) -> bool {
        self._has_src_addr
    }

    #[cfg(target_os = "windows")]
    pub fn family(&mut self) -> u16 {
        self._address.family()
    }
    #[cfg(target_os = "linux")]
    pub fn family(&mut self) -> sa_family_t {
        self._address.family() as sa_family_t
    }
}

#[derive(Default, Debug, Clone)]
pub struct tcp_address_mask_t {
    pub _network_address: ip_addr_t,
    pub _address_mask: i32,
}

impl tcp_address_mask_t {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub unsafe fn resolve(&mut self, name_: &str, ipv6_: bool) -> anyhow::Result<()> {
        let mut addr_str = String::new();
        let mut mask_str = String::new();
        let delimiter = name_.index("/");
        if delimiter.is_some() {
            addr_str = name_[..delimiter.unwrap()].to_string();
            mask_str = name_[delimiter.unwrap() + 1..].to_string();
        } else {
            addr_str = name_.to_string();
            mask_str = "0".to_string();
        }

        let mut resolver_opts: ip_resolver_options_t = ip_resolver_options_t::new();
        resolver_opts.bindable(false);
        resolver_opts.allow_nic_name(false);
        resolver_opts.allow_dns(false);
        resolver_opts.ipv6(ipv6_);
        resolver_opts.expect_port(false);

        let mut resolver = ip_resolver_t::new(&mut resolver_opts);
        resolver.resolve(&mut self._network_address, addr_str.as_str())?;

        Ok(())
    }
}
