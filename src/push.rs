use libc::option;
use crate::ctx::ctx_t;
use crate::defines::ZMQ_PUSH;
use crate::lb::lb_t;
use crate::msg::msg_t;
use crate::options::options_t;
use crate::pipe::pipe_t;
use crate::socket_base::socket_base_t;

pub struct push_t<'a> {
    pub socket_base: socket_base_t<'a>,
    pub _lb: lb_t,
}

impl push_t {
    pub unsafe fn new(options: &mut options_t, parent_: &mut ctx_t, tid_: u32, sid: i32) -> Self {
        options.type_ = ZMQ_PUSH;
        Self {
            socket_base: socket_base_t::new(parent_, tid_, sid, false),
            _lb: lb_t::new(),
        }
    }
    
    pub unsafe fn xattach_pipe(&mut self, pipe_: &mut pipe_t, subscribe_to_all_: bool, locally_initiated_: bool) {
        pipe_.set_nodelay();
        self._lb.attach(pipe_);
    }
    
    pub unsafe fn xwrite_activated(&mut self, pipe_: &mut pipe_t) {
        self._lb.activated(pipe_)
    }
    
    pub unsafe fn xpipe_terminated(&mut self, pipe_: &mut pipe_t) {
        self._lb.pipe_terminated(pipe_);
    }
    
    pub unsafe fn xsend(&mut self, msg_: &mut msg_t) -> i32 {
        self._lb.send(msg_)
    }
    
    pub unsafe fn xhas_out(&mut self) -> bool {
        self._lb.has_out()
    }
}