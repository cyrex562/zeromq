/*
    Copyright (c) 2007-2019 Contributors as noted in the AUTHORS file

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
// #include "ws_protocol.hpp"
// #include "ws_encoder.hpp"
// #include "msg.hpp"
// #include "likely.hpp"
// #include "wire.hpp"
// #include "random.hpp"

use std::ptr::null_mut;
use crate::encoder::EncoderBase;
use crate::message::{ZMQ_MSG_COMMAND, ZMQ_MSG_MORE, ZMQ_MSG_SHARED, ZmqMessage};
use crate::utils::put_u32;
use crate::v2_protocol::v2_protocol_msg_flag::{command_flag, more_flag};

// #include <limits.h>
pub struct ZmqWsEncoder
{
    // : public EncoderBase<ZmqWsEncoder>
    pub base: EncoderBase,
    // ZmqWsEncoder (bufsize_: usize, must_mask_: bool);
    // ~ZmqWsEncoder ();

    //
    //   void size_ready ();
    //   void message_ready ();

    // unsigned char _tmp_buf[16];
    pub _tmp_buf: Vec<u8>,
    pub _must_mask: bool,
    // unsigned char _mask[4];
    pub _mask: Vec<u8>,
    pub _masked_msg: ZmqMessage,
    pub _is_binary: bool,

    // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqWsEncoder)
}

impl ZmqWsEncoder {
    pub fn new(bufsize_: usize, must_mask_: bool) -> Self

    {
//  EncoderBase<ZmqWsEncoder> (bufsize_), _must_mask (must_mask_)
        //  Write 0 bytes to the batch and go to message_ready state.
        let mut out = Self {
            base: EncoderBase::new(bufsize_),
            _tmp_buf: vec![],
            _must_mask: false,
            _mask: vec![],
            _masked_msg: Default::default(),
            _is_binary: false,
        };
        next_step(null_mut(), 0, &ZmqWsEncoder::message_ready, true);
        _masked_msg.init();
        out
    }

// ZmqWsEncoder::~ZmqWsEncoder ()
// {
//     _masked_msg.close ();
// }

    pub fn message_ready(&mut self)
    {
        let mut offset = 0;

        let mut _is_binary = false;

        if (in_progress().is_ping()) {
            _tmp_buf[offset += 1] = 0x80 | opcode_ping;
        } else if (in_progress().is_pong()) {
            _tmp_buf[offset += 1] = 0x80 | opcode_pong;
        } else if (in_progress().is_close_cmd()) {
            _tmp_buf[offset += 1] = 0x80 | opcode_close;
        } else {
            _tmp_buf[offset += 1] = 0x82; // Final | binary
            _is_binary = true;
        }

        _tmp_buf[offset] = if _must_mask { 0x80 } else { 0x00 };

        let size = in_progress().size();
        if (_is_binary) {
            size += 1;
        }
        //  TODO: create an opcode for subscribe/cancel
        if (in_progress().is_subscribe() || in_progress().is_cancel()) {
            size += 1;
        }

        if (size <= 125) {
            _tmp_buf[offset += 1] |= (size & 127);
        } else if (size <= 0xFFFF) {
            _tmp_buf[offset += 1] |= 126;
            _tmp_buf[offset += 1] = ((size >> 8) & 0xFF);
            _tmp_buf[offset += 1] = (size & 0xFF);
        } else {
            _tmp_buf[offset += 1] |= 127;
            put_uint64(_tmp_buf + offset, size);
            offset += 8;
        }

        if (_must_mask) {
            let random = generate_random();
            put_u32(_tmp_buf + offset, 0, random);
            put_u32(_mask, 0, random);
            offset += 4;
        }

        let mut mask_index = 0;
        if (_is_binary) {
            //  Encode flags.
            let mut protocol_flags = 0;
            if (in_progress().flags() & ZMQ_MSG_MORE) {
                protocol_flags |= more_flag;
            }
            if (in_progress().flags() & ZMQ_MSG_COMMAND) {
                protocol_flags |= command_flag;
            }
            _tmp_buf[offset += 1] =
                if _must_mask { protocol_flags ^ _mask[mask_index += 1] } else { protocol_flags };
        }

        //  Encode the subscribe/cancel byte.
        //  TODO: remove once there is an opcode for subscribe/cancel
        if (in_progress().is_subscribe()) {
            _tmp_buf[offset += 1] = if _must_mask {
                1 ^ _mask[mask_index += 1]
            } else { 1 };
        } else if (in_progress().is_cancel()) {
            _tmp_buf[offset += 1] = if _must_mask {
                0 ^ _mask[mask_index += 1]
            } else { 0 };
        }

        next_step(_tmp_buf, offset, &ZmqWsEncoder::size_ready, false);
    }

    pub fn size_ready(&mut self)
    {
        if (_must_mask) {
            // assert (in_progress () != &_masked_msg);
            let size = in_progress().size();

            let src = (in_progress().data());
            let dest = src;

            //  If msg is shared or data is constant we cannot mask in-place, allocate a new msg for it
            if (in_progress().flags() & ZMQ_MSG_SHARED
                || in_progress().is_cmsg()) {
                _masked_msg.close();
                _masked_msg.init_size(size);
                dest = (_masked_msg.data());
            }

            let mut mask_index = 0;
            if (_is_binary) {
                mask_index += 1;
            }
            //  TODO: remove once there is an opcode for subscribe/cancel
            if (in_progress().is_subscribe() || in_progress().is_cancel()) {
                mask_index += 1;
            }
            // for (size_t i = 0; i < size; += 1i, mask_index+= 1)
            for i in 0..size
            {
                dest[i] = src[i] ^ _mask[mask_index % 4];
                mask_index += 1;
            }

            next_step(dest, size, &ZmqWsEncoder::message_ready, true);
        } else {
            next_step(in_progress().data(), in_progress().size(),
                      &ZmqWsEncoder::message_ready, true);
        }
    }
}
