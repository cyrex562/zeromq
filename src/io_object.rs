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

use std::ptr::null_mut;
use crate::defines::ZmqHandle;
use crate::fd::ZmqFileDesc;
use crate::io_thread::ZmqIoThread;
use crate::poll_events_interface::ZmqPollEventsInterface;

// #include "precompiled.hpp"
// #include "io_object.hpp"
// #include "io_thread.hpp"
// #include "err.hpp"
#[derive(Default,Debug,Clone)]
pub struct ZmqIoObject
{
    // pub ZmqPollEventsInterface: ZmqPollEventsInterface,
    pub poller: Option<ZmqHandle>,
    // ZMQ_NON_COPYABLE_NOR_MOVABLE (io_object_t)
}

impl ZmqIoObject {
    // ZmqIoObject (ZmqIoThread *io_thread_ = null_mut());
    pub fn new(io_thread_: Option<ZmqIoThread>) -> Self
    {
// : poller (null_mut())
//     if (io_thread_)
//         plug (io_thread_);
        let mut out = Self {
            // ZmqPollEventsInterface: Default::default(),
            poller: None,
        };
        if io_thread_.is_some() {
            out.plug(&mut io_thread_.unwrap());
        }

        out
    }

    // ~ZmqIoObject () ;

    //  When migrating an object from one I/O thread to another, first
    //  unplug it, then migrate it, then plug it to the new thread.
    // void plug (ZmqIoThread *io_thread_);
    pub fn plug (&mut self, io_thread: &mut ZmqIoThread)
    {
        // zmq_assert (io_thread_);
        // zmq_assert (!poller);
        //  Retrieve the poller from the thread we are running in.
        self.poller = io_thread_.get_poller ();
    }

    // void unplug ();
    pub fn unplug (&mut self)
    {
        // zmq_assert (poller);
        //  Forget about old poller in preparation to be migrated
        //  to a different I/O thread.
        self.poller = None;
    }

    //  Methods to access underlying poller object.
    // handle_t add_fd (ZmqFileDesc fd);
    pub fn add_fd (&mut self, fd: ZmqFileDesc) -> handle_t
    {
        return self.poller.add_fd (fd, this);
    }

    // void rm_fd (handle_t handle_);
    pub fn rm_fd (&mut self, handle: ZmqHandle)
    {
        self.poller.rm_fd (handle_);
    }

    // void set_pollin (handle_t handle_);
    pub fn set_pollin (&mut self, handle_: handle_t)
    {
        self.poller.set_pollin (handle_);
    }

    // void reset_pollin (handle_t handle_);
    pub fn reset_pollin (&mut self, handle_: handle_t)
    {
        self.poller.reset_pollin (handle_);
    }

    // void set_pollout (handle_t handle_);
    pub fn set_pollout (&mut self, handle_: handle_t)
    {
        self.poller.set_pollout (handle_);
    }

    // void reset_pollout (handle_t handle_);
    pub fn reset_pollout (&mut self, handle_: handle_t)
    {
        self.poller.reset_pollout (handle_);
    }

    // void add_timer (timeout: i32, id_: i32);
    pub fn add_timer (&mut self, timeout: i32, id_: i32)
    {
        self.poller.add_timer (timeout, self, id_);
    }

    // void cancel_timer (id_: i32);
    pub fn cancel_timer (&mut self, id_: i32)
    {
        self.poller.cancel_timer (self, id_);
    }

}

impl ZmqPollEventsInterface for ZmqIoObject {
    //  i_poll_events interface implementation.
    // void in_event () ;
    fn in_event (&mut self)
    {
        // zmq_assert (false);
    }

    // void out_event () ;
    fn out_event (&mut self)
    {
        // zmq_assert (false);
    }

    // void timer_event (id_: i32) ;
    fn timer_event(&mut self, id_: i32)
    {
    // zmq_assert (false);
    }
}




// ZmqIoObject::~ZmqIoObject ()
// {
// }


























