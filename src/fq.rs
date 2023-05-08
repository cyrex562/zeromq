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
// #include "fq.hpp"
// #include "pipe.hpp"
// #include "err.hpp"
// #include "msg.hpp"

use crate::message::{ZmqMessage, ZMQ_MSG_MORE};
use crate::pipe::ZmqPipe;

#[derive(Default, Debug, Clone)]
pub struct ZmqFq {
    //
    //     ZmqFq ();
    //     ~ZmqFq ();
    //
    //     void attach (pipe: &mut ZmqPipe);
    //     void activated (pipe: &mut ZmqPipe);
    //     void pipe_terminated (pipe: &mut ZmqPipe);
    //
    //     int recv (msg: &mut ZmqMessage);
    //     int recvpipe (msg: &mut ZmqMessage ZmqPipe **pipe);
    //     bool has_in ();

    //
    //  Inbound pipes.
    // typedef array_t<ZmqPipe, 1> pipes_t;
    // pipes_t pipes;
    pub pipes: Vec<ZmqPipe>,

    //  Number of active pipes. All the active pipes are located at the
    //  beginning of the pipes array.
    // pipes_t::size_type active;
    pub active: usize,

    //  Index of the next bound pipe to read a message from.
    // pipes_t::size_type _current;
    pub _current: usize,

    //  If true, part of a multipart message was already received, but
    //  there are following parts still waiting in the current pipe.
    pub more: bool, // // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqFq)
}

impl ZmqFq {
    // ZmqFq::ZmqFq () : active (0), _current (0), more (false)
    // {
    // }
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    // ZmqFq::~ZmqFq ()
    // {
    // zmq_assert (pipes.empty ());
    // }

    pub fn attach(&mut self, pipe: &mut ZmqPipe) {
        self.pipes.push_back(pipe);
        self.pipes.swap(self.active, self.pipes.size() - 1);
        self.active += 1;
    }

    pub fn pipe_terminated(&mut self, pipe: &mut ZmqPipe) {
        let index = self.pipes.binary_search(pipe).unwrap();

        //  Remove the pipe from the list; adjust number of active pipes
        //  accordingly.
        if (index < self.active) {
            self.active -= 1;
            self.pipes.swap(index, self.active);
            if (_current == self.active) {
                _current = 0;
            }
            self.pipes.erase(pipe);
        }
    }

    pub fn activated(&mut self, pipe: &mut ZmqPipe) {
        //  Move the pipe to the list of active pipes.
        pipes.swap(pipes.index(pipe), self.active);
        self.active += 1;
    }

    pub fn recv(&mut self, msg: &mut ZmqMessage) -> i32 {
        return self.recvpipe(msg, None);
    }

    pub fn recvpipe(&mut self, msg: &mut ZmqMessage, pipe: Option<&mut ZmqPipe>) -> i32 {
        //  Deallocate old content of the message.
        let mut rc = msg.close();
        // errno_assert (rc == 0);

        //  Round-robin over the pipes to get the next message.
        while (self.active > 0) {
            //  Try to fetch new message. If we've already read part of the message
            //  subsequent part should be immediately available.
            let fetched = self.pipes[self._current].read(msg);

            //  Note that when message is not fetched, current pipe is deactivated
            //  and replaced by another active pipe. Thus we don't have to increase
            //  the 'current' pointer.
            if (fetched) {
                if (pipe.is_some()) {
                    pipe.unwrap().replace(self.pipes[self._current].clone()) // = pipes[_current];
                }
                self.more = (msg.flags() & ZMQ_MSG_MORE) != 0;
                if (!self.more) {
                    self._current = (self._current + 1) % self.active;
                }
                return 0;
            }

            //  Check the atomicity of the message.
            //  If we've already received the first part of the message
            //  we should get the remaining parts without blocking.
            // zmq_assert (!more);

            self.active -= 1;
            self.pipes.swap(self._current, self.active);
            if (self._current == self.active) {
                self._current = 0;
            }
        }

        //  No message is available. Initialise the output parameter
        //  to be a 0-byte message.
        msg.init2();
        // errno_assert (rc == 0);
        // errno = EAGAIN;
        return -1;
    }

    pub fn has_in(&mut self) -> bool {
        //  There are subsequent parts of the partly-read message available.
        if (more) {
            return true;
        }

        //  Note that messing with current doesn't break the fairness of fair
        //  queueing algorithm. If there are no messages available current will
        //  get back to its original value. Otherwise it'll point to the first
        //  pipe holding messages, skipping only pipes with no messages available.
        while (self.active > 0) {
            if (self.pipes[self._current].check_read()) {
                return true;
            }

            //  Deactivate the pipe.
            self.active -= 1;
            self.pipes.swap(self._current, self.active);
            if (self._current == self.active) {
                self._current = 0;
            }
        }

        return false;
    }
}
