use std::collections::HashSet;
use crate::ctx::ctx_t;
use crate::defines::ZMQ_ROUTER;
use crate::fq::fq_t;
use crate::msg::msg_t;
use crate::options::options_t;
use crate::pipe::pipe_t;
use crate::socket_base::routing_socket_base_t;

pub struct router_t<'a> {
    pub routing_socket_base: routing_socket_base_t<'a>,
    pub _fq: fq_t,
    pub _prefetched: bool,
    pub _routing_id_sent: bool,
    pub _prefetched_id: msg_t,
    pub _prefetched_msg: msg_t,
    pub _current_in: &'a mut pipe_t<'a>,
    pub _terminate_current_in: bool,
    pub _more_in: bool,
    pub _anonymous_pipes: HashSet<&'a mut pipe_t<'a>>,
    pub _current_out: &'a mut pipe_t<'a>,
    pub _more_out: bool,
    pub _next_integral_routing_id: u32,
    pub _mandatory: bool,
    pub _raw_socket: bool,
    pub _probe_router: bool,
    pub _handover: bool
}

impl router_t {
    pub unsafe fn new(options: &mut options_t, parent_: &mut ctx_t, tid_: u32, sid_: i32) -> Self {
        options.type_ = ZMQ_ROUTER;
        options.recv_routing_id = true;
        options.raw_socket = false;
        options.can_send_hello_msg = true;
        options.can_recv_disconnect_msg = true;
        let mut out = Self {
            routing_socket_base: routing_socket_base_t::new(parent_, tid_, sid_),
            _fq: fq_t::default(),
            _prefetched: false,
            _routing_id_sent: false,
            _prefetched_id: Default::default(),
            _prefetched_msg: Default::default(),
            _current_in: &mut Default::default(),
            _terminate_current_in: false,
            _more_in: false,
            _anonymous_pipes: Default::default(),
            _current_out: &mut Default::default(),
            _more_out: false,
            _next_integral_routing_id: 0,
            _mandatory: false,
            _raw_socket: false,
            _probe_router: false,
            _handover: false,
        };
        out._prefetched_id.init2();
        out._prefetched_msg.init2();
        out
    }


}
