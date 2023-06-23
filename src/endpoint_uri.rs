use crate::endpoint::EndpointType;

#[derive(Default, Debug, Clone)]
pub struct EndpointUriPair {
    // std::string local, remote;
    pub local: String,
    pub remote: String,
    // endpoint_type_t local_type;
    pub local_type: EndpointType,
}

impl EndpointUriPair {
    // endpoint_uri_ZmqPair () : local_type (endpoint_type_none) {}

    // endpoint_uri_ZmqPair (const std::string &local,
    //                      const std::string &remote,
    //                      endpoint_type_t local_type) :
    //     local (local), remote (remote), local_type (local_type)
    // {
    // }
    pub fn new(local: &str, remote: &str, local_type: EndpointType) -> Self {
        Self {
            local: String::from(local),
            remote: String::from(remote),
            local_type,
        }
    }

    // const std::string &identifier () const
    pub fn identifier(&self) -> String {
        // return local_type == endpoint_type_bind ? local : remote;
        if self.local_type == EndpointType::endpoint_type_bind {
            self.local.clone()
        } else {
            self.remote.clone()
        }
    }
}
