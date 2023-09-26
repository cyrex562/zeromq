#![allow(non_camel_case_types)]

use crate::command::command_t;
use crate::ctx::reaper_tid;
use crate::defines::handle_t;
use crate::i_poll_events::i_poll_events;
use crate::mailbox::mailbox_t;
use crate::object::object_t;
use crate::poller::poller_t;

pub struct io_thread_t
{
    pub object: object_t,
    pub _mailbox: mailbox_t,
    pub _mailbox_handle: handle_t,
    pub _poller: *mut poller_t,
}

impl io_thread_t{
    pub fn start(&mut self) {
        let name = format!("IO/{}", self.object.get_tid() - reaper_tid - 1);
        self._poller.start(name);
    }

    pub fn stop(&mut self) {
        self.object.send_stop();
    }

    pub fn get_mailbox(&mut self) -> mailbox_t {
        return self._mailbox;
    }

    pub fn get_load(&mut self) -> i32 {
        return self._poller.get_load();
    }
}



impl i_poll_events for io_thread_t {
    fn in_event(&mut self) {
        let mut cmd = command_t::new();
        let rc = self._mailbox.recv(&mut cmd, 0);
        while rc == 0 {
            if rc == 0 {
                cmd.destination._ctx.process_command(&mut cmd);
            }
        }
    }

    fn out_event(&mut self) {
        unimplemented!()
    }

    fn timer_event(&mut self, id_: i32) {
        unimplemented!()
    }
}
