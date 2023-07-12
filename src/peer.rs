use std::error::Error;
use bincode::options;
use libc::EFAULT;
use crate::context::ZmqContext;
use crate::defines::ZMQ_PEER;
use crate::err::ZmqError;

use crate::pipe::ZmqPipe;
use crate::socket::ZmqSocket;


// #[derive(Default, Debug, Clone)]
// pub struct ZmqPeer {
//     //   : public ZmqServer
//     pub server: ZmqServer,
//     // u32 _peer_last_routing_id;
//     pub peer_last_routing_id: u32,
//
// }

// impl ZmqPeer {
//     // ZmqPeer (ZmqContext *parent_, tid: u32, sid_: i32);
//     pub fn new(&mut options: ZmqContext, parent: &mut ZmqContext, tid: u32, sid_: i32) -> Self
//
//     {
//         options.type_ = ZMQ_PEER;
//         options.can_send_hello_msg = true;
//         options.can_recv_disconnect_msg = true;
//         options.can_recv_hiccup_msg = true;
//         options.can_recv_routing_id = true;
// // ZmqServer (parent_, tid, sid_) -> Self
//         Self {
//             server: ZmqServer::new(parent, tid, sid_),
//             ..Default::default()
//         }
//     }
//
//
//
//
//     // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqPeer)
// }



//  Overrides of functions from ZmqSocketBase.
// void xattach_pipe (pipe: &mut ZmqPipe,
// subscribe_to_all_: bool,
// locally_initiated_: bool);
pub fn peer_xattach_pipe(sock: &mut ZmqSocket, pipe: &mut ZmqPipe,
                    subscribe_to_all_: bool,
                    locally_initiated_: bool) {
    sock.xattach_pipe(pipe, subscribe_to_all_, locally_initiated_);
    sock._peer_last_routing_id = pipe.get_server_socket_routing_id();
}

// u32 connect_peer (endpoint_uri_: &str);
pub fn connect_peer(sock: &mut ZmqSocket, endpoint_uri_: &str) -> Result<i32,Error> {
    // let mut sync_lock = scoped_optional_lock_t::new(&sync);

    // connect_peer cannot work with immediate enabled
    if options.immediate == 1 {
        return Err(ZmqError::Fault("efault".to_string()));
    }

    sock.connect_internal(endpoint_uri_)?;

    return Ok(sock._peer_last_routing_id);
}


