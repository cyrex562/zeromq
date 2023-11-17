use crate::ctx::ZmqContext;
use crate::defines::{ZMQ_MSG_MORE, ZMQ_MSG_ROUTING_ID};
use crate::defines::err::ZmqError;
use crate::defines::err::ZmqError::PipeError;
use crate::endpoint::ZmqEndpointUriPair;
use crate::engine::ZmqEngine;
use crate::msg::ZmqMsg;
use crate::object::{obj_send_activate_read, obj_send_activate_write, obj_send_hiccup, obj_send_pipe_hwm, obj_send_pipe_peer_stats, obj_send_pipe_stats_publish, obj_send_pipe_term, obj_send_pipe_term_ack};
use crate::options::ZmqOptions;
use crate::own::{own_process_own, own_process_seqnum, own_process_term, ZmqOwn};
use crate::pipe::ZmqPipeState::{
    Active, DelimiterReceived, TermAckSent, TermReqSent1, TermReqSent2, WaitingForDelimiter,
};
use crate::session::ZmqSession;
use crate::socket::ZmqSocket;
use crate::ypipe::ypipe_conflate::YPipeConflate;

pub mod pipes;
pub mod zmq_pipe;
pub mod out_pipe;
pub mod pipe_event;

pub trait IPipeEvents {
    fn read_activated(&self, pipe_: &mut ZmqPipe);
    fn write_activated(&self, pipe_: &mut ZmqPipe);
    fn hiccuped(&self, pipe_: &mut ZmqPipe);
    fn pipe_terminated(&self, pipe_: &mut ZmqPipe);
}

pub enum ZmqPipeState {
    Active,
    DelimiterReceived,
    WaitingForDelimiter,
    TermAckSent,
    TermReqSent1,
    TermReqSent2,
}

// pub type ZmqUpipe<'a> = ZmqYPipeBase<ZmqMsg>;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ZmqPipe<'a> {
    // pub base: &'a mut ZmqObject<'a>,
    // pub _array_item_1: ArrayItem<1>,
    // pub _array_item_2: ArrayItem<2>,
    // pub _array_item_3: ArrayItem<3>,
    pub in_pipe: Option<&'a mut YPipeConflate<'a, ZmqMsg>>,
    pub out_pipe: Option<&'a mut YPipeConflate<'a, ZmqMsg>>,
    pub in_active: bool,
    pub out_active: bool,
    pub hwm: i32,
    pub lwm: i32,
    pub in_hwm_boost: i32,
    pub out_hwm_boost: i32,
    pub msgs_read: u64,
    pub msgs_written: u64,
    pub peers_msgs_read: u64,
    pub peer: Option<&'a mut ZmqPipe<'a>>,
    pub sink: Option<&'a mut dyn IPipeEvents>,
    pub state: ZmqPipeState,
    pub delay: bool,
    pub router_socket_routing_id: Vec<u8>,
    pub server_socket_routing_id: i32,
    pub conflate: bool,
    pub endpoint_pair: ZmqEndpointUriPair,
    pub disconnect_msg: ZmqMsg,
}

impl<'a> ZmqPipe<'a> {
    fn new(
        // parent_: &mut ZmqObject,
        inpipe_: &mut YPipeConflate<ZmqMsg>,
        outpipe_: &mut YPipeConflate<ZmqMsg>,
        inhwm_: i32,
        outhwm_: i32,
        conflate_: bool,
    ) -> Self {
        Self {
            // base: parent_,
            // _array_item_1: ArrayItem::new(),
            // _array_item_2: ArrayItem::new(),
            // _array_item_3: ArrayItem::new(),
            in_pipe: Some(inpipe_),
            out_pipe: Some(outpipe_),
            in_active: false,
            out_active: false,
            hwm: inhwm_,
            lwm: 0,
            in_hwm_boost: 0,
            out_hwm_boost: 0,
            msgs_read: 0,
            msgs_written: 0,
            peers_msgs_read: 0,
            peer: None,
            sink: None,
            state: Active,
            delay: false,
            router_socket_routing_id: vec![],
            server_socket_routing_id: 0,
            conflate: conflate_,
            endpoint_pair: ZmqEndpointUriPair::default(),
            disconnect_msg: ZmqMsg::new(),
        }
    }

    pub fn set_peer(&mut self, peer_: &mut ZmqPipe) {
        self.peer = Some(peer_);
    }

    pub fn set_event_sink(&mut self, sink_: &mut dyn IPipeEvents) {
        self.sink = Some(sink_);
    }

    pub fn set_server_socket_router_id(&mut self, server_socket_routing_id_: u32) {
        self.server_socket_routing_id = server_socket_routing_id_ as i32;
    }

    pub fn get_server_socket_routing_id(&mut self) -> u32 {
        return self.server_socket_routing_id as u32;
    }

    pub fn set_router_socket_routing_id(&mut self, router_socket_routing_id_: &mut Vec<u8>) {
        self.router_socket_routing_id = router_socket_routing_id_.clone();
    }

    pub fn get_routing_id(&mut self) -> &mut Vec<u8> {
        return &mut self.router_socket_routing_id;
    }

    pub fn check_read(&mut self) -> bool {
        if self.in_active == false {
            return false;
        }

        if self.state != Active && self.state != WaitingForDelimiter {
            return false;
        }

        return true;
    }

    pub fn read(&mut self, ctx: &mut ZmqContext, msg: &mut ZmqMsg) -> Result<(), ZmqError> {
        if self.in_active == false {
            return Err(PipeError("Pipe is not readable"));
        }
        if self.state != Active && self.state != WaitingForDelimiter {
            return Err(PipeError("Pipe is not readable"));
        }

        loop {
            if (self.in_pipe).read(msg) == false {
                self.in_active = false;
                return Err(PipeError("Pipe is not readable"));
            }

            if msg.is_credential() {
                (msg).close()
            } else {
                break;
            }
        }

        if msg.is_delimiter() {
            self.process_delimiter(ctx);
            return Err(PipeError("Pipe is not readable"));
        }

        if !(msg.flags() & ZMQ_MSG_MORE > 0) && !msg.is_routing_id() {
            self.msgs_read += 1;
        }

        if self.lwm > 0 && self.msgs_read % self.lwm == 0 {
            obj_send_activate_write(ctx, self.peer.unwrap(), self.msgs_read);
        }

        Ok(())
    }

    pub fn check_write(&mut self) -> bool {
        if self.out_active == false || self.state != Active {
            return false;
        }

        let full = !self.check_hwm();
        if full {
            self.out_active = false;
            return false;
        }

        return true;
    }

    pub fn write(&mut self, msg: &mut ZmqMsg) -> Result<(), ZmqError> {
        if self.check_write() == false {
            return Err(PipeError("Pipe is not writable"));
        }

        let more = msg.flag_set(ZMQ_MSG_MORE);
        let is_routing_id = msg.is_routing_id();

        self.out_pipe.unwrap().write(msg, true);
        if more && !is_routing_id {
            self.msgs_written += 1;
        }
        Ok(())
    }

    pub fn rollback(&mut self) {
        let mut msg: ZmqMsg;
        if self.out_pipe {
            while self.out_pipe.unwrite(&mut msg) {
                let rc = msg.close();
            }
        }
    }

    pub fn flush(&mut self, ctx: &mut ZmqContext) {
        if self.state == TermAckSent {
            return;
        }

        if self.out_pipe.is_some() && (*self.out_pipe).flush() == 0 {
            obj_send_activate_read(ctx, self.peer.unwrap());
        }
    }

    pub fn process_activate_read(&mut self) {
        if self.in_active == false && (self.state == Active || self.state == WaitingForDelimiter) {
            self.in_active = true;
            self.sink.read_activated(self);
        }
    }

    pub fn process_own(&mut self, own: &mut ZmqOwn) {
        let mut a_own: ZmqOwn = ZmqOwn::default();
        own_process_own(&mut a_own, own)
    }

    pub fn process_activate_write(&mut self, msgs_read_: u64) {
        self.peers_msgs_read = msgs_read_;
        if self.out_active == false && self.state == Active {
            self.out_active = true;
            self.sink.write_activated(self);
        }
    }

    pub fn process_hiccup(&mut self, pipe_: &mut YPipeConflate<ZmqMsg>) -> Result<(), ZmqError> {
        self.out_pipe.flush()?;
        let mut msg: ZmqMsg = ZmqMsg::new();
        while (self.out_pipe).unwrap().read(&mut msg) {
            if msg.flags & ZMQ_MSG_MORE == 0 {
                self.msgs_written -= 1
            }
            msg.close()?;
        }

        self.out_pipe = Some(pipe_);
        self.out_active = true;

        if self.state == Active {
            self.sink.hiccuped(self)
        }

        Ok(())
    }

    pub fn process_pipe_term(&mut self, ctx: &mut ZmqContext) {
        if self.state == Active {
            if self.delay {
                self.state = WaitingForDelimiter;
            } else {
                self.state = TermAckSent;
                self.out_pipe = None;
                obj_send_pipe_term_ack(ctx, self.peer.unwrap());
            }
        } else if self.state == DelimiterReceived {
            self.state = TermAckSent;
            self.out_pipe = None;
            obj_send_pipe_term_ack(ctx, self.peer.unwrap());
        } else if self.state == TermReqSent1 {
            self.state = TermReqSent2;
            self.out_pipe = None;
            obj_send_pipe_term_ack(ctx, self.peer.unwrap());
        }
    }

    pub fn process_pipe_term_ack(&mut self, ctx: &mut ZmqContext) {
        self.sink.pipe_terminated(self);
        if self.state == TermReqSent1 {
            self.out_pipe = None;
            obj_send_pipe_term_ack(ctx, self.peer.unwrap());
        } else {}

        if !self.conflate {
            let msg = ZmqMsg::new();
            while (*self.in_pipe).read(&msg) {
                msg.close()
            }
        }
    }

    pub fn process_term(&mut self, ctx: &mut ZmqContext, own: &mut ZmqOwn, linger: i32) {
        own_process_term(
            ctx,
            &mut own.owned,
            &mut own.terminating,
            &mut own.term_acks,
            linger,
        )
    }

    pub fn process_seqnum(&mut self, own: &mut ZmqOwn) {
        own_process_seqnum(own)
    }

    pub fn process_plug(&mut self, options: &ZmqOptions, session: &mut ZmqSession) {
        session.process_plug(options);
    }

    pub fn process_attach(&mut self, options: &ZmqOptions, engine: &mut ZmqEngine) {
        engine.session.unwrap().process_attach(options, engine);
    }

    pub fn process_stop(&mut self, ctx: &mut ZmqContext, options: &mut ZmqOptions, socket: &mut ZmqSocket) -> Result<(), ZmqError> {
        socket.process_stop(ctx, options)
    }

    pub fn process_pipe_hwm(&mut self, inhwm_: i32, outhwm_: i32) {
        self.set_hwms(inhwm_, outhwm_);
    }

    pub fn set_nodelay(&mut self) {
        self.delay = false;
    }

    pub fn terminate(&mut self, ctx: &mut ZmqContext, delay_: bool) {
        self.delay = delay_;

        if self.state == TermReqSent1 || self.state == TermReqSent2 {
            return;
        }

        if self.state == TermAckSent {
            return;
        }

        if self.state == Active {
            obj_send_pipe_term(ctx, self.peer.unwrap());
            self.state = TermReqSent1;
        } else if self.state == WaitingForDelimiter && self.delay == false {
            self.rollback();
            self.out_pipe = None;
            obj_send_pipe_term_ack(ctx, self.peer.unwrap());
            self.state = TermAckSent;
        } else if self.state == WaitingForDelimiter {} else if self.state == DelimiterReceived {
            obj_send_pipe_term(ctx, self.peer.unwrap());
            self.state = TermReqSent1;
        } else {}
        self.out_active = false;
        if self.out_pipe {
            self.rollback();
            let mut msg = ZmqMsg::new();
            msg.init_delimiter();
            (*self.out_pipe).write(&mut msg, false);
            self.flush(ctx)
        }
    }

    pub fn is_delimiter(&mut self, msg_: &mut ZmqMsg) -> bool {
        msg_.is_delimiter()
    }

    pub fn compute_lwm(&mut self, hwm_: i32) -> i32 {
        (hwm_ + 1) / 2
    }

    pub fn process_delimiter(&mut self, ctx: &mut ZmqContext) {
        if self.state == Active {
            self.state = DelimiterReceived;
        } else {
            self.rollback();
            self.out_pipe = None;
            obj_send_pipe_term_ack(ctx, self.peer.unwrap());
            self.state = TermAckSent;
        }
    }

    pub fn hiccup(&mut self, ctx: &mut ZmqContext) {
        if self.state != Active {
            return;
        }
        if self.conflate == true {
            self.in_pipe = Some(&mut YPipeConflate::new())
        } else {
            self.in_pipe = Some(&mut YPipeConflate::new())
        };
        self.in_active = true;
        obj_send_hiccup(ctx, self.peer.unwrap(), self.in_pipe.unwrap());
    }

    pub fn set_hwms(&mut self, inhwm_: i32, outhwm_: i32) {
        let mut in_ = inhwm_ + i32::max(self.in_hwm_boost, 0);
        let mut out_ = outhwm_ + i32::max(self.out_hwm_boost, 0);

        if inhwm_ <= 0 || self.in_hwm_boost == 0 {
            in_ = 0;
        }
        if outhwm_ <= 0 || self.out_hwm_boost == 0 {
            out_ = 0;
        }

        self.lwm == self.compute_lwm(in_);
        self.hwm = out_;
    }

    pub fn set_hwms_boost(&mut self, inhwmboost_: i32, outhwmboost_: i32) {
        self.in_hwm_boost = inhwmboost_;
        self.out_hwm_boost = outhwmboost_;
    }

    pub fn check_hwm(&mut self) -> bool {
        let full = self.hwm >= 0 && self.msgs_written - self.peers_msgs_read >= self.hwm as u64;
        !full
    }

    pub fn send_hwms_to_peer(&mut self, ctx: &mut ZmqContext, inhwm_: i32, outhwm_: i32) {
        obj_send_pipe_hwm(ctx, self.peer.unwrap(), inhwm_, outhwm_);
    }

    pub fn set_endpoint_pair(&mut self, endpoint_pair_: ZmqEndpointUriPair) {
        self.endpoint_pair = endpoint_pair_;
    }

    pub fn send_stats_to_peer(&mut self, ctx: &mut ZmqContext, socket: &mut ZmqSocket) {
        let mut ep = ZmqEndpointUriPair::from_endpoint_uri_pair(&mut self.endpoint_pair);
        obj_send_pipe_peer_stats(
            ctx,
            self.peer.unwrap(),
            self.msgs_written - self.peers_msgs_read,
            socket,
            &mut ep,
        );
    }

    pub fn process_pipe_peer_stats(
        &mut self,
        ctx: &mut ZmqContext,
        queue_count_: u64,
        socket_base_: &mut ZmqSocket,
        endpoint_pair_: &mut ZmqEndpointUriPair,
    ) {
        obj_send_pipe_stats_publish(
            ctx,
            socket_base_,
            queue_count_,
            self.msgs_written - self.peers_msgs_read,
            endpoint_pair_,
        );
    }

    pub fn process_pipe_stats_publish(
        &mut self,
        context: &mut ZmqContext,
        options: &mut ZmqOptions,
        socket: &mut ZmqSocket,
        outbound_queue_count: u64,
        inbound_queue_count: u64,
        endpoint_pair: &mut ZmqEndpointUriPair,
    ) {
        socket.process_pipe_stats_publish(context, options, outbound_queue_count, inbound_queue_count, endpoint_pair);
    }


    pub fn send_disconnect_msg(&mut self, ctx: &mut ZmqContext) {
        if self.disconnect_msg.size() > 0 && self.out_pipe.is_some() {
            self.rollback();
            (self.out_pipe).unwrap().write(&mut self.disconnect_msg, false);
            self.flush(ctx);
            self.disconnect_msg.init2()
        }
    }

    pub fn set_disconnect_msg2(&mut self, disconnect: &mut Vec<u8>) -> Result<(), ZmqError> {
        self.disconnect_msg.close()?;
        self.disconnect_msg.init_buffer(disconnect, disconnect.len())
    }

    pub fn send_hiccup_msg(&mut self, ctx: &mut ZmqContext, hiccup: &mut Vec<u8>) -> Result<(), ZmqError> {
        if hiccup.is_empty() == false && self.out_pipe.is_some() {
            let mut msg: ZmqMsg = ZmqMsg::new();
            msg.init_buffer(hiccup, hiccup.len())?;
            (*self.out_pipe).write(&mut msg, false);
            self.flush(ctx);
        }

        Ok(())
    }
}

// type upipe_normal_t = ZmqYPipe<ZmqMsg, MESSAGE_PIPE_GRANULARITY>;
// type upipe_conflate_t = YPipeConflate<ZmqMsg>;

pub fn pipepair(
    parents_: (&mut ZmqSession, &mut ZmqSocket),
    pipes_: &mut [Option<&mut ZmqPipe>; 2],
    hwms_: [i32; 2],
    conflate_: [bool; 2],
) -> Result<(), ZmqError> {
    let mut upipe1: YPipeConflate<ZmqMsg>;
    if conflate_[0] == true {
        upipe1 = YPipeConflate::new();
    } else {
        upipe1 = YPipeConflate::new();
    }

    let mut upipe2: YPipeConflate<ZmqMsg>;
    if conflate_[1] == true {
        upipe2 = YPipeConflate::new();
    } else {
        upipe2 = YPipeConflate::new();
    }

    pipes_[0] = Some(&mut ZmqPipe::new(
        &mut upipe1,
        &mut upipe2,
        hwms_[1],
        hwms_[0],
        conflate_[0],
    ));
    pipes_[1] = Some(&mut ZmqPipe::new(
        &mut upipe2,
        &mut upipe1,
        hwms_[0],
        hwms_[1],
        conflate_[1],
    ));

    pipes_[0].unwrap().set_peer(pipes_[1].unwrap());
    pipes_[1].unwrap().set_peer(pipes_[0].unwrap());

    return Ok(());
}

pub fn send_routing_id(ctx: &mut ZmqContext, pipe_: &mut ZmqPipe, options_: &ZmqOptions) -> Result<(), ZmqError> {
    let mut id = ZmqMsg::new();
    id.init_size(options_.routing_id_size)?;
    // libc::memcpy(
    //     id.data(),
    //     &options_.routing_id,
    //     options_.routing_id_size as size_t,
    // );
    id.data_mut().copy_from_slice(&options_.routing_id);
    id.set_flags(ZMQ_MSG_ROUTING_ID);
    (pipe_).write(&mut id)?;
    (pipe_).flush(ctx);
    Ok(())
}

pub fn send_hello_msg(pipe_: &mut ZmqPipe, options_: &ZmqOptions) -> Result<(), ZmqError> {
    let mut hello_msg = ZmqMsg::new();
    hello_msg.init_buffer(&options_.hello_msg[0], options_.hello_msg.size())?;
    (pipe_).write(&mut hello_msg)?;
    Ok(())
}
