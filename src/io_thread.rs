#![allow(non_camel_case_types)]

use crate::command::command_t;
use crate::ctx::reaper_tid;
use crate::defines::handle_t;
use crate::i_poll_events::i_poll_events;
use crate::mailbox::mailbox_t;
use crate::object::object_t;
use crate::poller::poller_t;

pub struct io_thread_t {
    pub object: object_t,
    pub _mailbox: mailbox_t,
    pub _mailbox_handle: handle_t,
    pub _poller: *mut poller_t,
}

impl io_thread_t {
    pub fn start(&mut self) {
        let name = format!("IO/{}", self.object.get_tid() - reaper_tid - 1);
        self._poller.start(name);
    }

    pub fn stop(&mut self) {
        self.object.send_stop();
    }

    pub fn get_mailbox(&mut self) -> *mut mailbox_t {
        return &mut self._mailbox;
    }

    pub fn get_load(&mut self) -> i32 {
        return self._poller.get_load();
    }

    pub fn get_poller(&mut self) -> *mut poller_t {
        return self._poller;
    }

    pub fn process_stop(&mut self) {
        self._poller.rm_fd(self._mailbox_handle);
        self._poller.stop();
    }
}


impl i_poll_events for io_thread_t {
    unsafe fn in_event(&mut self) {
        let mut cmd = command_t::new();
        let rc = self._mailbox.recv(&mut cmd, 0);
        while rc == 0 {
            if rc == 0 {
                cmd.destination.process_command(&mut cmd);
            }
            rc = self._mailbox.recv(&mut cmd, 0);
        }
    }

    fn out_event(&mut self) {
        unimplemented!()
    }

    fn timer_event(&mut self, id_: i32) {
        unimplemented!()
    }
}
