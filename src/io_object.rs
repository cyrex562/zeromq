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
// #include "io_object.hpp"
// #include "io_thread.hpp"
// #include "err.hpp"
pub struct io_object_t : public i_poll_events
{
// public:
    io_object_t (io_thread_t *io_thread_ = NULL);
    ~io_object_t () ZMQ_OVERRIDE;

    //  When migrating an object from one I/O thread to another, first
    //  unplug it, then migrate it, then plug it to the new thread.
    void plug (io_thread_t *io_thread_);
    void unplug ();

  protected:
    typedef poller_t::handle_t handle_t;

    //  Methods to access underlying poller object.
    handle_t add_fd (fd_t fd_);
    void rm_fd (handle_t handle_);
    void set_pollin (handle_t handle_);
    void reset_pollin (handle_t handle_);
    void set_pollout (handle_t handle_);
    void reset_pollout (handle_t handle_);
    void add_timer (timeout_: i32, id_: i32);
    void cancel_timer (id_: i32);

    //  i_poll_events interface implementation.
    void in_event () ZMQ_OVERRIDE;
    void out_event () ZMQ_OVERRIDE;
    void timer_event (id_: i32) ZMQ_OVERRIDE;

  // private:
    poller_t *_poller;

    ZMQ_NON_COPYABLE_NOR_MOVABLE (io_object_t)
};

io_object_t::io_object_t (io_thread_t *io_thread_) : _poller (NULL)
{
    if (io_thread_)
        plug (io_thread_);
}

io_object_t::~io_object_t ()
{
}

void io_object_t::plug (io_thread_t *io_thread_)
{
    zmq_assert (io_thread_);
    zmq_assert (!_poller);

    //  Retrieve the poller from the thread we are running in.
    _poller = io_thread_->get_poller ();
}

void io_object_t::unplug ()
{
    zmq_assert (_poller);

    //  Forget about old poller in preparation to be migrated
    //  to a different I/O thread.
    _poller = NULL;
}

io_object_t::handle_t io_object_t::add_fd (fd_t fd_)
{
    return _poller->add_fd (fd_, this);
}

void io_object_t::rm_fd (handle_t handle_)
{
    _poller->rm_fd (handle_);
}

void io_object_t::set_pollin (handle_t handle_)
{
    _poller->set_pollin (handle_);
}

void io_object_t::reset_pollin (handle_t handle_)
{
    _poller->reset_pollin (handle_);
}

void io_object_t::set_pollout (handle_t handle_)
{
    _poller->set_pollout (handle_);
}

void io_object_t::reset_pollout (handle_t handle_)
{
    _poller->reset_pollout (handle_);
}

void io_object_t::add_timer (timeout_: i32, id_: i32)
{
    _poller->add_timer (timeout_, this, id_);
}

void io_object_t::cancel_timer (id_: i32)
{
    _poller->cancel_timer (this, id_);
}

void io_object_t::in_event ()
{
    zmq_assert (false);
}

void io_object_t::out_event ()
{
    zmq_assert (false);
}

void io_object_t::timer_event (int)
{
    zmq_assert (false);
}
