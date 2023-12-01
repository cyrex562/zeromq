use std::fmt::Display;

use crate::defines::{
    AF_INET, AF_INET6, IN6ADDR_ANY, INADDR_ANY, ZmqSockAddr, ZmqSockAddrIn, ZmqSockAddrIn6,
};
use crate::ip::ip_resolver;

#[derive(Default, Debug, Clone)]
pub struct ZmqIpAddress {
    pub addr_bytes: [u8; 16],
    pub address_family: i32,
    pub port: u16,
    pub flow_info: u32,
    pub scope_id: u32,
}

// impl Display for ZmqIpAddress {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(
//             f,
//             "ip_addr_t {{ generic: {}, ipv4: {}, ipv6: {} }}",
//             ip_resolver::sockaddr_to_str(&self.generic),
//             ip_resolver::sockaddr_in_to_str(&self.ipv4),
//             ip_resolver::sockaddr_in6_to_str(&self.ipv6)
//         )
//     }
// }
impl Display for ZmqIpAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = String::new();
        // if self.family() == AF_INET {
        //     out.push_str(&format!(
        //         "{}:{}",
        //         ip_resolver::sockaddr_to_str(&self.as_sockaddr()),
        //         self.port()
        //     ));
        // } else {
        //     out.push_str(&format!(
        //         "[{}]:{}",
        //         ip_resolver::sockaddr_to_str(&self.as_sockaddr()),
        //         self.port()
        //     ));
        // }
        // write!(f, "{}", out)
        write!(f, "family: {}, port: {}, flow_info: {}, scope_id: {}, address bytes: {}",self.address_family, self.port, self.flow_info, self.scope_id, self.addr_bytes.to_vec().iter().map(|x| format!("{:02x}", x)).collect::<String>())
    }
}

impl ZmqIpAddress {
    pub fn new(
        addr_bytes: &[u8],
        address_family: i32,
        port: Option<u16>,
        flow_info: Option<u32>,
        scope_id: Option<u32>,
    ) -> Self {
        let mut out = Self {
            addr_bytes: [0; 16],
            address_family: 0,
            port: 0,
            flow_info: 0,
            scope_id: 0,
        };
        out.addr_bytes.copy_from_slice(addr_bytes);
        out.address_family = address_family;
        out.port = port.unwrap_or(0);
        out.flow_info = flow_info.unwrap_or(0);
        out.scope_id = scope_id.unwrap_or(0);
        out
    }
    pub fn set_port(&mut self, port_: u16) {
        // if self.family() == AF_INET6 {
        //     self.ipv6.sin6_port = port_.to_be();
        // } else {
        //     self.ipv4.sin_port = port_.to_be();
        // }
        self.port = port_;
    }
    pub fn family(&mut self) -> i32 {
        // self.generic.sa_family.clone() as i32
        self.address_family
    }

    pub fn get_ip_addr_u32(&self) -> u32 {
        u32::from_be_bytes(self.addr_bytes[0..4].try_into().unwrap())
    }

    pub fn get_ipv6_addr_bytes(&self) -> [u8; 16] {
        self.addr_bytes
    }

    pub fn get_ipv4_addr_bytes(&self) -> [u8;4] {
        self.addr_bytes[0..4].try_into().unwrap()
    }

    pub fn is_multicast(&mut self) -> bool {
        if self.family() == AF_INET {
            return ip_resolver::IN_MULTICAST(self.get_ip_addr_u32());
        }
        return ip_resolver::in6_is_addr_multicast(&self.addr_bytes);
    }

    pub fn port(&mut self) -> u16 {
        // if self.family() == AF_INET6 {
        //     return self.ipv6.sin6_port.to_be();
        // }
        // self.ipv4.sin_port.to_be()
        self.port
    }

    pub fn as_sockaddr(&mut self) -> ZmqSockAddr {
        let mut out = ZmqSockAddr::default();
        out.sa_family = self.family() as u16;
        out.sa_data[0..16].copy_from_slice(&self.addr_bytes[0..16]);
        out
    }

    pub fn sockaddr_len(&mut self) -> usize {
        if self.family() == AF_INET6 {
            std::mem::size_of::<ZmqSockAddrIn6>()
        } else {
            std::mem::size_of::<ZmqSockAddrIn>()
        }
    }

    pub fn any(family_: i32) -> ZmqIpAddress {
        let mut addr = ZmqIpAddress::default();
        if family_ == AF_INET {
            addr.address_family = AF_INET;
            // addr.ipv4.sin_addr = INADDR_ANY.to_be();
            addr.addr_bytes.clone_from_slice(&INADDR_ANY.to_be_bytes());
        } else if family_ == AF_INET6 {
            addr.address_family = AF_INET6;
            // copy_bytes(&IN6ADDR_ANY.s6_addr, 0, 16, &mut addr.ipv6.sin6_addr, 0)?;
            addr.addr_bytes.clone_from_slice(&IN6ADDR_ANY.s6_addr);
        }
        addr
    }
}
