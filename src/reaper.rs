#![allow(non_camel_case_types)]

use crate::defines::handle_t;
use crate::i_poll_events::i_poll_events;
use crate::mailbox::mailbox_t;

pub struct reaper_t {
    pub _mailbox: mailbox_t,
    pub _mailbox_handle: handle_t,
    pub _poller: *mut poller_t,
    pub _sockets: i32,
    pub _terminating: bool,
    #[cfg(feature="have_fork")]
    pub _pid: pid_t,
}

impl i_poll_events for reaper_t {
    fn in_event(&mut self) {
        unimplemented!()
    }

    fn out_event(&mut self) {
        unimplemented!()
    }

    fn timer_event(&mut self, id_: i32) {
        unimplemented!()
    }
}
