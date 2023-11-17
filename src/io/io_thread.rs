use crate::command::ZmqCommand;
use crate::ctx::{reaper_tid, ZmqContext};
use crate::defines::err::ZmqError;
use crate::defines::ZmqHandle;
use crate::io::mailbox::ZmqMailbox;
use crate::object::{obj_process_command, obj_send_stop};
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;
use crate::poll::poller_base::ZmqPollerBase;

pub struct ZmqIoThread<'a> {
    // pub object: ZmqObject<'a>,
    pub thread_id: u32,
    pub _mailbox: ZmqMailbox<'a>,
    pub _mailbox_handle: ZmqHandle,
    pub _poller: &'a mut ZmqPollerBase<'a>,
}

impl<'a> ZmqIoThread<'a> {
    pub fn start(&mut self) {
        let name = format!("IO/{}", self.thread_id - reaper_tid - 1);
        self._poller.start(name);
    }

    pub fn stop(&mut self, ctx: &mut ZmqContext, pipe: &mut ZmqPipe) {
        obj_send_stop(ctx, pipe, self.thread_id);
    }

    pub fn get_mailbox(&mut self) -> *mut ZmqMailbox {
        return &mut self._mailbox;
    }

    pub fn get_load(&mut self) -> i32 {
        return self._poller.get_load();
    }

    pub fn get_poller(&mut self) -> &mut ZmqPollerBase {
        return self._poller;
    }

    pub fn process_stop(&mut self) {
        self._poller.rm_fd(self._mailbox_handle);
        self._poller.stop();
    }

    pub fn in_event(&mut self, options: &ZmqOptions) ->Result<(),ZmqError> {
        let mut cmd = ZmqCommand::new();
        let rc = self._mailbox.recv(&mut cmd, 0)?;
        // TODO: check error state and run while loop, etc
        // while rc == 0 {
        //     if rc == 0 {
        //         // cmd.destination.process_command(&mut cmd);
        //         obj_process_command(options, &mut cmd, cmd.dest_pipe.unwrap());
        //     }
        //     self._mailbox.recv(&mut cmd, 0)?;
        // }
        Ok(())
    }

    fn out_event(&mut self) {
        unimplemented!()
    }

    fn timer_event(&mut self, id_: i32) {
        unimplemented!()
    }
}
