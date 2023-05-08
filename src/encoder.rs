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

// #ifndef __ZMQ_ENCODER_HPP_INCLUDED__
// #define __ZMQ_ENCODER_HPP_INCLUDED__

// #if defined(_MSC_VER)
// #ifndef NOMINMAX
// #define NOMINMAX
// #endif
// #endif

// #include <stddef.h>
// #include <string.h>
// #include <stdlib.h>
// #include <algorithm>

// #include "err.hpp"
// #include "ZmqBaseEncoder.hpp"
// #include "msg.hpp"

// namespace zmq
// {
//  Helper base class for encoders. It implements the state machine that
//  fills the outgoing buffer. Derived classes should implement individual
//  state machine actions.

use crate::message::ZmqMessage;
use crate::utils::copy_bytes;

#[derive(Default, Debug, Clone)]
pub struct ZmqBaseEncoder {
    //
    //  Where to get the data to write from.
    // unsigned char *write_pos;
    pub write_pos: usize,

    //  How much data to write before next step should be executed.
    pub to_write: usize,

    //  Next step. If set to NULL, it means that associated data stream
    //  is dead.
    // step_t next;
    pub next: step_t,

    pub new_msg_flag: bool,

    //  The buffer for encoded data.
    // const size_t buf_size;
    pub buf_size: usize,

    // unsigned char *const buf;
    pub buf: Vec<u8>,

    // ZmqMessage *in_progress;
    pub in_progress: Option<ZmqMessage>,
    // // ZMQ_NON_COPYABLE_NOR_MOVABLE (encoder_base_t)
}

impl ZmqBaseEncoder {
    //
    // explicit encoder_base_t (bufsize_: usize) :
    // write_pos (0),
    // to_write (0),
    // next (null_mut()),
    // new_msg_flag (false),
    // buf_size (bufsize_),
    // buf ( (malloc (bufsize_))),
    // in_progress (null_mut())
    // {
    // alloc_assert (buf);
    // }
    pub fn new(buf_size: usize) -> Self {
        Self {
            buf: Vec::with_capacity(buf_size),
            in_progress: None,
            ..Default::default()
        }
    }

    // ~encoder_base_t ()  { free (buf); }

    //  The function returns a batch of binary data. The data
    //  are filled to a supplied buffer. If no buffer is supplied (data
    //  points to NULL) decoder object will provide buffer of its own.
    pub fn encode(&mut self, data: &mut Option<&mut [u8]>, size: usize) -> Option<&mut [u8]> {
        //
        // unsigned char *buffer = !*data ? buf : *data;
        let buffer = if data.is_some() {
            data.unwrap()
        } else {
            self.buf.as_mut_slice()
        };
        //   const size_t buffersize = !*data ? buf_size : size;
        let buffer_size = if data.is_some() { size } else { self.buf_size };

        if (self.in_progress.is_none()) {
            return None;
        }

        let mut pos = 0;
        while (pos < buffer_size) {
            //  If there are no more data to return, run the state machine.
            //  If there are still no data, return what we already have
            //  in the buffer.
            if (!self.to_write) {
                if (self.new_msg_flag) {
                    let mut rc = self.in_progress.unwrawp().close();
                    // errno_assert (rc == 0);
                    let mut rc = self.in_progress.unwrawp().init();
                    // errno_assert (rc == 0);
                    self.in_progress = None;
                    break;
                }
                self.next();
            }

            //  If there are no data in the buffer yet and we are able to
            //  fill whole buffer in a single go, let's use zero-copy.
            //  There's no disadvantage to it as we cannot stuck multiple
            //  messages into the buffer anyway. Note that subsequent
            //  write(s) are non-blocking, thus each single write writes
            //  at most SO_SNDBUF bytes at once not depending on how large
            //  is the chunk returned from here.
            //  As a consequence, large messages being sent won't block
            //  other engines running in the same I/O thread for excessive
            //  amounts of time.
            if (pos == 0 && data.is_none() && self.to_write >= buffersize) {
                // *data = write_pos;
                pos = to_write;
                self.write_pos = 0;
                self.to_write = 0;
                return Some(self.buf[pos..].as_mut());
            }

            //  Copy data to the buffer. If the buffer is full, return.
            let to_copy = usize::min(self.to_write, buffersize - pos);
            // memcpy (buffer + pos, write_pos, to_copy);
            copy_bytes(
                buffer,
                pos as i32,
                self.buf.as_slice(),
                self.write_pos,
                to_copy as i32,
            );
            pos += to_copy;
            self.write_pos += to_copy;
            self.to_write -= to_copy;
        }

        *data = Some(buffer);
        return Some(&mut self.buf[pos..]);
    }

    pub fn load_msg(&mut self, msg: &mut ZmqMessage) {
        // zmq_assert (in_progress () == null_mut());
        self.in_progress = Some(msg.clone());
        // (static_cast<T *> (this)->*next) ();
        self.next();
    }

    //
    //  Prototype of state machine action.
    // typedef void (T::*step_t) ();

    //  This function should be called from derived class to write the data
    //  to the buffer and schedule next state machine action.
    pub fn next_step(
        &mut self,
        write_pos_: usize,
        to_write_: usize,
        next_: step_t,
        new_msg_flag_: bool,
    ) {
        self.write_pos = write_pos_;
        to_write = to_write_;
        self.next = next_;
        self.new_msg_flag = new_msg_flag_;
    }

    // ZmqMessage *in_progress () { return in_progress; }
}

// #endif
