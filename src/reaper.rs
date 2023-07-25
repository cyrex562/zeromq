use libc::{EAGAIN, EINTR};

use crate::context::ZmqContext;
use crate::defines::{ZmqHandle, RETIRED_FD};
use crate::devpoll::ZmqPoller;
use crate::mailbox::ZmqMailbox;
use crate::thread_command::ZmqThreadCommand;

pub struct ZmqReaper<'a> {
    //  Reaper thread accesses incoming commands via this mailbox.
    pub mailbox: ZmqMailbox<'a>,
    //  Handle associated with mailbox' file descriptor.
    pub mailbox_handle: ZmqHandle,
    //  I/O multiplexing is performed using a poller object.
    pub poller: ZmqPoller<'a>,
    //  Number of sockets being Reaped at the moment.
    pub _sockets: i32,
    //  If true, we were already asked to terminate.
    pub terminating: bool,
}

impl ZmqReaper {
    pub fn new(ctx: &mut ZmqContext, tid: u32) -> Self {
        let mut out = Self {
            mailbox: Default::default(),
            mailbox_handle: 0,
            poller: ZmqPoller::new(ctx),
            _sockets: 0,
            terminating: false,
        };
        if out.mailbox.get_fd() != RETIRED_FD {
            out.mailbox_handle = out.poller.add_fd(&out.mailbox.get_fd(), &mut out);
            out.poller.set_pollin(&out.mailbox_handle);
        }
        out
    }

    pub fn start(&mut self) {
        // zmq_assert (mailbox.valid ());

        //  Start the thread.
        self.poller.start("Reaper");
    }

    pub fn stop(&mut self) {
        if (self.mailbox.valid()) {
            send_stop();
        }
    }

    pub fn in_event() {
        loop {
            // #ifdef HAVE_FORK
            //         if ( (_pid != getpid ())) {
            //             //printf("reaper_t::in_event return in child process %d\n", getpid());
            //             return;
            //         }
            // #endif

            //  Get the next command. If there is none, exit.
            let mut cmd = ZmqThreadCommand::default();
            let rc: i32 = mailbox.recv(&cmd, 0);
            if (rc != 0 && errno == EINTR) {
                continue;
            }
            if (rc != 0 && errno == EAGAIN) {
                break;
            }
            // errno_assert (rc == 0);

            //  Process the command.
            cmd.destination.process_command(&cmd);
        }
    }

    pub fn process_stop(&mut self) {
        terminating = true;

        //  If there are no sockets being Reaped finish immediately.
        if (!self._sockets) {
            send_done();
            self.poller.rm_fd(mailbox_handle);
            self.poller.stop();
        }
    }

    pub fn process_reap(&mut self, socket: &mut ZmqSocket) {
        //  Add the socket to the poller.
        socket.start_reaping(poller);
        self._sockets += 1;
    }

    pub fn process_reaped(&mut self) {
        self._sockets -= 1;

        //  If Reaped was already asked to terminate and there are no more sockets,
        //  finish immediately.
        if (!self._sockets && terminating) {
            send_done();
            poller.rm_fd(mailbox_handle);
            poller.stop();
        }
    }
}

impl ZmqObject for ZmqReaper {
    fn set_ctx(&mut self, ctx: &mut ZmqContext) {
        todo!()
    }
}

impl ZmqPollEventsInterface for ZmqReaper {
    fn in_event(&mut self) {
        todo!()
    }

    fn out_event(&mut self) {
        todo!()
    }

    fn timer_event(&mut self, id: i32) {
        todo!()
    }
}

// ZmqReaper::~ZmqReaper ()
// {
//     LIBZMQ_DELETE (poller);
// }

// ZmqMailbox *ZmqReaper::get_mailbox () -> &mut ZmqMailbox
// {
//     return &mailbox;
// }

// void ZmqReaper::out_event ()
// {
//     // zmq_assert (false);
// }

// void ZmqReaper::timer_event
// {
//     // zmq_assert (false);
// }
