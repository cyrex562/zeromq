use crate::command::{CommandType, ZmqCommand};
use crate::context::ZmqContext;
use crate::endpoint::{EndpointUriPair, ZmqEndpoint};
use crate::io_thread::ZmqThread;
use crate::own::ZmqOwn;
use crate::pipe::ZmqPipe;
use crate::session_base::ZmqSessionBase;
use crate::socket_base::ZmqSocketBase;
use anyhow::anyhow;
use std::ptr::null_mut;

// #[derive(Default,Debug,Clone)]
// pub struct object_t {
//     //  Context provides access to the global state.
//     // ZmqContext *const _ctx;
//     ctx: *const ZmqContext,
//
//     //  Thread ID of the thread the object belongs to.
//     // uint32_t _tid;
//     tid: u32,
//
//     // ZMQ_NON_COPYABLE_NOR_MOVABLE (object_t)
// }

pub trait ZmqObject {
    //  Context provides access to the global state.
    fn get_ctx(&self) -> &ZmqContext;
    fn get_ctx_mut(&mut self) -> &mut ZmqContext;
    fn set_ctx(&mut self, ctx: &mut ZmqContext);
    //  Thread ID of the thread the object belongs to.
    fn get_tid(&self) -> u32;
    fn set_tid(&mut self, tid: u32);
    fn process_command(&mut self, cmd: &ZmqCommand) {
        match cmd.cmd_type {
            CommandType::stop => {}
            CommandType::plug => {}
            CommandType::own => {}
            CommandType::attach => {}
            CommandType::bind => {}
            CommandType::activate_read => {}
            CommandType::activate_write => {}
            CommandType::hiccup => {}
            CommandType::pipe_term => {}
            CommandType::pipe_term_ack => {}
            CommandType::pipe_hwm => {}
            CommandType::term_req => {}
            CommandType::term => {}
            CommandType::term_ack => {}
            CommandType::term_endpoint => {}
            CommandType::reap => {}
            CommandType::reaped => {}
            CommandType::inproc_connected => {}
            CommandType::conn_failed => {}
            CommandType::pipe_peer_stats => {}
            CommandType::pipe_stats_publish => {}
            CommandType::done => {}
        }
    }

    //  Using following function, socket is able to access global
    //  repository of inproc endpoints.
    // int register_endpoint (addr_: *const c_char, const endpoint_t &endpoint_);
    fn register_endpoint(&mut self, addr: &str, endpoint: &mut ZmqEndpoint) -> anyhow::Result<()> {
        self.get_ctx().register_endpoint(addr, endpoint)
    }

    // int unregister_endpoint (const std::string &addr_, ZmqSocketBase *socket_);
    fn unregister_endpoint(
        &mut self,
        addr: &str,
        sock_base: &mut ZmqSocketBase,
    ) -> anyhow::Result<()> {
        return self.get_ctx().unregister_endpoint(addr, sock_base);
    }

    // void unregister_endpoints (ZmqSocketBase *socket_);
    fn unregister_endpoints(&mut self, sock_base: &mut ZmqSocketBase) {
        self.get_ctx().unregister_endpoints(sock_base);
    }

    // endpoint_t find_endpoint (addr_: *const c_char) const;
    fn find_endpoint(&self, addr: &str) -> Option<ZmqEndpoint> {
        return self.get_ctx().find_endpoint(addr);
    }

    // void pend_connection (const std::string &addr_,
    //                       const endpoint_t &endpoint_,
    //                       ZmqPipe **pipes_);
    fn pend_connection(&mut self, addr: &str, endpoint: &ZmqEndpoint, pipes: &[ZmqPipe]) {
        self.get_ctx().pend_connection(addr, endpoint, pipes);
    }

    // void connect_pending (addr_: *const c_char, ZmqSocketBase *bind_socket_);
    fn connect_pending(&self, addr: &str, bind_socket: &mut ZmqSocketBase) {
        self.get_ctx().connect_pending(addr, bind_socket);
    }

    // void destroy_socket (ZmqSocketBase *socket_);
    fn destroy_socket(&mut self, socket: &mut ZmqSocketBase) {
        // unimplemented!()
        self.get_ctx().destroy_socket(socket);
    }

    //  Logs an message.
    // void log (format_: *const c_char, ...);
    fn log(msg: &str) {
        unimplemented!()
    }

    // void send_inproc_connected (ZmqSocketBase *socket_);
    fn send_inproc_connected(&mut self, socket: &mut ZmqSocketBase) {
        // ZmqCommand cmd;
        let mut cmd = ZmqCommand::default();
        cmd.destination = socket;
        cmd.cmd_type = CommandType::inproc_connected;
        self.send_command(&mut cmd);
    }

    // void send_bind (ZmqOwn *destination_,
    //                 ZmqPipe *pipe_,
    //                 bool inc_seqnum_ = true);
    fn send_bind(&mut self, destination: &mut ZmqOwn, pipe: &mut ZmqPipe, inc_seqnum: bool) {
        if (inc_seqnum) {
            destination.inc_seqnum();
        }

        let mut cmd = ZmqCommand::default();
        cmd.destination = destination;
        cmd.cmd_type = ZmqCommand::bind;
        cmd.args.bind.pipe = pipe.clone();
        self.send_command(&mut cmd);
    }

    //  Chooses least loaded I/O thread.
    // ZmqThread *choose_io_thread (uint64_t affinity_) const;
    fn choose_io_thread(&mut self, affinity: u64) -> Option<ZmqThread> {
        self.get_ctx().choose_io_thread(affinity)
    }

    //  Derived object can use these functions to send commands
    //  to other objects.
    // void send_stop ();
    fn send_stop(&mut self) {
        //  'stop' command goes always from administrative thread to
        //  the current object.
        let mut cmd = ZmqCommand::default();
        cmd.destination = self;
        cmd.cmd_type = ZmqCommand::stop;
        self.get_ctx().send_command(self.get_tid(), &mut cmd);
    }

    // void send_plug (ZmqOwn *destination_, bool inc_seqnum_ = true);
    fn send_plug(&mut self, destination: &mut ZmqOwn, inc_seqnum: bool) {
        if (inc_seqnum_) {
            destination.inc_seqnum();
        }

        let mut cmd = ZmqCommand::default();
        cmd.destination = destination;
        cmd.cmd_type = ZmqCommand::plug;
        self.send_command(&mut cmd);
    }

    // void send_own (ZmqOwn *destination_, ZmqOwn *object_);
    fn send_own(&mut self, destination: &mut ZmqOwn, object: &mut ZmqOwn) {
        destination.inc_seqnum();
        let mut cmd = ZmqCommand::default();
        cmd.destination = destination;
        cmd.cmd_type = ZmqCommand::own;
        cmd.args.own.object = object;
        self.send_command(&mut cmd);
    }

    // void send_attach (ZmqSessionBase *destination_,
    //                   i_engine *engine_,
    //                   bool inc_seqnum_ = true);
    fn send_attach(
        &mut self,
        destination: &mut ZmqSessionbase,
        engine: &mut i_engine,
        inc_seqnum: bool,
    ) {
        if (inc_seqnum_) {
            destination.inc_seqnum();
        }

        let mut cmd = ZmqCommand::default();
        cmd.destination = destination;
        cmd.cmd_type = ZmqCommand::attach;
        cmd.args.attach.engine = engine_;
        self.send_command(&mut cmd);
    }

    // void send_activate_read (ZmqPipe *destination_);
    fn send_activate_read(&mut self, destination: &mut ZmqPipe) {
        let mut cmd = ZmqCommand::default();
        cmd.destination = destination;
        cmd.cmd_type = CommandType::activate_read;
        self.send_command(&mut cmd);
    }

    // void send_activate_write (ZmqPipe *destination_, uint64_t msgs_read_);
    fn send_activate_write(&mut self, destination: &mut ZmqPipe, msgs_read: u64) {
        let mut cmd = ZmqCommand::default();
        cmd.destination = destination;
        cmd.cmd_type = CommandType::activate_write;
        cmd.args.activate_write.msgs_read = msgs_read;
        self.send_command(&mut cmd);
    }

    // void send_hiccup (ZmqPipe *destination_, pipe_: *mut c_void);
    fn send_hiccup(&mut self, destination: &mut ZmqPipe, pipe: &mut [u8]) {
        let mut cmd = ZmqCommand::default();
        cmd.destination = destination;
        cmd.cmd_type = CommandType::hiccup;
        cmd.args.hiccup.pipe = pipe;
        self.send_command(&mut cmd);
    }

    // void send_pipe_peer_stats (ZmqPipe *destination_,
    //                            queue_count_: u64,
    //                            ZmqOwn *socket_base,
    //                            endpoint_uri_pair_t *endpoint_pair_);
    fn send_pipe_peer_stats(
        &mut self,
        destination: &mut ZmqPipe,
        queue_count: u64,
        socket_base: &mut ZmqOwn,
        endpoint_pair: &mut EndpointUriPair,
    ) {
        let mut cmd = ZmqCommand::default();
        cmd.destination = destination;
        cmd.cmd_type = CommandType::pipe_peer_stats;
        cmd.args.pipe_peer_stats.queue_count = queue_count;
        cmd.args.pipe_peer_stats.socket_base = socket_base;
        cmd.args.pipe_peer_stats.endpoint_pair = endpoint_pair;
        self.send_command(&mut cmd);
    }

    // void send_pipe_stats_publish (ZmqOwn *destination_,
    //                               outbound_queue_count_: u64,
    //                               inbound_queue_count_: u64,
    //                               endpoint_uri_pair_t *endpoint_pair_);
    fn send_pipe_stats_publish(
        &mut self,
        destination: &mut ZmqOwn,
        outbound_queue_count: u64,
        inbound_queue_count: u64,
        endpoint_pair: &mut EndpointUriPair,
    ) {
        let mut cmd = ZmqCommand::default();
        cmd.destination = destination;
        cmd.cmd_type = CommandType::pipe_stats_publish;
        cmd.args.pipe_stats_publish.outbound_queue_count = outbound_queue_count;
        cmd.args.pipe_stats_publish.inbound_queue_count = inbound_queue_count;
        cmd.args.pipe_stats_publish.endpoint_pair = endpoint_pair;
        self.send_command(&mut cmd);
    }

    // void send_pipe_term (ZmqPipe *destination_);
    fn send_pipe_term(&mut self, destination: &mut ZmqPipe) {
        let mut cmd = ZmqCommand::default();
        cmd.destination = destination;
        cmd.cmd_type = CommandType::pipe_term;
        self.send_command(&mut cmd);
    }

    // void send_pipe_term_ack (ZmqPipe *destination_);
    fn send_pipe_term_ack(&mut self, destination: &mut ZmqPipe) {
        let mut cmd = ZmqCommand::default();
        cmd.destination = destination;
        cmd.cmd_type = CommandType::pipe_term_ack;
        self.send_command(&mut cmd);
    }

    // void send_pipe_hwm (ZmqPipe *destination_, inhwm_: i32, outhwm_: i32);
    fn send_pipe_hwm(&mut self, destination: &mut ZmqPipe, inhwm: i32, outhwm: i32) {
        let mut cmd = ZmqCommand::default();
        cmd.destination = destination;
        cmd.cmd_type = CommandType::pipe_hwm;
        cmd.args.pipe_hwm.inhwm = inhwm;
        cmd.args.pipe_hwm.outhwm = outhwm;
        self.send_command(&mut cmd);
    }

    // void send_term_req (ZmqOwn *destination_, ZmqOwn *object_);
    fn send_term_req(&mut self, destination: &mut ZmqOwn, object: &mut ZmqOwn) {
        let mut cmd = ZmqCommand::default();
        cmd.destination = destination;
        cmd.cmd_type = CommandType::term_req;
        cmd.args.term_req.object = object;
        self.send_command(&mut cmd);
    }

    // void send_term (ZmqOwn *destination_, linger_: i32);
    fn send_term(&mut self, destination: &mut ZmqOwn, linger: i32) {
        let mut cmd = ZmqCommand::default();
        cmd.destination = destination;
        cmd.cmd_type = CommandType::term;
        cmd.args.term.linger = linger;
        self.send_command(&mut cmd);
    }

    // void send_term_ack (ZmqOwn *destination_);
    fn send_term_ack(&mut self, destination: &mut ZmqOwn) {
        let mut cmd = ZmqCommand::default();
        cmd.destination = destination;
        cmd.cmd_type = CommandType::term_ack;
        self.send_command(&mut cmd);
    }

    // void send_term_endpoint (ZmqOwn *destination_, std::string *endpoint_);
    fn send_term_endpoint(&mut self, destination: &mut ZmqOwn, endpoint: &str) {
        let mut cmd = ZmqCommand::default();
        cmd.destination = destination;
        cmd.cmd_type = CommandType::term_endpoint;
        cmd.args.term_endpoint.endpoint = endpoint.into_string();
        self.send_command(&mut cmd);
    }

    // void send_reap (ZmqSocketBase *socket_);
    fn send_reap(&mut self, socket: &mut ZmqSocketBase) {
        let mut cmd = ZmqCommand::default();
        cmd.destination = self.get_ctx().get_reaper().unwrap();
        cmd.cmd_type = CommandType::reap;
        cmd.args.reap.socket = socket;
        self.send_command(&mut cmd);
    }

    // void send_reaped ();
    fn send_reaped(&mut self) {
        let mut cmd = ZmqCommand::default();
        cmd.destination = self.ctx.get_reaper().unwrap();
        cmd.cmd_type = CommandType::reaped;
        self.send_command(&mut cmd);
    }

    // void send_done ();
    fn send_done(&mut self) {
        let mut cmd = ZmqCommand::default();
        cmd.destination = null_mut();
        cmd.cmd_type = CommandType::done;
        self.ctx.send_command(ZmqContext::TERM_TID, cmd);
    }

    // void send_conn_failed (ZmqSessionBase *destination_);
    fn send_conn_failed(&mut self, destination: &mut ZmqSessionBase) {
        let mut cmd = ZmqCommand::default();
        cmd.destination = destination;
        cmd.cmd_type = CommandType::conn_failed;
        self.send_command(&mut cmd);
    }

    //  These handlers can be overridden by the derived objects. They are
    //  called when command arrives from another thread.
    // virtual void process_stop ();
    fn process_stop(&mut self) {
        unimplemented!()
    }

    // virtual void process_plug ();
    fn process_plug(&mut self) {
        unimplemented!()
    }

    // virtual void process_own (ZmqOwn *object_);
    fn process_own(&mut self, object: &mut ZmqOwn) {
        unimplemented!()
    }

    // virtual void process_attach (i_engine *engine_);
    fn process_attached(&mut self, engine: &mut i_engine) {
        unimplemented!()
    }

    // virtual void process_bind (ZmqPipe *pipe_);
    fn process_bind(&mut self, pipe: &mut ZmqPipe) {
        unimplemented!()
    }

    // virtual void process_activate_read ();
    fn process_activate_read(&mut self) {
        unimplemented!()
    }

    // virtual void process_activate_write (uint64_t msgs_read_);
    fn process_activate_write(&mut self, msgs_read: u64) {
        unimplemented!()
    }

    // virtual void process_hiccup (pipe_: *mut c_void);
    fn process_hiccup(&mut self, pipe: &mut [u8]) {
        unimplemented!()
    }

    // virtual void process_pipe_peer_stats (queue_count_: u64,
    //                                       ZmqOwn *socket_base_,
    //                                       endpoint_uri_pair_t *endpoint_pair_);
    fn process_pipe_peer_stats(
        &mut self,
        queue_count: u64,
        socket_base: &mut ZmqOwn,
        endpoint_pair: &mut EndpointUriPair,
    ) {
        unimplemented!()
    }

    // virtual void
    // process_pipe_stats_publish (outbound_queue_count_: u64,
    //                             inbound_queue_count_: u64,
    //                             endpoint_uri_pair_t *endpoint_pair_);
    fn process_pipe_stats_publish(
        &mut self,
        outbound_queue_count: u64,
        inbound_queue_count: u64,
        endpoint_pair: &mut EndpointUriPair,
    ) {
        unimplemented!()
    }

    // virtual void process_pipe_term ();
    fn process_pipe_term(&mut self) {
        unimplemented!()
    }

    // virtual void process_pipe_term_ack ();
    fn process_pipe_term_ack(&mut self) {
        unimplemented!()
    }

    // virtual void process_pipe_hwm (inhwm_: i32, outhwm_: i32);
    fn process_pipe_hwm(&mut self, inhwm: i32, outhwm: i32) {
        unimplemented!()
    }

    // virtual void process_term_req (ZmqOwn *object_);
    fn process_term_req(&mut self, object: &mut ZmqOwn) {
        unimplemented!()
    }

    // virtual void process_term (linger_: i32);
    fn process_term(&mut self, linger: i32) {
        unimplemented!()
    }

    // virtual void process_term_ack ();
    fn process_term_ack(&mut self) {
        unimplemented!()
    }

    // virtual void process_term_endpoint (std::string *endpoint_);
    fn process_term_endpoint(&mut self, endpoint: &str) {
        unimplemented!()
    }

    // virtual void process_reap (ZmqSocketBase *socket_);
    fn process_reap(&mut self, socket: &mut ZmqSocketBase) {
        unimplemented!()
    }

    // virtual void process_reaped ();
    fn process_reaped(&mut self) {
        unimplemented!()
    }

    // virtual void process_conn_failed ();
    fn process_conn_failed(&mut self) {
        unimplemented!()
    }

    //  Special handler called after a command that requires a seqnum
    //  was processed. The implementation should catch up with its counter
    //  of processed commands here.
    // virtual void process_seqnum ();
    fn process_seqnum(&mut self) {
        unimplemented!()
    }

    // void send_command (const command_t &cmd);
    fn send_command(&mut self, cmd: &mut ZmqCommand) -> anyhow::Result<()> {
        match (cmd.cmd_type) {
            CommandType::activate_read => self.process_activate_read(),
            CommandType::activate_write => {
                self.process_activate_write(cmd.args.activate_write.msgs_read)
            }
            CommandType::stop => self.process_stop(),
            CommandType::plug => {
                self.process_plug();
                self.process_seqnum();
            }

            CommandType::own => {
                self.process_own(&mut cmd.args.own.object);
                self.process_seqnum();
            }

            CommandType::attach => {
                self.process_attach(cmd.args.attach.engine);
                self.process_seqnum();
            }

            CommandType::bind => {
                self.process_bind(&mut cmd.args.bind.pipe);
                self.process_seqnum();
            }

            CommandType::hiccup => self.process_hiccup(cmd.args.hiccup.pipe),

            CommandType::pipe_peer_stats => self.process_pipe_peer_stats(
                cmd.args.pipe_peer_stats.queue_count,
                &mut cmd.args.pipe_peer_stats.socket_base,
                cmd.args.pipe_peer_stats.endpoint_pair,
            ),

            CommandType::pipe_stats_publish => self.process_pipe_stats_publish(
                cmd.args.pipe_stats_publish.outbound_queue_count,
                cmd.args.pipe_stats_publish.inbound_queue_count,
                cmd.args.pipe_stats_publish.endpoint_pair,
            ),

            CommandType::pipe_term => self.process_pipe_term(),

            CommandType::pipe_term_ack => self.process_pipe_term_ack(),

            CommandType::pipe_hwm => {
                self.process_pipe_hwm(cmd.args.pipe_hwm.inhwm, cmd.args.pipe_hwm.outhwm)
            }

            CommandType::term_req => self.process_term_req(&mut cmd.args.term_req.object),

            CommandType::term => self.process_term(cmd.args.term.linger),

            CommandType::term_ack => self.process_term_ack(),

            CommandType::term_endpoint => {
                self.process_term_endpoint(&mut cmd.args.term_endpoint.endpoint)
            }

            CommandType::reap => self.process_reap(&mut cmd.args.reap.socket),

            CommandType::reaped => self.process_reaped(),

            CommandType::inproc_connected => process_seqnum(),

            CommandType::conn_failed => process_conn_failed(),

            CommandType::done => {}
            _ => {
                return Err(anyhow!("invalid command type: {}", cmd.cmd_type));
            }
        }

        Ok(())
    }
}

// int object_t::register_endpoint (addr_: &str,
//                                       const ZmqEndpoint &endpoint_)
// {
//     return _ctx.register_endpoint (addr_, endpoint_);
// }

// int object_t::unregister_endpoint (const std::string &addr_,
//                                         ZmqSocketBase *socket_)
// {
//     return _ctx.unregister_endpoint (addr_, socket_);
// }

// void object_t::unregister_endpoints (ZmqSocketBase *socket_)
// {
//     return _ctx.unregister_endpoints (socket_);
// }

// ZmqEndpoint object_t::find_endpoint (addr_: &str) const
// {
//     return _ctx.find_endpoint (addr_);
// }

// void object_t::pend_connection (const std::string &addr_,
//                                      const ZmqEndpoint &endpoint_,
//                                      ZmqPipe **pipes_)
// {
//     _ctx.pend_connection (addr_, endpoint_, pipes_);
// }

// void object_t::connect_pending (addr_: &str,
//                                      ZmqSocketBase *bind_socket_)
// {
//     return _ctx.connect_pending (addr_, bind_socket_);
// }

// void object_t::destroy_socket (ZmqSocketBase *socket_)
// {
//     _ctx.destroy_socket (socket_);
// }

// ZmqThread *object_t::choose_io_thread (u64 affinity_) const
// {
//     return _ctx.choose_io_thread (affinity_);
// }

// void object_t::send_stop ()
// {
//     //  'stop' command goes always from administrative thread to
//     //  the current object.
//     ZmqCommand cmd;
//     cmd.destination = this;
//     cmd.type = ZmqCommand::stop;
//     _ctx.send_command (_tid, cmd);
// }

// void object_t::send_plug (ZmqOwn *destination, inc_seqnum_: bool)
// {
//     if (inc_seqnum_)
//         destination.inc_seqnum ();
//
//     ZmqCommand cmd;
//     cmd.destination = destination;
//     cmd.cmd_type = ZmqCommand::plug;
//     send_command (cmd);
// }

// void object_t::send_own (ZmqOwn *destination, ZmqOwn *object_)
// {
//     destination.inc_seqnum ();
//     ZmqCommand cmd;
//     cmd.destination = destination;
//     cmd.cmd_type = ZmqCommand::own;
//     cmd.args.own.object = object_;
//     send_command (cmd);
// }

// void object_t::send_attach (ZmqSessionBase *destination,
//                                  i_engine *engine_,
//                                  inc_seqnum_: bool)
// {
//     if (inc_seqnum_)
//         destination.inc_seqnum ();
//
//     ZmqCommand cmd;
//     cmd.destination = destination;
//     cmd.cmd_type = ZmqCommand::attach;
//     cmd.args.attach.engine = engine_;
//     send_command (cmd);
// }

// void object_t::send_conn_failed (ZmqSessionBase *destination)
// {
//     let mut cmd = ZmqCommand::default();
//     cmd.destination = destination;
//     cmd.cmd_type = CommandType::conn_failed;
//     self.send_command(&cmd);
// }

// void object_t::send_bind (ZmqOwn *destination_,
//                                ZmqPipe *pipe_,
//                                inc_seqnum_: bool)
// {
//     if (inc_seqnum_)
//         destination_.inc_seqnum ();
//
//     ZmqCommand cmd;
//     cmd.destination = destination_;
//     cmd.type = ZmqCommand::bind;
//     cmd.args.bind.pipe = pipe_;
//     send_command (cmd);
// }

// void object_t::send_activate_read (ZmqPipe *destination)
// {
//     let mut cmd = ZmqCommand::default();
//     cmd.destination = destination;
//     cmd.cmd_type = ZmqCommand::activate_read;
//     send_command (cmd);
// }

// void object_t::send_activate_write (ZmqPipe *destination,
//                                          u64 msgs_read_)
// {
//     let mut cmd = ZmqCommand::default();
//     cmd.destination = destination;
//     cmd.cmd_type = ZmqCommand::activate_write;
//     cmd.args.activate_write.msgs_read = msgs_read_;
//     self.send_command(&cmd);
// }

// void object_t::send_hiccup (ZmqPipe *destination, pipe: *mut c_void)
// {
//     let mut cmd = ZmqCommand::default();
//     cmd.destination = destination;
//     cmd.cmd_type = CommandType::hiccup;
//     cmd.args.hiccup.pipe = pipe;
//     self.send_command(&cmd);
// }

// void object_t::send_pipe_peer_stats (ZmqPipe *destination,
//                                           queue_count_: u64,
//                                           ZmqOwn *socket_base_,
//                                           EndpointUriPair *endpoint_pair_)
// {
//     let mut cmd = ZmqCommand::default();
//     cmd.destination = destination;
//     cmd.cmd_type = CommandType::pipe_peer_stats;
//     cmd.args.pipe_peer_stats.queue_count = queue_count_;
//     cmd.args.pipe_peer_stats.socket_base = socket_base_;
//     cmd.args.pipe_peer_stats.endpoint_pair = endpoint_pair_;
//     self.send_command(&cmd);
// }

// void object_t::send_pipe_stats_publish (
//   ZmqOwn *destination,
//   outbound_queue_count_: u64,
//   inbound_queue_count_: u64,
//   EndpointUriPair *endpoint_pair)
// {
//     let mut cmd = ZmqCommand::default();
//     cmd.destination = destination;
//     cmd.cmd_type = CommandType::pipe_stats_publish;
//     cmd.args.pipe_stats_publish.outbound_queue_count = outbound_queue_count_;
//     cmd.args.pipe_stats_publish.inbound_queue_count = inbound_queue_count_;
//     cmd.args.pipe_stats_publish.endpoint_pair = endpoint_pair;
//     self.send_command(&cmd);
// }

// void object_t::send_pipe_term (ZmqPipe *destination)
// {
//     let mut cmd = ZmqCommand::default();
//     cmd.destination = destination;
//     cmd.cmd_type = CommandType::pipe_term;
//     self.send_command(&cmd);
// }

// void object_t::send_pipe_term_ack (ZmqPipe *destination)
// {
//     let mut cmd = ZmqCommand::default();
//     cmd.destination = destination;
//     cmd.cmd_type = CommandType::pipe_term_ack;
//     self.send_command(&cmd);
// }

// void object_t::send_pipe_hwm (ZmqPipe *destination,
//                                    inhwm_: i32,
//                                    outhwm_: i32)
// {
//     let mut cmd = ZmqCommand::default();
//     cmd.destination = destination;
//     cmd.cmd_type = CommandType::pipe_hwm;
//     cmd.args.pipe_hwm.inhwm = inhwm_;
//     cmd.args.pipe_hwm.outhwm = outhwm_;
//     self.send_command(&cmd);
// }

// void object_t::send_term_req (ZmqOwn *destination, ZmqOwn *object)
// {
//     let mut cmd = ZmqCommand::default();
//     cmd.destination = destination;
//     cmd.cmd_type = CommandType::term_req;
//     cmd.args.term_req.object = object;
//     self.send_command(&cmd);
// }

// void object_t::send_term (ZmqOwn *destination, linger_: i32)
// {
//     let mut cmd = ZmqCommand::default();
//     cmd.destination = destination;
//     cmd.cmd_type = CommandType::term;
//     cmd.args.term.linger = linger_;
//     self.send_command(&cmd);
// }

// void object_t::send_term_ack (ZmqOwn *destination)
// {
//     let mut cmd = ZmqCommand::default();
//     cmd.destination = destination;
//     cmd.cmd_type = CommandType::term_ack;
//     self.send_command(&cmd);
// }

// void object_t::send_term_endpoint (ZmqOwn *destination,
//                                         std::string *endpoint_)
// {
//     let mut cmd = ZmqCommand::default();
//     cmd.destination = destination;
//     cmd.cmd_type = CommandType::term_endpoint;
//     cmd.args.term_endpoint.endpoint = endpoint_;
//     self.send_command(&cmd);
// }

// void object_t::send_reap (class ZmqSocketBase *socket)
// {
//     let mut cmd = ZmqCommand::default();
//     cmd.destination = _ctx.get_reaper ();
//     cmd.cmd_type = CommandType::reap;
//     cmd.args.reap.socket = socket;
//     self.send_command(&cmd);
// }

// void object_t::send_reaped ()
// {
//     let mut cmd = ZmqCommand::default();
//     cmd.destination = _ctx.get_reaper ();
//     cmd.cmd_type = CommandType::reaped;
//     self.send_command(&cmd);
// }

// void object_t::send_inproc_connected (ZmqSocketBase *socket_)
// {
//     ZmqCommand cmd;
//     cmd.destination = socket_;
//     cmd.type = ZmqCommand::inproc_connected;
//     send_command (cmd);
// }

// void object_t::send_done ()
// {
//     let mut cmd = ZmqCommand::default();
//     cmd.destination = null_mut();
//     cmd.cmd_type = CommandType::done;
//     _ctx.send_command (ZmqContext::TERM_TID, cmd);
// }

// void object_t::process_stop ()
// {
//     zmq_assert (false);
// }

// void object_t::process_plug ()
// {
//     zmq_assert (false);
// }

// void object_t::process_own (ZmqOwn *)
// {
//     zmq_assert (false);
// }

// void object_t::process_attach (i_engine *)
// {
//     zmq_assert (false);
// }

// void object_t::process_bind (ZmqPipe *)
// {
//     zmq_assert (false);
// }

// void object_t::process_activate_read ()
// {
//     zmq_assert (false);
// }

// void object_t::process_activate_write (u64)
// {
//     zmq_assert (false);
// }

// void object_t::process_hiccup (void *)
// {
//     zmq_assert (false);
// }

// void object_t::process_pipe_peer_stats (u64,
//                                              ZmqOwn *,
//                                              EndpointUriPair *)
// {
//     zmq_assert (false);
// }

// void object_t::process_pipe_stats_publish (u64,
//                                                 u64,
//                                                 EndpointUriPair *)
// {
//     zmq_assert (false);
// }

// void object_t::process_pipe_term ()
// {
//     zmq_assert (false);
// }

// void object_t::process_pipe_term_ack ()
// {
//     zmq_assert (false);
// }

// void object_t::process_pipe_hwm (int, int)
// {
//     zmq_assert (false);
// }

// void object_t::process_term_req (ZmqOwn *)
// {
//     zmq_assert (false);
// }

// void object_t::process_term
// {
//     zmq_assert (false);
// }

// void object_t::process_term_ack ()
// {
//     zmq_assert (false);
// }

// void object_t::process_term_endpoint (std::string *)
// {
//     zmq_assert (false);
// }

// void object_t::process_reap (class ZmqSocketBase *)
// {
//     zmq_assert (false);
// }

// void object_t::process_reaped ()
// {
//     zmq_assert (false);
// }

// void object_t::process_seqnum ()
// {
//     zmq_assert (false);
// }

// void object_t::process_conn_failed ()
// {
//     zmq_assert (false);
// }

// void object_t::send_command (const ZmqCommand &cmd)
// {
//     self.ctx.send_command (cmd.destination.get_tid (), cmd);
// }
