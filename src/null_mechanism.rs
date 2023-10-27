use std::ffi::c_void;
use libc::EFAULT;
use crate::defines::{ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_ERROR, ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND};
use crate::mechanism;
use crate::mechanism::MechanismStatus::{Error, Handshaking, Ready};
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::session_base::ZmqSessionBase;
use crate::zap_client::ZapClient;

pub const error_command_name: &'static str = "\x05ERROR";
pub const error_command_name_len: usize = 6;
pub const error_reason_len_size: usize = 1;

pub const ready_command_name: &'static str = "\x05READY";
pub const ready_command_name_len: usize = 6;

pub struct ZmqNullMechanism {
    pub zap_client: ZapClient,
    pub _ready_command_sent: bool,
    pub _error_command_sent: bool,
    pub _ready_command_received: bool,
    pub _error_command_received: bool,
    pub _zap_request_sent: bool,
    pub _zap_reply_received: bool,
}

impl ZmqNullMechanism {
    pub fn new(session_: &mut ZmqSessionBase, peer_address_: &str, options_: &mut ZmqOptions) -> ZmqNullMechanism {
        let mut out = ZmqNullMechanism {
            zap_client: ZapClient::new(session_, peer_address_, options_),
            _ready_command_sent: false,
            _error_command_sent: false,
            _ready_command_received: false,
            _error_command_received: false,
            _zap_request_sent: false,
            _zap_reply_received: false,
        };

        out
    }

    pub unsafe fn next_handshake_command(&mut self, options: &ZmqOptions, msg_: &mut ZmqMsg) -> i32 {
        if (self._ready_command_sent || self._error_command_sent) {
            // errno = EAGAIN;
            return -1;
        }

        if (self.zap_required() && !self._zap_reply_received) {
            if (self._zap_request_sent) {
                // errno = EAGAIN;
                return -1;
            }
            //  Given this is a backward-incompatible change, it's behind a socket
            //  option disabled by default.
            let mut rc = self.zap_client.mechanism_base.session.zap_connect();
            if (rc == -1 && options.zap_enforce_domain) {
                self.zap_client.mechanism_base.session.get_socket().event_handshake_failed_no_detail(
                    self.zap_client.mechanism_base.session.get_endpoint(), EFAULT);
                return -1;
            }
            if (rc == 0) {
                self.send_zap_request();
                self._zap_request_sent = true;

                //  TODO actually, it is quite unlikely that we can read the ZAP
                //  reply already, but removing this has some strange side-effect
                //  (probably because the pipe's in_active flag is true until a read
                //  is attempted)
                rc = self.zap_client.receive_and_process_zap_reply();
                if (rc != 0) {
                    return -1;
                }

                self._zap_reply_received = true;
            }
        }

        if (self._zap_reply_received && self.zap_client.status_code != "200") {
            self._error_command_sent = true;
            if (self.zap_client.status_code != "300") {
                let status_code_len = 3;
                let rc = msg_.init_size(
                    error_command_name_len + error_reason_len_size + status_code_len);
                // zmq_assert (rc == 0);
                let mut msg_data = (msg_.data_mut());
                libc::memcpy(msg_data, error_command_name.as_ptr() as *const c_void, error_command_name_len);
                msg_data += error_command_name_len;
                *msg_data = status_code_len;
                msg_data += error_reason_len_size;
                libc::memcpy(msg_data, self.zap_client.status_code.as_ptr() as *const c_void, status_code_len);
                return 0;
            }
            // errno = EAGAIN;
            return -1;
        }

        self.zap_client.mechanism_base.make_command_with_basic_properties(msg_, ready_command_name,
                                                                          ready_command_name_len);

        self._ready_command_sent = true;

        return 0;
    }

    pub unsafe fn process_handshake_command(&mut self, msg_: &mut ZmqMsg) -> i32 {
        if (self._ready_command_received || self._error_command_received) {
            self.zap_client.mechanism_base.session.get_socket().event_handshake_failed_protocol(
                self.zap_client.mechanism_base.session.get_endpoint(), ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND);
            // errno = EPROTO;
            return -1;
        }

        // const unsigned char *cmd_data = static_cast<unsigned char *> (msg_->data ());
        let cmd_data = msg_.data_mut();
        // const size_t data_size = msg_->size ();
        let data_size = msg_.size();

        let mut rc = 0;
        if (data_size >= ready_command_name_len && libc::memcmp(cmd_data, ready_command_name, ready_command_name_len) == 0) {
            rc = self.process_ready_command(cmd_data, data_size);
        } else if (data_size >= error_command_name_len && libc::memcmp(cmd_data, error_command_name, error_command_name_len) == 0) {
            rc = self.process_error_command(cmd_data, data_size);
        } else {
            self.zap_client.mechanism_base.session.get_socket().event_handshake_failed_protocol(
                self.zap_client.mechanism_base.session.get_endpoint(), ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND);
            // errno = EPROTO;
            rc = -1;
        }

        if (rc == 0) {
            rc = msg_.close();
            // errno_assert (rc == 0);
            rc = msg_.init2();
            // errno_assert (rc == 0);
        }
        return rc;
    }

    pub unsafe fn process_ready_command(&mut self, cmd_data: &[u8], data_size_: usize) -> i32 {
        self._ready_command_received = true;
        return self.zap_client.mechanism_base.parse_metadata(cmd_data + ready_command_name_len, data_size_ - ready_command_name_len);
    }

    pub unsafe fn process_error_command(&mut self, cmd_data: &[u8], data_size_: usize) -> i32 {
        let fixed_prefix_size = error_command_name_len + error_reason_len_size;
        if (data_size_ < fixed_prefix_size) {
            self.zap_client.mechanism_base.session.get_socket().event_handshake_failed_protocol(
                self.zap_client.mechanism_base.session.get_endpoint(),
                ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_ERROR);

            // errno = EPROTO;
            return -1;
        }
        let error_reason_len = cmd_data[error_command_name_len];
        if (error_reason_len > (data_size_ - fixed_prefix_size) as u8) {
            self.zap_client.mechanism_base.session.get_socket().event_handshake_failed_protocol(
                self.zap_client.mechanism_base.session.get_endpoint(),
                ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_ERROR);

            // errno = EPROTO;
            return -1;
        }
        let error_reason = cmd_data[fixed_prefix_size..];
        self.zap_client.handle_error_reason(error_reason, error_reason_len);
        self._error_command_received = true;
        return 0;
    }

    pub unsafe fn zap_msg_available() -> i32 {
        if (self._zap_reply_received) {
            // errno = EFSM;
            return -1;
        }
        let rc = self.zap_client.receive_and_process_zap_reply();
        if (rc == 0) {
            self._zap_reply_received = true;
        }
        return if rc == -1 { -1 } else { 0 };
    }

    pub unsafe fn status(&mut self) -> mechanism::MechanismStatus {
        if (self._ready_command_sent && self._ready_command_received) {
            return Ready;
        }

        let command_sent = self._ready_command_sent || self._error_command_sent;
        let command_received = self._ready_command_received || self._error_command_received;
        return if command_sent && command_received { Error } else { Handshaking };
    }

    pub unsafe fn send_zap_request(&mut self, options: &mut ZmqOptions) {
        self.zap_client.send_zap_request(options, "NULL", &[]);
    }
}
