use crate::object::ZmqObject;
use crate::own::ZmqOwn;
use crate::socket_base::ZmqSocketBase;
use crate::endpoint::ZmqEndpointUriPair;
use crate::i_engine::IEngine;
use crate::pipe::ZmqPipe;

pub enum ZmqType {
    stop,
    plug,
    own,
    attach,
    bind,
    activate_read,
    activate_write,
    hiccup,
    pipe_term,
    pipe_term_ack,
    pipe_hwm,
    term_req,
    term,
    term_ack,
    term_endpoint,
    reap,
    reaped,
    inproc_connected,
    conn_failed,
    pipe_peer_stats,
    pipe_stats_publish,
    done,
}

pub struct StopArgs {}

pub struct PlugArgs {}

pub struct OwnArgs<'a> {
    pub object: &'a mut ZmqOwn<'a>,
}

pub struct AttachArgs<'a> {
    pub engine: &'a mut dyn IEngine,
}

pub struct BindArgs<'a> {
    pub pipe: &'a mut ZmqPipe<'a>,
}

pub struct ActivateReadArgs {}

pub struct ActivateWriteArgs {
    pub msgs_read: u64,
}

pub struct HiccupArgs<'a> {
    pub pipe: &'a mut ZmqPipe<'a>,
}

pub struct PipeTermArgs {}

pub struct PipeTermAckArgs {}

pub struct PipeHwmArgs {
    pub inhwm: i32,
    pub outhwm: i32,
}

pub struct TermReqArgs<'a> {
    pub object: &'a mut ZmqOwn<'a>,
}

pub struct TermArgs {
    pub linger: i32,
}

pub struct TermAckArgs {}

pub struct TermEndpointArgs {
    pub endpoint: String,
}

pub struct ReapArgs<'a> {
    pub socket: &'a mut ZmqSocketBase<'a>,
}

pub struct ReapedArgs {}

pub struct PipePeerStatsArgs<'a> {
    pub queue_count: u64,
    pub socket_base: &'a mut ZmqOwn<'a>,
    pub endpoint_pair: &'a mut ZmqEndpointUriPair,
}

pub struct PipeStatsPublishArgs<'a> {
    pub outbound_queue_count: u64,
    pub inbound_queue_count: u64,
    pub endpoint_pair: &'a mut ZmqEndpointUriPair,
}

pub struct DoneArgs {}

pub union ZmqArgs<'a> {
    pub stop: StopArgs,
    pub plug: PlugArgs,
    pub own: OwnArgs<'a>,
    pub attach: AttachArgs<'a>,
    pub bind: BindArgs<'a>,
    pub activate_read: ActivateReadArgs,
    pub activate_write: ActivateWriteArgs,
    pub hiccup: HiccupArgs<'a>,
    pub pipe_term: PipeTermArgs,
    pub pipe_term_ack: PipeTermAckArgs,
    pub pipe_hwm: PipeHwmArgs,
    pub term_req: TermReqArgs<'a>,
    pub term: TermArgs,
    pub term_ack: TermAckArgs,
    pub term_endpoint: TermEndpointArgs,
    pub reap: ReapArgs<'a>,
    pub reaped: ReapedArgs,
    pub pipe_peer_stats: PipePeerStatsArgs<'a>,
    pub pipe_stats_publish: PipeStatsPublishArgs<'a>,
    pub done: DoneArgs,
}

pub struct ZmqCommand<'a> {
    pub destination: Option<&'a mut ZmqObject<'a>>,
    pub args: ZmqArgs<'a>,
    pub type_: ZmqType,
}

impl ZmqCommand {
    pub fn new() -> Self {
        Self {
            destination: None,
            args: ZmqArgs { stop: StopArgs {} },
            type_: ZmqType::stop,
        }
    }
}


impl PartialEq for ZmqCommand {
    fn eq(&self, other: &Self) -> bool {
        self.destination == other.destination
    }
}
