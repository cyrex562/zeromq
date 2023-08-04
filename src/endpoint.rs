
#[derive(PartialEq)]
pub enum endpoint_type_t
{
    endpoint_type_none,
    endpoint_type_bind,
    endpoint_type_connect,
}

pub struct endpoint_uri_pair_t
{
    pub local: String,
    pub remote: String,
    pub local_type: endpoint_type_t,
}

impl endpoint_uri_pair_t
{
    pub fn new() -> Self
    {
        Self {
            local: String::new(),
            remote: String::new(),
            local_type: endpoint_type_t::endpoint_type_none,
        }
    }

    pub fn new2(local_: &str, remote_: &str, local_type_: endpoint_type_t) -> Self
    {
        Self {
            local: local_.to_string(),
            remote: remote_.to_string(),
            local_type: local_type_,
        }
    }

    pub fn new3(pair: &mut endpoint_uri_pair_t) -> Self
    {
        Self {
            local: pair.local.clone(),
            remote: pair.remote.clone(),
            local_type: pair.local_type.clone(),
        }
    }

    pub fn identifier(&self) -> &String {
        if self.local_type == endpoint_type_t::endpoint_type_bind {
            &self.local
        } else {
            &self.remote
        }
    }
}

pub fn make_unconnected_connect_endpoint_pair(endpoint_: &str) -> endpoint_uri_pair_t
{
    endpoint_uri_pair_t::new2("", endpoint_, endpoint_type_t::endpoint_type_connect)
}

pub fn make_unconnected_bind_endpoint_pair(endpoint_: &str) -> endpoint_uri_pair_t
{
    endpoint_uri_pair_t::new2(endpoint_, "", endpoint_type_t::endpoint_type_bind)
}