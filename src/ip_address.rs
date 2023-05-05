
#[derive(Default,Debug,Clone)]
pub struct ZmqIpAddress {
    address_type: ZmqSocketAddrType,
    address: [u8; 16],
}

impl ZmqIpAddress {
    pub fn new() -> Self {
        Self {
            address_type: ZmqSocketAddrType::IPv4,
            address: [0; 16],
        }
    }

    pub fn set_ipv4_address_from_u32(&mut self, addr: u32) {
        self.address_type = ZmqSocketAddrType::IPv4;
        self.address = addr.to_be_bytes();
    }

    pub fn set_ipv4_address_from_bytes(&mut self, addr: [u8;4]) {
        self.address_type = ZmqSocketAddrType::IPv4;
        self.address.clone_from_slice(&addr);
    }

    pub fn set_ipv6_address_from_bytes(&mut self, addr: [u8;16]) {
        self.address_type = ZmqSocketAddrType::IPv6;
        self.address.clone_from_slice(&addr);
    }

    pub fn set_ipv6_address_from_u128(&mut self, addr: u128) {
        self.address_type = ZmqSocketAddrType::IPv6;
        self.address = addr.to_be_bytes();
    }

    pub fn ipv4_address_bytes(&mut self) -> [u8;4] {
        self.address[0..4].into()
    }

    pub fn ipv6_address_bytes(&mut self) -> [u8;16] {
        self.address
    }

    pub fn ipv4_address_u32(&mut self) -> u32 {
        u32::from_be_bytes(self.address[0..4].try_into().unwrap())
    }

    pub fn ipv6_address_u128(&mut self) -> u128 {
        u128::from_be_bytes(self.address)
    }
}