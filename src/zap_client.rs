use std::ffi::c_void;
use crate::defines::{ZMQ_MSG_MORE, ZMQ_PROTOCOL_ERROR_ZAP_BAD_REQUEST_ID, ZMQ_PROTOCOL_ERROR_ZAP_BAD_VERSION, ZMQ_PROTOCOL_ERROR_ZAP_INVALID_METADATA, ZMQ_PROTOCOL_ERROR_ZAP_INVALID_STATUS_CODE, ZMQ_PROTOCOL_ERROR_ZAP_MALFORMED_REPLY, ZMQ_PROTOCOL_ERROR_ZAP_UNSPECIFIED};
use crate::err::ZmqError;
use crate::mechanism;
use crate::mechanism::ZmqMechanism;
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::session_base::ZmqSession;
use crate::zap_client::zap_client_state::{error_sent, ready, sending_error};

#[derive(Default,Debug,Clone)]
pub struct ZapClient<'a> {
    pub mechanism: ZmqMechanism<'a>,
    pub status_code: String,
    pub peer_address: String,
}

pub const zap_version: &'static str = "1.0";

pub const zap_version_len: usize = 3;

pub const id: &'static str = "1";

pub const id_len: usize = 1;

impl ZapClient {
    pub fn new(session_: &mut ZmqSession, peer_address_: &str, options: &ZmqOptions) -> Self {
        Self {
            mechanism: ZmqMechanismBase::new(session_, options),
            status_code: String::new(),
        }
    }

    pub unsafe fn send_zap_request(&mut self, options: &mut ZmqOptions, mechanism_: &str, credentials_: &[u8]) {
        let credential_list = [credentials_];
        self.send_zap_request2(options, mechanism_, &credential_list)
    }

    pub unsafe fn send_zap_request2(&mut self, options: &ZmqOptions, mechanism_: &str, credentials_: &[&[u8]]) {
        // write_zap_msg cannot fail. It could only fail if the HWM was exceeded,
        // but on the ZAP socket, the HWM is disabled.

        // int rc;
        let mut rc = 0i32;
        // msg_t msg;
        let mut msg = ZmqMsg::new();

        //  Address delimiter frame
        rc = msg.init2 ();
        // errno_assert (rc == 0);
        msg.set_flags (ZmqMsg::more);
        rc = self.mechanism.session.write_zap_msg (&msg);
        // errno_assert (rc == 0);

        //  Version frame
        rc = msg.init_size (zap_version_len);
        // errno_assert (rc == 0);
        libc::memcpy (msg.data (), zap_version.as_ptr() as *const c_void, zap_version_len);
        msg.set_flags (ZmqMsg::more);
        rc = self.mechanism.session.write_zap_msg (&msg);
        // errno_assert (rc == 0);

        //  Request ID frame
        rc = msg.init_size (id_len);
        // errno_assert (rc == 0);
        libc::memcpy (msg.data (), id.as_ptr() as *const c_void, id_len);
        msg.set_flags (ZmqMsg::more);
        rc = self.mechanism.session.write_zap_msg (&msg);
        // errno_assert (rc == 0);

        //  Domain frame
        rc = msg.init_size (options.zap_domain.length ());
        // errno_assert (rc == 0);
        libc::memcpy (msg.data (), options.zap_domain.as_ptr() as *const c_void, options.zap_domain.length ());

        msg.set_flags (ZmqMsg::more);
        rc = self.mechanism.session.write_zap_msg (&msg);
        // errno_assert (rc == 0);

        //  Address frame
        rc = msg.init_size (self.peer_address.length ());
        // errno_assert (rc == 0);
       libc::memcpy (msg.data (), self.peer_address.as_ptr() as *const c_void, self.peer_address.length ());
        msg.set_flags (ZmqMsg::more);
        rc = self.mechanism.session.write_zap_msg (&msg);
        // errno_assert (rc == 0);

        //  Routing id frame
        rc = msg.init_size (options.routing_id_size);
        // errno_assert (rc == 0);
        libc::memcpy (msg.data (), options.routing_id as *const c_void, options.routing_id_size as usize);
        msg.set_flags (ZmqMsg::more);
        rc = self.mechanism.session.write_zap_msg (&msg);
        // errno_assert (rc == 0);

        //  Mechanism frame
        rc = msg.init_size (mechanism_.len ());
        // errno_assert (rc == 0);
        libc::memcpy (msg.data (), mechanism_, mechanism_.len());
        if (credentials_.len())
            msg.set_flags (ZmqMsg::more);
        rc = self.mechanism.session.write_zap_msg (&msg);
        // errno_assert (rc == 0);

        //  Credentials frames
        // for (size_t i = 0; i < credentials_.len(); ++i)
        for i in 0 .. credentials_.len()
        {
            rc = msg.init_size (credentials_[i].len());
            // errno_assert (rc == 0);
            if i < credentials_.len() - 1 {
                msg.set_flags(ZmqMsg::more);
            }
            libc::memcpy (msg.data (), credentials_[i].as_ptr() as *const c_void, credentials_[i].len());
            rc = self.mechanism.session.write_zap_msg (&msg);
            // errno_assert (rc == 0);
        }
    }

    pub fn receive_and_process_zap_reply(&mut self, options: &ZmqOptions) -> Result<(),ZmqError> {
        let mut rc = 0i32;
        let zap_reply_frame_count = 7;
        let mut msg: Vec<ZmqMsg> = Vec::with_capacity(zap_reply_frame_count);

        //  Initialize all reply frames
        // for (size_t i = 0; i < zap_reply_frame_count; i++)
        for i in 0 .. zap_reply_frame_count
        {
            msg[i].init2()?;
            // errno_assert (rc == 0);
        }

        // for (size_t i = 0; i < zap_reply_frame_count; i++)
        for i in 0 .. zap_reply_frame_count
        {
            rc = self.mechanism.session.read_zap_msg (&mut msg[i]);
            if rc == -1 {
                // if (errno == EAGAIN) {
                //     return 1;
                // }
                // return close_and_return (msg, -1);
                return Err(ZapError("EAGAIN"));
            }
            if msg[i].flags_set(ZMQ_MSG_MORE) == (if i < zap_reply_frame_count - 1 { 0 } else { ZMQ_MSG_MORE })
            {
                self.mechanism.session.get_socket().event_handshake_failed_protocol (
                    options,
                    self.mechanism.session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZAP_MALFORMED_REPLY as i32);
                // errno = EPROTO;
                // return close_and_return (msg, -1);
                return Err(ZapError("EPROTO"));
            }
        }

        //  Address delimiter frame
        if msg[0].size () > 0 {
            //  TODO can a ZAP handler produce such a message at all?
            self.mechanism.session.get_socket ().event_handshake_failed_protocol (
                self.mechanism.session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZAP_UNSPECIFIED);
            // errno = EPROTO;
            // return close_and_return (msg, -1);
            return -1;
        }

        //  Version frame
        if msg[1].size () != zap_version_len
            || libc::memcmp (msg[1].data_mut(), zap_version.as_ptr() as *const c_void, zap_version_len) != 0
        {
            self.mechanism.session.get_socket ().event_handshake_failed_protocol (
                self.mechanism.session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZAP_BAD_VERSION);
            // errno = EPROTO;
            // return close_and_return (msg, -1);
            return -1;
        }

        //  Request id frame
        if (msg[2].size () != id_len || libc::memcmp(msg[2].data_mut(), id.as_ptr() as *const c_void, id_len) != 0) {
            self.mechanism.session.get_socket ().event_handshake_failed_protocol (
                self.mechanism.session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZAP_BAD_REQUEST_ID);
            // errno = EPROTO;
            // return close_and_return (msg, -1);
            return -1;
        }

        //  Status code frame, only 200, 300, 400 and 500 are valid status codes
        let status_code_data = (msg[3].data_mut());
        if (msg[3].size () != 3 || status_code_data[0] < '2'
            || status_code_data[0] > '5' || status_code_data[1] != '0'
            || status_code_data[2] != '0') {
            self.mechanism.session.get_socket ().event_handshake_failed_protocol (
                self.mechanism.session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZAP_INVALID_STATUS_CODE);
            // errno = EPROTO;
            // return close_and_return (msg, -1);
            return -1;
        }

        //  Save status code
        self.status_code.assign (msg[3].data_mut()), 3);

        //  Save user id
        self.mechanism.set_user_id (msg[5].data_mut(), msg[5].size ());

        //  Process metadata frame
        rc = self.mechanism.parse_metadata ((msg[6].data_mut()),
                                            msg[6].size (), true);

        if (rc != 0) {
            self.mechanism.session.get_socket ().event_handshake_failed_protocol (
                self.mechanism.session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZAP_INVALID_METADATA);
            // errno = EPROTO;
            // return close_and_return (msg, -1);
            return -1;
        }

        //  Close all reply frames
        // for (size_t i = 0; i < zap_reply_frame_count; i++)
        for i in 0 .. zap_reply_frame_count
        {
            let rc2 = msg[i].close ();
            // errno_assert (rc2 == 0);
        }

        self.handle_zap_status_code ();

        return 0;
    }

    pub unsafe fn handle_zap_status_code(&mut self)
    {
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

        self.mechanism.session.get_socket ().event_handshake_failed_auth (
            self.mechanism.session.get_endpoint (), status_code_numeric, "");
    }
}

pub enum zap_client_state {
    waiting_for_hello,
    sending_welcome,
    waiting_for_initiate,
    waiting_for_zap_reply,
    sending_ready,
    sending_error,
    error_sent,
    ready,
}

pub struct zap_client_common_handshake_t {
    pub zap_client: ZapClient,
    pub state: zap_client_state,
    pub _zap_reply_ok_state: zap_client_state,
}

impl zap_client_common_handshake_t {
    pub fn new(session_: &mut ZmqSession, peer_address_: &str, options_: &ZmqOptions, zap_reply_ok_state_: zap_client_state) -> Self {
        Self {
            zap_client: ZapClient::new(session_, peer_address_, options_),
            state: zap_client_state::waiting_for_hello,
        }
    }

    pub fn status(&mut self) -> mechanism::MechanismStatus {
        if self.state == ready {
            return mechanism::MechanismStatus::Ready;
        }
        if self.state == error_sent {
            return mechanism::MechanismStatus::Error;
        }
        return mechanism::MechanismStatus::Handshaking;
    }

    pub unsafe fn zap_msg_available(&mut self) -> i32 {
        return self.receive_and_process_zap_reply ();
    }

    pub unsafe fn handle_zap_status_code(&mut self) {
        self.zap_client.handle_zap_status_code ();
        if self.zap_client.status_code[0] == '2' {
            self.state = self._zap_reply_ok_state.clone();
        }
        else if self.zap_client.status_code[0] == '3' {
            self.state == error_sent;
        } else {
            self.state == sending_error;
        }
    }

    pub unsafe fn receive_and_process_zap_reply(&mut self) -> i32 {
        return self.zap_client.receive_and_process_zap_reply()
    }
}
