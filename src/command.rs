use std::fmt::{Display, Formatter};

use libc::c_void;

use crate::endpoint::EndpointUriPair;
use crate::engine_interface::ZmqEngineInterface;
use crate::object::ZmqObject;
use crate::own::ZmqOwn;
use crate::pipe::ZmqPipe;
use crate::session_base::ZmqSessionBase;
use crate::socket::ZmqSocket;
use crate::thread_context::ZmqThreadContext;

pub enum CommandType {
    Stop,
    Plug,
    Own,
    Attach,
    Bind,
    ActivateRead,
    ActivateWrite,
    Hiccup,
    PipeTerm,
    PipeTermAck,
    PipeHwm,
    TermReq,
    Term,
    TermAck,
    TermEndpoint,
    Reap,
    Reaped,
    InprocConnected,
    ConnFailed,
    PipePeerStats,
    PipeStatsPublish,
    Done,
}

impl Display for CommandType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

//  This structure defines the commands that can be sent between threads.
#[derive(Default, Debug, Clone)]
pub struct ZmqCommand {
    pub cmd_type: CommandType,
    //  Object to process the command.
    pub destination: ZmqSocket,
    pub object: Option<ZmqOwn>,
    pub pipe: Option<ZmqPipe>,
    pub msgs_read: u64,
    pub inhwm: i32,
    pub outhwm: i32,
    pub linger: i32,
    pub endpoint: String,
    pub socket: Option<ZmqSocket>,
    pub queue_count: u64,
    pub socket_base: Option<ZmqOwn>,
    pub endpoint_pair: EndpointUriPair,
    pub outbound_queue_count: u64,
    pub inbound_queue_count: u64,
}

impl ZmqEngineInterface for ZmqCommand {
    fn has_handshake_state(&self) -> bool {
        todo!()
    }

    fn plug(&mut self, io_thread: &mut ZmqThreadContext, session: &mut ZmqSessionBase) {
        todo!()
    }

    fn terminate(&mut self) {
        todo!()
    }

    fn restart_input(&mut self) -> bool {
        todo!()
    }

    fn restart_output(&mut self) {
        todo!()
    }

    fn zap_msg_available(&mut self) {
        todo!()
    }

    fn get_endpoint(&mut self) -> &EndpointUriPair {
        todo!()
    }
}
