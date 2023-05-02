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
// #include "io_object.hpp"
// #include "io_thread.hpp"
// #include "err.hpp"
pub struct io_object_t : public i_poll_events
{
// public:
    io_object_t (ZmqThread *io_thread_ = null_mut());
    ~io_object_t () ;

    //  When migrating an object from one I/O thread to another, first
    //  unplug it, then migrate it, then plug it to the new thread.
    void plug (ZmqThread *io_thread_);
    void unplug ();

  protected:
    typedef Poller::handle_t handle_t;

    //  Methods to access underlying poller object.
    handle_t add_fd (ZmqFileDesc fd);
    void rm_fd (handle_t handle_);
    void set_pollin (handle_t handle_);
    void reset_pollin (handle_t handle_);
    void set_pollout (handle_t handle_);
    void reset_pollout (handle_t handle_);
    void add_timer (timeout: i32, id_: i32);
    void cancel_timer (id_: i32);

    //  i_poll_events interface implementation.
    void in_event () ;
    void out_event () ;
    void timer_event (id_: i32) ;

  // private:
    Poller *poller;

    // ZMQ_NON_COPYABLE_NOR_MOVABLE (io_object_t)
};

io_object_t::io_object_t (ZmqThread *io_thread_) : poller (null_mut())
{
    if (io_thread_)
        plug (io_thread_);
}

io_object_t::~io_object_t ()
{
}

void io_object_t::plug (ZmqThread *io_thread_)
{
    zmq_assert (io_thread_);
    zmq_assert (!poller);

    //  Retrieve the poller from the thread we are running in.
    poller = io_thread_.get_poller ();
}

void io_object_t::unplug ()
{
    zmq_assert (poller);

    //  Forget about old poller in preparation to be migrated
    //  to a different I/O thread.
    poller = null_mut();
}

io_object_t::handle_t io_object_t::add_fd (ZmqFileDesc fd)
{
    return poller.add_fd (fd, this);
}

void io_object_t::rm_fd (handle_t handle_)
{
    poller.rm_fd (handle_);
}

void io_object_t::set_pollin (handle_t handle_)
{
    poller.set_pollin (handle_);
}

void io_object_t::reset_pollin (handle_t handle_)
{
    poller.reset_pollin (handle_);
}

void io_object_t::set_pollout (handle_t handle_)
{
    poller.set_pollout (handle_);
}

void io_object_t::reset_pollout (handle_t handle_)
{
    poller.reset_pollout (handle_);
}

void io_object_t::add_timer (timeout: i32, id_: i32)
{
    poller.add_timer (timeout, this, id_);
}

void io_object_t::cancel_timer (id_: i32)
{
    poller.cancel_timer (this, id_);
}

void io_object_t::in_event ()
{
    zmq_assert (false);
}

void io_object_t::out_event ()
{
    zmq_assert (false);
}

void io_object_t::timer_event
{
    zmq_assert (false);
}
