use crate::defines::{ZMQ_IO_THREADS, ZmqFd, ZmqHandle};
use std::ptr::null_mut;
use crate::io::io_thread::ZmqIoThread;
use crate::poll::poller_base::ZmqPollerBase;

pub struct IoObject<'a> {
    pub _poller: &'a mut ZmqPollerBase,
}

impl IoObject {
    pub fn new(io_thread_: &mut ZmqIoThread) -> Self {
        let mut out = Self {
            _poller: &mut ZmqPollerBase::default(),
        };
        if io_thread_ != null_mut() {
            out.plug(io_thread_);
        }
        out
    }

    pub fn plug(&mut self, io_thread_: &mut ZmqIoThread) {
        self._poller = io_thread_._poller;
    }

    pub fn unplug(&mut self) {
        self._poller = &mut ZmqIoThread::default();
    }

    pub fn add_fd(&mut self, fd_: ZmqFd) -> ZmqHandle {
        self._poller.add_fd(fd_, self)
    }

    pub fn rm_fd(&mut self, handle_: ZmqHandle) {
        self._poller.rm_fd(handle_)
    }

    pub fn set_pollin(&mut self, handle_: ZmqHandle) {
        self._poller.set_poll_in(handle_)
    }

    pub fn reset_pollin(&mut self, handle_: ZmqHandle) {
        self._poller.reset_pollin(handle_)
    }

    pub fn set_pollout(&mut self, handle_: ZmqHandle) {
        self._poller.set_poll_out(handle_)
    }

    pub fn reset_pollout(&mut self, handle_: ZmqHandle) {
        self._poller.reset_pollout(handle_)
    }

    pub fn add_timer(&mut self, timeout_: i32, id_: i32) {
        self._poller.add_timer(timeout_, self, id_)
    }

    pub fn cancel_timer(&mut self, id_: i32) {
        self._poller.cancel_timer(self, id_)
    }
}

// impl IPollEvents for IoObject {
//     fn in_event(&mut self) {
//         todo!()
//     }
//
//     fn out_event(&mut self) {
//         todo!()
//     }
//
//     fn timer_event(&mut self, id_: i32) {
//         todo!()
//     }
// }