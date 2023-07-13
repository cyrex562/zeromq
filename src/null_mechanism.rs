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

// #include <stddef.h>
// #include <string.h>
// #include <stdlib.h>

use crate::context::ZmqContext;
use crate::defines::{
    ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_ERROR, ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND,
};
use crate::mechanism::ZmqMechanismStatus::{error, handshaking, ready};
use crate::mechanism_base::ZmqMechanismBase;
use crate::message::ZmqMessage;

use crate::session_base::ZmqSessionBase;
use crate::utils::{cmp_bytes, copy_bytes};
use crate::zap_client::ZmqZapClient;
use bincode::options;
use libc::{memcmp, memcpy, read, EAGAIN, EFAULT, EPROTO};
use std::mem;
use std::ptr::null_mut;

pub const error_command_name: &'static [u8; 6] = b"\x05ERROR";
pub const error_command_name_len: usize = error_command_name.len();
pub const error_reason_len_size: usize = 1;

pub const ready_command_name: &'static [u8; 6] = b"\x05READY";
pub const ready_command_name_len: usize = ready_command_name.len();

// #include "err.hpp"
// #include "msg.hpp"
// #include "session_base.hpp"
// #include "null_mechanism.hpp"
#[derive(Default, Debug, Clone)]
pub struct ZmqNullMechanism<'a> {
    pub mechanism_base: ZmqMechanismBase<'a>,
    pub zap_client: ZmqZapClient,
    pub _ready_command_sent: bool,
    pub _error_command_sent: bool,
    pub _ready_command_received: bool,
    pub _error_command_received: bool,
    pub _zap_request_sent: bool,
    pub _zap_reply_received: bool,
    // int process_ready_command (const cmd_data_: &mut [u8],  data_size_: usize);
    pub process_ready_command: fn(&mut [u8], usize) -> i32,
    // int process_error_command (const cmd_data_: &mut [u8], data_size_: usize);
    pub process_error_command: fn(&mut [u8], usize) -> i32,
    // void send_zap_request ();
}

impl <'a>ZmqNullMechanism<'a> {
    // ZmqNullMechanism (ZmqSessionBase *session_,
    //                   const std::string &peer_address_,
    //                   options: &ZmqOptions);
    // ~ZmqNullMechanism ();
    // mechanism implementation
    // int next_handshake_command (msg: &mut ZmqMessage);
    // int process_handshake_command (msg: &mut ZmqMessage);
    // int zap_msg_available ();
    // status_t status () const;
    pub fn new(
        session: &mut ZmqSessionBase,
        peer_address_: &str,
        options: &mut ZmqContext,
    ) -> ZmqNullMechanism<'a> {
        // ZmqMechanismBase (session_, options_),
        // ZmqZapClient (session_, peer_address_, options_),
        // _ready_command_sent (false),
        // _error_command_sent (false),
        // _ready_command_received (false),
        // _error_command_received (false),
        // _zap_request_sent (false),
        // _zap_reply_received (false)
        let mut out = Self {
            zap_client: ZmqZapClient::new(options, session, peer_address_),
            mechanism_base: ZmqMechanismBase::new(options, session),
            ..Default::default()
        };
        out
    }

    pub fn next_handshake_command(&mut self, msg: &mut ZmqMessage) -> i32 {
        if (_ready_command_sent || _error_command_sent) {
            errno = EAGAIN;
            return -1;
        }

        if (zap_required() && !_zap_reply_received) {
            if (_zap_request_sent) {
                errno = EAGAIN;
                return -1;
            }
            //  Given this is a backward-incompatible change, it's behind a socket
            //  option disabled by default.
            let rc = session.zap_connect();
            if (rc == -1 && options.zap_enforce_domain) {
                session
                    .get_socket()
                    .event_handshake_failed_no_detail(session.get_endpoint(), EFAULT);
                return -1;
            }
            if (rc == 0) {
                send_zap_request();
                _zap_request_sent = true;

                //  TODO actually, it is quite unlikely that we can read the ZAP
                //  reply already, but removing this has some strange side-effect
                //  (probably because the pipe's in_active flag is true until a read
                //  is attempted)
                rc = receive_and_process_zap_reply();
                if (rc != 0) {
                    return -1;
                }

                _zap_reply_received = true;
            }
        }

        if (_zap_reply_received && status_code != "200") {
            _error_command_sent = true;
            if (status_code != "300") {
                let status_code_len = 3;
                let rc: i32 =
                    msg.init_size(error_command_name_len + error_reason_len_size + status_code_len);
                // zmq_assert (rc == 0);\
                let mut offset = 0usize;
                let msg_data = (msg.data_mut());
                copy_bytes(
                    msg_data,
                    offset as i32,
                    error_command_name,
                    0,
                    error_command_name_len as i32,
                );

                // msg_data += error_command_name_len;
                offset += error_command_name_len;
                copy_bytes(
                    msg_data,
                    offset as i32,
                    &[(status_code_len as u8)],
                    0,
                    error_reason_len_size as i32,
                );
                // *msg_data = status_code_len;
                // msg_data += error_reason_len_size;
                offset += error_reason_len_size;
                copy_bytes(
                    msg_data,
                    offset as i32,
                    status_code,
                    0,
                    status_code_len as i32,
                );
                return 0;
            }
            errno = EAGAIN;
            return -1;
        }

        make_command_with_basic_properties(msg, ready_command_name, ready_command_name_len);

        _ready_command_sent = true;

        return 0;
    }

    pub fn process_handshake_command(&mut self, msg: &mut ZmqMessage) -> i32 {
        if (_ready_command_received || _error_command_received) {
            session.get_socket().event_handshake_failed_protocol(
                session.get_endpoint(),
                ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND,
            );
            errno = EPROTO;
            return -1;
        }

        let cmd_data = (msg.data());
        let data_size = msg.size();

        let mut rc = 0;
        if (data_size >= ready_command_name_len
            && cmp_bytes(cmd_data, 0, ready_command_name, 0, ready_command_name_len) != 0)
        {
            rc = process_ready_command(cmd_data, data_size);
        } else if (data_size >= error_command_name_len
            && cmp_bytes(cmd_data, 0, error_command_name, 0, error_command_name_len) != 0)
        {
            rc = process_error_command(cmd_data, data_size);
        } else {
            session.get_socket().event_handshake_failed_protocol(
                session.get_endpoint(),
                ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND,
            );
            errno = EPROTO;
            rc = -1;
        }

        if (rc == 0) {
            msg.close();
            // errno_assert (rc == 0);
            msg.init2();
            // errno_assert (rc == 0);
        }
        return rc;
    }

    pub fn process_ready_command(&mut self, cmd_data_: &mut [u8], data_size_: usize) -> i32 {
        _ready_command_received = true;
        return parse_metadata(
            cmd_data_ + ready_command_name_len,
            data_size_ - ready_command_name_len,
        );
    }

    pub fn process_error_command(&mut self, cmd_data_: &mut [u8], data_size_: usize) -> i32 {
        let fixed_prefix_size = error_command_name_len + error_reason_len_size;
        if (data_size_ < fixed_prefix_size) {
            session.get_socket().event_handshake_failed_protocol(
                session.get_endpoint(),
                ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_ERROR,
            );

            errno = EPROTO;
            return -1;
        }
        let error_reason_len = (cmd_data_[error_command_name_len]);
        if (error_reason_len > (data_size_ - fixed_prefix_size) as u8) {
            session.get_socket().event_handshake_failed_protocol(
                session.get_endpoint(),
                ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_ERROR,
            );

            errno = EPROTO;
            return -1;
        }
        let error_reason = (cmd_data_) + fixed_prefix_size;
        handle_error_reason(error_reason, error_reason_len);
        _error_command_received = true;
        return 0;
    }
    pub fn zap_msg_available() -> i32 {
        if (_zap_reply_received) {
            errno = EFSM;
            return -1;
        }
        let rc: i32 = receive_and_process_zap_reply();
        if (rc == 0) {
            _zap_reply_received = true;
        }
        return if rc == -1 { -1 } else { 0 };
    }

    pub fn status(&mut self) -> status_t {
        if (_ready_command_sent && _ready_command_received) {
            return ready;
        }

        let command_sent = _ready_command_sent || _error_command_sent;
        let command_received = _ready_command_received || _error_command_received;
        return if command_sent && command_received {
            error
        } else {
            handshaking
        };
    }

    pub fn send_zap_request(&mut self) {
        send_zap_request("NULL", 4, null_mut(), null_mut(), 0);
    }
} // impl ZmqNullMechanism

// ZmqNullMechanism::~ZmqNullMechanism ()
// {
// }
