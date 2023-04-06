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
// #include "curve_mechanism_base.hpp"
// #include "msg.hpp"
// #include "wire.hpp"
// #include "session_base.hpp"

// #ifdef ZMQ_HAVE_CURVE

// #ifdef ZMQ_USE_LIBSODIUM
//  libsodium added crypto_box_easy_afternm and crypto_box_open_easy_afternm with
//  https: //github.com/jedisct1/libsodium/commit/aaf5fbf2e53a33b18d8ea9bdf2c6f73d7acc8c3e
// #if SODIUM_LIBRARY_VERSION_MAJOR > 7                                           \
//   || (SODIUM_LIBRARY_VERSION_MAJOR == 7 && SODIUM_LIBRARY_VERSION_MINOR >= 4)
// #define ZMQ_HAVE_CRYPTO_BOX_EASY_FNS 1
// #endif
// #endif

use std::mem;
use anyhow::anyhow;
use crate::config::CRYPTO_BOX_NONCEBYTES;
use crate::curve_encoding::{ZmqCurveEncoding, ZmqNonce};
use crate::mechanism_base::ZmqMechanismBase;
use crate::message::{CANCEL_CMD_NAME, CANCEL_CMD_NAME_SIZE, SUB_CMD_NAME, SUB_CMD_NAME_SIZE, ZMQ_MSG_COMMAND, ZMQ_MSG_MORE, ZmqMessage};
use crate::options::ZmqOptions;
use crate::session_base::ZmqSessionBase;
use crate::utils::copy_bytes;
use crate::zmq_hdr::{ZMQ_PROTOCOL_ERROR_ZMTP_INVALID_SEQUENCE, ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_MESSAGE, ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND};




pub type ZmqNonce = u64;

//  Right now, we only transport the lower two bit flags of ZmqMessage, so they
//  are binary identical, and we can just use a bitmask to select them. If we
//  happened to add more flags, this might change.
pub const flag_mask: u8 = ZMQ_MSG_MORE | ZMQ_MSG_COMMAND;
pub const flags_len: usize = 1;
pub const nonce_prefix_len: usize = 16;
pub const message_command: &[u8] = b"\x07MESSAGE";
pub const message_command_len: usize = message_command.len();
pub const message_header_len: usize = message_command_len + mem::size_of::<ZmqNonce>();

// #ifndef ZMQ_USE_LIBSODIUM
pub const CRYPTO_BOX_MACBYTES: usize = 16;

// pub struct curve_mechanism_base_t: public virtual mechanism_base_t,
// public ZmqCurveEncoding
#[derive(Default,Debug,Clone)]
pub struct ZmqCurveMechanismBase
{
    // public:
    // curve_mechanism_base_t (ZmqSessionBase *session_,
    // const ZmqOptions & options_,
    // encode_nonce_prefix_: * const c_char,
    // decode_nonce_prefix_: * const c_char,
    // const downgrade_sub_: bool);
    //
    // // mechanism implementation
    // int encode (msg: & mut ZmqMessage) ZMQ_OVERRIDE;
    // int decode (msg: & mut ZmqMessage) ZMQ_OVERRIDE;
    pub mechanism_base: ZmqMechanismBase,
    pub curve_encoding: ZmqCurveEncoding,
}


impl ZmqCurveMechanismBase {
    // ZmqCurveMechanismBase::ZmqCurveMechanismBase (
    // ZmqSessionBase *session_,
    // const ZmqOptions & options_,
    // encode_nonce_prefix_: * const c_char,
    // decode_nonce_prefix_: * const c_char,
    // const downgrade_sub_: bool):
    // mechanism_base_t (session_, options_),
    // ZmqCurveEncoding (
    // encode_nonce_prefix_, decode_nonce_prefix_, downgrade_sub_)
    // {}
    pub fn new(session: &mut ZmqSessionBase,
    options: &ZmqOptions,
    encode_nonce_prefix: &str,
    decode_nonce_prefix: &str,
    downgrade_sub: bool) -> Self {
        Self {
            mechanism_base: ZmqMechanismBase::new2(session,options),
            curve_encoding: ZmqCurveEncoding::new(encode_nonce_prefix, decode_nonce_prefix, downgrade_sub)
        }
    }

    pub fn encode (&mut self, msg: & mut ZmqMessage) -> anyhow::Result<()>
    {
        self.curve_encoding.encode (msg)
    }

    pub fn decode (&mut self, msg: &mut ZmqMessage) -> anyhow::Result<()>
    {
        check_basic_command_structure(msg)?;
    // if (rc == - 1)
    // return - 1;

    // error_event_code: i32;
        let mut error_event_code = 0u32;
    match self.curve_encoding.decode (msg, &mut error_event_code) {
        Ok(_) => Ok(()),
        Err(e) => {
            self.mechanism_base.session.get_socket().event_handshake_failed_protocol(self.mechanism_base.session.get_endpoint(), event_error_code);
            Err(anyhow!("decode failed: {}", e))
        }
    }
    // if ( - 1 == rc) {
    // session.get_socket () -> event_handshake_failed_protocol (
    // session.get_endpoint (), error_event_code);
    // }
    //
    // return rc;
    }
}


// #endif
