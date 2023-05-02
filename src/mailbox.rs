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
use crate::ypipe::Ypipe;
use std::sync::Mutex;
use crate::signaler::ZmqSignaler;

pub const COMMAND_PIPE_GRANULARITY: i32 = 16;

// #include "precompiled.hpp"
// #include "mailbox.hpp"
// #include "err.hpp"
#[derive(Default,Debug,Clone)]
//   : public i_mailbox
pub struct mailbox_t
{
// public:
  // private:
    //  The pipe to store actual commands.
    // typedef Ypipe<ZmqCommand, command_pipe_granularity> cpipe_t;
    // cpipe_t cpipe;
    pub cpipe: Ypipe<ZmqCommand>,

    //  Signaler to pass signals from writer thread to reader thread.
    pub signaler: ZmqSignaler,

    //  There's only one thread receiving from the mailbox, but there
    //  is arbitrary number of threads sending. Given that ypipe requires
    //  synchronised access on both of its endpoints, we have to synchronise
    //  the sending side.
    // mutex_t sync;
    // TODO: figure out how to implement sync primitives
    pub sync: Mutex<()>,

    //  True if the underlying pipe is active, ie. when we are allowed to
    //  read commands from it.
    pub active: bool,

    // // ZMQ_NON_COPYABLE_NOR_MOVABLE (mailbox_t)
}

impl mailbox_t {
    // mailbox_t ();
    // mailbox_t::mailbox_t ()
    pub fn new() -> Self
    {
        //  Get the pipe into passive state. That way, if the users starts by
        //  polling on the associated file descriptor it will get woken up when
        //  new command is posted. const bool ok = cpipe.check_read ();
        // zmq_assert ( ! ok); active = false;
        Self {
            cpipe: Ypipe::new(),
            signaler: ZmqSignaler::new(),
            active: false,
            sync: Mutex::new(()),
        }
    }

    // ~mailbox_t ();
    // mailbox_t::~mailbox_t ()
    // {
    // //  TODO: Retrieve and deallocate commands inside the cpipe.
    //
    // // Work around problem that other threads might still be in our
    // // send() method, by waiting on the mutex before disappearing.
    // sync.lock (); sync.unlock ();
    // }

    // ZmqFileDesc get_fd () const;
    // ZmqFileDesc mailbox_t::get_fd () const
    pub fn get_fd(&mut self) -> ZmqFileDesc
    {
    return self.signaler.get_fd ();
    }

    // void send (const ZmqCommand &cmd);
    pub fn send(&mut self, cmd: &ZmqCommand) -> anyhow::Result<()>
    {
        // sync.lock ();
        let guard = self.sync.lock()?;
        self.cpipe.write (cmd, false);
        let ok = self.cpipe.flush ();
        //sync.unlock ();
        std::mem::drop(guard);

        if ( ! ok) {
            self.signaler.send();
        }

        Ok(())
    }

    // int recv (cmd: &mut ZmqCommand timeout: i32);
    pub fn recv(&mut self, cmd: &mut ZmqCommand, timeout: i32) -> anyhow::Result<()> {
        //  Try to get the command straight away.
        if (active) {
            if (self.cpipe.read(cmd)) { return 0; }

            //  If there are no more commands available, switch into passive state.
            self.active = false;
        }

        //  Wait for signal from the command sender.
        self.signaler.wait(timeout)?;
        // if (rc == - 1) {
        //     // errno_assert (errno == EAGAIN | | errno == EINTR); return - 1;
        // }

        //  Receive the signal.
        self.signaler.recv_failable()?;
        // if (rc == - 1) {
        //     // errno_assert (errno == EAGAIN); return - 1;
        // }

        //  Switch into active state.
        self.active = true;

        //  Get a command. const bool ok = cpipe.read (cmd); zmq_assert (ok);
        // return 0;
        Ok(())
    }

// bool valid () const;
    bool mailbox_t::valid () const {
    return signaler.valid ();
    }

    // #ifdef HAVE_FORK
    // close the file descriptors in the signaller. This is used in a forked
    // child process to close the file descriptors so that they do not interfere
    // with the context in the parent process.
    void forked ()
    {
    signaler.forked ();
    }
    // #endif
}