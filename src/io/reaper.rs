use libc::{EAGAIN, EINTR};

use crate::command::ZmqCommand;
use crate::ctx::ZmqContext;
use crate::defines::ZmqHandle;
use crate::defines::RETIRED_FD;
use crate::io::mailbox::ZmqMailbox;
use crate::object::{obj_process_command, obj_send_stop};
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;
use crate::poll::poller_base::ZmqPollerBase;
use crate::socket::ZmqSocket;
use crate::utils::get_errno;

#[derive(Default, Debug, Clone)]
pub struct ZmqReaper<'a> {
    // pub _object: ZmqObject<'a>,
    pub thread_id: u32,
    pub mailbox: ZmqMailbox<'a>,
    pub mailbox_handle: ZmqHandle,
    pub poller: &'a mut ZmqPollerBase,
    pub sockets: i32,
    pub terminating: bool,
    #[cfg(feature = "have_fork")]
    pub _pid: pid_t,
}

impl ZmqReaper {
    pub fn new(ctx_: &mut ZmqContext, tid_: u32) -> Self {
        let mut out = Self {
            thread_id: 0,
            mailbox: ZmqMailbox::new(),
            mailbox_handle: 0 as ZmqHandle,
            poller: &mut ZmqPollerBase::new(ctx_),
            sockets: 0,
            terminating: false,
            #[cfg(feature = "have_fork")]
            _pid: 0,
            // _object: ZmqObject::new(ctx_, tid_),
        };
        if out.mailbox.get_fd() != RETIRED_FD {
            (*out.poller).add(out.mailbox.get_fd(), &mut out);
            out.sockets += 1;
            out.poller.set_pollin(&mut out.mailbox_handle);
        }

        out
    }
}

pub fn reaper_in_event(options: &ZmqOptions, reaper: &mut ZmqReaper, pipe: &mut ZmqPipe) {
    loop {
        #[cfg(feature = "have_fork")]
        {
            if self._pid != libc::getpid() {
                return;
            }
        }

        let mut cmd: ZmqCommand = ZmqCommand::new();
        let rc = reaper.mailbox.recv(&mut cmd, 0);
        if rc != 0 && get_errno() == EINTR {
            continue;
        }
        if rc != 0 && get_errno() == EAGAIN {
            break;
        }

        // TODO
        // cmd.destination.process_command(cmd);
        obj_process_command(options, &mut cmd, pipe)
    }
}

// pub fn get_mailbox(&mut self) -> &mut ZmqMailbox {
//         &mut self.mailbox
//     }

pub fn reaper_start(reaper: &mut ZmqReaper) {
    reaper.poller.start("Reaper")
}

pub fn reaper_stop(ctx: &mut ZmqContext, reaper: &mut ZmqReaper, pipe: &mut ZmqPipe) {
    if reaper.mailbox.valid() {
        // reaper._object.send_stop();
        obj_send_stop(ctx, pipe, reaper.thread_id)
    }
}

pub fn reaper_process_stop(reaper: &mut ZmqReaper) {
    reaper.terminating = true;
    if reaper.sockets == 0 {
        reaper.poller.rm_fd(&mut reaper.mailbox_handle);
        reaper.poller.stop();
    }
}

pub fn reaper_process_reap(reaper: &mut ZmqReaper, socket_: &mut ZmqSocket) {
    socket_.start_reaping(reaper.poller);
    reaper.sockets += 1;
}

pub fn reaper_process_reaped(reaper: &mut ZmqReaper) {
    reaper.sockets -= 1;
    if reaper.terminating && reaper.sockets == 0 {
        reaper.poller.rm_fd(&mut reaper.mailbox_handle);
        reaper.poller.stop();
    }
}
