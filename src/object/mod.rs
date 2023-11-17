
use crate::command::{ZmqCommand, ZmqCommandType};
use crate::ctx::{Endpoint, ZmqContext};
use crate::defines::err::ZmqError;
use crate::endpoint::ZmqEndpointUriPair;
use crate::engine::ZmqEngine;
use crate::io::io_thread::ZmqIoThread;
use crate::io::reaper::{reaper_process_reap, reaper_process_reaped};
use crate::msg::content::ZmqContent;
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::own::{own_process_own, own_process_seqnum, own_process_term_ack, ZmqOwn};
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

pub fn obj_process_command(
    options: &ZmqOptions,
    cmd: &mut ZmqCommand,
    pipe: &mut ZmqPipe,
    ctx: &mut ZmqContext
) -> Result<(),ZmqError> {
    match cmd.type_.clone() {
        ZmqCommandType::ActivateRead => {
            pipe.process_activate_read();
        }
        // break;
        ZmqCommandType::ActivateWrite => {
            pipe.process_activate_write(cmd.msgs_read);
        }
        // break;
        ZmqCommandType::Stop => {
            pipe.process_stop(options, cmd.engine.unwrap().socket.unwrap());
        }
        // break;
        ZmqCommandType::Plug => {
            pipe.process_plug(options, cmd.engine.unwrap().session.unwrap());
            pipe.process_seqnum(cmd.object.unwrap());
        }
        // break;
        ZmqCommandType::Own => {
            pipe.process_own(cmd.object.unwrap());
            pipe.process_seqnum(cmd.object.unwrap());
        }
        // break;
        ZmqCommandType::Attach => {
            pipe.process_attach(options, cmd.engine.unwrap());
            pipe.process_seqnum(cmd.object.unwrap());
        }
        // break;
        ZmqCommandType::Bind => {
            pipe.process_bind(cmd.pipe.unwrap().out_pipe.unwrap());
            pipe.process_seqnum(cmd.object.unwrap());
        }
        // break;
        ZmqCommandType::Hiccup => {
            pipe.process_hiccup(cmd.pipe.unwrap().out_pipe.unwrap())?;
        }
        // break;
        ZmqCommandType::PipePeerStats => {
            pipe.process_pipe_peer_stats(
                ctx,
                cmd.queue_count,
                cmd.socket.unwrap(),
                cmd.endpoint_pair.unwrap(),
            );
        }
        // break;
        ZmqCommandType::PipeStatsPublish => {
            pipe.process_pipe_stats_publish(
                options,
                cmd.engine.unwrap().socket.unwrap(),
                cmd.outbound_queue_count,
                cmd.inbound_queue_count,
                cmd.endpoint_pair.unwrap(),
            );
        }
        // break;
        ZmqCommandType::PipeTerm => {
            pipe.process_pipe_term(ctx);
        }
        // break;
        ZmqCommandType::PipeTermAck => {
            pipe.process_pipe_term_ack(ctx);
        }
        // break;
        ZmqCommandType::PipeHwm => {
            pipe.process_pipe_hwm(cmd.inhwm, cmd.outhwm);
        }
        // break;
        ZmqCommandType::TermReq => {
            pipe.process_term_req(cmd.object.unwrap());
        }
        // break;
        ZmqCommandType::Term => {
            pipe.process_term(cmd.object.unwrap(), cmd.linger);
        }
        // break;
        ZmqCommandType::TermAck => {
            own_process_term_ack(
                &mut cmd.object.unwrap().terminating,
                &mut cmd.object.unwrap().processed_seqnum,
                &mut cmd.object.unwrap().sent_seqnum,
                &mut cmd.object.unwrap().term_acks,
                &mut cmd.object.unwrap().owner,
            );
        }
        // break;
        ZmqCommandType::TermEndpoint => {
            cmd.socket
                .unwrap()
                .process_term_endpoint(options, &cmd.endpoint)?;
        }
        // break;
        ZmqCommandType::Reap => {
            reaper_process_reap(cmd.dest_reaper.unwrap(), cmd.socket.unwrap());
        }
        // break;
        ZmqCommandType::Reaped => {
            reaper_process_reaped(cmd.dest_reaper.unwrap());
        }
        // break;
        ZmqCommandType::InprocConnected => {
            own_process_seqnum(cmd.dest_own.unwrap());
        }
        // break;
        ZmqCommandType::ConnFailed => {
            // session_process_conn_failed();
            cmd.engine.unwrap().session.unwrap().process_conn_failed();
        }

        ZmqCommandType::Done => {} // default:
                   //     zmq_assert (false);
    }

    Ok(())
}

pub fn obj_register_endpoint(
    ctx: &mut ZmqContext,
    addr: &str,
    sock: &mut ZmqSocket,
    options: &mut ZmqOptions,
) -> Result<(),ZmqError> {
    ctx.register_endpoint(addr, sock, options)
}

pub fn obj_unregister_endpoint(ctx: &mut ZmqContext, addr: &str, socket: &mut ZmqSocket) -> Result<(),ZmqError>{
    ctx.unregister_endpoint(addr)
}

pub fn obj_unregister_endpoints(ctx: &mut ZmqContext, socket: &mut ZmqSocket) {
    ctx.unregister_endpoints(socket);
}

pub fn obj_find_endpoint<'a>(ctx: &mut ZmqContext, addr: &str) -> Option<&'a mut ZmqSocket<'a>> {
    ctx.find_endpoint(addr)
}

pub fn obj_pend_connection(
    ctx: &mut ZmqContext,
    addr: &str,
    endpoint: &Endpoint,
    pipes: &mut [&mut ZmqPipe],
) -> Result<(),ZmqError> {
    ctx.pend_connection(addr, endpoint, pipes)
}

pub fn obj_connect_pending(ctx: &mut ZmqContext, addr: &str, bind_socket: &mut ZmqSocket) -> Result<(),ZmqError> {
    ctx.connect_pending(addr, bind_socket)
}

pub fn obj_destroy_socket(ctx: &mut ZmqContext, socket: &mut ZmqSocket) {
    ctx.destroy_socket(socket);
}

pub fn obj_choose_io_thread<'a>(
    ctx: &mut ZmqContext,
    affinity: u64
) -> Option<&'a mut ZmqIoThread<'a>> {
    ctx.choose_io_thread(affinity)
}

pub fn obj_send_stop(ctx: &mut ZmqContext, pipe: &mut ZmqPipe, thread_id: u32) {
    let mut cmd = ZmqCommand::new();
    cmd.dest_pipe = Some(pipe);
    cmd.type_ = ZmqCommandType::Stop;
    ctx.send_command(thread_id, &mut cmd);
}

pub fn obj_send_plug(ctx: &mut ZmqContext, destination: &mut ZmqPipe, inc_seqnum: bool) {
    if inc_seqnum {
        destination.inc_seqnum();
    }

    let mut cmd = ZmqCommand::new();
    cmd.dest_pipe = Some(destination);
    cmd.type_ = ZmqCommandType::Plug;
    obj_send_command(ctx, &mut cmd);
}

pub fn obj_send_own(ctx: &mut ZmqContext, destination: &mut ZmqPipe, object: &mut ZmqOwn) {
    destination.inc_seqnum();
    let mut cmd = ZmqCommand::new();
    cmd.dest_pipe = Some(destination);
    cmd.type_ = ZmqCommandType::Own;
    cmd.object = Some(object);
    obj_send_command(ctx, &mut cmd);
}

pub  fn obj_send_attach(
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

pub fn obj_send_conn_failed(ctx: &mut ZmqContext, destination: &mut ZmqPipe) {
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
    cmd.type_ = ZmqCommandType::ActivateRead;
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

pub fn obj_send_hiccup(ctx: &mut ZmqContext, destination: &mut ZmqPipe, pipe: &mut YPipeConflate<ZmqMsg>) {
    let mut cmd = ZmqCommand::new();
    cmd.dest_pipe = Some(destination);
    cmd.type_ = ZmqCommandType::Hiccup;
    cmd.pipe = Some(pipe.out_pipe());
    obj_send_command(ctx, &mut cmd);
}

pub fn obj_send_pipe_peer_stats(
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
    cmd.dest_own = Some(&mut destination.own);
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

pub fn obj_send_term_endpoint(
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

pub fn obj_send_reap(ctx: &mut ZmqContext, socket: &mut ZmqSocket) {
    let mut cmd = ZmqCommand::new();
    cmd.dest_reaper = Some(ctx.get_reaper());
    cmd.type_ = ZmqCommandType::Reap;
    cmd.socket = Some(socket);
    obj_send_command(ctx, &mut cmd);
}

pub fn obj_send_reaped(ctx: &mut ZmqContext) {
    let mut cmd = ZmqCommand::new();
    cmd.dest_reaper = Some(ctx.get_reaper());
    cmd.type_ = ZmqCommandType::Reaped;
    obj_send_command(ctx, &mut cmd);
}

pub fn obj_send_inproc_connected(ctx: &mut ZmqContext, socket: &mut ZmqSocket) {
    let mut cmd = ZmqCommand::new();
    cmd.dest_sock = Some(socket);
    cmd.type_ = ZmqCommandType::InprocConnected;
    obj_send_command(ctx, &mut cmd);
}

pub fn obj_send_done(ctx: &mut ZmqContext) {
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
