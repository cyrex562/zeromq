use crate::ctx::ctx_t;
use crate::defines::ZMQ_PEER;
use crate::options::options_t;
use crate::pipe::pipe_t;
use crate::server::server_t;

pub struct peer_t<'a> {
    pub server: server_t<'a>,
    pub _peer_last_routing_id: u32,
}

impl peer_t {
    pub unsafe fn new(options: &mut options_t, parent_: &mut ctx_t, tid_: u32, sid_: i32) -> Self {
        options.type_ = ZMQ_PEER;
        options.can_send_hello_msg = true;
        options.can_recv_disconnect_msg = true;
        options.can_recv_hiccup_msg = true;

        Self {
            server: server_t::new(options, parent_, tid_, sid_),
            _peer_last_routing_id: 0,
        }
    }

    pub unsafe fn connect_peer(&mut self, options: &mut options_t, endpoint_uri_: &str) -> u32 {
        if options.immediate == 1 {
            return 0;
        }

        let rc = self.server.socket_base.connect_internal(endpoint_uri_);
        if rc != 0 {
            return 0;
        }

        return self._peer_last_routing_id;
    }

    pub unsafe fn xattach_pipe(&mut self, pipe_: &mut pipe_t, subscribe_to_all_: bool, locally_initiated_: bool) {
        self.server.xattach_pipe(pipe_, subscribe_to_all_, locally_initiated_);
        self._peer_last_routing_id = pipe_.get_server_socket_routing_id();
    }
}
