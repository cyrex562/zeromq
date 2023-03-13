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
// #include "macros.hpp"

// #ifdef ZMQ_HAVE_CURVE

// #include "msg.hpp"
// #include "session_base.hpp"
// #include "err.hpp"
// #include "curve_client.hpp"
// #include "wire.hpp"
// #include "curve_client_tools.hpp"
// #include "secure_allocator.hpp"

use libc::EPROTO;
use crate::curve_mechanism_base::curve_mechanism_base_t;
use crate::mechanism::ZmqMechanismStatus;
use crate::mechanism_base::mechanism_base_t;
use crate::message::ZmqMessage;
use crate::options::ZmqOptions;
use crate::session_base::session_base_t;
use crate::zmq_hdr::{ZMQ_PROTOCOL_ERROR_ZMTP_CRYPTOGRAPHIC, ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND};

pub enum ZmqCurveClientState
{
    send_hello,
    expect_welcome,
    send_initiate,
    expect_ready,
    error_received,
    connected
}

// pub struct curve_client_t ZMQ_FINAL : public curve_mechanism_base_t
pub struct ZmqCurveClient
{
    // public:
    // private:
    //  Current FSM state
    // state_t _state;
    pub state: ZmqCurveClientState,
    //  CURVE protocol tools
    // curve_client_tools_t _tools;
    pub tools: ZmqCurveClientTools,
    pub mechanism_base: mechanism_base_t,
    pub curve_mechanism_base: curve_mechanism_base_t,
}

impl ZmqCurveClient {
    // curve_client_t (session_base_t *session_,
    // const ZmqOptions &options_,
    // const downgrade_sub_: bool);
    // curve_client_t::curve_client_t (session_base_t *session_,
    // const ZmqOptions &options_,
    // const downgrade_sub_: bool) :
    // mechanism_base_t (session_, options_),
    // curve_mechanism_base_t (session_,
    // options_,
    // "CurveZMQMESSAGEC",
    // "CurveZMQMESSAGES",
    // downgrade_sub_),
    // _state (send_hello),
    // _tools (options_.curve_public_key,
    // options_.curve_secret_key,
    // options_.curve_server_key)
    // {
    // }
    pub fn new(session: &mut session_base_t,
               options: &mut ZmqOptions,
               downgrade_sub: bool) -> Self {
        Self {
            mechanism_base: mechanism_base_t::new(session, options),
            curve_mechanism_base: curve_mechanism_base_t::new(session, options, "CurveZMQMESSAGEC", "CurveZMQMESSAGES", downgrade_sub),
            state: ZmqCurveClientState::send_hello,
            tools: ZmqCurveClientTools::new(options.curve_public_key, options.curve_secret_key, options.curve_server_key),
        }
    }

    // ~curve_client_t () ZMQ_FINAL;
    // curve_client_t::~curve_client_t ()
    // {
    // }

    // // mechanism implementation
    // int next_handshake_command (msg: &mut ZmqMessage) ZMQ_FINAL;
    pub fn next_handshake_command (&mut self, msg: &mut ZmqMessage) -> anyhow::Result<()>
    {
        // int rc = 0;
        let mut rc = 0i32;

        match self.state {
            ZmqCurveClientState::send_hello => {
                rc = produce_hello(msg);
                if rc == 0 {
                    self.state = ZmqCurveClientState::expect_welcome;
                    return Ok(());
                } else {
                    return Err(anyhow!("produce_hello command failed"))
                }
            }
            // ZmqCurveClientState::expect_welcome => {
            //
            // }
            ZmqCurveClientState::send_initiate => {
                rc = produce_initiate(msg);
                if rc == 0 {
                    self.state = ZmqCurveClientState::expect_ready;
                    return Ok(());
                } else {
                    return Err(anyhow!("produce_initiate command failed"));
                }
            }
            // ZmqCurveClientState::expect_ready => {}
            // ZmqCurveClientState::error_received => {}
            // ZmqCurveClientState::connected => {}
            _ => {
                return Err(anyhow!("EAGAIN"));
            }
        }

        // case send_hello:
        // rc = produce_hello (msg);
        // if (rc == 0)
        // _state = expect_welcome;
        // break;
        // case send_initiate:
        // rc = produce_initiate (msg);
        // if (rc == 0)
        // _state = expect_ready;
        // break;
        // default:
        // errno = EAGAIN;
        // rc = -1;
        // }
        // return rc;
    }
    // int process_handshake_command (msg: &mut ZmqMessage) ZMQ_FINAL;
    pub fn process_handshake_command (&mut self, msg: &mut ZmqMessage) -> anyhow::Result<()>
    {
        let msg_data = msg.data()?;
        let msg_size = msg.size ();
        let rc = 0;
        if (is_handshake_command_welcome (msg_data.as_slice(), msg_size)) {
            self.process_welcome(msg_data.as_slice(), msg_size)?;
        }
        else if (is_handshake_command_ready (msg_data.as_slice(),
        msg_size)) {
            self.process_ready(msg_data.as_slice(), msg_size)?;
        }
        else if (is_handshake_command_error (msg_data.as_slice(),
        msg_size)) {
            self.process_error(msg_data, msg_size)?;
        }
        else {
            self.ession.get_socket().event_handshake_failed_protocol (
                self.session.get_endpoint (),
                ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND);
            // errno = EPROTO;
            // rc = -1;
            return Err(anyhow!("EPROTO"));
        }

        // if (rc == 0) {
        //     rc = msg.close();
        //     // errno_assert (rc == 0);
        //     rc = msg.init();
        //     // errno_assert(rc == 0);
        // }
        msg.close()?;
        msg.init2()?;
        Ok(())

        // return rc;
    }

    // int encode (msg: &mut ZmqMessage) ZMQ_FINAL;
    pub fn encode (&mut self, msg: &mut ZmqMessage) -> anyhow::Result<()>
    {
        // zmq_assert (_state == connected);
        self.curve_mechanism_base.encode (msg)
    }

    // int decode (msg: &mut ZmqMessage) ZMQ_FINAL;
    pub fn decode (&mut self, msg: &mut ZmqMessage) -> anyhow::Result<()>
    {
        // zmq_assert (_state == connected);
        self.curve_mechanism_base.decode (msg)
    }

    // status_t status () const ZMQ_FINAL;
    pub fn status(&mut self) -> ZmqMechanismStatus
    {
        if (self.state == ZmqCurveClientState::connected) {
            return ZmqMechanismStatus::ready;
        }
        if (self.state == ZmqCurveClientState::error_received) {
            return ZmqMechanismStatus::error;
        }
        return ZmqMechanismStatus::handshaking;
    }

    // int produce_hello (msg: &mut ZmqMessage);
    pub fn produce_hello(&mut self, msg: &mut ZmqMessage) -> anyhow::Result<()>
    {
        msg.init_size(200)?;
        // errno_assert (rc == 0);

        match tools.produce_hello(msg.data(), get_and_inc_nonce()) {
            Ok(_) => {
                Ok(())
            }
            Err(e) => {
                self.session.get_socket().event_handshake_failed_protocol(
                    self.session.get_endpoint(),
                    ZMQ_PROTOCOL_ERROR_ZMTP_CRYPTOGRAPHIC);

                // TODO this is somewhat inconsistent: we call init_size, but we may
                // not close msg; i.e. we assume that msg is initialized but empty
                // (if it were non-empty, calling init_size might cause a leak!)

                // msg->close ();
                Err(anyhow!("error occurred: {}", e))
            }
        }
        // if (rc == -1) {
        //     session.get_socket ()->event_handshake_failed_protocol (
        //     session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZMTP_CRYPTOGRAPHIC);
        //
        //     // TODO this is somewhat inconsistent: we call init_size, but we may
        //     // not close msg; i.e. we assume that msg is initialized but empty
        //     // (if it were non-empty, calling init_size might cause a leak!)
        //
        //     // msg->close ();
        //     return -1;
        // }
        //
        // return 0;
    }

    //     int process_welcome (const uint8_t *msg_data_, msg_size_: usize);
    pub fn process_welcome (&mut self, msg_data: &[u8], msg_size: usize) -> anyhow::Result<()>
    {
        match self.tools.process_welcome (msg_data, msg_size, get_writable_precom_buffer ()) {
            Ok(_) => {
                self.state = ZmqCurveClientState::send_initiate;
                return Ok(());
            }
            Err(e) => {
                self.session.get_socket().event_handshake_failed_protocol (
                    self.session.get_endpoint (),
                    ZMQ_PROTOCOL_ERROR_ZMTP_CRYPTOGRAPHIC)?;
                return Err(anyhow!("EPROTO: {}", e))
            }
        }

    // if (rc == -1) {
    // session.get_socket ()->event_handshake_failed_protocol (
    // session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZMTP_CRYPTOGRAPHIC);
    //
    // errno = EPROTO;
    // return -1;
    // }
    //
    // _state = send_initiate;
    //
    // return 0;
    }


    //     int produce_initiate (msg: &mut ZmqMessage);
    pub fn produce_initiate(&mut self, msg: &mut ZmqMessage) -> anyhow::Result<()>
    {
        let metadata_length = basic_properties_len();
        // std::vector<unsigned char, secure_allocator_t<unsigned char> >
        // metadata_plaintext (metadata_length);
        let mut metadata_plaintext: Vec<u8> = Vec::with_capacity(metadata_length);

        add_basic_properties(&metadata_plaintext[0], metadata_length);

        let msg_size = 113 + 128 + crypto_box_BOXZEROBYTES + metadata_length;
        msg.init_size(msg_size)?;
        // errno_assert (rc == 0);

        match self.tools.produce_initiate(msg.data(), msg_size, get_and_inc_nonce(),
                                          &metadata_plaintext[0], metadata_length) {
            Err(e) => {
                self.session.get_socket().event_handshake_failed_protocol(
                    session.get_endpoint(), ZMQ_PROTOCOL_ERROR_ZMTP_CRYPTOGRAPHIC)?;
                return Err(anyhow!("produce_initiate failed: {}", e))
            }
            Ok(_) => {
                Ok(())
            }
        }
        //
        // if (-1 == rc) {
        //
        //
        // // TODO see comment in produce_hello
        // return -1;
        // }
        //
        // return 0;
    }


    //     int process_ready (const uint8_t *msg_data_, msg_size_: usize);
    pub fn process_ready (&mut self, msg_data: &[u8], msg_size: usize) -> anyhow::Result<()>
    {
        if (msg_size_ < 30) {
            self.session.get_socket ().event_handshake_failed_protocol (
                self.session.get_endpoint (),
                ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_READY);
            errno = EPROTO;
            return -1;
        }

        let clen = (msg_size - 14) + crypto_box_BOXZEROBYTES;

        let mut ready_nonce: [u8;crypto_box_NONCEBYTES] = [0;crypto_box_NONCEBYTES];
        // std::vector<uint8_t, secure_allocator_t<uint8_t> > ready_plaintext (
        // crypto_box_ZEROBYTES + clen);
        let mut ready_plaintext: Vec<u8> = Vec::with_capacity(crypto_box_ZEROBYTES + clen);
        std::vector<uint8_t> ready_box (crypto_box_BOXZEROBYTES + 16 + clen);

        std::fill (ready_box.begin (), ready_box.begin () + crypto_box_BOXZEROBYTES,
        0);
        memcpy (&ready_box[crypto_box_BOXZEROBYTES], msg_data_ + 14,
        clen - crypto_box_BOXZEROBYTES);

        memcpy (ready_nonce, "CurveZMQREADY---", 16);
        memcpy (ready_nonce + 16, msg_data_ + 6, 8);
        set_peer_nonce (get_uint64 (msg_data_ + 6));

        int rc = crypto_box_open_afternm (&ready_plaintext[0], &ready_box[0], clen,
        ready_nonce, get_precom_buffer ());

        if (rc != 0) {
        session.get_socket ()->event_handshake_failed_protocol (
        session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZMTP_CRYPTOGRAPHIC);
        errno = EPROTO;
        return -1;
        }

        rc = parse_metadata (&ready_plaintext[crypto_box_ZEROBYTES],
        clen - crypto_box_ZEROBYTES);

        if (rc == 0)
        _state = connected;
        else {
        session.get_socket ()->event_handshake_failed_protocol (
        session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZMTP_INVALID_METADATA);
        errno = EPROTO;
        }

        return rc;
    }



    //     int process_error (const uint8_t *msg_data_, msg_size_: usize);





















    int curve_client_t::process_error (const uint8_t *msg_data_,
    msg_size_: usize)
    {
    if (_state != expect_welcome && _state != expect_ready) {
    session.get_socket ()->event_handshake_failed_protocol (
    session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND);
    errno = EPROTO;
    return -1;
    }
    if (msg_size_ < 7) {
    session.get_socket ()->event_handshake_failed_protocol (
    session.get_endpoint (),
    ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_ERROR);
    errno = EPROTO;
    return -1;
    }
    const size_t error_reason_len = static_cast<size_t> (msg_data_[6]);
    if (error_reason_len > msg_size_ - 7) {
    session.get_socket ()->event_handshake_failed_protocol (
    session.get_endpoint (),
    ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_ERROR);
    errno = EPROTO;
    return -1;
    }
    const char *error_reason = reinterpret_cast<const char *> (msg_data_) + 7;
    handle_error_reason (error_reason, error_reason_len);
    _state = error_received;
    return 0;
    }

}


// #endif
