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

// #include <new>

// #include "macros.hpp"
// #include "io_thread.hpp"
// #include "err.hpp"
// #include "ctx.hpp"
pub struct io_thread_t ZMQ_FINAL : public ZmqObject, public i_poll_events
{
// public:
    io_thread_t (ZmqContext *ctx, u32 tid_);

    //  Clean-up. If the thread was started, it's necessary to call 'stop'
    //  before invoking destructor. Otherwise the destructor would hang up.
    ~io_thread_t ();

    //  Launch the physical thread.
    void start ();

    //  Ask underlying thread to stop.
    void stop ();

    //  Returns mailbox associated with this I/O thread.
    mailbox_t *get_mailbox ();

    //  i_poll_events implementation.
    void in_event ();
    void out_event ();
    void timer_event (id_: i32);

    //  Used by io_objects to retrieve the associated poller object.
    poller_t *get_poller () const;

    //  Command handlers.
    void process_stop ();

    //  Returns load experienced by the I/O thread.
    int get_load () const;

  // private:
    //  I/O thread accesses incoming commands via this mailbox.
    mailbox_t _mailbox;

    //  Handle associated with mailbox' file descriptor.
    poller_t::handle_t _mailbox_handle;

    //  I/O multiplexing is performed using a poller object.
    poller_t *_poller;

    ZMQ_NON_COPYABLE_NOR_MOVABLE (io_thread_t)
};

io_thread_t::io_thread_t (ZmqContext *ctx, u32 tid_) :
    ZmqObject (ctx, tid_),
    _mailbox_handle (static_cast<poller_t::handle_t> (null_mut()))
{
    _poller = new (std::nothrow) poller_t (*ctx);
    alloc_assert (_poller);

    if (_mailbox.get_fd () != retired_fd) {
        _mailbox_handle = _poller.add_fd (_mailbox.get_fd (), this);
        _poller.set_pollin (_mailbox_handle);
    }
}

io_thread_t::~io_thread_t ()
{
    LIBZMQ_DELETE (_poller);
}

void io_thread_t::start ()
{
    char name[16] = "";
    snprintf (name, mem::size_of::<name>(), "IO/%u",
              get_tid () - ZmqContext::REAPER_TID - 1);
    //  Start the underlying I/O thread.
    _poller.start (name);
}

void io_thread_t::stop ()
{
    send_stop ();
}

mailbox_t *io_thread_t::get_mailbox ()
{
    return &_mailbox;
}

int io_thread_t::get_load () const
{
    return _poller.get_load ();
}

void io_thread_t::in_event ()
{
    //  TODO: Do we want to limit number of commands I/O thread can
    //  process in a single go?

    ZmqCommand cmd;
    int rc = _mailbox.recv (&cmd, 0);

    while (rc == 0 || errno == EINTR) {
        if (rc == 0)
            cmd.destination.process_command (cmd);
        rc = _mailbox.recv (&cmd, 0);
    }

    errno_assert (rc != 0 && errno == EAGAIN);
}

void io_thread_t::out_event ()
{
    //  We are never polling for POLLOUT here. This function is never called.
    zmq_assert (false);
}

void io_thread_t::timer_event (int)
{
    //  No timers here. This function is never called.
    zmq_assert (false);
}

poller_t *io_thread_t::get_poller () const
{
    zmq_assert (_poller);
    return _poller;
}

void io_thread_t::process_stop ()
{
    zmq_assert (_mailbox_handle);
    _poller.rm_fd (_mailbox_handle);
    _poller.stop ();
}
