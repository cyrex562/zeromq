use crate::ctx::ctx_t;
use crate::defines::ZMQ_SCATTER;
use crate::lb::lb_t;
use crate::msg::{MSG_MORE, msg_t};
use crate::options::options_t;
use crate::pipe::pipe_t;
use crate::socket_base::socket_base_t;

pub struct scatter_t {
    pub socket_base: socket_base_t,
    pub _lb: lb_t,
}

impl scatter_t {
    pub unsafe fn new(options: &mut options_t, parent_: &mut ctx_t, tid_: u32, sid_: i32) -> Self {
        options.type_ = ZMQ_SCATTER;
        Self {
            socket_base: socket_base_t::new(parent_, tid_, sid_, false),
            _lb: lb_t::new(),
        }
    }

    pub unsafe fn xattach_pipe(&mut self, pipe_: &mut pipe_t, subscribe_to_all: bool, locally_initiated_: bool) {
        pipe_.set_nodelay();
        self._lb.attach(pipe_);
    }

    pub unsafe fn xwrite_activated(&mut self, pipe_: &mut pipe_t) {
        self._lb.activated(pipe_);
    }

    pub unsafe fn xpipe_terminated(&mut self, pipe_: &mut pipe_t) {
        self._lb.terminated(pipe_);
    }

    pub unsafe fn xsend(&mut self, msg_: &mut msg_t) -> i32 {
        //  SCATTER sockets do not allow multipart data (ZMQ_SNDMORE)
        if (msg_.flags () & MSG_MORE) {
            // errno = EINVAL;
            return -1;
        }

        return self._lb.send (msg_);
    }

    // bool zmq::scatter_t::xhas_out ()
    pub unsafe fn xhas_out(&mut self) -> bool
    {
        return self._lb.has_out ();
    }
}
