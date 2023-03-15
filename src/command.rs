use crate::object::object_t;
use crate::own::own_t;
use crate::pipe::pipe_t;
use libc::c_void;
use crate::endpoint::EndpointUriPair;
use crate::socket_base::ZmqSocketBase;

pub enum CommandType {
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
        done
}

#[derive(Default,Debug,Clone)]
pub struct StopCommandArgs {}

#[derive(Default,Debug,Clone)]
pub struct PlugCommandArgs {}

#[derive(Default,Debug,Clone)]
pub struct OwnCommandArgs {
    pub object: *mut own_t,
}

#[derive(Default,Debug,Clone)]
pub struct AttachCommandArgs {
    pub engine: *mut i_engine,
}

#[derive(Default,Debug,Clone)]
pub struct BindCommandArgs {
    pub pipe: *mut pipe_t,
}

#[derive(Default,Debug,Clone)]
pub struct ActivateReadCommandArgs {}

#[derive(Default,Debug,Clone)]
pub struct ActivateWriteCommandArgs {
    pub msgs_read: u64,
}

#[derive(Default,Debug,Clone)]
pub struct HiccupCommandArgs {
    pub pipe: &mut [u8],
}

#[derive(Default,Debug,Clone)]
pub struct PipeTermCommandArgs {
}

#[derive(Default,Debug,Clone)]
pub struct PipeTermAckCommandArgs {}

#[derive(Default,Debug,Clone)]
pub struct PipeHwmCommandArgs {
    pub inhwm: i32,
    pub outhwm: i32
}

#[derive(Default,Debug,Clone)]
pub struct TermReqCommandArgs {
    pub object: *mut own_t
}

#[derive(Default,Debug,Clone)]
pub struct TermCommandArgs {
    pub linger: i32,}

#[derive(Default,Debug,Clone)]
pub struct TermAckCommandArgs {}

#[derive(Default,Debug,Clone)]
pub struct TermEndpointCommandArgs {
    pub endpoint: String
}

#[derive(Default,Debug,Clone)]
pub struct ReapCommandArgs {
    pub socket: *mut ZmqSocketBase
}

pub struct PipePeerStatsCommandArgs {
    pub queue_count: u64,
    pub socket_base: *mut own_t,
    pub enpoint_pair: EndpointUriPair
}

pub struct PipeStatsPublishCommandArgs {
    pub outbound_queue_count: u64,
    pub inbound_queue_count: u64,
    pub endpoint_pair: EndpointUriPair
}

pub struct DoneCommandArgs {}

#[derive(Default,Debug,Clone)]
pub struct ReapedCommandArgs {}

#[derive(Default,Debug,Clone)]
pub union ArgsUnion {
    //  Sent to I/O thread to let it know that it should
    //  terminate itself.
    pub stop: StopCommandArgs,
    //  Sent to I/O object to make it register with its I/O thread.
    pub plug: PlugCommandArgs,
    //  Sent to socket to let it know about the newly created object.
    pub own: OwnCommandArgs,
    //  Attach the engine to the session. If engine is NULL, it informs
    //  session that the connection have failed.
    pub attach: AttachCommandArgs,
    //  Sent from session to socket to establish pipe(s) between them.
    //  Caller have used inc_seqnum beforehand sending the command.
    pub bind: BindCommandArgs,
    //  Sent by pipe writer to inform dormant pipe reader that there
    //  are messages in the pipe.
    pub activate_read: ActivateReadCommandArgs,
    //  Sent by pipe reader to inform pipe writer about how many
    //  messages it has read so far.
    pub activate_write: ActivateWriteCommandArgs,
    //  Sent by pipe reader to writer after creating a new inpipe.
    //  The parameter is actually of type pipe_t::upipe_t, however,
    //  its definition is private so we'll have to do with void*.
    pub hiccup: HiccupCommandArgs,
    //  Sent by pipe reader to pipe writer to ask it to terminate
    //  its end of the pipe.
    pub pipe_term: PipeTermCommandArgs,
    //  Pipe writer acknowledges pipe_term command.
    pub pipe_term_ack: PipeTermAckCommandArgs,
    //  Sent by one of pipe to another part for modify hwm
    pub pipe_hwm: PipeHwmCommandArgs,
    //  Sent by I/O object ot the socket to request the shutdown of
    //  the I/O object.
    pub term_req: TermReqCommandArgs,
    //  Sent by socket to I/O object to start its shutdown.
    pub term: TermCommandArgs,
    //  Sent by I/O object to the socket to acknowledge it has
    //  shut down.
    pub term_ack: TermAckCommandArgs,
    //  Sent by session_base (I/O thread) to socket (application thread)
    //  to ask to disconnect the endpoint.
    pub term_endpoint: TermEndpointCommandArgs,
    //  Transfers the ownership of the closed socket
    //  to the reaper thread.
    pub reap: ReapCommandArgs,
    //  Closed socket notifies the reaper that it's already deallocated.
    pub reaped: ReapedCommandArgs,
    //  Send application-side pipe count and ask to send monitor event
    pub pipe_peer_stats: PipePeerStatsCommandArgs,
    //  Collate application thread and I/O thread pipe counts and endpoints
    //  and send as event
    pub pip_stats_publish: PipeStatsPublishCommandArgs,
    //  Sent by reaper thread to the term thread when all the sockets
    //  are successfully deallocated.
    pub done: DoneCommandArgs
}

//  This structure defines the commands that can be sent between threads.
#[derive(Default,Debug,Clone)]
pub struct ZmqCommand
{
    pub type_: CommandType,
    //  Object to process the command.
    // object_t *destination;
    pub destination: *mut object_t,
    //
    pub args: ArgsUnion,
}
