use crate::context::ZmqContext;
use crate::endpoint_uri::EndpointUriPair;
use crate::pipe::ZmqPipe;
use crate::socket::ZmqSocket;

pub enum EndpointType {
    endpoint_type_none,
    // a connection-less endpoint
    endpoint_type_bind,
    // a connection-oriented Bind endpoint
    endpoint_type_connect, // a connection-oriented connect endpoint
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
pub struct ZmqEndpoint<'a> {
    // ZmqSocketBase *socket;
    pub uri: EndpointUriPair,
    // pub pipe: &'a mut ZmqPipe,
    pub pipe: ZmqPipe,
    pub socket: &'a mut ZmqSocket<'a>,
}

impl<'a> ZmqEndpoint<'a> {
    pub fn new(socket: &mut ZmqSocket) -> Self {
        Self {
            uri: Default::default(),
            pipe: Default::default(),
            socket,
        }
    }
}
