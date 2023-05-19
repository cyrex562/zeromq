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
// #include <stdlib.h>
// #include <string.h>

// #include "raw_decoder.hpp"
// #include "err.hpp"

use crate::decoder_interface::ZmqDecoderInterface;
use crate::message::ZmqMessage;

pub struct RawDecoder {
    // : public ZmqDecoderInterface
    pub decoder_interface: ZmqDecoderInterface,
    //
// RawDecoder (bufsize_: usize);
// ~RawDecoder ();
//  ZmqDecoderInterface interface.
// void get_buffer (unsigned char **data, size: *mut usize);
// int decode (const data: &mut [u8], size: usize, size_t &bytes_used_);
// ZmqMessage *msg () { return &in_progress; }
// void resize_buffer (size_t) {}
// ZmqMessage in_progress;
    pub in_progress: ZmqMessage,
// shared_message_memory_allocator allocator;

// ZMQ_NON_COPYABLE_NOR_MOVABLE (RawDecoder)
}

impl RawDecoder {
    pub fn new(bufsize_: usize) -> Self {
        //: allocator (bufsize_, 1)
        // let rc: i32 = in_progress.init ();
        // errno_assert (rc == 0);
        Self {
            in_progress: ZmqMessage::new(),
            decoder_interface: ZmqDecoderInterface::new(),
        }
    }

    // RawDecoder::~RawDecoder ()
    // {
    // let rc: i32 = in_progress.close ();
    // // errno_assert (rc == 0);
    // }

    pub fn get_buffer(&mut self, data: &mut [u8]) {
        // * data = allocator.allocate (); *size = allocator.size ();
    }

    pub fn decode(&mut self, data: &[u8], size: usize, bytes_used_: &mut usize) -> i32 {
        todo!();
            //     let rc: i32 = self.in_progress.init ( (data), size,
            // shared_message_memory_allocator::call_dec_ref,
            // allocator.buffer (), allocator.provide_content ());

            // if the buffer serves as memory for a zero-copy message, release it
            // and allocate a new buffer in get_buffer for the next decode if (in_progress.is_zcmsg ()) {
            // allocator.advance_content (); allocator.release ();

            // errno_assert (rc != -1); * bytes_used_ = size;
        return 1;
    }
}
