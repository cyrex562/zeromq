use std::ffi::c_void;

use crate::ctx::ZmqContext;
use crate::defines::{ZMQ_MSG_MORE, ZMQ_PROTOCOL_ERROR_ZAP_BAD_REQUEST_ID, ZMQ_PROTOCOL_ERROR_ZAP_BAD_VERSION, ZMQ_PROTOCOL_ERROR_ZAP_INVALID_METADATA, ZMQ_PROTOCOL_ERROR_ZAP_INVALID_STATUS_CODE, ZMQ_PROTOCOL_ERROR_ZAP_MALFORMED_REPLY, ZMQ_PROTOCOL_ERROR_ZAP_UNSPECIFIED};
use crate::defines::err::ZmqError;
use crate::defines::err::ZmqError::ZapError;
use crate::mechanism;
use crate::mechanism::ZmqMechanism;
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::session::ZmqSession;
use crate::zap_client::ZapClientState::{ErrorSent, Ready, SendingError};

#[derive(Default, Debug, Clone)]
pub struct ZapClient<'a> {
    pub mechanism: ZmqMechanism<'a>,
    pub status_code: String,
    pub peer_address: String,
}

pub const ZAP_VERSION: &'static str = "1.0";

pub const ZAP_VERSION_LEN: usize = 3;

pub const id: &'static str = "1";

pub const ID_LEN: usize = 1;

impl<'a> ZapClient<'a> {
    pub fn new(
        session_: &mut ZmqSession,
        peer_address_: &str,
        options: &ZmqOptions,
    ) -> Self {
        Self {
            mechanism: ZmqMechanism::new(session_),
            status_code: String::new(),
            peer_address: "".to_string(),
        }
    }

    pub fn send_zap_request(&mut self,
                                   options: &mut ZmqOptions,
                                   mechanism_: &str,
                                   credentials_: &[u8],
    ) {
        let credential_list = [credentials_];
        self.send_zap_request2(options, mechanism_, &credential_list)
    }

    pub fn send_zap_request2(&mut self,
                                    options: &ZmqOptions,
                                    mechanism_: &str,
                                    credentials_: &[&[u8]],
    ) {
        // write_zap_msg cannot fail. It could only fail if the HWM was exceeded,
        // but on the ZAP socket, the HWM is disabled.

        // int rc;
        let mut rc = 0i32;
        // msg_t msg;
        let mut msg = ZmqMsg::new();

        //  Address delimiter frame
        rc = msg.init2();
        // errno_assert (rc == 0);
        msg.set_flags(ZmqMsg::more);
        self.mechanism.session.write_zap_msg(&msg)?;
        // errno_assert (rc == 0);

        //  Version frame
        rc = msg.init_size(ZAP_VERSION_LEN);
        // errno_assert (rc == 0);
        // libc::memcpy (msg.data (), ZAP_VERSION.as_ptr() as *const c_void, ZAP_VERSION_LEN);
        msg.data().copy_from_slice(ZAP_VERSION.as_bytes());
        msg.set_flags(ZmqMsg::more);
        self.mechanism.session.write_zap_msg(&msg)?;
        // errno_assert (rc == 0);

        //  Request ID frame
        rc = msg.init_size(ID_LEN);
        // errno_assert (rc == 0);
        // libc::memcpy (msg.data (), id.as_ptr() as *const c_void, ID_LEN);
        msg.data().copy_from_slice(id.as_bytes());
        msg.set_flags(ZmqMsg::more);
        self.mechanism.session.write_zap_msg(&msg)?;
        // errno_assert (rc == 0);

        //  Domain frame
        rc = msg.init_size(options.zap_domain.length());
        // errno_assert (rc == 0);
        // libc::memcpy (msg.data (), options.zap_domain.as_ptr() as *const c_void, options.zap_domain.length ());
        msg.data().copy_from_slice(options.zap_domain.as_bytes());

        msg.set_flags(ZmqMsg::more);
        self.mechanism.session.write_zap_msg(&msg)?;
        // errno_assert (rc == 0);

        //  Address frame
        rc = msg.init_size(self.peer_address.length());
        // errno_assert (rc == 0);
        // libc::memcpy (msg.data (), self.peer_address.as_ptr() as *const c_void, self.peer_address.length ());
        msg.data().copy_from_slice(self.peer_address.as_bytes());
        msg.set_flags(ZmqMsg::more);
        self.mechanism.session.write_zap_msg(&msg)?;
        // errno_assert (rc == 0);

        //  Routing id frame
        rc = msg.init_size(options.routing_id_size);
        // errno_assert (rc == 0);
        // libc::memcpy (msg.data (), options.routing_id as *const c_void, options.routing_id_size as usize);

        msg.data().copy_from_slice(options.routing_id);

        msg.set_flags(ZmqMsg::more);
        self.mechanism.session.write_zap_msg(&msg)?;
        // errno_assert (rc == 0);

        //  Mechanism frame
        rc = msg.init_size(mechanism_.len());
        // errno_assert (rc == 0);
        // libc::memcpy (msg.data (), mechanism_, mechanism_.len());

        msg.data().copy_from_slice(mechanism_.as_bytes());

        if credentials_.len() {
            msg.set_flags(ZmqMsg::more);
        }
        self.mechanism.session.write_zap_msg(&msg)?;
        // errno_assert (rc == 0);

        //  Credentials frames
        // for (size_t i = 0; i < credentials_.len(); ++i)
        for i in 0..credentials_.len() {
            rc = msg.init_size(credentials_[i].len());
            // errno_assert (rc == 0);
            if i < credentials_.len() - 1 {
                msg.set_flags(ZmqMsg::more);
            }
            // libc::memcpy(msg.data(), credentials_[i].as_ptr() as *const c_void, credentials_[i].len());
            msg.data().copy_from_slice(credentials_[i]);
            self.mechanism.session.write_zap_msg(&msg)?;
            // errno_assert (rc == 0);
        }
    }

    pub fn receive_and_process_zap_reply(&mut self, options: &mut ZmqOptions, ctx: &mut ZmqContext) -> Result<(), ZmqError> {
        let mut rc = 0i32;
        let zap_reply_frame_count = 7;
        let mut msg: Vec<ZmqMsg> = Vec::with_capacity(zap_reply_frame_count);

        //  Initialize all reply frames
        // for (size_t i = 0; i < zap_reply_frame_count; i++)
        for i in 0..zap_reply_frame_count {
            msg[i].init2()?;
            // errno_assert (rc == 0);
        }

        // for (size_t i = 0; i < zap_reply_frame_count; i++)
        for i in 0..zap_reply_frame_count {
            if self.mechanism.session.read_zap_msg(ctx, &mut msg[i]).ise_err() {
                // if (errno == EAGAIN) {
                //     return 1;
                // }
                // return close_and_return (msg, -1);
                return Err(ZapError("EAGAIN"));
            }
            if msg[i].flags_set(ZMQ_MSG_MORE) == (if i < zap_reply_frame_count - 1 { 0 } else { ZMQ_MSG_MORE }) {
                self.mechanism.session.get_socket().event_handshake_failed_protocol(
                    ctx,
                    options,
                    self.mechanism.session.get_endpoint(), ZMQ_PROTOCOL_ERROR_ZAP_MALFORMED_REPLY as i32);
                // errno = EPROTO;
                // return close_and_return (msg, -1);
                return Err(ZapError("EPROTO"));
            }
        }

        //  Address delimiter frame
        if msg[0].size() > 0 {
            //  TODO can a ZAP handler produce such a message at all?
            self.mechanism.session.get_socket().event_handshake_failed_protocol(
                ctx,
                options,
                self.mechanism.session.get_endpoint(), ZMQ_PROTOCOL_ERROR_ZAP_UNSPECIFIED as i32);
            // errno = EPROTO;
            // return close_and_return (msg, -1);
            return Err(ZapError("EPROTO"));
        }

        //  Version frame
        // if msg[1].size () != ZAP_VERSION_LEN
        //     || libc::memcmp (msg[1].data_mut(), ZAP_VERSION.as_ptr() as *const c_void, ZAP_VERSION_LEN) != 0
        if msg[1].size() != ZAP_VERSION_LEN || (msg[1].data_mut()).iter().zip(ZAP_VERSION.as_bytes()).any(|(a, b)| a != b) {
            self.mechanism.session.get_socket().event_handshake_failed_protocol(ctx,
                options,
                self.mechanism.session.get_endpoint(), ZMQ_PROTOCOL_ERROR_ZAP_BAD_VERSION as i32);
            // errno = EPROTO;
            // return close_and_return (msg, -1);
            return Err(ZapError("EPROTO"));
        }

        //  Request id frame
        // if (msg[2].size () != ID_LEN || libc::memcmp(msg[2].data_mut(), id.as_ptr() as *const c_void, ID_LEN) != 0)
        if msg[2].size() != ID_LEN || (msg[2].data_mut()).iter().zip(id.as_bytes()).any(|(a, b)| a != b) {
            self.mechanism.session.get_socket().event_handshake_failed_protocol(ctx,
                options,
                self.mechanism.session.get_endpoint(), ZMQ_PROTOCOL_ERROR_ZAP_BAD_REQUEST_ID as i32);
            // errno = EPROTO;
            // return close_and_return (msg, -1);
            return Err(ZapError("EPROTO"));
        }

        //  Status code frame, only 200, 300, 400 and 500 are valid status codes
        let status_code_data = (msg[3].data_mut());
        if msg[3].size() != 3 || status_code_data[0] < '2' as u8 || status_code_data[0] > '5' as u8 || status_code_data[1] != '0' as u8 || status_code_data[2] != '0' as u8 {
            self.mechanism.session.get_socket().event_handshake_failed_protocol(ctx,
                options,
                self.mechanism.session.get_endpoint(), ZMQ_PROTOCOL_ERROR_ZAP_INVALID_STATUS_CODE as i32);
            // errno = EPROTO;
            // return close_and_return (msg, -1);
            return Err(ZapError("EPROTO"));
        }

        //  Save status code
        self.status_code.assign(msg[3].data_mut(), 3);

        //  Save user id
        self.mechanism.set_user_id(
            msg[5].data_mut().as_ptr() as *mut c_void,
            msg[5].size(),
        );

        //  Process metadata frame
        if self.mechanism.parse_metadata(
            options,
            (msg[6].data_mut()),
            msg[6].size(),
            true,
        ).is_err() {
            self.mechanism.session.get_socket().event_handshake_failed_protocol(
                ctx,
                options,
                self.mechanism.session.get_endpoint(), ZMQ_PROTOCOL_ERROR_ZAP_INVALID_METADATA as i32);
            // errno = EPROTO;
            // return close_and_return (msg, -1);
            return Err(ZapError("EPROTO"));
        }

        //  Close all reply frames
        // for (size_t i = 0; i < zap_reply_frame_count; i++)
        for i in 0..zap_reply_frame_count {
            let rc2 = msg[i].close()?;
            // errno_assert (rc2 == 0);
        }

        self.handle_zap_status_code(ctx, options);

        return Ok(());
    }

    pub fn handle_zap_status_code(&mut self, ctx: &mut ZmqContext, options: &mut ZmqOptions) {
        //  we can assume here that status_code is a valid ZAP status code,
        //  i.e. 200, 300, 400 or 500
        let mut status_code_numeric = 0;

        let code_val = self.status_code[0];
        if code_val == '2' {
            return;
        } else if code_val == '3' {
            status_code_numeric = 300;
        } else if code_val == '4' {
            status_code_numeric = 400;
        } else if code_val == '5' {
            status_code_numeric = 500;
        }

        self.mechanism.session.get_socket().event_handshake_failed_auth(
            ctx,
            options,
            self.mechanism.session.get_endpoint(),
            status_code_numeric,
        );
    }
}

pub enum ZapClientState {
    WaitingForHello,
    SendingWelcome,
    WaitingForInitiate,
    WaitingForZapReply,
    SendingReady,
    SendingError,
    ErrorSent,
    Ready,
}

pub struct ZapClientCommonHandshake<'a> {
    pub zap_client: ZapClient<'a>,
    pub state: ZapClientState,
    pub _zap_reply_ok_state: ZapClientState,
}

impl<'a> ZapClientCommonHandshake<'a> {
    pub fn new(session_: &mut ZmqSession, peer_address_: &str, options_: &ZmqOptions, zap_reply_ok_state_: ZapClientState) -> Self {
        Self {
            zap_client: ZapClient::new(session_, peer_address_, options_),
            state: ZapClientState::WaitingForHello,
            _zap_reply_ok_state: zap_reply_ok_state_,
        }
    }

    pub fn status(&mut self) -> mechanism::MechanismStatus {
        if self.state == Ready {
            return mechanism::MechanismStatus::Ready;
        }
        if self.state == ErrorSent {
            return mechanism::MechanismStatus::Error;
        }
        return mechanism::MechanismStatus::Handshaking;
    }

    pub fn zap_msg_available(&mut self, options: &mut ZmqOptions, ctx: &mut ZmqContext) -> Result<(), ZmqError> {
        self.receive_and_process_zap_reply(options, ctx)
    }

    pub fn handle_zap_status_code(&mut self, ctx: &mut ZmqContext, options: &mut ZmqOptions) {
        self.zap_client.handle_zap_status_code(ctx, options);
        if self.zap_client.status_code[0] == '2' {
            self.state = self._zap_reply_ok_state.clone();
        } else if self.zap_client.status_code[0] == '3' {
            self.state == ErrorSent;
        } else {
            self.state == SendingError;
        }
    }

    pub fn receive_and_process_zap_reply(&mut self, options: &mut ZmqOptions, ctx: &mut ZmqContext) -> Result<(), ZmqError> {
        self.zap_client.receive_and_process_zap_reply(options, ctx)
    }
}
