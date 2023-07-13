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

pub const zap_version: &'static str = "1.0";

pub const zap_version_len: usize = zap_version.len();


#[derive(Default, Debug, Clone)]
pub struct ZmqZapClient {
    pub peer_address: String,
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
        mechanism: &str,
        mechanism_length: usize,
        credentials: &[u8],
        credentials_size: &[usize],
    ) {
        self.send_zap_request2(
            ctx,
            mechanism,
            mechanism_length,
            &mut credentials.to_vec(),
            &mut credentials_size.to_vec(),
            1,
        );
    }

    pub fn send_zap_request2(
        &mut self,
        ctx: &mut ZmqContext,
        mechanism: &str,
        mechanism_length: usize,
        credentials: &mut Vec<u8>,
        credentials_sizes: &mut Vec<usize>,
        credentials_count: usize,
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
        msg.init_size(4)?;
        // TODO
        // copy_bytes(msg.data_mut(), 0, id.as_bytes(), 0, 4);
        msg.set_flags(ZMQ_MSG_MORE);
        self.base.session.write_zap_msg(&mut msg)?;
        // errno_assert (rc == 0);

        //  Domain frame
        msg.init_size(ctx.zap_domain.length());
        // errno_assert (rc == 0);
        copy_bytes(
            msg.data_mut(),
            0,
            ctx.zap_domain.as_bytes(),
            0,
            ctx.zap_domain.length(),
        );
        msg.set_flags(ZMQ_MSG_MORE);
        self.base.session.write_zap_msg(&mut msg)?;
        // errno_assert (rc == 0);

        //  Address frame
        msg.init_size(self.peer_address.length())?;
        // errno_assert (rc == 0);
        copy_bytes(msg.data_mut(), 0, self.peer_address.as_bytes(), 0, self.peer_address.length());
        msg.set_flags(ZMQ_MSG_MORE);
        self.base.session.write_zap_msg(&mut msg)?;
        // errno_assert (rc == 0);

        //  Routing id frame
        msg.init_size(ctx.routing_id_size)?;
        // errno_assert (rc == 0);
        copy_bytes(
            msg.data_mut(),
            0,
            &ctx.routing_id.as_bytes(),
            0,
            ctx.routing_id_size,
        );
        msg.set_flags(ZMQ_MSG_MORE);
        self.base.session.write_zap_msg(&mut msg)?;
        // errno_assert (rc == 0);

        //  Mechanism frame
        msg.init_size(mechanism_length)?;        // errno_assert (rc == 0);
        copy_bytes(
            msg.data_mut(),
            0,
            mechanism.as_bytes(),
            0,
            mechanism_length,
        );
        if (credentials_count) {
            msg.set_flags(ZMQ_MSG_MORE);
        }
        self.base.session.write_zap_msg(&mut msg)?;
        // errno_assert (rc == 0);

        //  Credentials frames
        // for (size_t i = 0; i < credentials_count_; += 1i)
        for i in 0..credentials_count {
            msg.init_size(credentials_sizes[i])?;
            // errno_assert (rc == 0);
            if (i < credentials_count - 1) {
                msg.set_flags(ZMQ_MSG_MORE);
            }
            copy_bytes(
                msg.data_mut(),
                0,
                credentials[i].as_slice(),
                0,
                credentials_sizes[i],
            );
            self.base.session.write_zap_msg(&mut msg)?;
            // errno_assert (rc == 0);
        }
        Ok(())
    }

    pub fn receive_and_process_zap_reply(&mut self, ctx: &mut ZmqContext) -> anyhow::Result<()> {
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
            self.base.session.read_zap_msg(&mut msg[i])?;
            if (rc == -1) {
                // if (errno == EAGAIN) {
                //     // return 1;
                //     bail!("EAGAIN")
                // }
                return close_and_return(&mut msg[0], -1);
            }
            if ((msg[i].flags() & ZMQ_MSG_MORE)
                == (if i < zap_reply_frame_count - 1 {
                0
            } else {
                ZMQ_MSG_MORE
            }))
            {
                self.base.session.get_socket().event_handshake_failed_protocol(ctx,
                                                                               self.base.session.get_endpoint(),
                                                                               ZMQ_PROTOCOL_ERROR_ZAP_MALFORMED_REPLY as i32,
                );
                // errno = EPROTO;
                return close_and_return(&mut msg[0], -1);
            }
        }

        //  Address delimiter frame
        if (msg[0].size() > 0) {
            //  TODO can a ZAP handler produce such a message at all?
            self.base.session.get_socket().event_handshake_failed_protocol(ctx,
                                                                           self.base.session.get_endpoint(),
                                                                           ZMQ_PROTOCOL_ERROR_ZAP_UNSPECIFIED as i32,
            );
            // errno = EPROTO;
            return close_and_return(&mut msg[0], -1);
        }

        //  Version frame
        if (msg[1].size() != zap_version_len
            || cmp_bytes(msg[1].data(), 0, zap_version.as_bytes(), 0, zap_version_len) != 0)
        {
            self.base.session.get_socket().event_handshake_failed_protocol(ctx,
                                                                           self.base.session.get_endpoint(),
                                                                           ZMQ_PROTOCOL_ERROR_ZAP_BAD_VERSION as i32,
            );
            // errno = EPROTO;
            return close_and_return(&mut msg[0], -1);
        }

        //  Request id frame
        // TODO
        // if (msg[2].size() != 4 || cmp_bytes(msg[2].data(), 0, id.as_bytes(), 0, 4) != 0) {
        //     self.base.session.get_socket().event_handshake_failed_protocol(ctx,
        //                                                                    self.base.session.get_endpoint(),
        //                                                                    ZMQ_PROTOCOL_ERROR_ZAP_BAD_REQUEST_ID as i32,
        //     );
        //     // errno = EPROTO;
        //     return close_and_return(&mut msg[0], -1);
        // }

        //  Status code frame, only 200, 300, 400 and 500 are valid status codes
        let status_code_data = (msg[3].data());
        if msg[3].size() != 3
            || status_code_data[0] < b'2'
            || status_code_data[0] > b'5'
            || status_code_data[1] != b'0'
            || status_code_data[2] != b'0'
        {
            self.base.session.get_socket().event_handshake_failed_protocol(ctx,
                                                                           self.base.session.get_endpoint(),
                                                                           ZMQ_PROTOCOL_ERROR_ZAP_INVALID_STATUS_CODE as i32,
            );
            // errno = EPROTO;
            return close_and_return(&mut msg[0], -1);
        }

        //  Save status code
        self.status_code.assign((msg[3].data()), 3);

        //  Save user id
        self.set_user_id(msg[5].data(), msg[5].size());

        //  Process metadata frame
        self.base.parse_metadata((msg[6].data()), msg[6].size(), true);

        if (rc != 0) {
            self.base.session.get_socket().event_handshake_failed_protocol(ctx,
                                                                           self.base.session.get_endpoint(),
                                                                           ZMQ_PROTOCOL_ERROR_ZAP_INVALID_METADATA as i32,
            );
            // errno = EPROTO;
            return close_and_return(&mut msg[0], -1);
        }

        //  Close all reply frames
        // for (size_t i = 0; i < zap_reply_frame_count; i+= 1)
        for i in 0..zap_reply_frame_count {
            msg[i].close()?;
            // errno_assert(rc2 == 0);
        }

        self.handle_zap_status_code(ctx);

        Ok(())
    }

    pub fn handle_zap_status_code(&mut self, ctx: &mut AppContext) {
        //  we can assume here that status_code is a valid ZAP status code,
        //  i.e. 200, 300, 400 or 500
        let mut status_code_numeric = 0;
        match (self.status_code[0]) {
            b'2' => return,
            b'3' => status_code_numeric = 300,
            b'4' => status_code_numeric = 400,
            b'5' => status_code_numeric = 500,
        };

        self.base.session
            .get_socket()
            .event_handshake_failed_auth(ctx, self.base.session.get_endpoint(), status_code_numeric);
    }
}

pub enum ZmqZapClientCommonHandshakeState {
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
        if (self.base.state == ready) {
            return ZmqMechanism::ready;
        }
        if (self.base.state == error_sent) {
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
