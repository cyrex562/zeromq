

use crate::command::ZmqCommand;
use crate::ctx::reaper_tid;
use crate::defines::ZmqHandle;
use crate::i_poll_events::IPollEvents;
use crate::mailbox::ZmqMailbox;
use crate::object::ZmqObject;
use crate::poller::ZmqPoller;

pub struct ZmqIoThread<'a> {
    pub object: ZmqObject<'a>,
    pub _mailbox: ZmqMailbox,
    pub _mailbox_handle: ZmqHandle,
    pub _poller: *mut ZmqPoller,
}

impl ZmqIoThread {
    pub fn start(&mut self) {
        let name = format!("IO/{}", self.object.get_tid() - reaper_tid - 1);
        self._poller.start(name);
    }

    pub fn stop(&mut self) {
        self.object.send_stop();
    }

    pub fn get_mailbox(&mut self) -> *mut ZmqMailbox {
        return &mut self._mailbox;
    }

    pub fn get_load(&mut self) -> i32 {
        return self._poller.get_load();
    }

    pub fn get_poller(&mut self) -> *mut ZmqPoller {
        return self._poller;
    }

    pub fn process_stop(&mut self) {
        self._poller.rm_fd(self._mailbox_handle);
        self._poller.stop();
    }
}


impl IPollEvents for ZmqIoThread {
    unsafe fn in_event(&mut self) {
        let mut cmd = ZmqCommand::new();
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
