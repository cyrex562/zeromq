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

use crate::mechanism_base::ZmqMechanismBase;
use crate::message::{ZMQ_MSG_MORE, ZmqMessage};
use crate::options::ZmqOptions;
use crate::session_base::ZmqSessionBase;

const zap_reply_frame_count: usize = 7;

// const char zap_version[] = "1.0";
const zap_version: &'static str = "1.0";
// const size_t zap_version_len = mem::size_of::<zap_version>() - 1;
const id: &str = "1";

// const size_t id_len = mem::size_of::<id>() - 1;

// #include "zap_client.hpp"
// #include "msg.hpp"
// #include "session_base.hpp"
#[derive(Default, Debug, Clone)]
pub struct ZmqZapClient
{
    // public virtual ZmqMechanismBase
// public:
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
    // protected:
    //   const peer_address: String;
    pub peer_address: String,
    //  Status code as received from ZAP handler
    // status_code: String;
    pub status_code: String,
    pub mechanism_base: ZmqMechanismBase,
}

impl ZmqZapClient {
    pub fn new(session: &ZmqSessionBase,
               peer_address_: &str,
               options: &ZmqOptions) -> Self
    {
        // ZmqMechanismBase (session_, options_), peer_address (peer_address_)
        Self {
            mechanism_base: ZmqMechanismBase::new2(session, options),
            peer_address: peer_address_.to_string(),
            ..Default::default()
        }
    }


    pub fn send_zap_request(&mut self, mechanism_: &str,
                            mechanism_length_: usize,
                            credentials_: &[u8],
                            credentials_size_: usize)
    {
        send_zap_request(mechanism_, mechanism_length_, &credentials_,
                         &credentials_size_, 1);
    }

    pub fn send_zap_request2(&mut self, mechanism_: &str,
                             mechanism_length_: usize,
                             credentials_: &mut Vec<Vec<u8>>,
                             credentials_sizes_: &mut Vec<usize>,
                             credentials_count_: usize)
    {
        // write_zap_msg cannot fail. It could only fail if the HWM was exceeded,
        // but on the ZAP socket, the HWM is disabled.

        let mut rc: i32;
        let mut msg = ZmqMessage::default();

        //  Address delimiter frame
        rc = msg.init();
        // errno_assert (rc == 0);
        msg.set_flags(ZMQ_MSG_MORE);
        rc = session.write_zap_msg(&msg);
        // errno_assert (rc == 0);

        //  Version frame
        rc = msg.init_size(zap_version_len);
        // errno_assert (rc == 0);
        memcpy(msg.data(), zap_version, zap_version_len);
        msg.set_flags(ZMQ_MSG_MORE);
        rc = session.write_zap_msg(&msg);
        // errno_assert (rc == 0);

        //  Request ID frame
        rc = msg.init_size(id_len);
        // errno_assert (rc == 0);
        memcpy(msg.data(), id, id_len);
        msg.set_flags(ZMQ_MSG_MORE);
        rc = session.write_zap_msg(&msg);
        // errno_assert (rc == 0);

        //  Domain frame
        rc = msg.init_size(options.zap_domain.length());
        // errno_assert (rc == 0);
        memcpy(msg.data(), options.zap_domain,
               options.zap_domain.length());
        msg.set_flags(ZMQ_MSG_MORE);
        rc = session.write_zap_msg(&msg);
        // errno_assert (rc == 0);

        //  Address frame
        rc = msg.init_size(peer_address.length());
        // errno_assert (rc == 0);
        memcpy(msg.data(), peer_address, peer_address.length());
        msg.set_flags(ZMQ_MSG_MORE);
        rc = session.write_zap_msg(&msg);
        // errno_assert (rc == 0);

        //  Routing id frame
        rc = msg.init_size(options.routing_id_size);
        // errno_assert (rc == 0);
        memcpy(msg.data(), options.routing_id, options.routing_id_size);
        msg.set_flags(ZMQ_MSG_MORE);
        rc = session.write_zap_msg(&msg);
        // errno_assert (rc == 0);

        //  Mechanism frame
        rc = msg.init_size(mechanism_length_);
        // errno_assert (rc == 0);
        memcpy(msg.data(), mechanism_, mechanism_length_);
        if (credentials_count_)
        msg.set_flags(ZMQ_MSG_MORE);
        rc = session.write_zap_msg(&msg);
        // errno_assert (rc == 0);

        //  Credentials frames
        for (size_t i = 0; i < credentials_count_; += 1i) {
            rc = msg.init_size(credentials_sizes_[i]);
            // errno_assert (rc == 0);
            if (i < credentials_count_ - 1)
            msg.set_flags(ZMQ_MSG_MORE);
            memcpy(msg.data(), credentials_[i], credentials_sizes_[i]);
            rc = session.write_zap_msg(&msg);
            // errno_assert (rc == 0);
        }
    }

    pub fn receive_and_process_zap_reply(&mut self) -> i32
    {
        let mut rc = 0;

        let mut msg: [ZmqMessage; zap_reply_frame_count];

        //  Initialize all reply frames
        for (size_t i = 0; i < zap_reply_frame_count; i+= 1) {
            rc = msg[i].init();
            errno_assert(rc == 0);
        }

        for (size_t i = 0; i < zap_reply_frame_count; i+= 1) {
            rc = session.read_zap_msg(&msg[i]);
            if (rc == -1) {
                if (errno == EAGAIN) {
                    return 1;
                }
                return close_and_return(msg, -1);
            }
            if ((msg[i].flags() & ZMQ_MSG_MORE)
                == (i < zap_reply_frame_count - 1?
            0: ZMQ_MSG_MORE)) {
            session.get_socket ().event_handshake_failed_protocol (
            session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZAP_MALFORMED_REPLY);
            errno = EPROTO;
            return close_and_return (msg, - 1);
            }
        }

        //  Address delimiter frame
        if (msg[0].size() > 0) {
            //  TODO can a ZAP handler produce such a message at all?
            session.get_socket().event_handshake_failed_protocol(
                session.get_endpoint(), ZMQ_PROTOCOL_ERROR_ZAP_UNSPECIFIED);
            errno = EPROTO;
            return close_and_return(msg, -1);
        }

        //  Version frame
        if (msg[1].size() != zap_version_len
            || memcmp(msg[1].data(), zap_version, zap_version_len)) {
            session.get_socket().event_handshake_failed_protocol(
                session.get_endpoint(), ZMQ_PROTOCOL_ERROR_ZAP_BAD_VERSION);
            errno = EPROTO;
            return close_and_return(msg, -1);
        }

        //  Request id frame
        if (msg[2].size() != id_len || memcmp(msg[2].data(), id, id_len)) {
            session.get_socket().event_handshake_failed_protocol(
                session.get_endpoint(), ZMQ_PROTOCOL_ERROR_ZAP_BAD_REQUEST_ID);
            errno = EPROTO;
            return close_and_return(msg, -1);
        }

        //  Status code frame, only 200, 300, 400 and 500 are valid status codes
        const char
        *status_code_data = static_cast <const char
        * > (msg[3].data());
        if (msg[3].size() != 3 || status_code_data[0] < '2'
            || status_code_data[0] > '5' || status_code_data[1] != '0'
            || status_code_data[2] != '0') {
            session.get_socket().event_handshake_failed_protocol(
                session.get_endpoint(), ZMQ_PROTOCOL_ERROR_ZAP_INVALID_STATUS_CODE);
            errno = EPROTO;
            return close_and_return(msg, -1);
        }

        //  Save status code
        status_code.assign(static_cast < char * > (msg[3].data()), 3);

        //  Save user id
        set_user_id(msg[5].data(), msg[5].size());

        //  Process metadata frame
        rc = parse_metadata(static_cast < const unsigned char * > (msg[6].data()),
                            msg[6].size(), true);

        if (rc != 0) {
            session.get_socket().event_handshake_failed_protocol(
                session.get_endpoint(), ZMQ_PROTOCOL_ERROR_ZAP_INVALID_METADATA);
            errno = EPROTO;
            return close_and_return(msg, -1);
        }

        //  Close all reply frames
        for (size_t i = 0; i < zap_reply_frame_count; i+= 1) {
            let rc2: i32 = msg[i].close();
            errno_assert(rc2 == 0);
        }

        handle_zap_status_code();

        return 0;
    }

    void ZmqZapClient::handle_zap_status_code ()
    {
    //  we can assume here that status_code is a valid ZAP status code,
    //  i.e. 200, 300, 400 or 500
    int status_code_numeric = 0;
    switch (status_code[0]) {
    case '2':
    return;
    case '3':
    status_code_numeric = 300;
    break;
    case '4':
    status_code_numeric = 400;
    break;
    case '5':
    status_code_numeric = 500;
    break;
    }

    session.get_socket ().event_handshake_failed_auth (
    session.get_endpoint (), status_code_numeric);
    }
}


enum ZmqZapClientCommonHandshakeState
{
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
pub struct ZmqZapClientCommonHandshake
{
// public ZmqZapClient
//   protected:
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

    // private:
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
    pub fn new() -> Self {}

    pub fn status(&mut self) -> status_t
    {
        if (state == ready)
        return ZmqMechanism::ready;
        if (state == error_sent)
        return ZmqMechanism::error;

        return ZmqMechanism::handshaking;
    }

    pub fn zap_msg_available(&mut self) -> i32
    {
        zmq_assert(state == waiting_for_zap_reply);
        return receive_and_process_zap_reply() == -1? - 1;: 0;
    }

    pub fn handle_zap_status_code(&mut self)
    {
        ZmqZapClient::handle_zap_status_code();

        //  we can assume here that status_code is a valid ZAP status code,
        //  i.e. 200, 300, 400 or 500
        switch(status_code[0])
        {
            case
            '2':
                state = _zap_reply_ok_state;
            break;
            case
            '3':
                //  a 300 error code (temporary failure)
                //  should NOT result in an ERROR message, but instead the
                //  client should be silently disconnected (see CURVEZMQ RFC)
                //  therefore, go immediately to state error_sent
                state = error_sent;
            break;
            _ =>
            state = sending_error;
        }
    }

    pub fn receive_and_process_zap_reply(&mut self) -> i32
    {
        zmq_assert(state == waiting_for_zap_reply);
        return ZmqZapClient::receive_and_process_zap_reply();
    }
}

// namespace zmq
// {


// }
