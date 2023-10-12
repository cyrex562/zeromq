use crate::ctx::ctx_t;
use crate::defines::ZMQ_PULL;
use crate::fq::fq_t;
use crate::msg::msg_t;
use crate::options::options_t;
use crate::pipe::pipe_t;
use crate::socket_base::socket_base_t;

pub struct pull_t<'a> {
    pub socket_base: socket_base_t<'a>,
    pub _fq: fq_t,
}

impl pull_t {
    pub unsafe fn new(options: &mut options_t, parent_: &mut ctx_t, tid_: u32, sid_: i32) -> Self {
        options.type_ = ZMQ_PULL;
        Self {
            socket_base: socket_base_t::new( parent_, tid_, sid_, false),
            _fq: fq_t::new(),
        }
    }
    
    pub unsafe fn xattach_pipe(&mut self, pipe_: &mut pipe_t, subscribe_to_all_: bool, locally_initiated_: bool) {
        self._fq.attach(pipe_)
    }
    
    pub fn xread_activated(&mut self, pipe: &mut pipe_t) {
        self._fq.activated(pipe)
    }
    
    pub fn xpipe_terminated(&mut self, pipe: &mut pipe_t) {
        self._fq.pipe_terminated(pipe)
    }
    
    pub unsafe fn xrecv(&mut self, msg_: &mut msg_t) -> i32 {
        self._fq.recv(msg_)
    }
    
    pub unsafe fn xhas_in(&mut self) -> bool {
        self._fq.has_in()
    }
}