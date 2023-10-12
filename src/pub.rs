use crate::ctx::ctx_t;
use crate::defines::ZMQ_PUB;
use crate::msg::msg_t;
use crate::options::options_t;
use crate::pipe::pipe_t;
use crate::xpub::xpub_t;

pub struct pub_t<'a> {
    pub xpub: xpub_t<'a>,
}

impl pub_t {
    pub unsafe fn new(parent_: &mut ctx_t, options_: &mut options_t, tid_: u32, sid_: i32) -> Self {
        options_.type_ = ZMQ_PUB;
        Self {
            xpub: xpub_t::new(options_, parent_, tid_, sid_),
        }
    }
    
    pub unsafe fn xattach_pipe(&mut self, pipe_: &mut pipe_t, subscribe_to_all_: bool, locally_initiated_: bool) {
        //  Don't delay pipe termination as there is no one
        //  to receive the delimiter.
        pipe_.set_nodelay ();
    
        self.xpub.xattach_pipe (pipe_, subscribe_to_all_, locally_initiated_);
    }
    
    pub fn xrecv(&mut self, msg: &mut msg_t) -> i32 {
        -1
    }
    
    pub fn xhas_in(&mut self) -> bool {
        false
    }
}