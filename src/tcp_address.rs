use crate::err::ZmqError;
use crate::ip_resolver::IpResolver;
use anyhow::bail;
use libc::{
    AF_INET, AF_INET6, c_char, getnameinfo, in6_addr, in_addr, NI_MAXHOST, NI_NUMERICHOST, size_t,
    sockaddr, sockaddr_in, sockaddr_in6, socklen_t,
};
use std::ffi::c_void;
use std::mem;
use std::ops::Index;
use std::ptr::null_mut;
use crate::ip_address::ZmqIpAddress;
use crate::ip_resolver_options::IpResolverOptions;

#[derive(Default, Debug, Clone)]
pub struct ZmqTcpAddress {
    pub address: ZmqIpAddress,
    pub source_address: ZmqIpAddress,
    pub has_src_addr: bool,
}

impl ZmqTcpAddress {
    pub fn new() -> Self {
        Self {
            address: ZmqIpAddress::new(),
            source_address: ZmqIpAddress::new(),
            has_src_addr: false,
        }
    }

    pub fn new2(sa_: &sockaddr, sa_len_: socklen_t) -> Self {
        let mut out = Self {
            ..Default::default()
        };
        if sa_.sa_family == AF_INET && sa_len_ >= 4 {
            let sa_in = sa_ as *const sockaddr_in;
            out.address = ZmqIpAddress::new2(&sa_in.sin_addr, 4);
            out.source_address = ZmqIpAddress::new2(&sa_in.sin_addr, 4);
            out.has_src_addr = true;
        } else if sa_.sa_family == AF_INET6 && sa_len_ >= 16 {
            let sa_in6 = sa_ as *const sockaddr_in6;
            out.address = ZmqIpAddress::new2(&sa_in6.sin6_addr, 16);
            out.source_address = ZmqIpAddress::new2(&sa_in6.sin6_addr, 16);
            out.has_src_addr = true;
        }
        out
    }

    pub fn resolve(
        &mut self,
        name_: &mut String,
        local_: bool,
        ipv6_: bool,
    ) -> Result<(), ZmqError> {
        let src_delimiter = name_.index(";");
        if src_delimiter.is_some() {
            let mut src_resolver_opts = IpResolverOptions::new();
            src_resolver_opts.bindable(true);
            src_resolver_opts.allow_dns(true);
            src_resolver_opts.ipv6(ipv6_);
            src_resolver_opts.expect_port(true);

            let mut src_resolver = IpResolver::new(&mut src_resolver_opts);

            src_resolver.resolve(&mut self.address, name_)?;
            *name_ = name_[src_delimiter.unwrap() + 1..];
            self.has_src_addr = true;
        }

        let mut resolver_opts = IpResolverOptions::new();
        resolver_opts.bindable(local_);
        resolver_opts.allow_dns(true);
        resolver_opts.ipv6(ipv6_);
        resolver_opts.allow_nic_name(local_);
        resolver_opts.expect_port(true);

        let mut resolver: IpResolver = IpResolver::new(&mut resolver_opts);

        resolver.resolve(&mut self.address, name_)?;

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

    pub fn to_string(&mut self, addr_: &mut String) -> anyhow::Result<()> {
        if self.address.family() != AF_INET && self.address.family() != AF_INET6 {
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
        if self.address.family() == AF_INET6 {
            *addr_ = self.make_address_string(hbuf, self.address.port(), ipv6_prefix, ipv6_suffix);
        } else {
            *addr_ = self.make_address_string(hbuf, self.address.port(), ipv4_prefix, ipv4_suffix);
        }
        Ok(())
    }

    pub fn addr(&mut self) -> *mut sockaddr {
        self.address.as_sockaddr()
    }

    pub fn addrlen(&mut self) -> socklen_t {
        self.address.len()
    }

    pub fn src_addr(&mut self) -> *mut sockaddr {
        self.source_address.as_sockaddr()
    }

    pub fn src_addrlen(&mut self) -> socklen_t {
        self.source_address.len()
    }

    pub fn has_src_addr(&mut self) -> bool {
        self.has_src_addr
    }

    #[cfg(target_os = "windows")]
    pub fn family(&mut self) -> u16 {
        self.address.family()
    }
    #[cfg(target_os = "linux")]
    pub fn family(&mut self) -> sa_family_t {
        self.address.family() as sa_family_t
    }
}

#[derive(Default, Debug, Clone)]
pub struct tcp_address_mask_t {
    pub _network_address: ZmqIpAddress,
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

        let mut resolver_opts: IpResolverOptions = IpResolverOptions::new();
        resolver_opts.bindable(false);
        resolver_opts.allow_nic_name(false);
        resolver_opts.allow_dns(false);
        resolver_opts.ipv6(ipv6_);
        resolver_opts.expect_port(false);

        let mut resolver = IpResolver::new(&mut resolver_opts);
        resolver.resolve(&mut self._network_address, addr_str.as_str())?;

        let full_mask_ipv4 = 32;
        let full_mask_ipv6 = 128;

        if mask_str.len() == 0 {
            self._address_mask = 0;
        } else {
            let mask = mask_str.parse::<i32>().unwrap();
            if mask < 1
                || (self._network_address.family() == AF_INET6 && mask > full_mask_ipv6)
                || (self._network_address.family() != AF_INET6 && mask > full_mask_ipv4)
            {
                bail!("invalid address mask")
            }
            self._address_mask = mask;
        }

        Ok(())
    }

    pub unsafe fn match_address(&mut self, ss_: &sockaddr, ss_len_: socklen_t) -> bool {
        if ss_.sa_family != self._network_address.family() as u16 {
            return false;
        }

        if self._address_mask > 0 {
            let mut mask = 0i32;
            let mut our_bytes: *mut u8 = null_mut();
            let mut their_bytes: *mut u8 = null_mut();
            if ss_.sa_family == AF_INET6 as u16 {
                their_bytes = ss_.sa_data[0..16].as_mut_ptr() as *mut u8;
                our_bytes =
                    (*self._network_address.as_sockaddr()).sa_data[0..16].as_mut_ptr() as *mut u8;
                mask = (mem::size_of::<in6_addr>() * 8) as i32;
            } else {
                their_bytes = ss_.sa_data[0..4].as_mut_ptr() as *mut u8;
                our_bytes =
                    (*self._network_address.as_sockaddr()).sa_data[0..4].as_mut_ptr() as *mut u8;
                mask = (mem::size_of::<in_addr>() * 8) as i32;
            }
            if self._address_mask < mask {
                mask = self._address_mask;
            }

            let full_bytes = mask / 8;
            if libc::memcmp(
                our_bytes as *const c_void,
                their_bytes as *const c_void,
                full_bytes as size_t,
            ) != 0
            {
                return false;
            }

            let last_byte_bits = 0xff << (8 - (mask.clone() % 8));
            if last_byte_bits > 0
                && (*(their_bytes + full_bytes.clone()) & last_byte_bits)
                    != (*(our_bytes + full_bytes.clone()) & last_byte_bits.clone())
            {
                return false;
            }
        }

        return true;
    }
}
