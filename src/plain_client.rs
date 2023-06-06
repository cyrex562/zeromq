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
// #include "macros.hpp"

// #include <string>
// #include <limits.h>

use libc::{EAGAIN, EPROTO};
use crate::curve_client_tools::produce_initiate;
use crate::defines::{ZMQ_PROTOCOL_ERROR_ZMTP_INVALID_METADATA, ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_ERROR, ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_WELCOME, ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND};
use crate::mechanism::{ZmqMechanism, ZmqMechanismStatus};
use crate::mechanism::ZmqMechanismStatus::ready;
use crate::mechanism_base::ZmqMechanismBase;
use crate::message::ZmqMessage;

use crate::plain_client::PlainClientState::{error_command_received, sending_initiate, waiting_for_ready, waiting_for_welcome};
use crate::session_base::ZmqSessionBase;
use crate::utils::{cmp_bytes, copy_bytes, advance_ptr};

// #include "msg.hpp"
// #include "err.hpp"
// #include "plain_client.hpp"
// #include "session_base.hpp"
// #include "plain_common.hpp"

pub enum PlainClientState
{
    sending_hello,
    waiting_for_welcome,
    sending_initiate,
    waiting_for_ready,
    error_command_received,
    ready
}

#[derive(Default,Debug,Clone)]
pub struct PlainClient
{
// : public ZmqMechanismBase
    pub mechanism_base: ZmqMechanismBase,
    pub _state: PlainClientState,
}

impl PlainClient {
    // PlainClient (ZmqSessionBase *session_, options: &ZmqOptions);
    pub fn new(session: &mut ZmqSessionBase, options: &mut ZmqContext) -> Self {
        Self {
            mechanism_base: ZmqMechanismBase::new(options, session),
            _state: PlainClientState::sending_hello,
        }
    }

    // ~PlainClient ();

    // mechanism implementation
    // int next_handshake_command (msg: &mut ZmqMessage);
    pub fn next_handshake_command (&mut self, msg: &mut ZmqMessage) -> i32
    {
        let mut rc = 0;

        match (&self._state) {
            sending_hello=> {
                produce_hello(msg);
                self._state = PlainClientState::waiting_for_welcome;
            }
        sending_initiate=> {
            self.produce_initiate(msg);
            self._state = PlainClientState::waiting_for_ready;
        }
        _ => {
            errno = EAGAIN;
            rc = -1;
        }
    }
        return rc;
    }





    // void produce_hello (msg: &mut ZmqMessage) const;

    // void produce_initiate (msg: &mut ZmqMessage) const;


    // int process_welcome (const cmd_data_: &mut [u8], data_size_: usize);

    // int process_ready (const cmd_data_: &mut [u8], data_size_: usize);

    // int process_error (const cmd_data_: &mut [u8], data_size_: usize);

// int process_handshake_command (msg: &mut ZmqMessage);
    pub fn process_handshake_command (&mut self, msg: &mut ZmqMessage) -> i32
    {
        let cmd_data =
            (msg.data_mut ());
        let data_size = msg.size ();

        let mut rc = 0;
        if data_size >= welcome_prefix_len
            && !cmp_bytes (cmd_data, 0, welcome_prefix, 0, welcome_prefix_len) == 0 {
            rc = self.process_welcome(cmd_data, data_size);
        }
        else if data_size >= ready_prefix_len
            && !cmp_bytes (cmd_data, 0, ready_prefix, 0, ready_prefix_len) == 0 {
            rc = process_ready(cmd_data, data_size);
        }
        else if data_size >= error_prefix_len
            && !cmp_bytes (cmd_data, 0,error_prefix, 0,error_prefix_len) == 0 {
            rc = process_error(cmd_data, data_size);
        }
        else {
            session.get_socket ().event_handshake_failed_protocol (
                session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND);
            errno = EPROTO;
            rc = -1;
        }

        if (rc == 0) {
            msg.close ();
            // errno_assert (rc == 0);
            msg.init2();
            // errno_assert (rc == 0);
        }

        return rc;
    }

    // status_t status () const;
    pub fn status (&mut self) -> ZmqMechanismStatus
    {
        return match &self._state {
            ready => ZmqMechanismStatus::ready,
            error_command_received => ZmqMechanismStatus::error,
            _ => ZmqMechanismStatus::handshaking,
        }
    }

    pub fn produce_hello (&mut self, msg: &mut ZmqMessage)
    {
        let username = self.options.plain_username;
        // zmq_assert (username.length () <= UCHAR_MAX);

        let password = self.options.plain_password;
        // zmq_assert (password.length () <= UCHAR_MAX);

        let command_size = hello_prefix_len + brief_len_size
            + username.length () + brief_len_size
            + password.length ();

        let rc: i32 = msg.init_size (command_size);
        // errno_assert (rc == 0);

        let mut ptr =  (msg.data_mut());
        copy_bytes(ptr, 0, hello_prefix, 0, hello_prefix_len);
        // ptr += hello_prefix_len;
        ptr = advance_ptr(ptr, hello_prefix_len);

        // *ptr+= 1 =  (username.length ());
        copy_bytes(ptr, 0, username.length(), 0, 1);
        ptr = advance_ptr(ptr, 1);
        // memcpy (ptr, username, username.length ());
        copy_bytes(ptr, 0, username, 0, username.length());
        ptr = advance_ptr(ptr, username.length ());

        // *ptr+= 1 =  (password.length ());
        copy_bytes(ptr, 0, password.length(), 0, 1);
        ptr = advance_ptr(ptr, 1);

        copy_bytes(ptr, 0, password, 0,password.length ());
    }

    pub fn process_welcome (&mut self, cmd_data_: &mut [u8], data_size_: usize) -> i32
    {
        // LIBZMQ_UNUSED (cmd_data_);

        if (self._state != waiting_for_welcome) {
            session.get_socket ().event_handshake_failed_protocol (
                session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND);
            errno = EPROTO;
            return -1;
        }
        if (data_size_ != welcome_prefix_len) {
            session.get_socket ().event_handshake_failed_protocol (
                session.get_endpoint (),
                ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_WELCOME);
            errno = EPROTO;
            return -1;
        }
        self._state = sending_initiate;
        return 0;
    }

    pub fn produce_initiate (&mut self, msg: &mut ZmqMessage)
    {
        make_command_with_basic_properties (msg, initiate_prefix,
                                            initiate_prefix_len);
    }

    pub fn process_ready (&mut self, cmd_data_: &mut [u8], data_size_: usize) -> i32
    {
        if (_state != waiting_for_ready) {
            session.get_socket ().event_handshake_failed_protocol (
                session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND);
            errno = EPROTO;
            return -1;
        }
        let rc: i32 = parse_metadata (cmd_data_ + ready_prefix_len,
                                      data_size_ - ready_prefix_len);
        if (rc == 0) {
            self._state = PlainClientState::ready;
        }
        else {
            session.get_socket().event_handshake_failed_protocol(
                session.get_endpoint(), ZMQ_PROTOCOL_ERROR_ZMTP_INVALID_METADATA);
        }

        return rc;
    }

    pub fn process_error (&mut self, cmd_data_: &mut [u8], data_size_: usize) -> i32
    {
        if (_state != waiting_for_welcome && _state != waiting_for_ready) {
            session.get_socket ().event_handshake_failed_protocol (
                session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND);
            errno = EPROTO;
            return -1;
        }
        let start_of_error_reason = error_prefix_len + brief_len_size;
        if (data_size_ < start_of_error_reason) {
            session.get_socket ().event_handshake_failed_protocol (
                session.get_endpoint (),
                ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_ERROR);
            errno = EPROTO;
            return -1;
        }
        let error_reason_len =
            (cmd_data_[error_prefix_len]);
        if (error_reason_len > data_size_ - start_of_error_reason) {
            session.get_socket ().event_handshake_failed_protocol (
                session.get_endpoint (),
                ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_ERROR);
            errno = EPROTO;
            return -1;
        }
        let error_reason =
            (cmd_data_) + start_of_error_reason;
        handle_error_reason (error_reason, error_reason_len);
        _state = error_command_received;
        return 0;
    }

} // end of impl plain_client

// PlainClient::PlainClient (ZmqSessionBase *const session_,
//                                      options: &ZmqOptions) :
//     ZmqMechanismBase (session_, options_), _state (sending_hello)
// {
// }
//
// PlainClient::~PlainClient ()
// {
// }
















