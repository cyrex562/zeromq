use crate::context::ZmqContext;
use crate::socket_base::ZmqSocketBase;

pub enum EndpointType {
    endpoint_type_none,
    // a connection-less endpoint
    endpoint_type_bind,
    // a connection-oriented bind endpoint
    endpoint_type_connect, // a connection-oriented connect endpoint
}

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


// endpoint_uri_ZmqPair
// make_unconnected_connect_endpoint_pair (const std::string &endpoint_);
// endpoint_uri_ZmqPair
// make_unconnected_connect_endpoint_pair (const std::string &endpoint_)
pub fn make_unconnected_connected_endpoint_pair(endpoint: &str) -> EndpointUriPair {
    // return endpoint_uri_ZmqPair (std::string (), endpoint_,
    //                             endpoint_type_connect);
    EndpointUriPair::new("", endpoint, EndpointType::endpoint_type_connect)
}

// endpoint_uri_ZmqPair
// make_unconnected_bind_endpoint_pair (const std::string &endpoint_);
// endpoint_uri_ZmqPair
// make_unconnected_bind_endpoint_pair (const std::string &endpoint_)
pub fn make_unconnected_bind_endpoint_pair(endpoint: &str) -> EndpointUriPair {
    // return endpoint_uri_ZmqPair (endpoint_, std::string (), endpoint_type_bind);
    EndpointUriPair::new(endpoint, "", EndpointType::endpoint_type_bind)
}

//  Information associated with inproc endpoint. Note that endpoint options
//  are registered as well so that the peer can access them without a need
//  for synchronisation, handshaking or similar.
#[derive(Default, Debug, Clone)]
pub struct ZmqEndpoint {
    // ZmqSocketBase *socket;
    pub socket: ZmqSocketBase,
    // ZmqOptions options;
    pub context: ZmqContext,
}
