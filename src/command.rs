use std::fmt::{Display, Formatter};

use libc::c_void;
use crate::address::ZmqAddress;

use crate::endpoint_uri::EndpointUriPair;
use crate::own::ZmqOwn;
use crate::pipe::ZmqPipe;
use crate::reaper::ZmqReaper;
use crate::session_base::ZmqSessionBase;
use crate::socket::ZmqSocket;

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
pub struct ZmqCommand<'a> {
    pub cmd_type: CommandType,
    //  Object to process the command.
    pub destination: ZmqAddress,
    pub object: Option<ZmqOwn>,
    pub pipe: Option<&'a mut ZmqPipe>,
    pub msgs_read: u64,
    pub inhwm: i32,
    pub outhwm: i32,
    pub linger: i32,
    pub endpoint: String,
    pub socket: Option<&'a mut ZmqSocket<'a>>,
    pub queue_count: u64,
    pub socket_base: Option<ZmqOwn>,
    pub endpoint_pair: EndpointUriPair,
    pub outbound_queue_count: u64,
    pub inbound_queue_count: u64,
    pub reaper: Option<ZmqReaper>,
    pub session: Option<&'a mut ZmqSessionBase>
}

// impl ZmqEngineInterface for ZmqCommand {
//     fn has_handshake_state(&self) -> bool {
//         todo!()
//     }
// 
//     fn plug(&mut self, io_thread: &mut ZmqThreadContext, session: &mut ZmqSessionBase) {
//         todo!()
//     }
// 
//     fn terminate(&mut self) {
//         todo!()
//     }
// 
//     fn restart_input(&mut self) -> bool {
//         todo!()
//     }
// 
//     fn restart_output(&mut self) {
//         todo!()
//     }
// 
//     fn zap_msg_available(&mut self) {
//         todo!()
//     }
// 
//     fn get_endpoint(&mut self) -> &EndpointUriPair {
//         todo!()
//     }
// }
