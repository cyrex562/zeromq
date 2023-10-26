

#[derive(PartialEq)]
pub enum ZmqEndpointType
{
    endpoint_type_none,
    endpoint_type_bind,
    endpoint_type_connect,
}

pub struct ZmqEndpointUriPair
{
    pub local: String,
    pub remote: String,
    pub local_type: ZmqEndpointType,
}

impl ZmqEndpointUriPair
{
    pub fn new() -> Self
    {
        Self {
            local: String::new(),
            remote: String::new(),
            local_type: ZmqEndpointType::endpoint_type_none,
        }
    }

    pub fn new2(local_: &str, remote_: &str, local_type_: ZmqEndpointType) -> Self
    {
        Self {
            local: local_.to_string(),
            remote: remote_.to_string(),
            local_type: local_type_,
        }
    }

    pub fn new3(pair: &mut ZmqEndpointUriPair) -> Self
    {
        Self {
            local: pair.local.clone(),
            remote: pair.remote.clone(),
            local_type: pair.local_type.clone(),
        }
    }

    pub fn identifier(&self) -> &String {
        if self.local_type == ZmqEndpointType::endpoint_type_bind {
            &self.local
        } else {
            &self.remote
        }
    }
}

pub fn make_unconnected_connect_endpoint_pair(endpoint_: &str) -> ZmqEndpointUriPair
{
    ZmqEndpointUriPair::new2("", endpoint_, ZmqEndpointType::endpoint_type_connect)
}

pub fn make_unconnected_bind_endpoint_pair(endpoint_: &str) -> ZmqEndpointUriPair
{
    ZmqEndpointUriPair::new2(endpoint_, "", ZmqEndpointType::endpoint_type_bind)
}
