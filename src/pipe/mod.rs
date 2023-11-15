use crate::defines::array::ArrayItem;
use crate::defines::{MESSAGE_PIPE_GRANULARITY, ZMQ_MSG_MORE, ZMQ_MSG_ROUTING_ID};
use crate::endpoint::ZmqEndpointUriPair;
use crate::err::ZmqError;
use crate::err::ZmqError::PipeError;
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipeState::{
    Active, DelimiterReceived, TermAckSent, TermReqSent1, TermReqSent2, WaitingForDelimiter,
};
use crate::session::ZmqSession;
use crate::socket::ZmqSocket;
use crate::ypipe::ypipe_base::ZmqYPipeBase;
use crate::ypipe::ypipe_conflate::YPipeConflate;
use crate::ypipe::ZmqYPipe;
use libc::size_t;
use std::ffi::c_void;
use std::ptr::null_mut;
use crate::ctx::ZmqContext;
use crate::defines::err::ZmqError;
use crate::object::{obj_send_activate_read, obj_send_activate_write, obj_send_hiccup, obj_send_pipe_hwm, obj_send_pipe_peer_stats, obj_send_pipe_stats_publish, obj_send_pipe_term, obj_send_pipe_term_ack};

pub mod pipes;
pub mod zmq_pipe;
pub mod out_pipe;

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
    pub _in_pipe: Option<&'a mut YPipeConflate<'a, ZmqMsg>>,
    pub out_pipe: Option<&'a mut YPipeConflate<'a, ZmqMsg>>,
    pub _in_active: bool,
    pub _out_active: bool,
    pub _hwm: i32,
    pub _lwm: i32,
    pub _in_hwm_boost: i32,
    pub _out_hwm_boost: i32,
    pub _msgs_read: u64,
    pub _msgs_written: u64,
    pub _peers_msgs_read: u64,
    pub peer: Option<&'a mut ZmqPipe<'a>>,
    pub _sink: Option<&'a mut dyn IPipeEvents>,
    pub _state: ZmqPipeState,
    pub _delay: bool,
    pub _router_socket_routing_id: Vec<u8>,
    pub _server_socket_routing_id: i32,
    pub _conflate: bool,
    pub endpoint_pair: ZmqEndpointUriPair,
    pub _disconnect_msg: ZmqMsg,
}

impl ZmqPipe {
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
            _in_pipe: Some(inpipe_),
            out_pipe: Some(outpipe_),
            _in_active: false,
            _out_active: false,
            _hwm: inhwm_,
            _lwm: 0,
            _in_hwm_boost: 0,
            _out_hwm_boost: 0,
            _msgs_read: 0,
            _msgs_written: 0,
            _peers_msgs_read: 0,
            peer: None,
            _sink: None,
            _state: Active,
            _delay: false,
            _router_socket_routing_id: vec![],
            _server_socket_routing_id: 0,
            _conflate: false,
            endpoint_pair: ZmqEndpointUriPair::default(),
            _disconnect_msg: ZmqMsg::new(),
        }
    }

    pub fn set_peer(&mut self, peer_: &mut ZmqPipe) {
        self.peer = Some(peer_);
    }

    pub fn set_event_sink(&mut self, sink_: &mut dyn IPipeEvents) {
        self._sink = Some(sink_);
    }

    pub fn set_server_socket_router_id(&mut self, server_socket_routing_id_: u32) {
        self._server_socket_routing_id = server_socket_routing_id_ as i32;
    }

    pub fn get_server_socket_routing_id(&mut self) -> u32 {
        return self._server_socket_routing_id as u32;
    }

    pub fn set_router_socket_routing_id(&mut self, router_socket_routing_id_: &mut Vec<u8>) {
        self._router_socket_routing_id = router_socket_routing_id_.clone();
    }

    pub fn get_routing_id(&mut self) -> &mut Vec<u8> {
        return &mut self._router_socket_routing_id;
    }

    pub fn check_read(&mut self) -> bool {
        if self._in_active == false {
            return false;
        }

        if self._state != Active && self._state != WaitingForDelimiter {
            return false;
        }

        return true;
    }

    pub fn read(&mut self, ctx: &mut ZmqContext, msg: &mut ZmqMsg) -> Result<(), ZmqError> {
        if self._in_active == false {
            return Err(PipeError("Pipe is not readable"));
        }
        if self._state != Active && self._state != WaitingForDelimiter {
            return Err(PipeError("Pipe is not readable"));
        }

        loop {
            if (self._in_pipe).read(msg) == false {
                self._in_active = false;
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
            self._msgs_read += 1;
        }

        if self._lwm > 0 && self._msgs_read % self._lwm == 0 {
            obj_send_activate_write(ctx, self.peer.unwrap(), self._msgs_read);
        }

        Ok(())
    }

    pub fn check_write(&mut self) -> bool {
        if self._out_active == false || self._state != Active {
            return false;
        }

        let full = !self.check_hwm();
        if full {
            self._out_active = false;
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
            self._msgs_written += 1;
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
        if self._state == TermAckSent {
            return;
        }

        if self.out_pipe.is_some() && (*self.out_pipe).flush() == 0 {
            obj_send_activate_read(ctx, self.peer.unwrap());
        }
    }

    pub fn process_activate_read(&mut self) {
        if self._in_active == false && (self._state == Active || self._state == WaitingForDelimiter) {
            self._in_active = true;
            self._sink.read_activated(self);
        }
    }

    pub fn process_activate_write(&mut self, msgs_read_: u64) {
        self._peers_msgs_read = msgs_read_;
        if self._out_active == false && self._state == Active {
            self._out_active = true;
            self._sink.write_activated(self);
        }
    }

    pub fn process_hiccup(&mut self, pipe_: &mut YPipeConflate<ZmqMsg>) -> Result<(), ZmqError> {
        self.out_pipe.flush()?;
        let mut msg: ZmqMsg = ZmqMsg::new();
        while (self.out_pipe).unwrap().read(&mut msg) {
            if msg.flags & ZMQ_MSG_MORE == 0 {
                self._msgs_written -= 1
            }
            msg.close();
        }

        self.out_pipe = Some(pipe_);
        self._out_active = true;

        if self._state == Active {
            self._sink.hiccuped(self)
        }

        Ok(())
    }

    pub fn process_pipe_term(&mut self, ctx: &mut ZmqContext) {
        if self._state == Active {
            if (self._delay) {
                self._state = WaitingForDelimiter;
            } else {
                self._state = TermAckSent;
                self.out_pipe = None;
                obj_send_pipe_term_ack(ctx, self.peer.unwrap());
            }
        } else if self._state == DelimiterReceived {
            self._state = TermAckSent;
            self.out_pipe = None;
            obj_send_pipe_term_ack(ctx, self.peer.unwrap());
        } else if self._state == TermReqSent1 {
            self._state = TermReqSent2;
            self.out_pipe = None;
            obj_send_pipe_term_ack(ctx, self.peer.unwrap());
        }
    }

    pub fn process_pipe_term_ack(&mut self, ctx: &mut ZmqContext) {
        self._sink.pipe_terminated(self);
        if self._state == TermReqSent1 {
            self.out_pipe = None;
            obj_send_pipe_term_ack(ctx, self.peer.unwrap());
        } else {}

        if !self._conflate {
            let msg = ZmqMsg::new();
            while (*self._in_pipe).read(&msg) {
                msg.close()
            }
        }
    }

    pub fn process_pipe_hwm(&mut self, inhwm_: i32, outhwm_: i32) {
        self.set_hwms(inhwm_, outhwm_);
    }

    pub fn set_nodelay(&mut self) {
        self._delay = false;
    }

    pub fn terminate(&mut self, ctx: &mut ZmqContext, delay_: bool) {
        self._delay = delay_;

        if self._state == TermReqSent1 || self._state == TermReqSent2 {
            return;
        }

        if self._state == TermAckSent {
            return;
        }

        if self._state == Active {
            obj_send_pipe_term(ctx, self.peer.unwrap());
            self._state = TermReqSent1;
        } else if self._state == WaitingForDelimiter && self._delay == false {
            self.rollback();
            self.out_pipe = None;
            obj_send_pipe_term_ack(ctx, self.peer.unwrap());
            self._state = TermAckSent;
        } else if self._state == WaitingForDelimiter {} else if self._state == DelimiterReceived {
            obj_send_pipe_term(ctx, self.peer.unwrap());
            self._state = TermReqSent1;
        } else {}
        self._out_active = false;
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
        if self._state == Active {
            self._state = DelimiterReceived;
        } else {
            self.rollback();
            self.out_pipe = None;
            obj_send_pipe_term_ack(ctx, self.peer.unwrap());
            self._state = TermAckSent;
        }
    }

    pub fn hiccup(&mut self, ctx: &mut ZmqContext) {
        if self._state != Active {
            return;
        }
        if self._conflate == true {
            self._in_pipe = Some(&mut YPipeConflate::new())
        } else {
            self._in_pipe = Some(&mut YPipeConflate::new())
        };
        self._in_active = true;
        obj_send_hiccup(ctx, self.peer.unwrap(), self._in_pipe.unwrap());
    }

    pub fn set_hwms(&mut self, inhwm_: i32, outhwm_: i32) {
        let mut in_ = inhwm_ + i32::max(self._in_hwm_boost, 0);
        let mut out_ = outhwm_ + i32::max(self._out_hwm_boost, 0);

        if inhwm_ <= 0 || self._in_hwm_boost == 0 {
            in_ = 0;
        }
        if outhwm_ <= 0 || self._out_hwm_boost == 0 {
            out_ = 0;
        }

        self._lwm == self.compute_lwm(in_);
        self._hwm = out_;
    }

    pub fn set_hwms_boost(&mut self, inhwmboost_: i32, outhwmboost_: i32) {
        self._in_hwm_boost = inhwmboost_;
        self._out_hwm_boost = outhwmboost_;
    }

    pub fn check_hwm(&mut self) -> bool {
        let full = self._hwm >= 0 && self._msgs_written - self._peers_msgs_read >= self._hwm as u64;
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
            self._msgs_written - self._peers_msgs_read,
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
            ctx: &mut ZmqContext,
            socket_base_,
            queue_count_,
            self._msgs_written - self._peers_msgs_read,
            endpoint_pair_,
        );
    }

    pub fn send_disconnect_msg(&mut self, ctx: &mut ZmqContext) {
        if self._disconnect_msg.size() > 0 && self.out_pipe.is_some() {
            self.rollback();
            (*self.out_pipe).write(self._disconnect_msg, false);
            self.flush(ctx);
            self._disconnect_msg.init2()
        }
    }

    pub fn set_disconnect_msg2(&mut self, disconnect: &mut Vec<u8>) -> Result<(), ZmqError> {
        self._disconnect_msg.close()?;
        self._disconnect_msg.init_buffer(disconnect, disconnect.len())
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
) -> Result<(),ZmqError> {
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
