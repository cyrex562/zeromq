use crate::ctx::ZmqContext;
use crate::defines::ZMQ_PEER;
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;
use crate::server::ZmqServer;
use crate::socket::ZmqSocket;

pub struct ZmqPeer<'a> {
    pub server: ZmqServer<'a>,
    pub _peer_last_routing_id: u32,
}

impl ZmqPeer {
    pub unsafe fn new(options: &mut ZmqOptions, parent_: &mut ZmqContext, tid_: u32, sid_: i32) -> Self {
        options.type_ = ZMQ_PEER;
        options.can_send_hello_msg = true;
        options.can_recv_disconnect_msg = true;
        options.can_recv_hiccup_msg = true;

        Self {
            server: ZmqServer::new(options, parent_, tid_, sid_),
            _peer_last_routing_id: 0,
        }
    }


}


pub fn peer_connect_peer(socket: &mut ZmqSocket, options: &mut ZmqOptions, endpoint_uri_: &str) -> u32 {
    if options.immediate == 1 {
        return 0;
    }

    let rc = socket.server.socket_base.connect_internal(endpoint_uri_);
    if rc != 0 {
        return 0;
    }

    return socket.server._peer_last_routing_id;
}

pub fn peer_xattach_pipe(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe, subscribe_to_all_: bool, locally_initiated_: bool) {
    socket.server.xattach_pipe(pipe_, subscribe_to_all_, locally_initiated_);
    socket.server._peer_last_routing_id = pipe_.get_server_socket_routing_id();
}

pub fn peer_xsetsockopt(socket: &mut ZmqSocket, option_: i32, optval_: &[u8], optvallen_: usize) -> i32 {
    unimplemented!()
}

pub fn peer_xgetsockopt(socket: &mut ZmqSocket, option: u32) -> Result<[u8], ZmqError> {
    unimplemented!();
}

pub fn peer_xjoin(socket: &mut ZmqSocket, group: &str) -> i32 {
    unimplemented!();
}

pub fn peer_xsend(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> i32 {
    unimplemented!()
}

pub fn peer_xrecv(socket: &mut ZmqSocket, msg: &mut ZmqMsg) -> i32 {
    unimplemented!()
}

pub fn peer_xhas_in(socket: &mut ZmqSocket) -> bool {
    unimplemented!()
}

pub fn peer_xhas_out(socket: &mut ZmqSocket) -> i32 {
    unimplemented!()
}

pub fn peer_xread_activated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    unimplemented!()
}

pub fn peer_xwrite_activated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    unimplemented!()
}

pub fn peer_xpipe_terminated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    unimplemented!()
}
