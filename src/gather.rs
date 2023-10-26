use crate::ctx::ZmqContext;
use crate::defines::ZMQ_GATHER;
use crate::fair_queue::ZmqFairQueue;
use crate::msg::{MSG_MORE, ZmqMsg};
use crate::options::ZmqOptions;
use crate::socket_base::ZmqSocketBase;

pub struct ZmqGather<'a> {
    pub socket_base: ZmqSocketBase<'a>,
    pub _fq: ZmqFairQueue,
}

impl ZmqGather {
    pub unsafe fn new(options: &mut ZmqOptions, parent_: &mut ZmqContext, tid_: u32, sid_: i32) -> Self {
        options.type_ = ZMQ_GATHER;
        Self {
            socket_base: ZmqSocketBase::new(parent_, tid_, sid_, true),
            _fq: ZmqFairQueue::new(),
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

    pub unsafe fn xrecv(&mut self, msg_: &mut ZmqMsg) -> i32 {
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
