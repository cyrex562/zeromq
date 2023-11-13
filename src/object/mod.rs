use crate::command::ZmqCommandType::{ActivateRead, Stop};
use crate::command::{ZmqCommand, ZmqCommandType};
use crate::ctx::{Endpoint, ZmqContext};
use crate::endpoint::ZmqEndpointUriPair;
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::own::{own_process_seqnum, own_process_term_ack, ZmqOwn};
use crate::pipe::ZmqPipe;
use crate::socket::ZmqSocket;
use crate::ypipe::ypipe_conflate::YPipeConflate;

// pub struct ZmqObject<'a> {
//     pub context: &'a mut ZmqContext<'a>,
//     pub thread_id: u32,
// }

// impl ZmqObject {
//     pub fn new(ctx: &mut ZmqContext, tid: u32) -> Self {
//         Self {
//             context: ctx,
//             thread_id: tid,
//         }
//     }
//
//     pub unsafe fn new2(parent: &mut Self) -> Self {
//         Self {
//             context: (*parent).context,
//             thread_id: (*parent).thread_id,
//         }
//     }
//
//
// }

// pub fn get_tid(&self) -> u32 {
//         self.thread_id
//     }

// pub fn set_tid(&mut self, id: u32) {
//     self.thread_id = id;
// }

pub fn obj_process_command(options: &ZmqOptions, cmd: &mut ZmqCommand, pipe: &mut ZmqPipe) {
    match cmd.type_.clone() {
        ActivateRead => {
            pipe.process_activate_read();
        }
        // break;
        activate_write => {
            pipe.process_activate_write(cmd.msgs_read);
        }
        // break;
        Stop => {
            pipe.process_stop();
        }
        // break;
        plug => {
            pipe.process_plug();
            pipe.process_seqnum();
        }
        // break;
        own => {
            pipe.process_own(cmd.object);
            pipe.process_seqnum();
        }
        // break;
        attach => {
            pipe.process_attach(cmd.engine);
            pipe.process_seqnum();
        }
        // break;
        bind => {
            pipe.process_bind(cmd.pipe);
            pipe.process_seqnum();
        }
        // break;
        hiccup => {
            pipe.process_hiccup(cmd.pipe.unwrap());
        }
        // break;
        pipe_peer_stats => {
            pipe.process_pipe_peer_stats(
                cmd.queue_count,
                cmd.socket.unwrap(),
                cmd.endpoint_pair.unwrap(),
            );
        }
        // break;
        pipe_stats_publish => {
            pipe.process_pipe_stats_publish(
                cmd.outbound_queue_count,
                cmd.inbound_queue_count,
                cmd.endpoint_pair,
            );
        }
        // break;
        pipe_term => {
            pipe.process_pipe_term();
        }
        // break;
        pipe_term_ack => {
            pipe.process_pipe_term_ack();
        }
        // break;
        pipe_hwm => {
            pipe.process_pipe_hwm(cmd.inhwm, cmd.outhwm);
        }
        // break;
        term_req => {
            pipe.process_term_req(cmd.object);
        }
        // break;
        term => {
            pipe.process_term(cmd.linger);
        }
        // break;
        term_ack => {
            own_process_term_ack(
                &mut cmd.socket.unwrap().terminating,
                &mut cmd.socket.unwrap().processed_seqnum,
                &mut cmd.socket.unwrap().sent_seqnum,
                &mut cmd.socket.unwrap().term_acks,
                &mut cmd.socket.unwrap().owner,
            );
        }
        // break;
        term_endpoint => {
            cmd.socket
                .unwrap()
                .process_term_endpoint(options, &cmd.endpoint);
        }
        // break;
        reap => {
            reaper_process_reap(cmd.dest_reaper.unwrap(), cmd.socket.unwrap());
        }
        // break;
        reaped => {
            reaper_process_reaped(cmd.dest_reaper.unwrap());
        }
        // break;
        inproc_connected => {
            own_process_seqnum(cmd.dest_own.unwrap());
        }
        // break;
        conn_failed => {
            session_process_conn_failed();
        }

        done => {} // default:
                   //     zmq_assert (false);
    }
}

pub fn obj_register_endpoint(
    ctx: &mut ZmqContext,
    addr: &str,
    sock: &mut ZmqSocket,
    options: &mut ZmqOptions,
) -> i32 {
    ctx.register_endpoint(addr, sock, options)
}

pub fn obj_unregister_endpoint(ctx: &mut ZmqContext, addr: &str, socket: &mut ZmqSocket) {
    ctx.unregister_endpoint(addr);
}

pub fn obj_unregister_endpoints(ctx: &mut ZmqContext, socket: &mut ZmqSocket) {
    ctx.unregister_endpoints(socket);
}

pub fn obj_find_endpoint<'a>(ctx: &mut ZmqContext, addr: &str) -> Option<&'a mut ZmqSocket> {
    ctx.find_endpoint(addr)
}

pub unsafe fn obj_pend_connection(
    ctx: &mut ZmqContext,
    addr: &str,
    endpoint: &Endpoint,
    pipes: &mut [&mut ZmqPipe],
) {
    ctx.pend_connection(addr, endpoint, pipes);
}

pub unsafe fn obj_connect_pending(ctx: &mut ZmqContext, addr: &str, bind_socket: &mut ZmqSocket) {
    ctx.connect_pending(addr, bind_socket);
}

pub unsafe fn obj_destroy_socket(ctx: &mut ZmqContext, socket: &mut ZmqSocket) {
    ctx.destroy_socket(socket);
}

pub unsafe fn obj_choose_io_thread<'a>(ctx: &mut ZmqContext, affinity: u64) -> &'a mut ZmqIoThread {
    ctx.choose_io_thread(affinity)
}

pub fn obj_send_stop(ctx: &mut ZmqContext, pipe: &mut ZmqPipe, thread_id: u32) {
    let mut cmd = ZmqCommand::new();
    cmd.dest_pipe = Some(pipe);
    cmd.type_ = ZmqCommandType::Stop;
    ctx.send_command(thread_id, &mut cmd);
}

pub unsafe fn obj_send_plug(ctx: &mut ZmqContext, destination: &mut ZmqPipe, inc_seqnum: bool) {
    if (inc_seqnum) {
        destination.inc_seqnum();
    }

    let mut cmd = ZmqCommand::new();
    cmd.dest_pipe = Some(destination);
    cmd.type_ = ZmqCommandType::Plug;
    obj_send_command(ctx, &mut cmd);
}

pub unsafe fn obj_send_own(ctx: &mut ZmqContext, destination: &mut ZmqPipe, object: &mut ZmqOwn) {
    destination.inc_seqnum();
    let mut cmd = ZmqCommand::new();
    cmd.dest_pipe = Some(destination);
    cmd.type_ = ZmqCommandType::Own;
    cmd.object = Some(object);
    obj_send_command(ctx, &mut cmd);
}

pub unsafe fn obj_send_attach(
    ctx: &mut ZmqContext,
    destination: &mut ZmqPipe,
    engine: &mut ZmqEngine,
    inc_seqnum: bool,
) {
    if (inc_seqnum) {
        destination.inc_seqnum();
    }

    let mut cmd = ZmqCommand::new();
    cmd.dest_pipe = Some(destination);
    cmd.type_ = ZmqCommandType::Attach;
    cmd.engine = Some(engine);
    obj_send_command(ctx, &mut cmd);
}

pub unsafe fn obj_send_conn_failed(ctx: &mut ZmqContext, destination: &mut ZmqPipe) {
    let mut cmd = ZmqCommand::new();
    cmd.dest_pipe = Some(destination);
    cmd.type_ = ZmqCommandType::ConnFailed;
    obj_send_command(ctx, &mut cmd);
}

pub fn obj_send_bind(
    ctx: &mut ZmqContext,
    destination: &mut ZmqPipe,
    pipe: &mut ZmqPipe,
    inc_seqnum: bool,
) {
    if inc_seqnum {
        destination.inc_seqnum();
    }

    let mut cmd = ZmqCommand::new();
    cmd.dest_pipe = Some(destination);
    cmd.type_ = ZmqCommandType::Bind;
    cmd.pipe = Some(pipe);
    obj_send_command(ctx, &mut cmd);
}

pub fn obj_send_activate_read(ctx: &mut ZmqContext, destination: &mut ZmqPipe) {
    let mut cmd = ZmqCommand::new();
    cmd.dest_pipe = Some(destination);
    cmd.type_ = ActivateRead;
    obj_send_command(ctx, &mut cmd);
}

pub fn obj_send_activate_write(
    ctx: &mut ZmqContext,
    destination: &mut ZmqPipe,
    msgs_read: u64,
) {
    let mut cmd = ZmqCommand::new();
    cmd.dest_pipe = Some(destination);
    cmd.type_ = ZmqCommandType::ActivateWrite;
    cmd.msgs_read = msgs_read;
    obj_send_command(ctx, &mut cmd);
}

pub unsafe fn obj_send_hiccup(ctx: &mut ZmqContext, destination: &mut ZmqPipe, pipe: &mut YPipeConflate<ZmqMsg>) {
    let mut cmd = ZmqCommand::new();
    cmd.dest_pipe = Some(destination);
    cmd.type_ = ZmqCommandType::Hiccup;
    cmd.pipe = Some(pipe);
    obj_send_command(ctx, &mut cmd);
}

pub unsafe fn obj_send_pipe_peer_stats(
    ctx: &mut ZmqContext,
    destination: &mut ZmqPipe,
    queue_count: u64,
    socket: &mut ZmqSocket,
    endpoint_pair: &mut ZmqEndpointUriPair,
) {
    let mut cmd = ZmqCommand::new();
    cmd.dest_pipe = Some(destination);
    cmd.type_ = ZmqCommandType::PipePeerStats;
    cmd.queue_count = queue_count;
    cmd.socket = Some(socket);
    cmd.endpoint_pair = Some(endpoint_pair);
    obj_send_command(ctx, &mut cmd);
}

// void zmq::object_t::send_pipe_hwm (pipe_t *destination_,
//                                    int inhwm_,
//                                    int outhwm_)
// {
//     command_t cmd;
//     cmd.destination = destination_;
//     cmd.type = command_t::pipe_hwm;
//     cmd.args.pipe_hwm.inhwm = inhwm_;
//     cmd.args.pipe_hwm.outhwm = outhwm_;
//     send_command (cmd);
// }
pub fn obj_send_pipe_hwm(
    ctx: &mut ZmqContext,
    destination_: &mut ZmqPipe,
    inhwm_: i32,
    outhwm_: i32,
) {
    let mut cmd = ZmqCommand::new();
    cmd.dest_pipe = Some(destination_);
    cmd.type_ = ZmqCommandType::PipeHwm;
    cmd.inhwm = inhwm_;
    cmd.outhwm = outhwm_;
    obj_send_command(ctx, &mut cmd);
}

// void zmq::object_t::send_term_req (own_t *destination_, own_t *object_)
// {
//     command_t cmd;
//     cmd.destination = destination_;
//     cmd.type = command_t::term_req;
//     cmd.args.term_req.object = object_;
//     send_command (cmd);
// }
pub fn send_term_req(ctx: &mut ZmqContext, destination: &mut ZmqOwn, object: &mut ZmqOwn) {
    let mut cmd = ZmqCommand::new();
    // cmd.destination = Some(destination);
    cmd.dest_own = Some(destination);
    cmd.type_ = ZmqCommandType::TermReq;
    cmd.object = Some(object);
    obj_send_command(ctx, &mut cmd);
}

pub fn obj_send_pipe_stats_publish(
    ctx: &mut ZmqContext,
    destination: &mut ZmqSocket,
    outbound_queue_count: u64,
    inbound_queue_count: u64,
    endpoint_pair: &mut ZmqEndpointUriPair,
) {
    let mut cmd = ZmqCommand::new();
    // cmd.destination = Some(destination);
    cmd.dest_own = Some(destination);
    cmd.type_ = ZmqCommandType::PipeStatsPublish;
    cmd.outbound_queue_count = outbound_queue_count;
    cmd.inbound_queue_count = inbound_queue_count;
    cmd.endpoint_pair = Some(endpoint_pair);
    obj_send_command(ctx, &mut cmd);
}

pub fn obj_send_pipe_term(ctx: &mut ZmqContext, destination: &mut ZmqPipe) {
    let mut cmd = ZmqCommand::new();
    cmd.dest_pipe = Some(destination);
    cmd.type_ = ZmqCommandType::PipeTerm;
    obj_send_command(ctx, &mut cmd);
}

pub fn obj_send_pipe_term_ack(ctx: &mut ZmqContext, destination: &mut ZmqPipe) {
    let mut cmd = ZmqCommand::new();
    cmd.dest_pipe = Some(destination);
    cmd.type_ = ZmqCommandType::PipeTermAck;
    obj_send_command(ctx, &mut cmd);
}

pub unsafe fn obj_send_term_endpoint(
    ctx: &mut ZmqContext,
    destination: &mut ZmqPipe,
    endpoint: &str,
) {
    let mut cmd = ZmqCommand::new();
    cmd.dest_pipe = Some(destination);
    cmd.type_ = ZmqCommandType::TermEndpoint;
    cmd.endpoint = endpoint.to_string();
    obj_send_command(ctx, &mut cmd);
}

// void zmq::object_t::send_term_ack (own_t *destination_)
// {
//     command_t cmd;
//     cmd.destination = destination_;
//     cmd.type = command_t::term_ack;
//     send_command (cmd);
// }
pub fn obj_send_term_ack(ctx: &mut ZmqContext, destination: &mut ZmqOwn) {
    let mut cmd = ZmqCommand::new();
    cmd.dest_own = Some(destination);
    cmd.type_ = ZmqCommandType::TermAck;
    obj_send_command(ctx, &mut cmd);
}

// void zmq::object_t::send_term (own_t *destination_, int linger_)
// {
//     command_t cmd;
//     cmd.destination = destination_;
//     cmd.type = command_t::term;
//     cmd.args.term.linger = linger_;
//     send_command (cmd);
// }
pub fn obj_send_term(ctx: &mut ZmqContext, destination: &mut ZmqOwn, linger: i32) {
    let mut cmd = ZmqCommand::new();
    cmd.dest_own = Some(destination);
    cmd.type_ = ZmqCommandType::Term;
    cmd.linger = linger;
    obj_send_command(ctx, &mut cmd);
}

pub unsafe fn obj_send_reap(ctx: &mut ZmqContext, socket: &mut ZmqSocket) {
    let mut cmd = ZmqCommand::new();
    cmd.dest_reaper = Some(ctx.get_reaper());
    cmd.type_ = ZmqCommandType::Reap;
    cmd.socket = Some(socket);
    obj_send_command(ctx, &mut cmd);
}

pub unsafe fn obj_send_reaped(ctx: &mut ZmqContext) {
    let mut cmd = ZmqCommand::new();
    cmd.dest_reaper = Some(ctx.get_reaper());
    cmd.type_ = ZmqCommandType::Reaped;
    obj_send_command(ctx, &mut cmd);
}

pub unsafe fn obj_send_inproc_connected(ctx: &mut ZmqContext, socket: &mut ZmqSocket) {
    let mut cmd = ZmqCommand::new();
    cmd.dest_sock = Some(socket);
    cmd.type_ = ZmqCommandType::InprocConnected;
    obj_send_command(ctx, &mut cmd);
}

pub unsafe fn obj_send_done(ctx: &mut ZmqContext) {
    let mut cmd = ZmqCommand::new();
    // cmd.destination = None;
    cmd.type_ = ZmqCommandType::Done;
    ctx.send_command(ZmqContext::term_tid, &mut cmd);
}

pub fn obj_send_command(ctx: &mut ZmqContext, cmd: &mut ZmqCommand) {
    // todo
    let tid = 0u32;
    ctx.send_command(tid, cmd);
}
