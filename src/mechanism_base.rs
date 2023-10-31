use std::ffi::c_void;
use crate::defines::ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_UNSPECIFIED;
use crate::mechanism::{mechanism_ops, ZmqMechanism, MechanismStatus};
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::session_base::ZmqSession;

pub struct ZmqMechanismBase
{
    pub session: *const ZmqSession,
    pub mechanism: ZmqMechanism,
}

impl ZmqMechanismBase
{
    pub fn new(session_: &mut ZmqSession, options: &ZmqOptions) -> Self
    {
        Self {
            session: session_,
            mechanism: ZmqMechanism::new(options)
        }
    }






}

impl mechanism_ops for ZmqMechanismBase
{
    fn next_handshake_command(&mut self, msg_: *mut ZmqMsg) -> i32 {
        todo!()
    }

    fn process_handshake_command(&mut self, msg_: *mut ZmqMsg) -> i32 {
        todo!()
    }

    fn status(&mut self) -> MechanismStatus {
        todo!()
    }

    fn property(&mut self, name_: &str, value_: *mut c_void, length: usize) {
        todo!()
    }
}
