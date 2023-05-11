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

// #include "precompiled.hpp"
// #include "mailbox_safe.hpp"
// #include "clock.hpp"
// #include "err.hpp"

use libc::EAGAIN;
use crate::command::ZmqCommand;
use crate::mailbox_interface::ZmqMailboxInterface;
use crate::signaler::ZmqSignaler;
use crate::ypipe::Ypipe;

// #include <algorithm>
pub struct ZmqMailboxSafe {
    // : public ZmqMailboxInterface
//
// #endif
    //
    //  The pipe to store actual commands.
    // typedef Ypipe<ZmqCommand, command_pipe_granularity> cpipe_t;
    // cpipe_t cpipe;
    //   pub cpipe: Ypipe<ZmqCommand, command_pipe_granularity>,
    //  Condition variable to pass signals from writer thread to reader thread.
    // condition_variable_t _cond_var;
    pub _cond_var: condition_variable_t,
    //  Synchronize access to the mailbox from receivers and senders
    // mutex_t *const sync;
    pub sync: *mut mutex_t,
    // std::vector<ZmqSignaler *> _signalers;
    pub _signalers: Vec<ZmqSignaler>,
    // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqMailboxSafe)
}

impl ZmqMailboxInterface for ZmqMailboxSafe {
    fn send(&mut self, cmd: &ZmqCommand) {
        todo!()
    }

    fn recv(&mut self, cmd: &mut ZmqCommand, timeout: i32) -> i32 {
        todo!()
    }
}

impl ZmqMailboxSafe {
    // ZmqMailboxSafe (mutex_t *sync_);
    pub fn new(sync_: &mut mutex_t) -> Self {
        // : sync (sync_)
        //  Get the pipe into passive state. That way, if the users starts by
        //  polling on the associated file descriptor it will get woken up when
        //  new command is posted.
        // const bool ok = cpipe.check_read ();
        // zmq_assert (!ok);
        Self {
            sync: sync_,
            // cpipe: cpipe_t::new(),
            _cond_var: condition_variable_t::new(),
            _signalers: vec![],
        }
    }


    // ~ZmqMailboxSafe ();

    // void send (const ZmqCommand &cmd);

    // int recv (cmd: &mut ZmqCommand timeout: i32);

    // Add signaler to mailbox which will be called when a message is ready
    // void add_signaler (ZmqSignaler *signaler_);
    pub fn add_signaler(&mut self, signaler: &mut ZmqSignaler) {
        sel_signalers.push_back(signaler_);
    }

    // void remove_signaler (ZmqSignaler *signaler_);

    // void clear_signalers ();

    // #ifdef HAVE_FORK
    // close the file descriptors in the signaller. This is used in a forked
    // child process to close the file descriptors so that they do not interfere
    // with the context in the parent process.
    pub fn forked() {
        // TODO: call fork on the condition variable
    }

    pub fn remove_signaler(&mut self, signaler_: &mut ZmqSignaler) {
        // TODO: make a copy of array and signal outside the lock
        // const std::vector<ZmqSignaler *>::iterator end = _signalers.end ();
        // const std::vector<ZmqSignaler *>::iterator it =
        //   std::find (_signalers.begin (), end, signaler_);
        // if (it != end) {
        //     _signalers.erase(it);
        // }
    }

    pub fn clear_signalers(&mut self) {
        self._signalers.clear();
    }


    pub fn send(&mut self, cmd: &ZmqCommand) {
        sync.lock();
        // cpipe.write (cmd, false);
        // let ok = cpipe.flush ();

        if (!ok) {
            _cond_var.broadcast();

            // for (std::vector<ZmqSignaler *>::iterator it = _signalers.begin (),
            //                                          end = _signalers.end ();
            //      it != end; += 1it) {
            //     (*it)->send ();
            // }
        }

        sync.unlock();
    }

    pub fn recv(&mut self, cmd: &mut ZmqCommand, timeout: i32) -> i32 {
        //  Try to get the command straight away.
        if (cpipe.read(cmd)) {
            return 0;
        }

        //  If the timeout is zero, it will be quicker to release the lock, giving other a chance to send a command
        //  and immediately relock it.
        if (timeout == 0) {
            sync.unlock();
            sync.lock();
        } else {
            //  Wait for signal from the command sender.
            let rc: i32 = _cond_var.wait(sync, timeout);
            if (rc == -1) {
                // errno_assert (errno == EAGAIN || errno == EINTR);
                return -1;
            }
        }

        //  Another thread may already fetch the command
        let ok = cpipe.read(cmd);

        if (!ok) {
            errno = EAGAIN;
            return -1;
        }

        return 0;
    }
}

// ZmqMailboxSafe::~ZmqMailboxSafe ()
// {
//     //  TODO: Retrieve and deallocate commands inside the cpipe.
//
//     // Work around problem that other threads might still be in our
//     // send() method, by waiting on the mutex before disappearing.
//     sync.lock ();
//     sync.unlock ();
// }






