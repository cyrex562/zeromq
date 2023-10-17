use crate::ctx::ctx_t;
use crate::defines::ZMQ_GATHER;
use crate::fq::fq_t;
use crate::msg::{MSG_MORE, msg_t};
use crate::options::options_t;
use crate::socket_base::socket_base_t;

pub struct gather_t<'a> {
    pub socket_base: socket_base_t<'a>,
    pub _fq: fq_t,
}

impl gather_t {
    pub unsafe fn new(options: &mut options_t, parent_: &mut ctx_t, tid_: u32, sid_: i32) -> Self {
        options.type_ = ZMQ_GATHER;
        Self {
            socket_base: socket_base_t::new(parent_, tid_, sid_, true),
            _fq: fq_t::new(),
        }
    }

    pub fn xattach_pipe(&mut self, pipe_: &mut pipe_t, subscribe_to_all_: bool, locally_initiated_: bool) {
            self._fq.attach(pipe_);
    }

    pub fn xread_activated(&mut self, pipe_: &mut pipe_t) {
        self._fq.activated(pipe_);
    }

    pub fn xpipe_terminated(&mut self, pipe_: &mut pipe_t) {
        self._fq.pipe_terminated(pipe_);
    }

    pub unsafe fn xrecv(&mut self, msg_: &mut msg_t) -> i32 {
        let mut rc = self._fq.recvpipe (msg_, &mut None);

        // Drop any messages with more flag
        while rc == 0 && msg_.flag_set(MSG_MORE) {
            // drop all frames of the current multi-frame message
            rc = self._fq.recvpipe (msg_, &mut None);

            while rc == 0 && msg_.flag_set(MSG_MORE) {
                rc = self._fq.recvpipe(msg_, &mut None);
            }

            // get the new message
            if rc == 0 {
                rc = self._fq.recvpipe(msg_, &mut None);
            }
        }

        return rc;
    }

    pub unsafe fn xhas_in(&mut self) -> bool {
        self._fq.has_in()
    }
}
