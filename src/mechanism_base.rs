use std::ffi::c_void;
use crate::defines::ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_UNSPECIFIED;
use crate::mechanism::{mechanism_ops, mechanism_t, status_t};
use crate::msg::msg_t;
use crate::options::options_t;
use crate::session_base::session_base_t;

pub struct mechanism_base_t
{
    pub session: *const session_base_t,
    pub mechanism: mechanism_t,
}

impl mechanism_base_t
{
    pub fn new(session_: &mut session_base_t, options: &options_t) -> Self
    {
        Self {
            session: session_,
            mechanism: mechanism_t::new(options)
        }
    }

    pub unsafe fn check_basic_command_structure(&mut self, msg_: *mut msg_t) -> i32 {
        if ((*msg_).size () <= 1
            || (*msg_).size () <= ( ((*msg_).data ()))[0]) {
            self.session.get_socket ().event_handshake_failed_protocol (
                self.session.get_endpoint (),
                ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_UNSPECIFIED);
            // errno = EPROTO;
            return -1;
        }
        return 0;
    }

    pub fn handle_error_reason(&mut self, error_reason_: &str, error_reason_len_: usize) {
        let status_code_len = 3;
        let zero_digit = '0';
        let significant_digit_index = 0;
        let first_zero_digit_index = 1;

        let second_zero_digit_index = 2;
        let factor = 100;
        if error_reason_len_ == status_code_len
            && error_reason_.chars().nth(first_zero_digit_index).unwrap() == zero_digit
            && error_reason_.chars().nth(second_zero_digit_index).unwrap() == zero_digit
            && error_reason_.chars().nth(significant_digit_index).unwrap() >= '3'
            && error_reason_.chars().nth(significant_digit_index).unwrap() <= '5' {
            // it is a ZAP error status code (300, 400 or 500), so emit an authentication failure event
            self.session.get_socket ().event_handshake_failed_auth (
                self.session.get_endpoint (),
                (error_reason_.chars().nth(significant_digit_index).unwrap() as u8 - zero_digit as u8) * factor);
        } else {
            // this is a violation of the ZAP protocol
            // TODO zmq_assert in this case?
        }
    }

    pub fn zap_required(&mut self) -> bool {
        return !self.mechanism.options.zap_domain.empty ();
    }
}

impl mechanism_ops for mechanism_base_t
{
    fn next_handshake_command(&mut self, msg_: *mut msg_t) -> i32 {
        todo!()
    }

    fn process_handshake_command(&mut self, msg_: *mut msg_t) -> i32 {
        todo!()
    }

    fn status(&mut self) -> status_t {
        todo!()
    }

    fn property(&mut self, name_: &str, value_: *mut c_void, length: usize) {
        todo!()
    }
}
