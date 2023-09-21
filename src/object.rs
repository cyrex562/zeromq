use crate::command::{command_t, type_t};
use crate::ctx::{ctx_t, endpoint_t};
use crate::endpoint::endpoint_uri_pair_t;
use crate::i_engine::i_engine;
use crate::io_thread::io_thread_t;
use crate::own::own_t;
use crate::pipe::pipe_t;
use crate::session_base::session_base_t;
use crate::socket_base::socket_base_t;
use std::ffi::{c_char, c_void};

pub struct object_t {
    pub _ctx: *const ctx_t,
    pub _tid: u32,
}

impl object_t {
    pub fn new(ctx_: *mut ctx_t, tid_: u32) -> Self {
        Self {
            _ctx: ctx_,
            _tid: tid_,
        }
    }

    pub unsafe fn new2(parent: *mut Self) -> Self {
        Self {
            _ctx: (*parent)._ctx,
            _tid: (*parent)._tid,
        }
    }

    pub fn get_tid(&self) -> u32 {
        self._tid
    }

    pub fn set_tid(&mut self, id_: u32) {
        self._tid = id_;
    }

    pub fn process_command(&mut self, cmd_: &command_t) {
        match cmd_.type_ {
            type_t::stop => {}
            type_t::plug => {}
            type_t::own => {}
            type_t::attach => {}
            type_t::bind => {}
            type_t::activate_read => {}
            type_t::activate_write => {}
            type_t::hiccup => {}
            type_t::pipe_term => {}
            type_t::pipe_term_ack => {}
            type_t::pipe_hwm => {}
            type_t::term_req => {}
            type_t::term => {}
            type_t::term_ack => {}
            type_t::term_endpoint => {}
            type_t::reap => {}
            type_t::reaped => {}
            type_t::inproc_connected => {}
            type_t::conn_failed => {}
            type_t::pipe_peer_stats => {}
            type_t::pipe_stats_publish => {}
            type_t::done => {}
        }
    }

    pub fn register_endpoint(&mut self, addr_: *mut c_char, endpoint_: *mut endpoint_t) -> i32 {
        let ctx = unsafe { &mut *(self._ctx as *mut ctx_t) };
        ctx.register_endpoint(addr_, endpoint_)
    }

    pub fn unregister_endpoint(&mut self, addr_: &mut String, socket_: *mut socket_base_t) -> i32 {
        let ctx = unsafe { &mut *(self._ctx as *mut ctx_t) };
        ctx.unregister_endpoint(addr_, socket_)
    }

    pub fn unregister_endpoints(&mut self, socket_: *mut socket_base_t) -> i32 {
        let ctx = unsafe { &mut *(self._ctx as *mut ctx_t) };
        ctx.unregister_endpoints(socket_)
    }

    pub fn find_endpoint(&mut self, addr_: *mut c_char) -> endpoint_t {
        let ctx = unsafe { &mut *(self._ctx as *mut ctx_t) };
        ctx.find_endpoint(addr_)
    }

    pub fn pend_connection(
        &mut self,
        addr_: &mut String,
        endpoint: &mut endpoint_t,
        pipes_: *mut *mut pipe_t,
    ) {
        self._ctx.pend_connection(addr_, endpoint, pipes_)
    }

    pub fn connect_pending(&mut self, addr_: *const c_char, bind_socket: *mut socket_base_t) {
        self._ctx.connect_pending(addr_, bind_socket)
    }

    pub fn destroy_socket(&mut self, socket_: *mut socket_base_t) {
        self._ctx.destroy_socket(socket_)
    }

    pub fn choose_io_thread(&mut self, affinity_: u64) -> *mut io_thread_t {
        self._ctx.choose_io_thread(affinity_)
    }

    pub fn send_command(&mut self, cmd_: &mut command_t) {
        self._ctx.send_comand(cmd_.destination.get_tid(), cmd_);
    }

    pub fn send_stop(&mut self) {
        let mut cmd = command_t::new();
        cmd.destination = self;
        cmd.type_ = type_t::stop;
        self._ctx.send_command(&mut cmd);
    }

    pub fn send_plug(&mut self, destination_: *mut own_t, inc_seqnum_: bool) {
        if inc_seqnum_ {
            destination_.inc_seqnum();
        }

        let mut cmd = command_t::new();
        cmd.destination = destination_ as *mut object_t;
        cmd.type_ = type_t::plug;
        self.send_command(&mut cmd);
    }

    pub unsafe fn send_attach(
        &mut self,
        destination_: *mut session_base_t,
        engine_: *mut dyn i_engine,
        inc_seqnum_: bool,
    ) {
        if inc_seqnum_ {
            destination_.inc_seqnum();
        }

        let mut cmd = command_t::new();
        cmd.destination = destination_ as *mut object_t;
        cmd.args.attach.engine = engine_;
        cmd.type_ = type_t::attach;
        self.send_command(&mut cmd);
    }

    pub fn send_conn_failed(&mut self, destination_: *mut session_base_t) {
        let mut cmd = command_t::new();
        cmd.destination = destination_ as *mut object_t;
        cmd.type_ = type_t::conn_failed;
        self.send_command(&mut cmd);
    }

    pub unsafe fn send_bind(
        &mut self,
        destination_: *mut own_t,
        pipe_: *mut pipe_t,
        inc_seqnum_: bool,
    ) {
        if inc_seqnum_ {
            destination_.inc_seqnum();
        }

        let mut cmd = command_t::new();
        cmd.destination = destination_ as *mut object_t;
        cmd.args.bind.pipe = pipe_;
        cmd.type_ = type_t::bind;
        self.send_command(&mut cmd);
    }

    pub fn send_activate_read(&mut self, destination_: *mut own_t) {
        let mut cmd = command_t::new();
        cmd.destination = destination_ as *mut object_t;
        cmd.type_ = type_t::activate_read;
        self.send_command(&mut cmd);
    }

    pub unsafe fn send_activate_write(&mut self, destination_: *mut pipe_t, msgs_read_: u64) {
        let mut cmd = command_t::new();
        cmd.destination = destination_ as *mut object_t;
        cmd.args.activate_write.msgs_read = msgs_read_;
        cmd.type_ = type_t::activate_write;
        self.send_command(&mut cmd);
    }

    pub unsafe fn send_hiccup(&mut self, destination_: *mut pipe_t, pipe_: *mut c_void) {
        let mut cmd = command_t::new();
        cmd.destination = destination_ as *mut object_t;
        cmd.args.hiccup.pipe = pipe_;
        cmd.type_ = type_t::hiccup;
        self.send_command(&mut cmd);
    }

    pub unsafe fn send_pipe_peer_stats(
        &mut self,
        destination_: *mut pipe_t,
        queue_count: u64,
        socket_base_: *mut own_t,
        endpoint_pair_: *mut endpoint_uri_pair_t,
    ) {
        let mut cmd = command_t::new();
        cmd.destination = destination_ as *mut object_t;
        cmd.args.pipe_peer_stats.queue_count = queue_count;
        cmd.args.pipe_peer_stats.socket_base = socket_base_;
        cmd.args.pipe_peer_stats.endpoint_pair = endpoint_pair_;
        cmd.type_ = type_t::pipe_peer_stats;
        self.send_command(&mut cmd);
    }

    pub unsafe fn send_pipe_stats_publish(
        &mut self,
        destination_: *mut own_t,
        outbound_queue_count_: u64,
        inbound_queue_count_: u64,
        endpoint_pair_: endpoint_uri_pair_t,
    ) {
        let mut cmd = command_t::new();
        cmd.destination = destination_ as *mut object_t;
        cmd.args.pipe_stats_publish.outbound_queue_count = outbound_queue_count_;
        cmd.args.pipe_stats_publish.inbound_queue_count = inbound_queue_count_;
        cmd.type_ = type_t::pipe_stats_publish;
        self.send_command(&mut cmd);
    }

    pub fn send_pipe_term(&mut self, destination_: *mut pipe_t) {
        let mut cmd = command_t::new();
        cmd.destination = destination_ as *mut object_t;
        cmd.type_ = type_t::pipe_term;
        self.send_command(&mut cmd);
    }

    pub fn send__pipe_term_ack(&mut self, destination_: *mut pipe_t) {
        let mut cmd = command_t::new();
        cmd.destination = destination_ as *mut object_t;
        cmd.type_ = type_t::pipe_term_ack;
        self.send_command(&mut cmd);
    }

    pub unsafe fn send_pipe_hwm(&mut self, destination_: *mut pipe_t, inhwm_: i32, outhwm_: i32) {
        let mut cmd = command_t::new();
        cmd.destination = destination_ as *mut object_t;
        cmd.args.pipe_hwm.inhwm = inhwm_;
        cmd.args.pipe_hwm.outhwm = outhwm_;
        cmd.type_ = type_t::pipe_hwm;
        self.send_command(&mut cmd);
    }

    pub unsafe fn send_term_req(&mut self, destination_: *mut own_t, object_: *mut own_t) {
        let mut cmd = command_t::new();
        cmd.destination = destination_ as *mut object_t;
        cmd.args.term_req.object = object_;
        cmd.type_ = type_t::term_req;
        self.send_command(&mut cmd);
    }

    pub unsafe fn send_term(&mut self, destination_: *mut own_t, linger_: i32) {
        let mut cmd = command_t::new();
        cmd.destination = destination_ as *mut object_t;
        cmd.args.term.linger = linger_;
        cmd.type_ = type_t::term;
        self.send_command(&mut cmd);
    }

    pub fn send_term_ack(&mut self, destination_: *mut own_t) {
        let mut cmd = command_t::new();
        cmd.destination = destination_ as *mut object_t;
        cmd.type_ = type_t::term_ack;
        self.send_command(&mut cmd);
    }

    pub unsafe fn send_term_endpoint(&mut self, destination_: *mut own_t, endpoint_: *mut String) {
        let mut cmd = command_t::new();
        cmd.destination = destination_ as *mut object_t;
        cmd.args.term_endpoint.endpoint = endpoint_;
        cmd.type_ = type_t::term_endpoint;
        self.send_command(&mut cmd);
    }

    pub unsafe fn send_reap(&mut self, socket_: *mut socket_base_t) {
        let mut cmd = command_t::new();
        cmd.destination = self._ctx.get_reaper();
        cmd.args.reap.socket = socket_;
        cmd.type_ = type_t::reap;
        self.send_command(&mut cmd);
    }

    pub unsafe fn send_reaped(&mut self) {
        let mut cmd = command_t::new();
        cmd.destination = self._ctx.get_reaper();
        cmd.type_ = type_t::reaped;
        self.send_command(&mut cmd);
    }

    pub fn send_inproc_connected(&mut self, socket_: *mut socket_base_t) {
        let mut cmd = command_t::new();
        cmd.destination = socket_;
        cmd.type_ = type_t::inproc_connected;
        self.send_command(&mut cmd);
    }

    pub fn send_done(&mut self) {
        let mut cmd = command_t::new();
        cmd.type_ = type_t::done;
        self.send_command(&mut cmd);
    }

    pub fn process_stop(&mut self) {
        unimplemented!()
    }

    pub fn process_plug(&mut self) {
        unimplemented!()
    }

    pub fn process_own(&mut self, own: *mut own_t) {
        unimplemented!()
    }

    pub fn process_attach(&mut self, engine: *mut dyn i_engine) {
        unimplemented!()
    }

    pub fn process_bind(&mut self, pipe: *mut pipe_t) {
        unimplemented!()
    }

    pub fn process_activate_read(&mut self) {
        unimplemented!()
    }

    pub fn process_activate_write(&mut self, val: u64) {
        unimplemented!()
    }

    pub fn process_hiccup(&mut self, val: *mut c_void) {
        unimplemented!()
    }

    pub fn process_pipe_peer_stats(
        &mut self,
        val: u64,
        own: *mut own_t,
        pair: *mut endpoint_uri_pair_t,
    ) {
        unimplemented!()
    }

    pub fn process_pipe_stats_publish(&mut self, a: u64, b: u64, par: *mut endpoint_uri_pair_t) {
        unimplemented!()
    }

    pub fn process_pipe_term(&mut self) {
        unimplemented!()
    }

    pub fn process_pipe_term_ack(&mut self) {
        unimplemented!()
    }

    pub fn process_pipe_hwm(&mut self, a: i32, b: i32) {
        unimplemented!()
    }

    pub fn process_term_req(&mut self, own: *mut own_t) {
        unimplemented!()
    }

    pub fn process_term(&mut self, linger: i32) {
        unimplemented!()
    }

    pub fn process_term_ack(&mut self) {
        unimplemented!()
    }

    pub fn process_term_endpoint(&mut self, endpoint: *mut String) {
        unimplemented!()
    }

    pub fn process_reap(&mut self, socket: *mut socket_base_t) {
        unimplemented!()
    }

    pub fn process_reaped(&mut self) {
        unimplemented!()
    }

    pub fn process_seqnum(&mut self) {
        unimplemented!()
    }

    pub fn process_conn_failed(&mut self) {
        unimplemented!()
    }
}
