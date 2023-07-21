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

// #include <string>

// #include "msg.hpp"
// #include "session_base.hpp"
// #include "err.hpp"
// #include "plain_server.hpp"
// #include "wire.hpp"
// #include "plain_common.hpp"

//     PlainServer (ZmqSessionBase *session_,
//                     const std::string &peer_address_,
//                     options: &ZmqOptions);
//     ~PlainServer ();

//     // mechanism implementation
//     int next_handshake_command (msg: &mut ZmqMessage);
//     int process_handshake_command (msg: &mut ZmqMessage);

//   //
//     static void produce_welcome (msg: &mut ZmqMessage);
//     void produce_ready (msg: &mut ZmqMessage) const;
//     void produce_error (msg: &mut ZmqMessage) const;

//     int process_hello (msg: &mut ZmqMessage);
//     int process_initiate (msg: &mut ZmqMessage);

//     void send_zap_request (const std::string &username_,
//                            password_: &str);
// };

use std::mem;
use anyhow::bail;
use libc::{EFAULT, EPROTO};
use crate::defines::{ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_HELLO, ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND, ZMQ_PROTOCOL_ERROR_ZMTP_UNSPECIFIED};
use crate::mechanism_base::ZmqMechanismBase;
use crate::message::ZmqMessage;

use crate::plain_common::{error_prefix, hello_prefix, initiate_prefix, ready_prefix, welcome_prefix};
use crate::session_base::ZmqSessionBase;
use crate::utils::{cmp_bytes, copy_bytes, advance_ptr};
use crate::zap_client::{ZmqZapClient, ZmqZapClientCommonHandshake, ZmqZapClientCommonHandshakeState};

#[derive(Debug)]
pub enum PlainServerState {
    plain_server_state_ready,
    plain_server_state_waiting_for_zap_reply,
    plain_server_state_error,
}

pub struct PlainServer<'a> {
    mechanism_base: ZmqMechanismBase<'a>,
    ZmqZapClientCommonHandshake: ZmqZapClientCommonHandshake,
    state: PlainServerState,
    username: String,
    password: String,
    peer_address: String,
    // options: ZmqOptions,
}

impl <'a>PlainServer<'a> {
    pub fn new(session: &mut ZmqSessionBase, peer_address: &str, options: &mut ZmqContext) -> Self {
        Self {
            mechanism_base: ZmqMechanismBase::new(options, session),
            ZmqZapClientCommonHandshake: ZmqZapClientCommonHandshake::new(options, session, peer_address, ZmqZapClientCommonHandshakeState::ready),
            state: PlainServerState::plain_server_state_ready,
            username: String::new(),
            password: String::new(),
            peer_address: String::from(peer_address),
            // options: options.clone(),
        }
    }

    pub fn next_handshake_command(&mut self, msg: &mut ZmqMessage) -> anyhow::Result<()> {
        match (&self.state) {
            sending_welcome => {
                self.produce_welcome(msg);
                self.state = waiting_for_initiate;
            }
            sending_ready => {
                self.produce_ready(msg);
                self.state = PlainServerState::plain_server_state_ready;
            }
            sending_error => {
                self.produce_error(msg);
                self.state = error_sent;
            }

            _ => {
                // errno = EAGAIN;
                // rc = -1;
                bail!("unhandled state: {:?}", self.state);
            }
        }
        Ok(())
    }

    pub fn process_handshake_command (&mut self, msg: &mut ZmqMessage) -> anyhow::Result<()>
    {
        let mut rc = 0;

        match (&self.state) {
            waiting_for_hello => {
                rc = process_hello(msg);
            }
            waiting_for_initiate => {
                rc = process_initiate(msg);
            }
            _ => {
                //  TODO see comment in ZmqCurveServer::process_handshake_command
                session.get_socket().event_handshake_failed_protocol(
                    session.get_endpoint(), ZMQ_PROTOCOL_ERROR_ZMTP_UNSPECIFIED);
              // errno = EPROTO;
                rc = -1;
            }
        }
        if rc == 0 {
            msg.close ()?;
            // errno_assert (rc == 0);
            msg.init2 ()?;
            // errno_assert (rc == 0);
            Ok(())
        }
        bail!("error")

    }


    pub fn process_hello (&mut self, msg: &mut ZmqMessage) -> i32
    {
        let mut rc = check_basic_command_structure (msg);
        if (rc == -1) {
            return -1;
        }

        let mut ptr =  (msg.data_mut());
        let mut bytes_left = msg.size ();

        if (bytes_left < hello_prefix_len
            || cmp_bytes (ptr, 0, hello_prefix, 0, hello_prefix.len()) != 0) {
            session.get_socket ().event_handshake_failed_protocol (
                session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND);
          // errno = EPROTO;
            return -1;
        }
        ptr += hello_prefix_len;
        bytes_left -= hello_prefix_len;

        if (bytes_left < 1) {
            //  PLAIN I: invalid PLAIN client, did not send username
            session.get_socket ().event_handshake_failed_protocol (
                session.get_endpoint (),
                ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_HELLO);
          // errno = EPROTO;
            return -1;
        }
        let username_length = ptr[0];
        //+= 1;
        ptr = advance_ptr(ptr, 1);
        bytes_left -= mem::size_of_val(&username_length);

        if bytes_left < username_length as usize {
            //  PLAIN I: invalid PLAIN client, sent malformed username
            session.get_socket().event_handshake_failed_protocol (
                session.get_endpoint (),
                ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_HELLO);
          // errno = EPROTO;
            return -1;
        }
        // let username = std::string (ptr, username_length);
        let username = String::from_utf8_lossy(ptr[..username_length]).to_string();
        ptr += username_length;
        bytes_left -= username_length;
        if bytes_left < 1 {
            //  PLAIN I: invalid PLAIN client, did not send password
            session.get_socket ().event_handshake_failed_protocol (
                session.get_endpoint (),
                ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_HELLO);
          // errno = EPROTO;
            return -1;
        }

        let password_length = ptr[0]; //*ptr+= 1;
        ptr = advance_ptr(ptr, 1);
        bytes_left -= mem::size_of_val(&password_length);
        if bytes_left != password_length as usize {
            //  PLAIN I: invalid PLAIN client, sent malformed password or
            //  extraneous data
            session.get_socket ().event_handshake_failed_protocol (
                session.get_endpoint (),
                ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_HELLO);
          // errno = EPROTO;
            return -1;
        }

        let password = String::from_utf8_lossy(&ptr[..password_length]);

        //  Use ZAP protocol (RFC 27) to authenticate the user.
        rc = session.zap_connect ();
        if rc != 0 {
            session.get_socket ().event_handshake_failed_no_detail (
                session.get_endpoint (), EFAULT);
            return -1;
        }

        send_zap_request (username, password);
        state = waiting_for_zap_reply;

        //  TODO actually, it is quite unlikely that we can read the ZAP
        //  reply already, but removing this has some strange side-effect
        //  (probably because the pipe's in_active flag is true until a read
        //  is attempted)
        return if receive_and_process_zap_reply () == -1 { -1 } else { 0 };
    }

    pub fn produce_welcome (&mut self, msg: &mut ZmqMessage)
    {
        let rc: i32 = msg.init_size (welcome_prefix_len);
        // errno_assert (rc == 0);
        copy_bytes (msg.data_mut(), 0, welcome_prefix, 0, welcome_prefix.len() as i32);
    }

    pub fn process_initiate (&mut self, msg: &mut ZmqMessage) -> i32
    {
        let ptr =  (msg.data_mut());
        let mut bytes_left = msg.size ();

        if (bytes_left < initiate_prefix_len
            || cmp_bytes (ptr, 0,  initiate_prefix, 0, initiate_prefix_len) != 0) {
            session.get_socket ().event_handshake_failed_protocol (
                session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND);
          // errno = EPROTO;
            return -1;
        }
        let rc: i32 = parse_metadata (ptr + initiate_prefix_len,
                                      bytes_left - initiate_prefix_len);
        if (rc == 0) {
            state = sending_ready;
        }
        return rc;
    }

    pub fn produce_ready (&mut self, msg: &mut ZmqMessage)
    {
        make_command_with_basic_properties (msg, ready_prefix, ready_prefix_len);
    }

    pub fn produce_error (&mut self, msg: &mut ZmqMessage)
    {
        let mut expected_status_code_len = 3;
        // zmq_assert (status_code.length ()
        //             ==  (expected_status_code_len));
        let status_code_len_size = 4usize;
        let rc: i32 = msg.init_size (error_prefix_len + status_code_len_size
            + expected_status_code_len);
        // zmq_assert (rc == 0);
        let msg_data =  (msg.data_mut());
        copy_bytes(msg_data, 0,error_prefix, 0,error_prefix_len);
        msg_data[error_prefix_len] = expected_status_code_len;
        copy_bytes (msg_data + error_prefix_len + status_code_len_size,0,
                    status_code, 0,status_code.length ());
    }

    pub fn send_zap_request ( &mut self, username_: &str, password_: &str)
    {
        let credentials: [&str;2] = [
            username_,
            password_];
        let credentials_sizes: [usize;2] = [username_.len(), password_.len()];
        let plain_mechanism_name: &str = "PLAIN";
        self.zap_client.send_zap_request (
            plain_mechanism_name, plain_mechanism_name.len() - 1, credentials,
            credentials_sizes, credentials.len() / sizeof (credentials[0].len()));
    }
} // impl plain server







