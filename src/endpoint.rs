

#[derive(PartialEq)]
pub enum ZmqEndpointType
{
    EndpointTypeNone,
    EndpointTypeBind,
    EndpointTypeConnect,
}

#[derive(Default,Debug,Clone)]
pub struct ZmqEndpointUriPair
{
    pub local: String,
    pub remote: String,
    pub local_type: ZmqEndpointType,
}

impl ZmqEndpointUriPair
{
    pub fn new(local: &str, remote: &str, local_type: ZmqEndpointType) -> Self
    {
        Self {
            local: local.to_string(),
            remote: remote.to_string(),
            local_type,
        }
    }
    
    pub fn from_endpoint_uri_pair(pair: &mut ZmqEndpointUriPair) -> Self
    {
        Self {
            local: pair.local.clone(),
            remote: pair.remote.clone(),
            local_type: pair.local_type.clone(),
        }
    }

    pub fn identifier(&self) -> &String {
        if self.local_type == ZmqEndpointType::EndpointTypeBind {
            &self.local
        } else {
            &self.remote
        }
    }
}

pub fn make_unconnected_connect_endpoint_pair(endpoint_: &str) -> ZmqEndpointUriPair
{
    ZmqEndpointUriPair::new2("", endpoint_, ZmqEndpointType::EndpointTypeConnect)
}

pub fn make_unconnected_bind_endpoint_pair(endpoint_: &str) -> ZmqEndpointUriPair
{
    ZmqEndpointUriPair::new2(endpoint_, "", ZmqEndpointType::EndpointTypeBind)
}
