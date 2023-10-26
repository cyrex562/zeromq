

use crate::endpoint::ZmqEndpointUriPair;
use crate::io_thread::ZmqIoThread;

pub enum ErrorReason {
    ProtocolError,
    ConnectionError,
    TimeoutError,
}
pub trait IEngine {
    fn has_handshake_stage(&mut self) -> bool;
    fn plug(&mut self, io_thread_: *mut ZmqIoThread, session_: *mut session_base_t);

    fn terminate(&mut self);

    fn restart_input(&mut self);

    fn restart_output(&mut self);

    fn zap_msg_available(&mut self);

    fn get_endpoint(&mut self) -> &mut ZmqEndpointUriPair;
}
