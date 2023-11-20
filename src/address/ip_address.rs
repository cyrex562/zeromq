use crate::defines::{
    ZmqSockAddr, ZmqSockAddrIn, ZmqSockAddrIn6, AF_INET, AF_INET6, IN6ADDR_ANY, INADDR_ANY,
};
use crate::ip::ip_resolver;
use crate::utils::copy_bytes;
use std::fmt::{Display, Formatter};
use crate::defines::err::ZmqError;
use crate::defines::err::ZmqError::PlatformError;

#[derive(Default, Debug, Clone)]
pub struct ZmqIpAddress {
    pub generic: ZmqSockAddr,
    pub ipv4: ZmqSockAddrIn,
    pub ipv6: ZmqSockAddrIn6,
}

impl Display for ZmqIpAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ip_addr_t {{ generic: {}, ipv4: {}, ipv6: {} }}",
            ip_resolver::sockaddr_to_str(&self.generic),
            ip_resolver::sockaddr_in_to_str(&self.ipv4),
            ip_resolver::sockaddr_in6_to_str(&self.ipv6)
        )
    }
}

impl ZmqIpAddress {
    pub fn set_port(&mut self, port_: u16) {
        if self.family() == AF_INET6 {
            self.ipv6.sin6_port = port_.to_be();
        } else {
            self.ipv4.sin_port = port_.to_be();
        }
    }
    pub fn family(&mut self) -> i32 {
        self.generic.sa_family.clone() as i32
    }

    pub fn is_multicast(&mut self) -> bool {
        if self.family() == AF_INET {
            return ip_resolver::IN_MULTICAST(self.ipv4.sin_addr.to_be());
        }
        return ip_resolver::IN6_IS_ADDR_MULTICAST(self.ipv6.sin6_addr.as_mut_ptr()) != false;
    }

    pub fn port(&mut self) -> u16 {
        if self.family() == AF_INET6 {
            return self.ipv6.sin6_port.to_be();
        }
        self.ipv4.sin_port.to_be()
    }

    pub fn as_sockaddr(&mut self) -> &mut ZmqSockAddr {
        &mut self.generic
    }

    pub fn sockaddr_len(&mut self) -> usize {
        if self.family() == AF_INET6 {
            std::mem::size_of::<ZmqSockAddrIn6>()
        } else {
            std::mem::size_of::<ZmqSockAddrIn>()
        }
    }

    pub fn any(family_: i32) -> Result<ZmqIpAddress, ZmqError> {
        let mut addr = ZmqIpAddress::default();
        if family_ == AF_INET {
            addr.ipv4.sin_family = AF_INET as u16;
            addr.ipv4.sin_addr = INADDR_ANY.to_be();
        } else if family_ == AF_INET6 {
            addr.ipv6.sin6_family = AF_INET6 as u16;
            copy_bytes(&IN6ADDR_ANY.s6_addr, 0, 16, &mut addr.ipv6.sin6_addr, 0)?;
        } else {
            return Err(PlatformError("invalid address family"));
        }
        Ok(addr)
    }
}
