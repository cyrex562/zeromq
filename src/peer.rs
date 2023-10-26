use crate::ctx::ZmqContext;
use crate::defines::ZMQ_PEER;
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;
use crate::server::ZmqServer;

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

    pub unsafe fn connect_peer(&mut self, options: &mut ZmqOptions, endpoint_uri_: &str) -> u32 {
        if options.immediate == 1 {
            return 0;
        }

        let rc = self.server.socket_base.connect_internal(endpoint_uri_);
        if rc != 0 {
            return 0;
        }

        return self._peer_last_routing_id;
    }

    pub unsafe fn xattach_pipe(&mut self, pipe_: &mut ZmqPipe, subscribe_to_all_: bool, locally_initiated_: bool) {
        self.server.xattach_pipe(pipe_, subscribe_to_all_, locally_initiated_);
        self._peer_last_routing_id = pipe_.get_server_socket_routing_id();
    }
}
