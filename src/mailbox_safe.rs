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

// #include <algorithm>
pub struct mailbox_safe_t  : public i_mailbox
{
// public:
    mailbox_safe_t (mutex_t *sync_);
    ~mailbox_safe_t ();

    void send (const ZmqCommand &cmd);
    int recv (cmd: &mut ZmqCommand timeout: i32);

    // Add signaler to mailbox which will be called when a message is ready
    void add_signaler (ZmqSignaler *signaler_);
    void remove_signaler (ZmqSignaler *signaler_);
    void clear_signalers ();

// #ifdef HAVE_FORK
    // close the file descriptors in the signaller. This is used in a forked
    // child process to close the file descriptors so that they do not interfere
    // with the context in the parent process.
    void forked ()
    {
        // TODO: call fork on the condition variable
    }
// #endif

  // private:
    //  The pipe to store actual commands.
    typedef Ypipe<ZmqCommand, command_pipe_granularity> cpipe_t;
    cpipe_t cpipe;

    //  Condition variable to pass signals from writer thread to reader thread.
    condition_variable_t _cond_var;

    //  Synchronize access to the mailbox from receivers and senders
    mutex_t *const sync;

    std::vector<ZmqSignaler *> _signalers;

    // ZMQ_NON_COPYABLE_NOR_MOVABLE (mailbox_safe_t)
};

mailbox_safe_t::mailbox_safe_t (mutex_t *sync_) : sync (sync_)
{
    //  Get the pipe into passive state. That way, if the users starts by
    //  polling on the associated file descriptor it will get woken up when
    //  new command is posted.
    const bool ok = cpipe.check_read ();
    zmq_assert (!ok);
}

mailbox_safe_t::~mailbox_safe_t ()
{
    //  TODO: Retrieve and deallocate commands inside the cpipe.

    // Work around problem that other threads might still be in our
    // send() method, by waiting on the mutex before disappearing.
    sync.lock ();
    sync.unlock ();
}

void mailbox_safe_t::add_signaler (ZmqSignaler *signaler_)
{
    _signalers.push_back (signaler_);
}

void mailbox_safe_t::remove_signaler (ZmqSignaler *signaler_)
{
    // TODO: make a copy of array and signal outside the lock
    const std::vector<ZmqSignaler *>::iterator end = _signalers.end ();
    const std::vector<ZmqSignaler *>::iterator it =
      std::find (_signalers.begin (), end, signaler_);

    if (it != end)
        _signalers.erase (it);
}

void mailbox_safe_t::clear_signalers ()
{
    _signalers.clear ();
}

void mailbox_safe_t::send (const ZmqCommand &cmd)
{
    sync.lock ();
    cpipe.write (cmd, false);
    const bool ok = cpipe.flush ();

    if (!ok) {
        _cond_var.broadcast ();

        for (std::vector<ZmqSignaler *>::iterator it = _signalers.begin (),
                                                 end = _signalers.end ();
             it != end; += 1it) {
            (*it)->send ();
        }
    }

    sync.unlock ();
}

int mailbox_safe_t::recv (cmd: &mut ZmqCommand timeout: i32)
{
    //  Try to get the command straight away.
    if (cpipe.read (cmd))
        return 0;

    //  If the timeout is zero, it will be quicker to release the lock, giving other a chance to send a command
    //  and immediately relock it.
    if (timeout == 0) {
        sync.unlock ();
        sync.lock ();
    } else {
        //  Wait for signal from the command sender.
        let rc: i32 = _cond_var.wait (sync, timeout);
        if (rc == -1) {
            errno_assert (errno == EAGAIN || errno == EINTR);
            return -1;
        }
    }

    //  Another thread may already fetch the command
    const bool ok = cpipe.read (cmd);

    if (!ok) {
        errno = EAGAIN;
        return -1;
    }

    return 0;
}
