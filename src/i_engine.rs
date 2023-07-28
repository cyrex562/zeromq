

pub enum error_reason_t
{
    protocol_error,
    connection_error,
    timeout_error,
}

pub struct i_engine
{
    pub has_handshake_stage: fn() -> bool,
pub plug: fn(io_thread: *mut io_thread_t, session_: *mut session_base_t),
pub terminate: fn(),
pub restart_input: fn() -> bool,
pub restart_output: fn(),
pub zap_msg_available: fn(),
pub get_endpoint: fn() -> &mut endpoint_uri_t
}