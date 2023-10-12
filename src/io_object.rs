use std::ptr::null_mut;
use crate::defines::handle_t;
use crate::fd::fd_t;
use crate::i_poll_events::i_poll_events;
use crate::io_thread::io_thread_t;
use crate::poller::poller_t;

pub struct io_object_t {
    pub _poller: *mut poller_t,
}

impl io_object_t {
    pub fn new(io_thread_: *mut io_thread_t) -> Self {
        let mut out = Self {
            _poller: null_mut(),
        };
        if io_thread_ != null_mut() {
            out.plug(io_thread_);
        }
        out
    }

    pub fn plug(&mut self, io_thread_: *mut io_thread_t) {
        self._poller = unsafe { (*io_thread_)._poller };
    }

    pub fn unplug(&mut self) {
        self._poller = null_mut();
    }

    pub fn add_fd(&mut self, fd_: fd_t) -> handle_t {
        self._poller.add_fd(fd_, self)
    }

    pub fn rm_fd(&mut self, handle_: handle_t) {
        self._poller.rm_fd(handle_)
    }

    pub fn set_pollin(&mut self, handle_: handle_t) {
        self._poller.set_poll_in(handle_)
    }

    pub fn reset_pollin(&mut self, handle_: handle_t) {
        self._poller.reset_pollin(handle_)
    }

    pub fn set_pollout(&mut self, handle_: handle_t) {
        self._poller.set_poll_out(handle_)
    }

    pub fn reset_pollout(&mut self, handle_: handle_t) {
        self._poller.reset_pollout(handle_)
    }

    pub fn add_timer(&mut self, timeout_: i32, id_: i32) {
        self._poller.add_timer(timeout_, self, id_)
    }

    pub fn cancel_timer(&mut self, id_: i32) {
        self._poller.cancel_timer(self, id_)
    }
}

impl i_poll_events for io_object_t {
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
