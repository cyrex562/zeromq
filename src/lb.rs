/*
    Copyright (c) 2007-2018 Contributors as noted in the AUTHORS file

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
// #include "lb.hpp"
// #include "pipe.hpp"
// #include "err.hpp"
// #include "msg.hpp"

use crate::message::{ZmqMessage, ZMQ_MSG_MORE};
use crate::pipe::{pipe_erase, pipe_index, pipe_swap, ZmqPipe};
use anyhow::anyhow;
use libc::EAGAIN;
use std::ptr::null_mut;

//  This class manages a set of outbound pipes. On send it load balances
//  messages fairly among the pipes.
pub struct LoadBalancer {
    //  List of outbound pipes.
    // typedef array_t<ZmqPipe, 2> pipes_t;
    // pipes_t pipes;
    pub pipes: [ZmqPipe; 2],
    //  Number of active pipes. All the active pipes are located at the
    //  beginning of the pipes array.
    // pipes_t::size_type active;
    pub active: usize,
    //  Points to the last pipe that the most recent message was sent to.
    // pipes_t::size_type _current;
    pub _current: usize,
    //  True if last we are in the middle of a multipart message.
    pub more: bool,
    //  True if we are dropping current message.
    pub _dropping: bool,
    // ZMQ_NON_COPYABLE_NOR_MOVABLE (LoadBalancer)
}

impl LoadBalancer {
    // LoadBalancer ();
    pub fn new() -> Self {
        // : active (0), _current (0), more (false), _dropping (false)
        Self {
            pipes: [ZmqPipe::default(), ZmqPipe::default()],
            active: 0,
            _current: 0,
            more: false,
            _dropping: false,
        }
    }

    // ~LoadBalancer ();

    // void attach (pipe: &mut ZmqPipe);
    pub fn attach(&mut self, pipe: &mut ZmqPipe) {
        self.pipes.push_back(pipe);
        self.activated(pipe);
    }

    // void activated (pipe: &mut ZmqPipe);
    pub fn activated(&mut self, pipe: &mut ZmqPipe) {
        //  Move the pipe to the list of active pipes.
        pipes.swap(pipes.index(pipe), self.active);
        self.active += 1;
    }

    // void pipe_terminated (pipe: &mut ZmqPipe);
    pub fn pipe_terminated(&mut self, pipe: &mut ZmqPipe) {
        let index = pipe_index(pipe, &self.pipes);

        //  If we are in the middle of multipart message and current pipe
        //  have disconnected, we have to drop the remainder of the message.
        if (index == self._current && more) {
            self._dropping = true;
        }

        //  Remove the pipe from the list; adjust number of active pipes
        //  accordingly.
        if (index < self.active as i32) {
            self.active -= 1;
            pipe_swap(index as usize, self.active, &mut self.pipes);
            if (_current == self.active) {
                _current = 0;
            }
        }
        pipe_erase(pipe, &self.pipes);
    }

    // int send (msg: &mut ZmqMessage);
    pub fn send(msg: &mut ZmqMessage) -> i32 {
        return sendpipe(msg, null_mut());
    }

    //  Sends a message and stores the pipe that was used in pipe_.
    //  It is possible for this function to return success but keep pipe_
    //  unset if the rest of a multipart message to a terminated pipe is
    //  being dropped. For the first frame, this will never happen.
    // int sendpipe (msg: &mut ZmqMessage ZmqPipe **pipe);

    pub fn sendpipe(
        &mut self,
        msg: &mut ZmqMessage,
        pipe: Option<&mut &mut [ZmqPipe]>,
    ) -> anyhow::Result<()> {
        //  Drop the message if required. If we are at the end of the message
        //  switch back to non-dropping mode.
        if (self._dropping) {
            more = (msg.flags() & ZMQ_MSG_MORE) != 0;
            _dropping = more;

            let mut rc = msg.close();
            // errno_assert (rc == 0);
            msg.init2();
            // errno_assert (rc == 0);
            return Ok(());
        }

        while (self.active > 0) {
            if (pipes[self._current].write(msg)) {
                if (pipe) {
                    *pipe = pipes[self._current];
                }
                break;
            }

            // If send fails for multi-part msg rollback other
            // parts sent earlier and return EAGAIN.
            // Application should handle this as suitable
            if (more) {
                pipes[self._current].rollback();
                // At this point the pipe is already being deallocated
                // and the first N frames are unreachable (_outpipe is
                // most likely already NULL so rollback won't actually do
                // anything and they can't be un-written to deliver later).
                // Return EFAULT to socket_base caller to drop current message
                // and any other subsequent frames to avoid them being
                // "stuck" and received when a new client reconnects, which
                // would break atomicity of multi-part messages (in blocking mode
                // socket_base just tries again and again to send the same message)
                // Note that given dropping mode returns 0, the user will
                // never know that the message could not be delivered, but
                // can't really fix it without breaking backward compatibility.
                // -2/EAGAIN will make sure socket_base caller does not re-enter
                // immediately or after a short sleep in blocking mode.
                self._dropping = (msg.flags() & ZMQ_MSG_MORE) != 0;
                more = false;
                errno = EAGAIN;
                return Err(anyhow!("EAGAIN"));
            }

            self.active -= 1;
            if (_current < self.active) {
                pipes.swap(_current, self.active);
            } else {
                self._current = 0;
            }
        }

        //  If there are no pipes we cannot send the message.
        if (self.active == 0) {
            errno = EAGAIN;
            return Err(anyhow!("EAGAIN"));
        }

        //  If it's final part of the message we can flush it downstream and
        //  continue round-robining (load balance).
        more = (msg.flags() & ZMQ_MSG_MORE) != 0;
        if (!more) {
            pipes[self._current].flush();

            if (self._current += 1 >= self.active) {
                _current = 0;
            }
        }

        //  Detach the message from the data buffer.
        msg.init2();
        // errno_assert (rc == 0);

        Ok(())
    }

    // bool has_out ();
    pub fn has_out(&mut self) -> bool {
        //  If one part of the message was already written we can definitely
        //  write the rest of the message.
        if (more) {
            return true;
        }

        while (self.active > 0) {
            //  Check whether a pipe has room for another message.
            if (pipes[self._current].check_write()) {
                return true;
            }

            //  Deactivate the pipe.
            self.active -= 1;
            pipes.swap(_current, self.active);
            if (self._current == self.active) {
                self._current = 0;
            }
        }
        return false;
    }
}

// LoadBalancer::~LoadBalancer ()
// {
//     // zmq_assert (pipes.empty ());
// }
