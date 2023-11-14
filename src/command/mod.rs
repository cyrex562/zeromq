use crate::endpoint::ZmqEndpointUriPair;
use crate::engine::ZmqEngine;
use crate::io::reaper::ZmqReaper;
use crate::own::ZmqOwn;
use crate::pipe::ZmqPipe;
use crate::session::ZmqSession;
use crate::socket::ZmqSocket;

#[derive(Clone)]
pub enum ZmqCommandType {
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

// pub struct StopArgs {}

// pub struct PlugArgs {}

// pub struct OwnArgs<'a> {
//     pub object: &'a mut ZmqOwn<'a>,
// }

// pub struct AttachArgs<'a> {
//     pub engine: &'a mut dyn IEngine,
// }

// pub struct BindArgs<'a> {
//     pub pipe: &'a mut ZmqPipe<'a>,
// }

// pub struct ActivateReadArgs {}

// pub struct ActivateWriteArgs {
//     pub msgs_read: u64,
// }

// pub struct HiccupArgs<'a> {
//     pub pipe: &'a mut ZmqPipe<'a>,
// }

// pub struct PipeTermArgs {}

// pub struct PipeTermAckArgs {}

// pub struct PipeHwmArgs {
//     pub inhwm: i32,
//     pub outhwm: i32,
// }

// pub struct TermReqArgs<'a> {
//     pub object: &'a mut ZmqOwn<'a>,
// }

// pub struct TermArgs {
//     pub linger: i32,
// }

// pub struct TermAckArgs {}

// pub struct TermEndpointArgs {
//     pub endpoint: String,
// }

// pub struct ReapArgs<'a> {
//     pub socket: &'a mut ZmqSocket<'a>,
// }

// pub struct ReapedArgs {}

// pub struct PipePeerStatsArgs<'a> {
//     pub queue_count: u64,
//     pub socket_base: &'a mut ZmqSocket<'a>,
//     pub endpoint_pair: &'a mut ZmqEndpointUriPair,
// }
//
// pub struct PipeStatsPublishArgs<'a> {
//     pub outbound_queue_count: u64,
//     pub inbound_queue_count: u64,
//     pub endpoint_pair: &'a mut ZmqEndpointUriPair,
// }

// pub struct DoneArgs {}

// pub union ZmqArgs<'a> {
//     pub stop: StopArgs,
//     pub plug: PlugArgs,
//     pub own: OwnArgs<'a>,
//     pub attach: AttachArgs<'a>,
//     pub bind: BindArgs<'a>,
//     pub activate_read: ActivateReadArgs,
//     pub activate_write: ActivateWriteArgs,
//     pub hiccup: HiccupArgs<'a>,
//     pub pipe_term: PipeTermArgs,
//     pub pipe_term_ack: PipeTermAckArgs,
//     pub pipe_hwm: PipeHwmArgs,
//     pub term_req: TermReqArgs<'a>,
//     pub term: TermArgs,
//     pub term_ack: TermAckArgs,
//     pub term_endpoint: TermEndpointArgs,
//     pub reap: ReapArgs<'a>,
//     pub reaped: ReapedArgs,
//     pub pipe_peer_stats: PipePeerStatsArgs<'a>,
//     pub pipe_stats_publish: PipeStatsPublishArgs<'a>,
//     pub done: DoneArgs,
// }

#[derive(Default,Debug,Clone,PartialOrd, PartialEq)]
pub struct ZmqCommand<'a> {
    // pub destination: Option<&'a mut ZmqPipe<'a>>,
    pub dest_pipe: Option<&'a mut ZmqPipe<'a>>,
    pub dest_own: Option<&'a mut ZmqOwn<'a>>,
    pub dest_sock: Option<&'a mut ZmqSocket<'a>>,
    pub dest_session: Option<&'a mut ZmqSession<'a>>,
    pub dest_reaper: Option<&'a mut ZmqReaper<'a>>,
    // pub args: ZmqArgs<'a>,
    // pub stop: StopArgs,
    // pub plug: PlugArgs,
    // pub own: OwnArgs<'a>,
    pub object: Option<&'a mut ZmqOwn<'a>>,
    // pub attach: AttachArgs<'a>,
    pub engine: Option<&'a mut ZmqEngine<'a>>,
    // pub bind: BindArgs<'a>,
    pub pipe: Option<&'a mut ZmqPipe<'a>>,
    // pub activate_read: ActivateReadArgs,
    // pub activate_write: ActivateWriteArgs,
    pub msgs_read: u64,
    // pub hiccup: HiccupArgs<'a>,
    // pub pipe_term: PipeTermArgs,
    // pub pipe_term_ack: PipeTermAckArgs,
    // pub pipe_hwm: PipeHwmArgs,
    pub inhwm: i32,
    pub outhwm: i32,
    // pub term_req: TermReqArgs<'a>,
    // pub term: TermArgs,
    pub linger: i32,
    // pub term_ack: TermAckArgs,
    // pub term_endpoint: TermEndpointArgs,
    pub endpoint: String,
    // pub reap: ReapArgs<'a>,
    pub socket: Option<&'a mut ZmqSocket<'a>>,
    // pub reaped: ReapedArgs,
    // pub pipe_peer_stats: PipePeerStatsArgs<'a>,
    pub queue_count: u64,
    // pub socket_base: Option<&'a mut ZmqSocket<'a>>,
    pub endpoint_pair: Option<&'a mut ZmqEndpointUriPair>,
    // pub pipe_stats_publish: PipeStatsPublishArgs<'a>,
    pub outbound_queue_count: u64,
    pub inbound_queue_count: u64,
    // pub done: DoneArgs,
    pub type_: ZmqCommandType,
}

impl ZmqCommand {
    pub fn new() -> Self {
        Self {
            // destination: None,
            dest_pipe: None,
            dest_own: None,
            dest_sock: None,
            dest_session: None,
            dest_reaper: None,
            object: None,
            engine: None,
            pipe: None,
            msgs_read: 0,
            inhwm: 0,
            outhwm: 0,
            linger: 0,
            endpoint: "".to_string(),
            socket: None,
            queue_count: 0,
            endpoint_pair: None,
            outbound_queue_count: 0,
            inbound_queue_count: 0,
            type_: ZmqCommandType::Stop,
        }
    }
}

// impl PartialEq for ZmqCommand {
//     fn eq(&self, other: &Self) -> bool {
//         self.destination == other.destination
//     }
// }
