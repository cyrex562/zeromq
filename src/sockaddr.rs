
pub enum ZmqSockAddrType {
    IPv4,
    IPv6
}

#[derive(Default, Debug, Clone)]
pub struct ZmqSockaddr {
    pub family: u16,
    pub data: [u8; 16],
    pub port: u16,
    pub sockaddr_type: ZmqSockAddrType,
    pub flowinfo: u32,
    pub scope_id: u32,
}

impl ZmqSockaddr {
    pub fn new() -> ZmqSockaddr {
        ZmqSockaddr {
            family: 0,
            data: [0; 16],
            port: 0,
            sockaddr_type: ZmqSockAddrType::IPv4,
            flowinfo: 0,
            scope_id: 0,
        }
    }

    pub fn socklen(&self) -> u32 {
        if self.sockaddr_type == ZmqSockAddrType::IPv4 {
            4
        } else {
            16
        }
    }

    pub fn sa_family(&mut self) -> u16 {
        self.family
    }

    pub fn set_sa_family(&mut self, family: u16) {
        self.family = family;
    }

    pub fn set_sin_family(&mut self, family: u16) {
        self.family = family;
    }

    pub fn sin_port(&mut self) -> u16 {
        self.port
    }

    pub fn set_sin_port(&mut self, port: u16) {
        self.port = port;
    }

    pub fn sin6_port(&mut self) -> u16 {
        self.port
    }

    pub fn set_sin6_port(&mut self, port: u16) {
        self.port = port;
    }

    pub fn sa_data(&mut self) -> [u8;14] {
        self.data[0..14].into()
    }

    pub fn set_sa_data(&mut self, sa_data: [u8; 14]) {
        self.data.clear();
        self.data.clone_from_slice(&sa_data);
    }

    pub fn sin_addr(&mut self) -> u32 {
        let mut addr: u32 = 0;
        addr |= (self.data[0] as u32) << 24;
        addr |= (self.data[1] as u32) << 16;
        addr |= (self.data[2] as u32) << 8;
        addr |= (self.data[3] as u32);
        addr
    }

    pub fn sin_family(&mut self) -> u16 {
        self.family
    }

    pub fn sin6_family(&mut self) -> u16 {
        self.family
    }

    pub fn sin6_flowinfo(&mut self) -> u32 {
        self.flowinfo
    }

    pub fn set_sin6_flowinfo(&mut self, flowinfo: u32) {
        self.flowinfo = flowinfo;
    }

    pub fn sin6_addr(&mut self) -> u128 {
        let mut addr: u128 = 0;
        addr |= (self.data[0] as u128) << 120;
        addr |= (self.data[1] as u128) << 112;
        addr |= (self.data[2] as u128) << 104;
        addr |= (self.data[3] as u128) << 96;
        addr |= (self.data[4] as u128) << 88;
        addr |= (self.data[5] as u128) << 80;
        addr |= (self.data[6] as u128) << 72;
        addr |= (self.data[7] as u128) << 64;
        addr |= (self.data[8] as u128) << 56;
        addr |= (self.data[9] as u128) << 48;
        addr |= (self.data[10] as u128) << 40;
        addr |= (self.data[11] as u128) << 32;
        addr |= (self.data[12] as u128) << 24;
        addr |= (self.data[13] as u128) << 16;
        addr |= (self.data[14] as u128) << 8;
        addr |= (self.data[15] as u128);
        addr
    }

    pub fn set_sin6_addr(&mut self, in_addr: u128) {
        addr = in_addr.to_le_bytes();
    }

    pub fn sin6_scope_id(&mut self) -> u32 {
        self.scope_id
    }

    pub fn set_sin6_scope_id(&mut self, scope_id: u32) {
        self.scope_id = scope_id;
    }


}