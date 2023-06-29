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

use anyhow::bail;
use libc::{EAGAIN, EPROTO};
use crate::context::ZmqContext;

use crate::defines::{
    ZMQ_PROTOCOL_ERROR_ZAP_BAD_REQUEST_ID, ZMQ_PROTOCOL_ERROR_ZAP_BAD_VERSION,
    ZMQ_PROTOCOL_ERROR_ZAP_INVALID_METADATA, ZMQ_PROTOCOL_ERROR_ZAP_INVALID_STATUS_CODE,
    ZMQ_PROTOCOL_ERROR_ZAP_MALFORMED_REPLY, ZMQ_PROTOCOL_ERROR_ZAP_UNSPECIFIED,
};
use crate::mechanism::ZmqMechanism;
use crate::mechanism_base::ZmqMechanismBase;
use crate::message::{close_and_return, ZmqMessage, ZMQ_MSG_MORE};

use crate::session_base::ZmqSessionBase;
use crate::utils::{cmp_bytes, copy_bytes};
use crate::zap_client::ZmqZapClientCommonHandshakeState::{
    error_sent, ready, sending_error, waiting_for_zap_reply,
};

const zap_reply_frame_count: usize = 7;

// const char zap_version[] = "1.0"; const zap_version: &'static str = "1.0";
pub const zap_version: &'static str = "1.0";
// const size_t zap_version_len = mem::size_of::<zap_version>() - 1; const id: &str = "1";
pub const zap_version_len: usize = zap_version.len();

// const size_t id_len = mem::size_of::<id>() - 1;

// #include "zap_client.hpp"
// #include "msg.hpp"
// #include "session_base.hpp"
#[derive(Default, Debug, Clone)]
pub struct ZmqZapClient {
    // public virtual ZmqMechanismBase
    //
    //     ZmqZapClient (ZmqSessionBase *session_,
    //                   const std::string &peer_address_,
    //                   options: &ZmqOptions);
    //
    //     void send_zap_request (mechanism_: &str,
    //                            mechanism_length_: usize,
    //                            credentials_: &[u8],
    //                            credentials_size_: usize);
    //
    //     void send_zap_request (mechanism_: &str,
    //                            mechanism_length_: usize,
    //                            const uint8_t **credentials_,
    //                            size_t *credentials_sizes_,
    //                            credentials_count_: usize);
    //
    //     virtual int receive_and_process_zap_reply ();
    //     virtual void handle_zap_status_code ();
    //
    //   const peer_address: String;
    pub peer_address: String,
    //  Status code as received from ZAP handler
    // status_code: String;
    pub status_code: String,
    pub base: ZmqMechanismBase,
}

impl ZmqZapClient {
    pub fn new(ctx: &mut ZmqContext, session: &mut ZmqSessionBase, peer_address_: &str) -> Self {
        // ZmqMechanismBase (session_, options_), peer_address (peer_address_)
        Self {
            base: ZmqMechanismBase::new(ctx, session),
            peer_address: peer_address_.to_string(),
            ..Default::default()
        }
    }

    pub fn send_zap_request(
        &mut self,
        ctx: &mut ZmqContext,
        mechanism_: &str,
        mechanism_length_: usize,
        credentials_: &[u8],
        credentials_size_: &[usize],
    ) {
        self.send_zap_request2(
            ctx,
            mechanism_,
            mechanism_length_,
            &mut credentials_.to_vec(),
            &mut credentials_size_.to_vec(),
            1,
        );
    }

    pub fn send_zap_request2(
        &mut self,
        ctx: &mut ZmqContext,
        mechanism_: &str,
        mechanism_length_: usize,
        credentials_: &mut Vec<u8>,
        credentials_sizes_: &mut Vec<usize>,
        credentials_count_: usize,
    ) -> anyhow::Result<()> {
        // write_zap_msg cannot fail. It could only fail if the HWM was exceeded,
        // but on the ZAP socket, the HWM is disabled.

        let mut rc: i32;
        let mut msg = ZmqMessage::default();

        //  Address delimiter frame
        msg.init2()?;
        // errno_assert (rc == 0);
        msg.set_flags(ZMQ_MSG_MORE);
        self.base.session.write_zap_msg(&mut msg)?;
        // errno_assert (rc == 0);

        //  Version frame
        msg.init_size(zap_version_len)?;
        // errno_assert (rc == 0);
        copy_bytes(
            msg.data_mut(),
            0,
            zap_version.as_bytes(),
            0,
            zap_version_len,
        );
        msg.set_flags(ZMQ_MSG_MORE);
        self.base.session.write_zap_msg(&mut msg)?;
        // errno_assert (rc == 0);

        //  Request ID frame
        msg.init_size(id_len)?;
        // errno_assert (rc == 0);
        copy_bytes(msg.data_mut(), 0, id.as_bytes(), 0, id_len);
        msg.set_flags(ZMQ_MSG_MORE);
        rc = session.write_zap_msg(&msg);
        // errno_assert (rc == 0);

        //  Domain frame
        rc = msg.init_size(ctx.zap_domain.length());
        // errno_assert (rc == 0);
        copy_bytes(
            msg.data_mut(),
            0,
            ctx.zap_domain.as_bytes(),
            0,
            ctx.zap_domain.length(),
        );
        msg.set_flags(ZMQ_MSG_MORE);
        rc = session.write_zap_msg(&msg);
        // errno_assert (rc == 0);

        //  Address frame
        rc = msg.init_size(peer_address.length());
        // errno_assert (rc == 0);
        copy_bytes(msg.data_mut(), 0, peer_address, 0, peer_address.length());
        msg.set_flags(ZMQ_MSG_MORE);
        rc = session.write_zap_msg(&msg);
        // errno_assert (rc == 0);

        //  Routing id frame
        rc = msg.init_size(ctx.routing_id_size);
        // errno_assert (rc == 0);
        copy_bytes(
            msg.data_mut(),
            0,
            &ctx.routing_id,
            0,
            ctx.routing_id_size,
        );
        msg.set_flags(ZMQ_MSG_MORE);
        rc = session.write_zap_msg(&msg);
        // errno_assert (rc == 0);

        //  Mechanism frame
        rc = msg.init_size(mechanism_length_);
        // errno_assert (rc == 0);
        copy_bytes(
            msg.data_mut(),
            0,
            mechanism_.as_bytes(),
            0,
            mechanism_length_,
        );
        if (credentials_count_) {
            msg.set_flags(ZMQ_MSG_MORE);
        }
        rc = session.write_zap_msg(&msg);
        // errno_assert (rc == 0);

        //  Credentials frames
        // for (size_t i = 0; i < credentials_count_; += 1i)
        for i in 0..credentials_count_ {
            rc = msg.init_size(credentials_sizes_[i]);
            // errno_assert (rc == 0);
            if (i < credentials_count_ - 1) {
                msg.set_flags(ZMQ_MSG_MORE);
            }
            copy_bytes(
                msg.data_mut(),
                0,
                credentials_[i].as_slice(),
                0,
                credentials_sizes_[i],
            );
            rc = session.write_zap_msg(&msg);
            // errno_assert (rc == 0);
        }
        Ok(())
    }

    pub fn receive_and_process_zap_reply(&mut self) -> anyhow::Result<()> {
        let mut rc = 0;

        let mut msg: [ZmqMessage; zap_reply_frame_count];

        //  Initialize all reply frames
        // for (size_t i = 0; i < zap_reply_frame_count; i+= 1)
        for i in 0..zap_reply_frame_count {
            msg[i].init2()?;
            // errno_assert(rc == 0);
        }

        // for (size_t i = 0; i < zap_reply_frame_count; i+= 1)
        for i in 0..zap_reply_frame_count {
            rc = session.read_zap_msg(&msg[i]);
            if (rc == -1) {
                if (errno == EAGAIN) {
                    // return 1;
                    bail!("EAGAIN")
                }
                return close_and_return(&mut msg[0], -1);
            }
            if ((msg[i].flags() & ZMQ_MSG_MORE)
                == (if i < zap_reply_frame_count - 1 {
                0
            } else {
                ZMQ_MSG_MORE
            }))
            {
                session.get_socket().event_handshake_failed_protocol(
                    session.get_endpoint(),
                    ZMQ_PROTOCOL_ERROR_ZAP_MALFORMED_REPLY,
                );
                errno = EPROTO;
                return close_and_return(&mut msg[0], -1);
            }
        }

        //  Address delimiter frame
        if (msg[0].size() > 0) {
            //  TODO can a ZAP handler produce such a message at all?
            session.get_socket().event_handshake_failed_protocol(
                session.get_endpoint(),
                ZMQ_PROTOCOL_ERROR_ZAP_UNSPECIFIED,
            );
            errno = EPROTO;
            return close_and_return(&mut msg[0], -1);
        }

        //  Version frame
        if (msg[1].size() != zap_version_len
            || cmp_bytes(msg[1].data(), 0, zap_version.as_bytes(), 0, zap_version_len) != 0)
        {
            session.get_socket().event_handshake_failed_protocol(
                session.get_endpoint(),
                ZMQ_PROTOCOL_ERROR_ZAP_BAD_VERSION,
            );
            errno = EPROTO;
            return close_and_return(&mut msg[0], -1);
        }

        //  Request id frame
        if (msg[2].size() != id_len || cmp_bytes(msg[2].data(), 0, id.as_bytes(), 0, id_len) != 0) {
            session.get_socket().event_handshake_failed_protocol(
                session.get_endpoint(),
                ZMQ_PROTOCOL_ERROR_ZAP_BAD_REQUEST_ID,
            );
            errno = EPROTO;
            return close_and_return(&mut msg[0], -1);
        }

        //  Status code frame, only 200, 300, 400 and 500 are valid status codes
        let status_code_data = (msg[3].data());
        if msg[3].size() != 3
            || status_code_data[0] < b'2'
            || status_code_data[0] > b'5'
            || status_code_data[1] != b'0'
            || status_code_data[2] != b'0'
        {
            session.get_socket().event_handshake_failed_protocol(
                session.get_endpoint(),
                ZMQ_PROTOCOL_ERROR_ZAP_INVALID_STATUS_CODE,
            );
            errno = EPROTO;
            return close_and_return(&mut msg[0], -1);
        }

        //  Save status code
        status_code.assign((msg[3].data()), 3);

        //  Save user id
        set_user_id(msg[5].data(), msg[5].size());

        //  Process metadata frame
        rc = parse_metadata(static_cast(msg[6].data()), msg[6].size(), true);

        if (rc != 0) {
            session.get_socket().event_handshake_failed_protocol(
                session.get_endpoint(),
                ZMQ_PROTOCOL_ERROR_ZAP_INVALID_METADATA,
            );
            errno = EPROTO;
            return close_and_return(&mut msg[0], -1);
        }

        //  Close all reply frames
        // for (size_t i = 0; i < zap_reply_frame_count; i+= 1)
        for i in 0..zap_reply_frame_count {
            msg[i].close()?;
            // errno_assert(rc2 == 0);
        }

        handle_zap_status_code();

        Ok(())
    }

    pub fn handle_zap_status_code(&mut self) {
        //  we can assume here that status_code is a valid ZAP status code,
        //  i.e. 200, 300, 400 or 500
        let mut status_code_numeric = 0;
        match (status_code[0]) {
            b'2' => return,
            b'3' => status_code_numeric = 300,
            b'4' => status_code_numeric = 400,
            b'5' => status_code_numeric = 500,
        };

        session
            .get_socket()
            .event_handshake_failed_auth(session.get_endpoint(), status_code_numeric);
    }
}

enum ZmqZapClientCommonHandshakeState {
    waiting_for_hello,
    sending_welcome,
    waiting_for_initiate,
    waiting_for_zap_reply,
    sending_ready,
    sending_error,
    error_sent,
    ready,
}

#[derive(Default, Debug, Clone)]
pub struct ZmqZapClientCommonHandshake {
    // public ZmqZapClient
    //
    // ZmqZapClientCommonHandshake (ZmqSessionBase *session_,
    //                                const std::string &peer_address_,
    //                                options: &ZmqOptions,
    //                                state_t zap_reply_ok_state_);

    //  methods from mechanism_t
    // status_t status () const ;
    // int zap_msg_available () ;

    //  ZmqZapClient methods
    // int receive_and_process_zap_reply () ;
    // void handle_zap_status_code () ;

    //  Current FSM state
    // state_t state;
    pub state: ZmqZapClientCommonHandshakeState,

    //
    //   const state_t _zap_reply_ok_state;
    pub _zap_reply_ok_state: ZmqZapClientCommonHandshakeState,
}

impl ZmqZapClientCommonHandshake {
    // ZmqZapClientCommonHandshake::ZmqZapClientCommonHandshake (
    // ZmqSessionBase *const session_,
    // const std::string &peer_address_,
    // options: &ZmqOptions,
    // state_t zap_reply_ok_state_) :
    // ZmqMechanismBase (session_, options_),
    // ZmqZapClient (session_, peer_address_, options_),
    // state (waiting_for_hello),
    // _zap_reply_ok_state (zap_reply_ok_state_)
    // {
    // }
    pub fn new(
        ctx: &mut ZmqContext,
        session: &mut ZmqSessionBase,
        peer_address: &str,
        sending_ready: ZmqZapClientCommonHandshakeState,
    ) -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn status(&mut self) -> status_t {
        if (state == ready) {
            return ZmqMechanism::ready;
        }
        if (state == error_sent) {
            return ZmqMechanism::error;
        }

        return ZmqMechanism::handshaking;
    }

    pub fn zap_msg_available(&mut self) -> i32 {
        // zmq_assert(state == waiting_for_zap_reply);
        if receive_and_process_zap_reply() == -1 {
            -1
        }
        {
            0
        }
    }

    pub fn handle_zap_status_code(&mut self) {
        // self.handle_zap_status_code();

        //  we can assume here that status_code is a valid ZAP status code,
        //  i.e. 200, 300, 400 or 500
        match (status_code[0]) {
            b'2' => state = _zap_reply_ok_state,
            b'3' => state = error_sent,
            _ => state = sending_error,
        };
    }

    pub fn receive_and_process_zap_reply(&mut self) -> anyhow::Result<()> {
        // zmq_assert(state == waiting_for_zap_reply);
        // return ZmqZapClient::receive_and_process_zap_reply();
        Ok(())
    }
}

// namespace zmq
// {

// }
