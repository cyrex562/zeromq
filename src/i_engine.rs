#![allow(non_camel_case_types)]

use crate::endpoint::endpoint_uri_pair_t;
use crate::io_thread::io_thread_t;

pub enum error_reason_t {
    protocol_error,
    connection_error,
    timeout_error,
}
pub trait i_engine {
    fn has_handshake_stage(&mut self) -> bool;
    fn plug(&mut self, io_thread_: *mut io_thread_t, session_: *mut session_base_t);

    fn terminate(&mut self);

    fn restart_input(&mut self);

    fn restart_output(&mut self);

    fn zap_msg_available(&mut self);

    fn get_endpoint(&mut self) -> &mut endpoint_uri_pair_t;
}
