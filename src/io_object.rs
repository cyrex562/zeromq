use std::ptr::null_mut;
use crate::defines::ZmqHandle;
use crate::fd::fd_t;
use crate::i_poll_events::IPollEvents;
use crate::io_thread::ZmqIoThread;
use crate::poller::ZmqPoller;

pub struct IoObject {
    pub _poller: *mut ZmqPoller,
}

impl IoObject {
    pub fn new(io_thread_: *mut ZmqIoThread) -> Self {
        let mut out = Self {
            _poller: null_mut(),
        };
        if io_thread_ != null_mut() {
            out.plug(io_thread_);
        }
        out
    }

    pub fn plug(&mut self, io_thread_: *mut ZmqIoThread) {
        self._poller = unsafe { (*io_thread_)._poller };
    }

    pub fn unplug(&mut self) {
        self._poller = null_mut();
    }

    pub fn add_fd(&mut self, fd_: fd_t) -> ZmqHandle {
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

impl IPollEvents for IoObject {
    fn in_event(&mut self) {
        todo!()
    }

    fn out_event(&mut self) {
        todo!()
    }

    fn timer_event(&mut self, id_: i32) {
        todo!()
    }
}
