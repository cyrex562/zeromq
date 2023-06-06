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

// #ifndef __ZMQ_DECODER_HPP_INCLUDED__
// #define __ZMQ_DECODER_HPP_INCLUDED__

// #include <algorithm>
// #include <cstddef>
// #include <cstring>

// #include "decoder_allocators.hpp"
// #include "err.hpp"
// #include "i_decoder.hpp"
// #include "stdint.hpp"

// namespace zmq
// {
//  Helper base class for decoders that know the amount of data to read
//  in advance at any moment. Knowing the amount in advance is a property
//  of the protocol used. 0MQ framing protocol is based size-prefixed
//  paradigm, which qualifies it to be parsed by this class.
//  On the other hand, XML-based transports (like XMPP or SOAP) don't allow
//  for knowing the size of data to read in advance and should use different
//  decoding algorithms.
//
//  This class implements the state machine that parses the incoming buffer.
//  Derived class should implement individual state machine actions.
//
//  Buffer management is done by an allocator policy.
// template <typename T, typename A = c_single_allocator>

use crate::utils::copy_bytes;

pub trait i_decoder {}

#[derive(Default, Debug, Clone)]
pub struct DecoderBase {
    // public:

    // private:
    //  Next step. If set to NULL, it means that associated data stream
    //  is dead. Note that there can be still data in the process in such
    //  case.
    // step_t next;
    next: usize,

    //  Where to store the read data.
    // unsigned char *read_pos;
    read_pos: Vec<u8>,

    //  How much data to read before taking next step.
    // std::size_t to_read;
    to_read: usize,

    //  The duffer for data to decode.
    // A allocator;
    // allocator: T,
    // unsigned char *buf;
    buf: Vec<u8>, // // ZMQ_NON_COPYABLE_NOR_MOVABLE (DecoderBase)
}

impl DecoderBase {
    // explicit DecoderBase (const buf_size_: usize) :
    // next (null_mut()), read_pos (null_mut()), to_read (0), allocator (buf_size_)
    // {
    //     buf = allocator.allocate ();
    // }
    pub fn new(buf_size: usize) -> Self {
        let mut out = Self {
            next: 0,
            read_pos: vec![],
            to_read: 0,
            // allocator: Allocator::new(),
            buf: vec![],
        };
        out
    }

    // ~DecoderBase ()  { allocator.deallocate (); }

    //  Returns a buffer to be filled with binary data.
    // void get_buffer (unsigned char **data, std::size: *mut usize)
    pub fn get_buffer(&mut self, data: &mut Vec<u8>, size: &mut usize) {
        self.buf = self.allocator.allocate();

        //  If we are expected to read large message, we'll opt for zero-
        //  copy, i.e. we'll ask caller to fill the data directly to the
        //  message. Note that subsequent read(s) are non-blocking, thus
        //  each single read reads at most SO_RCVBUF bytes at once not
        //  depending on how large is the chunk returned from here.
        //  As a consequence, large messages being received won't block
        //  other engines running in the same I/O thread for excessive
        //  amounts of time.
        if (self.to_read >= self.allocator.size()) {
            *data = self.buf[self.read_pos..];
            *size = self.to_read;
            return;
        }

        *data = self.buf.clone();
        *size = self.allocator.size();
    }

    //  Processes the data in the buffer previously allocated using
    //  get_buffer function. size argument specifies number of bytes
    //  actually filled into the buffer. Function returns 1 when the
    //  whole message was decoded or 0 when more data is required.
    //  On error, -1 is returned and errno set accordingly.
    //  Number of bytes processed is returned in bytes_used_.
    pub fn decode(&mut self, data: &[u8], size: usize, bytes_used: &mut usize) -> i32 {
        let mut bytes_used_ = 0;

        //  In case of zero-copy simply adjust the pointers, no copying
        //  is required. Also, run the state machine in case all the data
        //  were processed.
        if (data == self.read_pos) {
            // zmq_assert(size <= to_read);
            self.read_pos += size;
            self.to_read -= size;
            bytes_used_ = size;

            while (!self.to_read) {
                let rc: i32 = 0;
                // TODO
                // (static_cast<T *> (this)->*next) (data + bytes_used_);
                if (rc != 0) {
                    return rc;
                }
            }
            return 0;
        }

        while (bytes_used_ < size) {
            //  Copy the data from buffer to the message.
            let to_copy = usize::min(self.to_read, size - bytes_used_);
            // Only copy when destination address is different from the
            // current address in the buffer.
            if (self.read_pos != data + bytes_used_) {
                copy_bytes(self.read_pos.as_mut_slice(), 0, data, bytes_used_, to_copy);
            }

            self.read_pos += to_copy;
            self.to_read -= to_copy;
            bytes_used_ += to_copy;
            //  Try to get more space in the message to fill in.
            //  If none is available, return.
            while (self.to_read == 0) {
                // pass current address in the buffer
                let rc: i32 = 0;
                // TODO
                // (static_cast<T *> (this)->*next) (data + bytes_used_);
                if (rc != 0) {
                    return rc;
                }
            }
        }

        return 0;
    }

    pub fn resize_buffer(&mut self, new_size: usize) {
        self.allocator.resize(new_size);
    }

    //   protected:
    //  Prototype of state machine action. Action should return false if
    //  it is unable to push the data to the system.
    // typedef int (T::*step_t) (unsigned char const *);

    //  This function should be called from derived class to read data
    //  from the buffer and schedule next state machine action.
    pub fn next_step(&mut self, read_pos_: &mut [u8], to_read_: usize, next_: usize) {
        self.read_pos = read_pos_.clone();
        self.to_read = to_read_;
        self.next = next_;
    }

    // A &get_allocator () { return allocator; }
    // pub fn get_allocator(&mut self) -> &mut T {
    //     &mut self.allocator
    // }
}
// }

// #endif
