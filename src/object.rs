use std::ffi::c_void;
use std::ptr::null_mut;
use crate::command::{command_t, type_t};
use crate::command::type_t::{activate_read, stop};
use crate::ctx::{ctx_t, endpoint_t};
use crate::endpoint::endpoint_uri_pair_t;
use crate::i_engine::i_engine;
use crate::io_thread::io_thread_t;
use crate::options::options_t;
use crate::own::own_t;
use crate::pipe::pipe_t;
use crate::socket_base::socket_base_t;

pub struct object_t<'a> {
    pub _ctx: &'a mut ctx_t<'a>,
    pub _tid: u32,
}

impl object_t {
    pub fn new(ctx_: &mut ctx_t, tid_: u32) -> Self {
        Self { _ctx: ctx_, _tid: tid_ }
    }

    pub unsafe fn new2(parent: &mut Self) -> Self {
        Self { _ctx: (*parent)._ctx, _tid: (*parent)._tid }
    }

    pub fn get_tid(&self) -> u32 {
        self._tid
    }

    pub fn set_tid(&mut self, id_: u32) {
        self._tid = id_;
    }

    pub unsafe fn process_command(&mut self, cmd_: command_t) {
        match cmd_.type_ {
            activate_read => {
                self.process_activate_read();
            }
            // break;

            activate_write => {
                self.process_activate_write(cmd_.args.activate_write.msgs_read);
            }
            // break;

            stop => {
                self.process_stop();
            }
            // break;

            plug => {
                self.process_plug();
                self.process_seqnum();
            }
            // break;

            own => {
                self.process_own(cmd_.args.own.object);
                self.process_seqnum();
            }
            // break;

            attach => {
                self.process_attach(cmd_.args.attach.engine);
                self.process_seqnum();
            }
            // break;

            bind => {
                self.process_bind(cmd_.args.bind.pipe);
                self.process_seqnum();
            }
            // break;

            hiccup => {
                self.process_hiccup(cmd_.args.hiccup.pipe);
            }
            // break;

            pipe_peer_stats => {
                self.process_pipe_peer_stats(cmd_.args.pipe_peer_stats.queue_count,
                                             cmd_.args.pipe_peer_stats.socket_base,
                                             cmd_.args.pipe_peer_stats.endpoint_pair);
            }
            // break;

            pipe_stats_publish => {
                self.process_pipe_stats_publish(
                    cmd_.args.pipe_stats_publish.outbound_queue_count,
                    cmd_.args.pipe_stats_publish.inbound_queue_count,
                    cmd_.args.pipe_stats_publish.endpoint_pair);
            }
            // break;

            pipe_term => {
                self.process_pipe_term();
            }
            // break;

            pipe_term_ack => {
                self.process_pipe_term_ack();
            }
            // break;

            pipe_hwm => {
                self.process_pipe_hwm(cmd_.args.pipe_hwm.inhwm,
                                      cmd_.args.pipe_hwm.outhwm);
            }
            // break;

            term_req => {
                self.process_term_req(cmd_.args.term_req.object);
            }
            // break;

            term => {
                self.process_term(cmd_.args.term.linger);
            }
            // break;

            term_ack => {
                self.process_term_ack();
            }
            // break;

            term_endpoint => {
                self.process_term_endpoint(&cmd_.args.term_endpoint.endpoint);
            }
            // break;

            reap => {
                self.process_reap(cmd_.args.reap.socket);
            }
            // break;

            reaped => {
                self.process_reaped();
            }
            // break;

            inproc_connected => {
                self.process_seqnum();
            }
            // break;

            conn_failed => {
                self.process_conn_failed();
            }

            done => {}
            // default:
            //     zmq_assert (false);
        }
    }

    pub fn register_endpoint(&mut self, addr_: &str, endpoint_: &endpoint_t, options: &mut options_t) -> i32 {
        self._ctx.register_endpoint(addr_, endpoint_, options)
    }

    pub fn unregister_endpoint(&mut self, addr_: &str, socket_: &mut socket_base_t) {
        self._ctx.unregister_endpoint(addr_);
    }

    pub fn unregister_endpoints(&mut self, socket_: &mut socket_base_t) {
        self._ctx.unregister_endpoints(socket_);
    }

    pub fn find_endpoint(&mut self, addr_: &str) -> &mut endpoint_t {
        self._ctx.find_endpoint(addr_) as &mut endpoint_t
    }

    pub unsafe fn pend_connection(&mut self,
                                  addr_: &str,
                                  endpoint_: &endpoint_t,
                                  pipes_: &mut [&mut pipe_t]) {
        self._ctx.pend_connection(addr_, endpoint_, pipes_);
    }

    pub unsafe fn connect_pending(&mut self, addr_: &str, bind_socket_: *mut socket_base_t) {
        self._ctx.connect_pending(addr_, bind_socket_);
    }

    pub unsafe fn destroy_socket(&mut self, socket_: *mut socket_base_t) {
        self._ctx.destroy_socket(socket_);
    }

    pub unsafe fn choose_io_thread(&mut self, affinity_: u64) -> *mut io_thread_t {
        self._ctx.choose_io_thread(affinity_)
    }

    pub fn send_stop(&mut self) {
        let mut cmd = command_t::new();
        cmd.destination = self;
        cmd.type_ = type_t::stop;
        self._ctx.send_command(self._tid, &mut cmd);
    }

    pub unsafe fn send_plug(&mut self, destination_: *mut own_t, inc_seqnum_: bool) {
        if (inc_seqnum_) {
            destination_.inc_seqnum();
        }

        let mut cmd = command_t::new();
        cmd.destination = destination_;
        cmd.type_ = type_t::plug;
        self.send_command(&cmd);
    }

    pub unsafe fn send_own(&mut self, destination_: *mut own_t, object_: *mut own_t) {
        destination_.inc_seqnum();
        let mut cmd = command_t::new();
        cmd.destination = destination_;
        cmd.type_ = type_t::own;
        cmd.args.own.object = object_;
        self.send_command(&mut cmd);
    }

    pub unsafe fn send_attach(&mut self, destination_: *mut session_base_t, engine_: *mut dyn i_engine, inc_seqnum_: bool) {
        if (inc_seqnum_) {
            destination_.inc_seqnum();
        }

        let mut cmd = command_t::new();
        cmd.destination = destination_;
        cmd.type_ = type_t::attach;
        cmd.args.attach.engine = engine_;
        self.send_command(&mut cmd);
    }

    pub unsafe fn send_conn_failed(&mut self, destination_: *mut session_base_t) {
        let mut cmd = command_t::new();
        cmd.destination = destination_;
        cmd.type_ = type_t::conn_failed;
        self.send_command(&cmd);
    }

    pub unsafe fn send_bind(&mut self, destination_: *mut own_t,
                            pipe_: *mut pipe_t,
                            inc_seqnum_: bool) {
        if (inc_seqnum_) {
            destination_.inc_seqnum();
        }

        let mut cmd = command_t::new();
        cmd.destination = destination_;
        cmd.type_ = type_t::bind;
        cmd.args.bind.pipe = pipe_;
        self.send_command(&cmd);
    }

    pub unsafe fn send_activate_read(&mut self, destination_: &mut pipe_t) {
        let mut cmd = command_t::new();
        cmd.destination = destination_;
        cmd.type_ = type_t::activate_read;
        self.send_command(&cmd);
    }

    pub unsafe fn send_activate_write(&mut self, destination_: &mut pipe_t,
                                      msgs_read_: u64) {
        let mut cmd = command_t::new();
        cmd.destination = destination_;
        cmd.type_ = type_t::activate_write;
        cmd.args.activate_write.msgs_read = msgs_read_;
        self.send_command(&cmd);
    }

    pub unsafe fn send_hiccup(&mut self, destination_: &mut pipe_t, pipe_: *mut c_void) {
        let mut cmd = command_t::new();
        cmd.destination = destination_;
        cmd.type_ = type_t::hiccup;
        cmd.args.hiccup.pipe = pipe_;
        self.send_command(&cmd);
    }

    pub unsafe fn send_pipe_peer_stats(&mut self,
                                       destination_: *mut pipe_t,
                                       queue_count_: u64,
                                       socket_base_: *mut own_t,
                                       endpoint_pair_: *mut endpoint_uri_pair_t) {
        let mut cmd = command_t::new();
        cmd.destination = destination_;
        cmd.type_ = type_t::pipe_peer_stats;
        cmd.args.pipe_peer_stats.queue_count = queue_count_;
        cmd.args.pipe_peer_stats.socket_base = socket_base_;
        cmd.args.pipe_peer_stats.endpoint_pair = endpoint_pair_;
        self.send_command(&cmd);
    }

    pub unsafe fn send_pipe_stats_publish(&mut self,
        destination_: *mut own_t,
        outbound_queue_count_: u64,
        inbound_queue_count_: u64,
        endpoint_pair_: *mut endpoint_uri_pair_t) {
        let mut cmd = command_t::new();
        cmd.destination = destination_;
        cmd.type_ = type_t::pipe_stats_publish;
        cmd.args.pipe_stats_publish.outbound_queue_count = outbound_queue_count_;
        cmd.args.pipe_stats_publish.inbound_queue_count = inbound_queue_count_;
        cmd.args.pipe_stats_publish.endpoint_pair = endpoint_pair_;
        self.send_command(&cmd);
    }

    pub unsafe fn send_pipe_term (&mut self, destination_: *mut pipe_t)
    {
        let mut cmd = command_t::new();
        cmd.destination = destination_;
        cmd.type_ = type_t::pipe_term;
        self.send_command (&cmd);
    }

    pub unsafe fn send_pipe_term_ack (&mut self, destination_: &mut pipe_t)
    {
        let mut cmd = command_t::new();
        cmd.destination = destination_;
        cmd.type_ = type_t::pipe_term_ack;
        self.send_command (&cmd);
    }

    pub unsafe fn send_term_endpoint (&mut self, destination_: &mut own_t,
                                        endpoint_: &str)
    {
        let mut cmd = command_t::new();
        cmd.destination = destination_;
        cmd.type_ = type_t::term_endpoint;
        cmd.args.term_endpoint.endpoint = endpoint_.to_string();
        self.send_command (&cmd);
    }

    pub unsafe fn send_reap (&mut self, socket_: *mut socket_base_t)
    {
        let mut cmd = command_t::new();
        cmd.destination = self._ctx.get_reaper ();
        cmd.type_ = type_t::reap;
        cmd.args.reap.socket = socket_;
        self.send_command (&cmd);
    }

    pub unsafe fn send_reaped (&mut self)
    {
        let mut cmd = command_t::new();
        cmd.destination = self._ctx.get_reaper ();
        cmd.type_ = type_t::reaped;
        self.send_command (&cmd);
    }

    pub unsafe fn send_inproc_connected (&mut self, socket_: *mut socket_base_t)
    {
        let mut cmd = command_t::new();
        cmd.destination = socket_;
        cmd.type_ = type_t::inproc_connected;
        self.send_command (&cmd);
    }

    pub unsafe fn send_done (&mut self)
    {
        let mut cmd = command_t::new();
        cmd.destination = null_mut();
        cmd.type_ = type_t::done;
        self._ctx.send_command (ctx_t::term_tid, &mut cmd);
    }

    pub unsafe fn send_command (&mut self, cmd_: &command_t)
    {
        self._ctx.send_command (cmd_.destination.get_tid (), cmd_);
    }
}

impl object_ops for object_t {
    fn process_stop(&mut self) {
        todo!()
    }

    fn process_plug(&mut self) {
        todo!()
    }

    fn process_own(&mut self, object_: *mut own_t) {
        todo!()
    }

    fn process_attach(&mut self, engine_: *mut dyn i_engine) {
        todo!()
    }

    fn process_bind(&mut self, pipe_: *mut pipe_t) {
        todo!()
    }

    fn process_activate_read(&mut self) {
        todo!()
    }

    fn process_activate_write(&mut self, msgs_read_: u64) {
        todo!()
    }

    fn process_hiccup(&mut self, pipe_: *mut c_void) {
        todo!()
    }

    fn process_pipe_peer_stats(&mut self, queue_count_: u64, socket_base: *mut own_t, endpoint_pair_: *mut endpoint_uri_pair_t) {
        todo!()
    }

    fn process_pipe_stats_publish(&mut self, outbound_queue_count_: u64, inbound_queue_count: u64, endpoint_pair_: *mut endpoint_uri_pair_t) {
        todo!()
    }

    fn process_pipe_term(&mut self) {
        todo!()
    }

    fn process_pipe_term_ack(&mut self) {
        todo!()
    }

    fn process_pipe_hwm(&mut self, inhwm_: i32, outhwm_: i32) {
        todo!()
    }

    fn process_pipe_term_req(&mut self, object_: *mut own_t) {
        todo!()
    }

    fn process_term(&mut self, linger_: i32) {
        todo!()
    }

    fn process_term_ack(&mut self) {
        todo!()
    }

    fn process_term_endpoint(&mut self, endpoint_: &str) {
        todo!()
    }

    fn process_reap(&mut self, socket_: *mut socket_base_t) {
        todo!()
    }

    fn process_reaped(&mut self) {
        todo!()
    }

    fn process_conn_failed(&mut self) {
        todo!()
    }

    fn process_seqnum(&mut self) {
        todo!()
    }
}

pub trait object_ops {
    fn process_stop(&mut self);
    fn process_plug(&mut self);
    fn process_own(&mut self, object_: *mut own_t);

    fn process_attach(&mut self, engine_: *mut dyn i_engine);

    fn process_bind(&mut self, pipe_: *mut pipe_t);

    fn process_activate_read(&mut self);

    fn process_activate_write(&mut self, msgs_read_: u64);

    fn process_hiccup(&mut self, pipe_: *mut c_void);

    fn process_pipe_peer_stats(&mut self, queue_count_: u64, socket_base: *mut own_t, endpoint_pair_: *mut endpoint_uri_pair_t);

    fn process_pipe_stats_publish(&mut self, outbound_queue_count_: u64, inbound_queue_count: u64, endpoint_pair_: *mut endpoint_uri_pair_t);

    fn process_pipe_term(&mut self);

    fn process_pipe_term_ack(&mut self);

    fn process_pipe_hwm(&mut self, inhwm_: i32, outhwm_: i32);

    fn process_pipe_term_req(&mut self, object_: *mut own_t);

    fn process_term(&mut self, linger_: i32);

    fn process_term_ack(&mut self);

    fn process_term_endpoint(&mut self, endpoint_: &str);

    fn process_reap(&mut self, socket_: *mut socket_base_t);

    fn process_reaped(&mut self);

    fn process_conn_failed(&mut self);

    fn process_seqnum(&mut self);
}
