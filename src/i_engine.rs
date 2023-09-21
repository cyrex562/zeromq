#![allow(non_camel_case_types)]

use crate::io_thread::io_thread_t;
use crate::endpoint::endpoint_uri_pair_t;

pub enum error_reason_t
{
    protocol_error,
    connection_error,
    timeout_error,
}

// pub struct i_engine
// {
//     pub has_handshake_stage: fn() -> bool,
// pub plug: fn(io_thread: *mut io_thread_t, session_: *mut session_base_t),
// pub terminate: fn(),
// pub restart_input: fn() -> bool,
// pub restart_output: fn(),
// pub zap_msg_available: fn(),
// pub get_endpoint: fn() -> &mut endpoint_uri_t
// }

pub trait i_engine {
    fn has_handshake_stage(&self) -> bool;
    fn plug(&self, io_thread: *mut io_thread_t, session_: *mut session_base_t);
    fn terminate(&self);
    fn restart_input(&self) -> bool;
    fn restart_output(&self);
    fn zap_msg_available(&self);
    fn get_endpoint(&self) -> &mut endpoint_uri_pair_t;
}