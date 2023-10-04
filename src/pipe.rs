use std::ffi::c_void;
use std::ptr::null_mut;
use libc::size_t;
use crate::array::array_item_t;
use crate::blob::blob_t;
use crate::endpoint::endpoint_uri_pair_t;
use crate::msg::{more, msg_t, routing_id};
use crate::object::object_t;
use crate::ypipe::ypipe_t;
use crate::ypipe_base::ypipe_base_t;
use crate::defines::message_pipe_granularity;
use crate::options::options_t;
use crate::own::own_t;
use crate::pipe::pipe_state::{active, delimiter_received, term_ack_sent, term_req_sent1, term_req_sent2, waiting_for_delimiter};
use crate::ypipe_conflate::ypipe_conflate_t;

pub trait i_pipe_events {
    fn read_activated(&self, pipe_: &mut pipe_t);
    fn write_activated(&self, pipe_: &mut pipe_t);
    fn hiccuped(&self, pipe_: &mut pipe_t);
    fn pipe_terminated(&self, pipe_: &mut pipe_t);
}

pub enum pipe_state {
    active,
    delimiter_received,
    waiting_for_delimiter,
    term_ack_sent,
    term_req_sent1,
    term_req_sent2,
}

pub type upipe_t = ypipe_base_t<msg_t>;

pub struct pipe_t {
    pub _base: *mut object_t,
    pub _array_item_1: array_item_t<1>,
    pub _array_item_2: array_item_t<2>,
    pub _array_item_3: array_item_t<3>,
    pub _in_pipe: *mut upipe_t,
    pub _out_pipe: *mut upipe_t,
    pub _in_active: bool,
    pub _out_active: bool,
    pub _hwm: i32,
    pub _lwm: i32,
    pub _in_hwm_boost: i32,
    pub _out_hwm_boost: i32,
    pub _msgs_read: u64,
    pub _msgs_written: u64,
    pub _peers_msgs_read: u64,
    pub _peer: *mut pipe_t,
    pub _sink: *mut dyn i_pipe_events,
    pub _state: pipe_state,
    pub _delay: bool,
    pub _router_socket_routing_id: blob_t,
    pub _server_socket_routing_id: i32,
    pub _conflate: bool,
    pub _endpoint_pair: endpoint_uri_pair_t,
    pub _disconnect_msg: msg_t,
}

impl pipe_t {

    fn new(parent_: *mut object_t, inpipe_: *mut upipe_t, outpipe_: *mut upipe_t, inhwm_: i32, outhwm_: i32, conflate_: bool) -> Self {
        Self {
            _base: parent_,
            _array_item_1: array_item_t::new(),
            _array_item_2: array_item_t::new(),
            _array_item_3: array_item_t::new(),
            _in_pipe: null_mut(),
            _out_pipe: null_mut(),
            _in_active: false,
            _out_active: false,
            _hwm: 0,
            _lwm: 0,
            _in_hwm_boost: 0,
            _out_hwm_boost: 0,
            _msgs_read: 0,
            _msgs_written: 0,
            _peers_msgs_read: 0,
            _peer: null_mut(),
            _sink: null_mut(),
            _state: pipe_state::active,
            _delay: false,
            _router_socket_routing_id: blob_t::new(),
            _server_socket_routing_id: 0,
            _conflate: false,
            _endpoint_pair: endpoint_uri_pair_t::new(),
            _disconnect_msg: msg_t::new(),
        }
    }

    pub fn set_peer(&mut self, peer_: *mut pipe_t){
        self._peer = peer_;
    }

    pub fn set_event_sink(&mut self, sink_: *mut dyn i_pipe_events){
        self._sink = sink_;
    }

    pub fn set_server_socket_router_id(&mut self, server_socket_routing_id_: u32) {
        self._server_socket_routing_id = server_socket_routing_id_ as i32;
    }

    pub fn get_server_socket_routing_id(&mut self) -> u32 {
        return self._server_socket_routing_id as u32;
    }

    pub fn set_router_socket_routing_id(&mut self, router_socket_routing_id_: blob_t) {
        self._router_socket_routing_id = router_socket_routing_id_;
    }

    pub fn get_routing_id(&mut self) -> &mut blob_t {
        return &mut self._router_socket_routing_id;
    }

    pub fn check_read(&mut self) -> bool {
        if self._in_active == false {
            return false;
        }

        if self._state != active && self._state != waiting_for_delimiter {
            return false;
        }

        return true;
    }

    pub unsafe fn read(&mut self, msg_: *mut msg_t) -> bool {
        if self._in_active == false {
            return false;
        }
        if self._state != active && self._state != waiting_for_delimiter {
            return false;
        }

        loop {
            if (*self._in_pipe).read(msg_) == false {
                self._in_active = false;
                return false;
            }

            if msg_.is_credential() {
                (*msg_).close()
            } else {
                break;
            }
        }


        if msg_.is_delimiter() {
            self.process_delimiter();
            return false;
        }

        if !(msg_.flags() & more > 0) && !msg_.is_routing_id() {
            self._msgs_read += 1;
        }

        if self._lwm > 0 && self._msgs_read % self._lwm == 0 {
            self._base.send_activate_write(self._peer, self._msgs_read);
        }

        return true;
    }

    pub fn check_write(&mut self) -> bool {
        if self._out_active == false || self._state != active {
            return false;
        }

        let full = !self.check_hwm();
        if full {
            self._out_active = false;
            return false;
        }

        return true;
    }

    pub unsafe fn write(&mut self, msg_: *mut msg_t) -> bool {
        if self.check_write() == false {
            return false;
        }

        let more = msg_.flags() & more > 0;

        let is_routing_id = msg_.is_routing_id();

        (*self._out_pipe).write(*msg_, more);
        if (more != 0 && !is_routing_id) {
            self._msgs_written += 1;
        }

        return true;
    }

    pub unsafe fn rollback(&mut self) {
        let mut msg: msg_t;
        if self._out_pipe {
            while self._out_pipe.unwrite(&msg) {
                let rc = msg.close();
            }
        }
    }

    pub unsafe fn flush(&mut self) {
        if self._state == term_ack_sent {
            return;
        }

        if self._out_pipe != null_mut() && (*self._out_pipe).flush() == 0 {
            self._base.send_activate_read(self._peer);
        }
    }

    pub unsafe fn process_activate_read(&mut self) {
        if self._in_active == false && (self._state == active || self._state == waiting_for_delimiter) {
            self._in_active = true;
            self._sink.read_activated(self);
        }
    }

    pub unsafe fn process_activate_write(&mut self, msgs_read_: u64) {
        self._peers_msgs_read = msgs_read_;
        if self._out_active == false && self._state == active {
            self._out_active = true;
            self._sink.write_activated(self);
        }
    }

    pub unsafe fn process_hiccup(&mut self, pipe_: *mut c_void)
    {
        self._out_pipe.flush();
        let mut msg: msg_t = msg_t::new();
        while (*self._out_pipe).read(&mut msg) {
            if msg.flags & more == 0 {
                self._msgs_written -= 1
            }
            msg.close();
        }

        self._out_pipe = pipe_ as *mut upipe_t;
        self._out_active = true;

        if self._state == active {
             self._sink.hiccuped(self)
        }
    }

    pub unsafe fn process_pipe_term(&mut self) {
        if self._state == active {
            if (self._delay) {
                self._state = waiting_for_delimiter;
            } else {
                self._state = term_ack_sent;
                self._out_pipe = null_mut();
                self._base.send_pipe_term_ack(self._peer);
            }
        } else if self._state == delimiter_received {
            self._state = term_ack_sent;
            self._out_pipe = null_mut();
            self._base.send_pipe_term_ack(self._peer);
        } else if self._state == term_req_sent1 {
            self._state = term_req_sent2;
            self._out_pipe = null_mut();
            self._base.send_pipe_term_ack(self._peer);
        }
    }

    pub unsafe fn process_pipe_term_ack(&mut self)
    {
        self._sink.pipe_terminated(self);
        if self._state == term_req_sent1 {
           self._out_pipe = null_mut();
            self._base.send_pipe_term_ack(self._peer);
        } else {

        }

        if !self._conflate {
            let msg = msg_t::new();
            while (*self._in_pipe).read(&msg) {
                msg.close()
            }
        }
    }

    pub unsafe fn process_pipe_hwm(&mut self, inhwm_: i32, outhwm_: i32) {
        self.set_hwms(inhwm_, outhwm_);
    }

    pub unsafe fn set_nodelay(&mut self) {
        self._delay = false;
    }

    pub unsafe fn terminate(&mut self, delay_: bool) {
        self._delay = delay_;

        if self._state == term_req_sent1 || self._state == term_req_sent2 {
            return;
        }

        if self._state == term_ack_sent {
            return;
        }

        if self._state == active {
            self._base.send_pipe_term(self._peer);
            self._state = term_req_sent1;
        }
        else if self._state == waiting_for_delimiter && self._delay == false {
            self.rollback();
            self._out_pipe = null_mut();
            self._base.send_pipe_term_ack(self._peer);
            self._state = term_ack_sent;
        }
        else if self._state == waiting_for_delimiter {}
        else if self._state == delimiter_received {
            self._base.send_pipe_term(self._peer);
            self._state = term_req_sent1;
        }
        else {

        }
        self._out_active = false;
        if self._out_pipe {
            self.rollback();
            let mut msg = msg_t::new();
            msg.init_delimiter();
            (*self._out_pipe).write(&mut msg, false);
            self.flush()
        }
    }

    pub unsafe fn is_delimiter(&mut self, msg_: &mut msg_t) -> bool {
        msg_.is_delimiter()
    }

    pub fn compute_lwm(&mut self, hwm_: i32) -> i32 {
        (hwm_ + 1) / 2
    }

    pub unsafe fn process_delimiter(&mut self) {
        if self._state == active {
            self._state = delimiter_received;
        } else {
            self.rollback();
            self._out_pipe = null_mut();
            self._base.send_pipe_term_ack(self._peer);
            self._state = term_ack_sent;
        }
    }

    pub unsafe fn hiccup(&mut self) {
        if self._state != active {
            return;
        }
        if self._conflate == true {
             self._in_pipe: ypipe_conflate_t<msg_t> = ypipe_conflate_t::new()
        } else {
             self._in_pipe: ypipe_t<msg_t, message_pipe_granularity> = ypipe_t::new()
        };
        self._in_active = true;
        self._base.send_hiccup(self._peer, self._in_pipe);
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

    pub fn send_hwms_to_peer(&mut self, inhwm_: i32, outhwm_: i32) {
        self._base.send_pipe_hwm(self._peer, inhwm_, outhwm_);
    }

    pub fn set_endpoint_pair(&mut self, endpoint_pair_: &mut endpoint_uri_pair_t)
    {
        self._endpoint_pair = endpoint_pair_;
    }

    pub fn send_stats_to_peer(&mut self, socket_base_: *mut own_t)
    {
        let mut ep = endpoint_uri_pair_t::new3(&mut self._endpoint_pair);
        self._base.send_pipe_peer_stats(self._peer, self._msgs_written - self._peers_msgs_read, socket_base_, ep);
    }

    pub fn process_pipe_peer_stats(&mut self, queue_count_: u64, socket_base_: *mut own_t, endpoint_pair_: *mut endpoint_uri_pair_t){
        self._base.send_pipe_stats_publish(socket_base_, queue_count_, self._msgs_written - self._peers_msgs_read, endpoint_pair_);
    }

    pub unsafe fn send_disconnect_msg(&mut self) {
        if self._disconnect_msg.size() > 0 && self._out_pipe != null_mut() {
            self.rollback();
            (*self._out_pipe).write(self._disconnect_msg, false);
            self.flush();
            self._disconnect_msg.init2()
        }
    }

    pub unsafe fn set_disconnect_msg2(&mut self, disconnect: &mut Vec<u8>) {
        self._disconnect_msg.close();
        let rc = self._disconnect_msg.init_buffer(disconnect.as_mut_ptr() as *const c_void, disconnect.len());
    }

    pub unsafe fn send_hiccup_msg(&mut self, hiccup: &mut Vec<u8>) {
        if hiccup.is_empty() == false && self._out_pipe != null_mut() {
            let mut msg: msg_t = msg_t::new();
            let rc = msg.init_buffer(hiccup.as_mut_ptr() as *const c_void, hiccup.len());
            (*self._out_pipe).write(&mut msg, false);
            self.flush();
        }
    }

}

type upipe_normal_t = ypipe_t<msg_t, message_pipe_granularity>;
type upipe_conflate_t = ypipe_conflate_t<msg_t>;


pub unsafe fn pipepair(parents_: [*mut object_t; 2],
                pipes_: &mut [*mut pipe_t; 2],
                hwms_: [i32; 2],
                conflate_: [bool; 2],
) -> i32 {
    let mut upipe1: *mut upipe_t;
    if conflate_[0] == true {
        upipe1 = &mut upipe_conflate_t::new() as *mut upipe_t;
    } else {
        upipe1 = &mut upipe_normal_t::new() as *mut upipe_t;
    }

    let upipe2: *mut upipe_t;
    if conflate_[1] == true {
        upipe2 = &mut upipe_conflate_t::new() as *mut upipe_t;
    } else {
        upipe2 = &mut upipe_normal_t::new() as *mut upipe_t;
    }

    pipes_[0] = &mut pipe_t::new(parents_[0], upipe1, upipe2, hwms_[1], hwms_[0], conflate_[0]) as *mut pipe_t;
    pipes_[1] = &mut pipe_t::new(parents_[1], upipe2, upipe1, hwms_[0], hwms_[1], conflate_[1]) as *mut pipe_t;

    pipes_[0].set_peer(pipes_[1]);
    pipes_[1].set_peer(pipes_[0]);

    return 0;
}

pub unsafe fn send_routing_id(pipe_: *mut pipe_t, options_: &options_t) {
    let mut id = msg_t::new();
    let mut rc = id.init_size(options_.routing_id_size);
    libc::memcpy(id.data(), &options_.routing_id as *const c_void, options_.routing_id_size as size_t);
    id.set_flags(routing_id);
    let mut written = (*pipe_).write(&mut id);
    (*pipe_).flush();
}

pub unsafe fn send_hello_msg(pipe_: *mut pipe_t, options_: &options_t)
{
    let mut hello_msg = msg_t::new();
    let rc = hello_msg.init_buffer(&options_.hello_msg[0], options_.hello_msg.size());
    let written = (*pipe_).write(&mut hello_msg);
}
