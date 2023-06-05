/*
    Copyright (c) 2007-2016 Contributors as noted in the AUTHORS file

    This file is part of libzmq, the ZeroMQ core engine in C+= 1.

    libzmq is free software; you can redistribute it and/or modify it under
    the terms of the GNU Lesser General Public License (LGPL) as published
    by the Free Software Foundation; either version 3 of the License, or
    (at your option) any later version.

    As a special exception, the Contributors give you permission to link
    this library with independent modules to produce an executable,
    regardless of the license terms of these independent modules, and to
    copy and distribute the resulting executable under terms of your choice,
    provided that you also meet, for each linked independent module, the
    terms and conditions of the license of that module. An independent
    module is a module which is not derived from or based on this library.
    If you modify this library, you must extend this exception to your
    version of the library.

    libzmq is distributed in the hope that it will be useful, but WITHOUT
    ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
    FITNESS FOR A PARTICULAR PURPOSE. See the GNU Lesser General Public
    License for more details.

    You should have received a copy of the GNU Lesser General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

use crate::command::ZmqCommand;
use crate::context::ZmqContext;
use crate::defines::ZmqHandle;
use crate::devpoll::ZmqPoller;
use crate::mailbox::ZmqMailbox;
use crate::object::ZmqObject;
use crate::poll_events_interface::ZmqPollEventsInterface;
use crate::socket_base::ZmqSocketBase;
use libc::{EAGAIN, EINTR};

// #include "precompiled.hpp"
// #include "macros.hpp"
// #include "reaper.hpp"
// #include "socket_base.hpp"
// #include "err.hpp"
pub struct ZmqReaper {
    // : public ZmqObject
    // pub object: ZmqObject,
    // public ZmqPollEventsInterface

    // ZmqReaper (ctx: &mut ZmqContext, tid: u32);

    // ~ZmqReaper ();

    // ZmqMailbox *get_mailbox ();

    // void start ();

    // void stop ();

    //  i_poll_events implementation.

    // void in_event ();

    // void out_event ();

    // void timer_event (id_: i32);

    //  Command handlers.
    // void process_stop ();

    // void process_reap (ZmqSocketBase *socket);

    // void process_reaped ();

    //  Reaper thread accesses incoming commands via this mailbox.
    // ZmqMailbox mailbox;
    pub mailbox: ZmqMailbox,
    //  Handle associated with mailbox' file descriptor.
    // Poller::handle_t mailbox_handle;
    pub mailbox_handle: ZmqHandle,
    //  I/O multiplexing is performed using a poller object.
    // Poller *poller;
    pub poller: ZmqPoller,
    //  Number of sockets being reaped at the moment.
    pub _sockets: i32,
    //  If true, we were already asked to terminate.
    pub terminating: bool,
    // #ifdef HAVE_FORK
    // the process that created this context. Used to detect forking.
    // pid_t _pid;
    // #endif

    // ZMQ_NON_COPYABLE_NOR_MOVABLE (reaper_t)
}

impl ZmqReaper {
    pub fn new(ctx: &mut ZmqContext, tid: u32) -> Self {
        //  :
        //     ZmqObject (ctx, tid),
        //     mailbox_handle (static_cast<Poller::handle_t> (null_mut())),
        //     poller (null_mut()),
        //     self._sockets (0),
        //     terminating (false)
        let mut out = Self {
            mailbox: Default::default(),
            mailbox_handle: 0,
            poller: ZmqPoller::new(ctx),
            _sockets: 0,
            terminating: false,
        };

        // if (!mailbox.valid ()) {
        //     return;
        // }

        // poller =  Poller (*ctx);
        // alloc_assert (poller);

        if (out.mailbox.get_fd() != retired_fd) {
            out.mailbox_handle = out.poller.add_fd(mailbox.get_fd(), &mut out);
            out.poller.set_pollin(&out.mailbox_handle);
        }

        // #ifdef HAVE_FORK
        //         _pid = getpid ();
        // #endif
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
            let mut cmd = ZmqCommand::default();
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

        //  If there are no sockets being reaped finish immediately.
        if (!self._sockets) {
            send_done();
            self.poller.rm_fd(mailbox_handle);
            self.poller.stop();
        }
    }

    pub fn process_reap(&mut self, socket: &mut ZmqSocketBase) {
        //  Add the socket to the poller.
        socket.start_reaping(poller);
        self._sockets += 1;
    }

    pub fn process_reaped(&mut self) {
        self._sockets -= 1;

        //  If reaped was already asked to terminate and there are no more sockets,
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
