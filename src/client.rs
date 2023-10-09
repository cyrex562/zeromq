use crate::ctx::ctx_t;
use crate::defines::ZMQ_CLIENT;
use crate::fq::fq_t;
use crate::lb::lb_t;
use crate::msg::{more, msg_t};
use crate::options::options_t;
use crate::pipe::pipe_t;
use crate::socket_base::socket_base_t;

pub struct client_t
{
    pub socket_base: socket_base_t,
    pub _fq: fq_t,
    pub _lb: lb_t,
}

impl client_t {
    pub unsafe fn new(options: &mut options_t, parent_: &mut ctx_t, tid_: u32, sid_: i32) -> Self {
        let mut out = Self {
            socket_base: socket_base_t::new(parent_, tid_, sid_, true),
            _fq: fq_t::default(),
            _lb: lb_t::default(),
        };
        options.type_ = ZMQ_CLIENT;
        options.can_send_hello_msg = true;
        options.can_recv_hiccup_msg = true;
        out
    }

    pub fn xattach_pipe(&mut self, pipe_: &mut pipe_t, subscribe_to_all_: bool, locally_initiated_: bool)
    {
        self._fq.attach(pipe_);
        self._lb.attach(pipe_);
    }

    pub unsafe fn xsend(&mut self, msg_: &mut msg_t) -> i32
    {
        if msg_.flags() & more != 0 {
            return -1;
        }
        self._lb.sendpipe(msg_, &mut None)
    }

    pub unsafe fn xrecv(&mut self, msg_: &mut msg_t) -> i32 {
        let mut rc = self._fq.recvpipe(msg_, &mut None);

        while rc == 0 && msg_.flags() & more > 0 {
            rc = self._fq.recvpipe(msg_, &mut None) ;
            while rc == 0 && msg_.flags() & more > 0 {
                rc = self._fq.recvpipe(msg_, &mut None);
            }

            if rc == 0 {
                rc = self._fq.recvpipe(msg_, &mut None)
            }
        }

        rc
    }

    pub fn xhas_in(&mut self) -> bool {
        self._fq.has_in()
    }

    pub unsafe fn xhas_out(&mut self) -> bool {
        self._lb.has_out()
    }

    pub fn xread_activated(&mut self, pipe_: &mut pipe_t) {
        self._fq.activated(pipe_)
    }

    pub fn xwrite_activated(&mut self, pipe_: &mut pipe_t) {
        self._lb.activated(pipe_)
    }

    pub fn xpipe_terminated(&mut self, pipe_: &mut pipe_t) {
        self._fq.pipe_terminated(pipe_);
        self._lb.pipe_terminated(pipe_);
    }
}