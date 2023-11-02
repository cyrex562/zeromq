#[derive(Default, Debug, Clone)]
pub struct IpResolverOptions {
    pub _bindable_wanted: bool,
    pub _nic_name_allowed: bool,
    pub _ipv6_wanted: bool,
    pub _port_expected: bool,
    pub _dns_allowed: bool,
    pub _path_allowed: bool,
}

impl IpResolverOptions {
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
