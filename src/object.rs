use crate::command::{CommandType, ZmqCommand};
use crate::context::ZmqContext;
use crate::engine_interface::ZmqEngineInterface;
use crate::own::ZmqOwn;
use crate::pipe::ZmqPipe;
use crate::session_base::ZmqSessionBase;
use crate::socket::ZmqSocket;
use crate::thread_context::ZmqThreadContext;
use anyhow::anyhow;
use std::ptr::null_mut;
use crate::endpoint_uri::EndpointUriPair;
use crate::engine::ZmqEngine;


fn obj_process_command(cmd: &ZmqCommand) {
    match cmd.cmd_type {
        CommandType::Stop => {}
        CommandType::Plug => {}
        CommandType::Own => {}
        CommandType::Attach => {}
        CommandType::Bind => {}
        CommandType::ActivateRead => {}
        CommandType::ActivateWrite => {}
        CommandType::Hiccup => {}
        CommandType::PipeTerm => {}
        CommandType::PipeTermAck => {}
        CommandType::PipeHwm => {}
        CommandType::TermReq => {}
        CommandType::Term => {}
        CommandType::TermAck => {}
        CommandType::TermEndpoint => {}
        CommandType::Reap => {}
        CommandType::Reaped => {}
        CommandType::InprocConnected => {}
        CommandType::ConnFailed => {}
        CommandType::PipePeerStats => {}
        CommandType::PipeStatsPublish => {}
        CommandType::Done => {}
    }
}



fn obj_send_attach(

    destination: &mut ZmqSessionBase,
    engine: &mut ZmqEngine,
    inc_seqnum: bool,
) {
    if (inc_seqnum) {
        destination.inc_seqnum();
    }

    let mut cmd = ZmqCommand::default();
    cmd.session = Some(destination);
    cmd.cmd_type = ZmqCommand::attach;
    cmd.args.attach.engine = engine;
    obj_send_command(&mut cmd);
}

fn obj_send_activate_read(pipe: &mut ZmqPipe) {
    let mut cmd = ZmqCommand::default();
    cmd.pipe = Some(pipe);
    cmd.cmd_type = CommandType::ActivateRead;
    obj_send_command(&mut cmd);
}

fn obj_send_activate_write(pipe: &mut ZmqPipe, msgs_read: u64) {
    let mut cmd = ZmqCommand::default();
    cmd.pipe = Some(pipe);
    cmd.cmd_type = CommandType::ActivateWrite;
    cmd.args.activate_write.msgs_read = msgs_read;
    obj_send_command(&mut cmd);
}

fn obj_end_hiccup(pipe_a: &mut ZmqPipe, pipe: &mut [u8]) {
    let mut cmd = ZmqCommand::default();
    cmd.pipe = Some(pipe_a);
    cmd.cmd_type = CommandType::Hiccup;
    cmd.args.hiccup.pipe = pipe;
    obj_send_command(&mut cmd);
}

fn obj_send_pipe_peer_stats(

    pipe: &mut ZmqPipe,
    queue_count: u64,
    socket_base: &mut ZmqOwn,
    endpoint_pair: &mut EndpointUriPair,
) {
    let mut cmd = ZmqCommand::default();
    cmd.pipe = Some(pipe);
    cmd.cmd_type = CommandType::PipePeerStats;
    cmd.args.pipe_peer_stats.queue_count = queue_count;
    cmd.args.pipe_peer_stats.socket_base = socket_base;
    cmd.args.pipe_peer_stats.endpoint_pair = endpoint_pair;
    obj_send_command(&mut cmd);
}

fn obj_send_pipe_stats_publish(

    object: &mut ZmqOwn,
    outbound_queue_count: u64,
    inbound_queue_count: u64,
    endpoint_pair: &mut EndpointUriPair,
) {
    let mut cmd = ZmqCommand::default();
    cmd.object = Some(object.clone());
    cmd.cmd_type = CommandType::PipeStatsPublish;
    cmd.args.pipe_stats_publish.outbound_queue_count = outbound_queue_count;
    cmd.args.pipe_stats_publish.inbound_queue_count = inbound_queue_count;
    cmd.args.pipe_stats_publish.endpoint_pair = endpoint_pair;
    obj_send_command(&mut cmd);
}

fn obj_send_pipe_term(pipe: &mut ZmqPipe) {
    let mut cmd = ZmqCommand::default();
    cmd.pipe = Some(pipe);
    cmd.cmd_type = CommandType::PipeTerm;
    obj_send_command(&mut cmd);
}

fn obj_send_pipe_term_ack(pipe: &mut ZmqPipe) {
    let mut cmd = ZmqCommand::default();
    cmd.pipe = Some(pipe);
    cmd.cmd_type = CommandType::PipeTermAck;
    obj_send_command(&mut cmd);
}

fn obj_send_pipe_hwm(pipe: &mut ZmqPipe, inhwm: i32, outhwm: i32) {
    let mut cmd = ZmqCommand::default();
    cmd.pipe = Some(pipe);
    cmd.cmd_type = CommandType::PipeHwm;
    cmd.args.pipe_hwm.inhwm = inhwm;
    cmd.args.pipe_hwm.outhwm = outhwm;
    obj_send_command(&mut cmd);
}

fn obj_send_term_req(object_a: &mut ZmqOwn, object: &mut ZmqOwn) {
    let mut cmd = ZmqCommand::default();
    cmd.object = Some(object_a.clone());
    cmd.cmd_type = CommandType::TermReq;
    cmd.args.term_req.object = object;
    obj_send_command(&mut cmd);
}

fn obj_send_term(object: &mut ZmqOwn, linger: i32) {
    let mut cmd = ZmqCommand::default();
    cmd.object = Some(object.clone());
    cmd.cmd_type = CommandType::Term;
    cmd.args.term.linger = linger;
    obj_send_command(&mut cmd);
}

fn obj_send_term_ack(object: &mut ZmqOwn) {
    let mut cmd = ZmqCommand::default();
    cmd.object = Some(object.clone());
    cmd.cmd_type = CommandType::TermAck;
    obj_send_command(&mut cmd);
}

fn obj_send_term_endpoint(object: &mut ZmqOwn, endpoint: &str) {
    let mut cmd = ZmqCommand::default();
    cmd.object = Some(object.clone());
    cmd.cmd_type = CommandType::TermEndpoint;
    cmd.args.term_endpoint.endpoint = endpoint.into_string();
    obj_send_command(&mut cmd);
}

fn obj_send_reap( socket: &mut ZmqSocket) {
    let mut cmd = ZmqCommand::default();
    cmd.reaper = Some(socket.context.get_reaper().unwrap().clone());
    cmd.cmd_type = CommandType::Reap;
    cmd.args.reap.socket = socket;
    obj_send_command(&mut cmd);
}

fn obj_send_reaped(ctx: &mut ZmqContext) {
    let mut cmd = ZmqCommand::default();
    cmd.reaper = Some(ctx.get_reaper().unwrap().clone());
    cmd.cmd_type = CommandType::Reaped;
    obj_send_command(&mut cmd);
}

fn obj_send_done(ctx: &mut ZmqContext) {
    let mut cmd = ZmqCommand::default();
    // cmd.destination = None;
    cmd.cmd_type = CommandType::Done;
    ctx.send_command(ZmqContext::TERM_TID, &mut cmd);
}

fn obj_send_conn_failed( destination: &mut ZmqSessionBase) {
    let mut cmd = ZmqCommand::default();
    cmd.session = Some(destination);
    cmd.cmd_type = CommandType::ConnFailed;
    obj_send_command(&mut cmd);
}

//  These handlers can be overridden by the derived objects. They are
//  called when command arrives from another thread.
fn obj_process_stop() {
    unimplemented!()
}

fn obj_process_plug() {
    unimplemented!()
}

fn obj_process_own( object: &mut ZmqOwn) {
    unimplemented!()
}

fn obj_process_attached( engine: &mut ZmqEngine) {
    unimplemented!()
}

fn obj_process_bind( pipe: &mut ZmqPipe) {
    unimplemented!()
}

fn obj_process_activate_read() {
    unimplemented!()
}

fn obj_process_activate_write( msgs_read: u64) {
    unimplemented!()
}

fn obj_process_hiccup( pipe: &mut [u8]) {
    unimplemented!()
}

fn obj_process_pipe_peer_stats(

    queue_count: u64,
    socket_base: &mut ZmqOwn,
    endpoint_pair: &mut EndpointUriPair,
) {
    unimplemented!()
}

fn obj_process_pipe_stats_publish(

    outbound_queue_count: u64,
    inbound_queue_count: u64,
    endpoint_pair: &mut EndpointUriPair,
) {
    unimplemented!()
}



pub fn obj_process_pipe_term() {
    unimplemented!()
}

pub fn obj_process_pipe_term_ack() {
    unimplemented!()
}

pub fn obj_process_pipe_hwm( inhwm: i32, outhwm: i32) {
    unimplemented!()
}

pub fn obj_process_term_req( object: &mut ZmqOwn) {
    unimplemented!()
}

pub fn obj_process_term( linger: i32) {
    unimplemented!()
}

pub fn obj_process_term_ack() {
    unimplemented!()
}

pub fn obj_process_term_endpoint( endpoint: &str) {
    unimplemented!()
}

pub fn obj_process_reap( socket: &mut ZmqSocket) {
    unimplemented!()
}

pub fn obj_process_reaped() {
    unimplemented!()
}

pub fn obj_process_conn_failed() {
    unimplemented!()
}

//  Special handler called after a command that requires a seqnum
//  was processed. The implementation should catch up with its counter
//  of processed commands here.
fn obj_process_seqnum() {
    unimplemented!()
}

pub fn obj_send_command(cmd: &mut ZmqCommand) -> anyhow::Result<()> {
    match (cmd.cmd_type) {
        CommandType::ActivateRead => obj_process_activate_read(),
        CommandType::ActivateWrite => {
            obj_process_activate_write(cmd.args.activate_write.msgs_read)
        }
        CommandType::Stop => obj_process_stop(),
        CommandType::Plug => {
            obj_process_plug();
            obj_process_seqnum();
        }

        CommandType::Own => {
            obj_process_own(&mut cmd.args.own.object);
            obj_process_seqnum();
        }

        CommandType::Attach => {
            obj_process_attached(cmd.args.attach.engine);
            obj_process_seqnum();
        }

        CommandType::Bind => {
            obj_process_bind(&mut cmd.args.bind.pipe);
            obj_process_seqnum();
        }

        CommandType::Hiccup => obj_process_hiccup(cmd.args.hiccup.pipe),

        CommandType::PipePeerStats => obj_process_pipe_peer_stats(
            cmd.args.pipe_peer_stats.queue_count,
            &mut cmd.args.pipe_peer_stats.socket_base,
            cmd.args.pipe_peer_stats.endpoint_pair,
        ),

        CommandType::PipeStatsPublish => obj_process_pipe_stats_publish(
            cmd.args.pipe_stats_publish.outbound_queue_count,
            cmd.args.pipe_stats_publish.inbound_queue_count,
            cmd.args.pipe_stats_publish.endpoint_pair,
        ),

        CommandType::PipeTerm => obj_process_pipe_term(),

        CommandType::PipeTermAck => obj_process_pipe_term_ack(),

        CommandType::PipeHwm => {
            obj_process_pipe_hwm(cmd.args.pipe_hwm.inhwm, cmd.args.pipe_hwm.outhwm)
        }

        CommandType::TermReq => obj_process_term_req(&mut cmd.args.term_req.object),

        CommandType::Term => obj_process_term(cmd.args.term.linger),

        CommandType::TermAck => obj_process_term_ack(),

        CommandType::TermEndpoint => {
            obj_process_term_endpoint(&mut cmd.args.term_endpoint.endpoint)
        }

        CommandType::Reap => obj_process_reap(&mut cmd.args.reap.socket),

        CommandType::Reaped => obj_process_reaped(),

        CommandType::InprocConnected => obj_process_seqnum(),

        CommandType::ConnFailed => obj_process_conn_failed(),

        CommandType::Done => {}
        _ => {
            return Err(anyhow!("invalid command type: {}", cmd.cmd_type));
        }
    }

    Ok(())
}

pub fn obj_send_own(obj_a: &mut ZmqOwn, object: &mut ZmqOwn) {
    obj_a.inc_seqnum();
    let mut cmd = ZmqCommand::default();
    cmd.object = Some(obj_a.clone());
    cmd.cmd_type = ZmqCommand::own;
    cmd.args.own.object = object;
    obj_send_command(&mut cmd);
}