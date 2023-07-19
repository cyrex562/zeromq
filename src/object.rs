use crate::thread_command::{ThreadCommandType, ZmqThreadCommand};
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


fn obj_process_command(cmd: &ZmqThreadCommand) {
    match cmd.cmd_type {
        ThreadCommandType::Stop => {}
        ThreadCommandType::Plug => {}
        ThreadCommandType::Own => {}
        ThreadCommandType::Attach => {}
        ThreadCommandType::Bind => {}
        ThreadCommandType::ActivateRead => {}
        ThreadCommandType::ActivateWrite => {}
        ThreadCommandType::Hiccup => {}
        ThreadCommandType::PipeTerm => {}
        ThreadCommandType::PipeTermAck => {}
        ThreadCommandType::PipeHwm => {}
        ThreadCommandType::TermReq => {}
        ThreadCommandType::Term => {}
        ThreadCommandType::TermAck => {}
        ThreadCommandType::TermEndpoint => {}
        ThreadCommandType::Reap => {}
        ThreadCommandType::Reaped => {}
        ThreadCommandType::InprocConnected => {}
        ThreadCommandType::ConnFailed => {}
        ThreadCommandType::PipePeerStats => {}
        ThreadCommandType::PipeStatsPublish => {}
        ThreadCommandType::Done => {}
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

    let mut cmd = ZmqThreadCommand::default();
    cmd.session = Some(destination);
    cmd.cmd_type = ZmqThreadCommand::attach;
    cmd.args.attach.engine = engine;
    obj_send_command(&mut cmd);
}

fn obj_send_activate_read(pipe: &mut ZmqPipe) {
    let mut cmd = ZmqThreadCommand::default();
    cmd.pipe = Some(pipe);
    cmd.cmd_type = ThreadCommandType::ActivateRead;
    obj_send_command(&mut cmd);
}

fn obj_send_activate_write(pipe: &mut ZmqPipe, msgs_read: u64) {
    let mut cmd = ZmqThreadCommand::default();
    cmd.pipe = Some(pipe);
    cmd.cmd_type = ThreadCommandType::ActivateWrite;
    cmd.args.activate_write.msgs_read = msgs_read;
    obj_send_command(&mut cmd);
}

fn obj_end_hiccup(pipe_a: &mut ZmqPipe, pipe: &mut [u8]) {
    let mut cmd = ZmqThreadCommand::default();
    cmd.pipe = Some(pipe_a);
    cmd.cmd_type = ThreadCommandType::Hiccup;
    cmd.args.hiccup.pipe = pipe;
    obj_send_command(&mut cmd);
}

fn obj_send_pipe_peer_stats(

    pipe: &mut ZmqPipe,
    queue_count: u64,
    socket_base: &mut ZmqOwn,
    endpoint_pair: &mut EndpointUriPair,
) {
    let mut cmd = ZmqThreadCommand::default();
    cmd.pipe = Some(pipe);
    cmd.cmd_type = ThreadCommandType::PipePeerStats;
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
    let mut cmd = ZmqThreadCommand::default();
    cmd.object = Some(object.clone());
    cmd.cmd_type = ThreadCommandType::PipeStatsPublish;
    cmd.args.pipe_stats_publish.outbound_queue_count = outbound_queue_count;
    cmd.args.pipe_stats_publish.inbound_queue_count = inbound_queue_count;
    cmd.args.pipe_stats_publish.endpoint_pair = endpoint_pair;
    obj_send_command(&mut cmd);
}

fn obj_send_pipe_term(pipe: &mut ZmqPipe) {
    let mut cmd = ZmqThreadCommand::default();
    cmd.pipe = Some(pipe);
    cmd.cmd_type = ThreadCommandType::PipeTerm;
    obj_send_command(&mut cmd);
}

fn obj_send_pipe_term_ack(pipe: &mut ZmqPipe) {
    let mut cmd = ZmqThreadCommand::default();
    cmd.pipe = Some(pipe);
    cmd.cmd_type = ThreadCommandType::PipeTermAck;
    obj_send_command(&mut cmd);
}

fn obj_send_pipe_hwm(pipe: &mut ZmqPipe, inhwm: i32, outhwm: i32) {
    let mut cmd = ZmqThreadCommand::default();
    cmd.pipe = Some(pipe);
    cmd.cmd_type = ThreadCommandType::PipeHwm;
    cmd.args.pipe_hwm.inhwm = inhwm;
    cmd.args.pipe_hwm.outhwm = outhwm;
    obj_send_command(&mut cmd);
}

fn obj_send_term_req(object_a: &mut ZmqOwn, object: &mut ZmqOwn) {
    let mut cmd = ZmqThreadCommand::default();
    cmd.object = Some(object_a.clone());
    cmd.cmd_type = ThreadCommandType::TermReq;
    cmd.args.term_req.object = object;
    obj_send_command(&mut cmd);
}

fn obj_send_term(object: &mut ZmqOwn, linger: i32) {
    let mut cmd = ZmqThreadCommand::default();
    cmd.object = Some(object.clone());
    cmd.cmd_type = ThreadCommandType::Term;
    cmd.args.term.linger = linger;
    obj_send_command(&mut cmd);
}

fn obj_send_term_ack(object: &mut ZmqOwn) {
    let mut cmd = ZmqThreadCommand::default();
    cmd.object = Some(object.clone());
    cmd.cmd_type = ThreadCommandType::TermAck;
    obj_send_command(&mut cmd);
}

fn obj_send_term_endpoint(object: &mut ZmqOwn, endpoint: &str) {
    let mut cmd = ZmqThreadCommand::default();
    cmd.object = Some(object.clone());
    cmd.cmd_type = ThreadCommandType::TermEndpoint;
    cmd.args.term_endpoint.endpoint = endpoint.into_string();
    obj_send_command(&mut cmd);
}

fn obj_send_reap( socket: &mut ZmqSocket) {
    let mut cmd = ZmqThreadCommand::default();
    cmd.reaper = Some(socket.context.get_reaper().unwrap().clone());
    cmd.cmd_type = ThreadCommandType::Reap;
    cmd.args.reap.socket = socket;
    obj_send_command(&mut cmd);
}

fn obj_send_reaped(ctx: &mut ZmqContext) {
    let mut cmd = ZmqThreadCommand::default();
    cmd.reaper = Some(ctx.get_reaper().unwrap().clone());
    cmd.cmd_type = ThreadCommandType::Reaped;
    obj_send_command(&mut cmd);
}

fn obj_send_done(ctx: &mut ZmqContext) {
    let mut cmd = ZmqThreadCommand::default();
    // cmd.destination = None;
    cmd.cmd_type = ThreadCommandType::Done;
    ctx.send_command(ZmqContext::TERM_TID, &mut cmd);
}

fn obj_send_conn_failed( destination: &mut ZmqSessionBase) {
    let mut cmd = ZmqThreadCommand::default();
    cmd.session = Some(destination);
    cmd.cmd_type = ThreadCommandType::ConnFailed;
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

pub fn obj_send_command(cmd: &mut ZmqThreadCommand) -> anyhow::Result<()> {
    match (cmd.cmd_type) {
        ThreadCommandType::ActivateRead => obj_process_activate_read(),
        ThreadCommandType::ActivateWrite => {
            obj_process_activate_write(cmd.args.activate_write.msgs_read)
        }
        ThreadCommandType::Stop => obj_process_stop(),
        ThreadCommandType::Plug => {
            obj_process_plug();
            obj_process_seqnum();
        }

        ThreadCommandType::Own => {
            obj_process_own(&mut cmd.args.own.object);
            obj_process_seqnum();
        }

        ThreadCommandType::Attach => {
            obj_process_attached(cmd.args.attach.engine);
            obj_process_seqnum();
        }

        ThreadCommandType::Bind => {
            obj_process_bind(&mut cmd.args.bind.pipe);
            obj_process_seqnum();
        }

        ThreadCommandType::Hiccup => obj_process_hiccup(cmd.args.hiccup.pipe),

        ThreadCommandType::PipePeerStats => obj_process_pipe_peer_stats(
            cmd.args.pipe_peer_stats.queue_count,
            &mut cmd.args.pipe_peer_stats.socket_base,
            cmd.args.pipe_peer_stats.endpoint_pair,
        ),

        ThreadCommandType::PipeStatsPublish => obj_process_pipe_stats_publish(
            cmd.args.pipe_stats_publish.outbound_queue_count,
            cmd.args.pipe_stats_publish.inbound_queue_count,
            cmd.args.pipe_stats_publish.endpoint_pair,
        ),

        ThreadCommandType::PipeTerm => obj_process_pipe_term(),

        ThreadCommandType::PipeTermAck => obj_process_pipe_term_ack(),

        ThreadCommandType::PipeHwm => {
            obj_process_pipe_hwm(cmd.args.pipe_hwm.inhwm, cmd.args.pipe_hwm.outhwm)
        }

        ThreadCommandType::TermReq => obj_process_term_req(&mut cmd.args.term_req.object),

        ThreadCommandType::Term => obj_process_term(cmd.args.term.linger),

        ThreadCommandType::TermAck => obj_process_term_ack(),

        ThreadCommandType::TermEndpoint => {
            obj_process_term_endpoint(&mut cmd.args.term_endpoint.endpoint)
        }

        ThreadCommandType::Reap => obj_process_reap(&mut cmd.args.reap.socket),

        ThreadCommandType::Reaped => obj_process_reaped(),

        ThreadCommandType::InprocConnected => obj_process_seqnum(),

        ThreadCommandType::ConnFailed => obj_process_conn_failed(),

        ThreadCommandType::Done => {}
        _ => {
            return Err(anyhow!("invalid command type: {}", cmd.cmd_type));
        }
    }

    Ok(())
}

pub fn obj_send_own(obj_a: &mut ZmqOwn, object: &mut ZmqOwn) {
    obj_a.inc_seqnum();
    let mut cmd = ZmqThreadCommand::default();
    cmd.object = Some(obj_a.clone());
    cmd.cmd_type = ZmqThreadCommand::own;
    cmd.args.own.object = object;
    obj_send_command(&mut cmd);
}