use crate::address::ip_address::ZmqIpAddress;
use crate::err::ZmqError;
use crate::ip::ip_resolver::IpResolver;
use crate::ip::ip_resolver_options::IpResolverOptions;
use crate::options::ZmqOptions;
use anyhow::bail;
#[cfg(not(target_os = "windows"))]
use libc::if_nametoindex;
use std::ffi::c_char;
#[cfg(target_os = "windows")]
use windows::Win32::NetworkManagement::IpHelper::if_nametoindex;

#[derive(Default, Debug, Clone)]
pub struct UdpAddress {
    pub _bind_address: ZmqIpAddress,
    pub _bind_interface: i32,
    pub _target_address: ZmqIpAddress,
    pub _is_multicast: bool,
    pub _address: String,
}

impl UdpAddress {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn resolve(
        &mut self,
        options: &ZmqOptions,
        name_: &mut String,
        bind_: bool,
        ipv6_: bool,
    ) -> Result<(), ZmqError> {
        let mut has_interface = false;
        self._address = name_.to_string();
        let mut src_delimiter = name_.find(";");

        if src_delimiter.is_some() {
            let src_name = name_[..src_delimiter.unwrap()].to_string();

            let mut src_resolver_opts: IpResolverOptions = IpResolverOptions::new();
            src_resolver_opts.bindable(true);
            src_resolver_opts.allow_dns(false);
            src_resolver_opts.allow_nic_name(true);
            src_resolver_opts.ipv6(ipv6_);
            src_resolver_opts.expect_port(false);

            let mut src_resolver = IpResolver::new(&mut src_resolver_opts);
            src_resolver.resolve(options, &mut self._bind_address, src_name.as_str())?;

            if self._bind_address.is_multicast() {
                bail!("multicast address not allowed as source address");
            }

            if src_name == "*" {
                self._bind_interface = 0;
            } else {
                unsafe { self._bind_interface = if_nametoindex(src_name.as_ptr() as *const c_char) as i32; }
                if self._bind_interface == 0 {
                    self._bind_interface = -1;
                }
            }

            has_interface = true;
            *name_ = name_[src_delimiter.unwrap() + 1..].to_string();
        }

        let mut resolver_opts: IpResolverOptions = IpResolverOptions::new();
        resolver_opts.bindable(bind_);
        resolver_opts.allow_dns(true);
        resolver_opts.allow_nic_name(bind_.clone());
        resolver_opts.expect_port(true);
        resolver_opts.ipv6(ipv6_.clone());

        let mut resolver = IpResolver::new(&mut resolver_opts);

        resolver.resolve(options, &mut self._target_address, name_.as_str())?;

        self._is_multicast = self._target_address.is_multicast();
        let port = self._target_address.port();

        if has_interface {
            if self._is_multicast == false {
                bail!("source address is set but target address is not multicast");
            }

            self._bind_address.set_port(port);
        } else if self._is_multicast.clone() || !bind_.clone() {
            self._bind_address = ZmqIpAddress::any(self._target_address.family())?;
            self._bind_address.set_port(port);
            self._bind_interface = 0;
        } else {
            self._bind_address = self._target_address.clone();
        }

        if self._bind_address.family() != self._target_address.family() {
            bail!("source and target address families do not match");
        }

        if ipv6_.clone() && self._is_multicast.clone() && self._bind_interface < 0 {
            bail!("multicast requires a source interface");
        }

        Ok(())
    }

    pub fn family(&mut self) -> i32 {
        self._target_address.family()
    }

    pub fn is_mcast(&mut self) -> bool {
        self._is_multicast.clone()
    }

    pub fn bind_addr(&mut self) -> &mut ZmqIpAddress {
        &mut self._bind_address
    }

    pub fn bind_if(&mut self) -> i32 {
        self._bind_interface.clone()
    }

    pub fn target_addr(&mut self) -> &mut ZmqIpAddress {
        &mut self._target_address
    }

    pub fn to_string(&mut self, addr_: &mut String) -> Result<(),ZmqError> {
        if self._is_multicast {
            addr_.push_str(self._target_address.to_string().as_str());
            addr_.push_str(";");
            addr_.push_str(self._bind_address.to_string().as_str());
        } else {
            addr_.push_str(self._target_address.to_string().as_str());
        }

        Ok(())
    }
}
