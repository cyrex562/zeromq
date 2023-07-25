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
}

#[derive(Default, Debug, Clone)]
pub struct tcp_address_mask_t {
    pub _network_address: ip_addr_t,
    pub _address_mask: i32,
}
