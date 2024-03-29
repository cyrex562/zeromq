/*
    Copyright (c) 2020 Contributors as noted in the AUTHORS file

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
// #include "v2_protocol.hpp"
// #include "v3_1_encoder.hpp"
// #include "msg.hpp"
// #include "likely.hpp"
// #include "wire.hpp"

use crate::decoder_allocators::size;
use crate::encoder::EncoderBase;
use crate::message::{ZmqMessage, CANCEL_CMD_NAME, SUB_CMD_NAME, ZMQ_MSG_COMMAND, ZMQ_MSG_MORE};
use libc::{memcpy, size_t};
use std::ptr::null_mut;

// #include <limits.h>
pub struct ZmqV31Encoder {
    // : public EncoderBase<ZmqV31Encoder>
    pub encoder_base: EncoderBase,
    //
    //     ZmqV31Encoder (bufsize_: usize);
    //     ~ZmqV31Encoder () ;

    //
    //   void size_ready ();
    //   void message_ready ();

    // unsigned char _tmp_buf[9 + ZmqMessage::SUB_CMD_NAME_SIZE];
    pub tmp_buf: Vec<u8>,
    // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqV31Encoder)
}

impl ZmqV31Encoder {
    // ZmqV31Encoder::ZmqV31Encoder (bufsize_: usize) :
    //     EncoderBase<ZmqV31Encoder> (bufsize_)
    pub fn new(bufsize_: usize) -> Self {
        let mut out = Self {
            encoder_base: EncoderBase::new(bufsize_),
            tmp_buf: Vec::with_capacity(9 + ZmqMessage::SUB_CMD_NAME_SIZE),
        };

        //  Write 0 bytes to the batch and go to message_ready state.
        out.next_step(null_mut(), 0, &ZmqV31Encoder::message_ready, true);
        out
    }

    pub fn message_ready(&mut self) {
        //  Encode flags.
        let size_ = in_progress().size();
        let mut header_size = 2; // flags byte + size byte
        let protocol_flags = _tmp_buf[0];
        protocol_flags = 0;
        if (in_progress().flags() & ZMQ_MSG_MORE) {
            protocol_flags |= v2_protocol_t::more_flag;
        }
        if (in_progress().size() > UCHAR_MAX) {
            protocol_flags |= v2_protocol_t::large_flag;
        }
        if (in_progress().flags() & ZMQ_MSG_COMMAND
            || in_progress().is_subscribe()
            || in_progress().is_cancel())
        {
            protocol_flags |= v2_protocol_t::command_flag;
            if (in_progress().is_subscribe()) {
                size += ZmqMessage::SUB_CMD_NAME_SIZE;
            } else if (in_progress().is_cancel()) {
                size += ZmqMessage::CANCEL_CMD_NAME_SIZE;
            }
        }

        //  Encode the message length. For messages less then 256 bytes,
        //  the length is encoded as 8-bit unsigned integer. For larger
        //  messages, 64-bit unsigned integer in network byte order is used.
        if (size > UCHAR_MAX) {
            put_uint64(_tmp_buf + 1, size);
            header_size = 9; // flags byte + size 8 bytes
        } else {
            _tmp_buf[1] = (size);
        }

        //  Encode the sub/cancel command string. This is Done in the encoder as
        //  opposed to when the subscribe message is created to allow different
        //  protocol behaviour on the wire in the v3.1 and legacy encoders.
        //  It results in the work being Done multiple times in case the sub
        //  is sending the subscription/cancel to multiple pubs, but it cannot
        //  be avoided. This processing can be moved to xsub once support for
        //  ZMTP < 3.1 is dropped.
        if (in_progress().is_subscribe()) {
            // memcpy (_tmp_buf + header_size, SUB_CMD_NAME,
            //         ZmqMessage::SUB_CMD_NAME_SIZE);
            header_size += ZmqMessage::SUB_CMD_NAME_SIZE;
        } else if (in_progress().is_cancel()) {
            // memcpy (_tmp_buf + header_size, CANCEL_CMD_NAME,
            //         ZmqMessage::CANCEL_CMD_NAME_SIZE);
            header_size += ZmqMessage::CANCEL_CMD_NAME_SIZE;
        }

        next_step(_tmp_buf, header_size, &ZmqV31Encoder::size_ready, false);
    }

    pub fn size_ready() {
        //  Write message Body into the buffer.
        next_step(
            in_progress().data(),
            in_progress().size(),
            &ZmqV31Encoder::message_ready,
            true,
        );
    }
}
