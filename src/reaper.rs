/*
    Copyright (c) 2007-2016 Contributors as noted in the AUTHORS file

    This file is part of libzmq, the ZeroMQ core engine in C++.

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
// #include "macros.hpp"
// #include "reaper.hpp"
// #include "socket_base.hpp"
// #include "err.hpp"
pub struct reaper_t ZMQ_FINAL : public object_t, public i_poll_events
{
// public:
    reaper_t (ZmqContext *ctx, u32 tid_);
    ~reaper_t ();

    mailbox_t *get_mailbox ();

    void start ();
    void stop ();

    //  i_poll_events implementation.
    void in_event ();
    void out_event ();
    void timer_event (id_: i32);

  // private:
    //  Command handlers.
    void process_stop ();
    void process_reap (ZmqSocketBase *socket_);
    void process_reaped ();

    //  Reaper thread accesses incoming commands via this mailbox.
    mailbox_t _mailbox;

    //  Handle associated with mailbox' file descriptor.
    poller_t::handle_t _mailbox_handle;

    //  I/O multiplexing is performed using a poller object.
    poller_t *_poller;

    //  Number of sockets being reaped at the moment.
    _sockets: i32;

    //  If true, we were already asked to terminate.
    _terminating: bool

// #ifdef HAVE_FORK
    // the process that created this context. Used to detect forking.
    pid_t _pid;
// #endif

    ZMQ_NON_COPYABLE_NOR_MOVABLE (reaper_t)
};

reaper_t::reaper_t (class ZmqContext *ctx, u32 tid_) :
    object_t (ctx, tid_),
    _mailbox_handle (static_cast<poller_t::handle_t> (null_mut())),
    _poller (null_mut()),
    _sockets (0),
    _terminating (false)
{
    if (!_mailbox.valid ())
        return;

    _poller = new (std::nothrow) poller_t (*ctx);
    alloc_assert (_poller);

    if (_mailbox.get_fd () != retired_fd) {
        _mailbox_handle = _poller->add_fd (_mailbox.get_fd (), this);
        _poller->set_pollin (_mailbox_handle);
    }

// #ifdef HAVE_FORK
    _pid = getpid ();
// #endif
}

reaper_t::~reaper_t ()
{
    LIBZMQ_DELETE (_poller);
}

mailbox_t *reaper_t::get_mailbox ()
{
    return &_mailbox;
}

void reaper_t::start ()
{
    zmq_assert (_mailbox.valid ());

    //  Start the thread.
    _poller->start ("Reaper");
}

void reaper_t::stop ()
{
    if (get_mailbox ()->valid ()) {
        send_stop ();
    }
}

void reaper_t::in_event ()
{
    while (true) {
// #ifdef HAVE_FORK
        if (unlikely (_pid != getpid ())) {
            //printf("reaper_t::in_event return in child process %d\n", (int)getpid());
            return;
        }
// #endif

        //  Get the next command. If there is none, exit.
        ZmqCommand cmd;
        let rc: i32 = _mailbox.recv (&cmd, 0);
        if (rc != 0 && errno == EINTR)
            continue;
        if (rc != 0 && errno == EAGAIN)
            break;
        errno_assert (rc == 0);

        //  Process the command.
        cmd.destination->process_command (cmd);
    }
}

void reaper_t::out_event ()
{
    zmq_assert (false);
}

void reaper_t::timer_event (int)
{
    zmq_assert (false);
}

void reaper_t::process_stop ()
{
    _terminating = true;

    //  If there are no sockets being reaped finish immediately.
    if (!_sockets) {
        send_done ();
        _poller->rm_fd (_mailbox_handle);
        _poller->stop ();
    }
}

void reaper_t::process_reap (ZmqSocketBase *socket_)
{
    //  Add the socket to the poller.
    socket_->start_reaping (_poller);

    ++_sockets;
}

void reaper_t::process_reaped ()
{
    --_sockets;

    //  If reaped was already asked to terminate and there are no more sockets,
    //  finish immediately.
    if (!_sockets && _terminating) {
        send_done ();
        _poller->rm_fd (_mailbox_handle);
        _poller->stop ();
    }
}
