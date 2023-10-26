use std::ffi::c_void;
use std::ptr::null_mut;
use crate::command::{ZmqCommand, ZmqType};
use crate::command::ZmqType::{activate_read, stop};
use crate::ctx::{ZmqContext, Endpoint};
use crate::endpoint::ZmqEndpointUriPair;
use crate::i_engine::IEngine;
use crate::io_thread::ZmqIoThread;
use crate::options::ZmqOptions;
use crate::own::ZmqOwn;
use crate::pipe::ZmqPipe;
use crate::session_base::ZmqSessionBase;
use crate::socket_base::ZmqSocketBase;

pub struct ZmqObject<'a> {
    pub context: &'a mut ZmqContext<'a>,
    pub thread_id: u32,
}

impl ZmqObject {
    pub fn new(ctx: &mut ZmqContext, tid: u32) -> Self {
        Self { context: ctx, thread_id: tid }
    }

    pub unsafe fn new2(parent: &mut Self) -> Self {
        Self { context: (*parent).context, thread_id: (*parent).thread_id }
    }

    pub fn get_tid(&self) -> u32 {
        self.thread_id
    }

    pub fn set_tid(&mut self, id: u32) {
        self.thread_id = id;
    }

    pub unsafe fn process_command(&mut self, cmd: ZmqCommand) {
        match cmd.type_ {
            activate_read => {
                self.process_activate_read();
            }
            // break;

            activate_write => {
                self.process_activate_write(cmd.args.activate_write.msgs_read);
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
                self.process_own(cmd.args.own.object);
                self.process_seqnum();
            }
            // break;

            attach => {
                self.process_attach(cmd.args.attach.engine);
                self.process_seqnum();
            }
            // break;

            bind => {
                self.process_bind(cmd.args.bind.pipe);
                self.process_seqnum();
            }
            // break;

            hiccup => {
                self.process_hiccup(cmd.args.hiccup.pipe);
            }
            // break;

            pipe_peer_stats => {
                self.process_pipe_peer_stats(cmd.args.pipe_peer_stats.queue_count,
                                             cmd.args.pipe_peer_stats.socket_base,
                                             cmd.args.pipe_peer_stats.endpoint_pair);
            }
            // break;

            pipe_stats_publish => {
                self.process_pipe_stats_publish(
                    cmd.args.pipe_stats_publish.outbound_queue_count,
                    cmd.args.pipe_stats_publish.inbound_queue_count,
                    cmd.args.pipe_stats_publish.endpoint_pair);
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
                self.process_pipe_hwm(cmd.args.pipe_hwm.inhwm,
                                      cmd.args.pipe_hwm.outhwm);
            }
            // break;

            term_req => {
                self.process_term_req(cmd.args.term_req.object);
            }
            // break;

            term => {
                self.process_term(cmd.args.term.linger);
            }
            // break;

            term_ack => {
                self.process_term_ack();
            }
            // break;

            term_endpoint => {
                self.process_term_endpoint(&cmd.args.term_endpoint.endpoint);
            }
            // break;

            reap => {
                self.process_reap(cmd.args.reap.socket);
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

    pub fn register_endpoint(&mut self, addr: &str, endpoint: &Endpoint, options: &mut ZmqOptions) -> i32 {
        self.context.register_endpoint(addr, endpoint, options)
    }

    pub fn unregister_endpoint(&mut self, addr: &str, socket: &mut ZmqSocketBase) {
        self.context.unregister_endpoint(addr);
    }

    pub fn unregister_endpoints(&mut self, socket: &mut ZmqSocketBase) {
        self.context.unregister_endpoints(socket);
    }

    pub fn find_endpoint(&mut self, addr: &str) -> &mut Endpoint {
        self.context.find_endpoint(addr) as &mut Endpoint
    }

    pub unsafe fn pend_connection(&mut self,
                                  addr: &str,
                                  endpoint: &Endpoint,
                                  pipes: &mut [&mut ZmqPipe]) {
        self.context.pend_connection(addr, endpoint, pipes);
    }

    pub unsafe fn connect_pending(&mut self, addr: &str, bind_socket: *mut ZmqSocketBase) {
        self.context.connect_pending(addr, bind_socket);
    }

    pub unsafe fn destroy_socket(&mut self, socket: *mut ZmqSocketBase) {
        self.context.destroy_socket(socket);
    }

    pub unsafe fn choose_io_thread(&mut self, affinity: u64) -> *mut ZmqIoThread {
        self.context.choose_io_thread(affinity)
    }

    pub fn send_stop(&mut self) {
        let mut cmd = ZmqCommand::new();
        cmd.destination = Some(self);
        cmd.type_ = ZmqType::stop;
        self.context.send_command(self.thread_id, &mut cmd);
    }

    pub unsafe fn send_plug(&mut self, destination: &mut ZmqOwn, inc_seqnum: bool) {
        if (inc_seqnum) {
            destination.inc_seqnum();
        }

        let mut cmd = ZmqCommand::new();
        cmd.destination = Some(destination);
        cmd.type_ = ZmqType::plug;
        self.send_command(&cmd);
    }

    pub unsafe fn send_own(&mut self, destination: &mut ZmqOwn, object: &mut ZmqOwn) {
        destination.inc_seqnum();
        let mut cmd = ZmqCommand::new();
        cmd.destination = Some(destination);
        cmd.type_ = ZmqType::own;
        cmd.args.own.object = object;
        self.send_command(&mut cmd);
    }

    pub unsafe fn send_attach(&mut self, destination: &mut ZmqSessionBase, engine: &mut dyn IEngine, inc_seqnum: bool) {
        if (inc_seqnum) {
            destination.inc_seqnum();
        }

        let mut cmd = ZmqCommand::new();
        cmd.destination = Some(destination);
        cmd.type_ = ZmqType::attach;
        cmd.args.attach.engine = engine;
        self.send_command(&mut cmd);
    }

    pub unsafe fn send_conn_failed(&mut self, destination: &mut ZmqSessionBase) {
        let mut cmd = ZmqCommand::new();
        cmd.destination = Some(destination);
        cmd.type_ = ZmqType::conn_failed;
        self.send_command(&cmd);
    }

    pub unsafe fn send_bind(&mut self, destination: &mut ZmqOwn,
                            pipe: &mut ZmqPipe,
                            inc_seqnum: bool) {
        if (inc_seqnum) {
            destination.inc_seqnum();
        }

        let mut cmd = ZmqCommand::new();
        cmd.destination = Some(destination);
        cmd.type_ = ZmqType::bind;
        cmd.args.bind.pipe = pipe;
        self.send_command(&cmd);
    }

    pub unsafe fn send_activate_read(&mut self, destination: &mut ZmqPipe) {
        let mut cmd = ZmqCommand::new();
        cmd.destination = Some(destination);
        cmd.type_ = ZmqType::activate_read;
        self.send_command(&cmd);
    }

    pub unsafe fn send_activate_write(&mut self, destination: &mut ZmqPipe,
                                      msgs_read: u64) {
        let mut cmd = ZmqCommand::new();
        cmd.destination = Some(destination);
        cmd.type_ = ZmqType::activate_write;
        cmd.args.activate_write.msgs_read = msgs_read;
        self.send_command(&cmd);
    }

    pub unsafe fn send_hiccup(&mut self, destination: &mut ZmqPipe, pipe: &mut ZmqPipe) {
        let mut cmd = ZmqCommand::new();
        cmd.destination = Some(destination);
        cmd.type_ = ZmqType::hiccup;
        cmd.args.hiccup.pipe = pipe;
        self.send_command(&cmd);
    }

    pub unsafe fn send_pipe_peer_stats(&mut self,
                                       destination: &mut ZmqPipe,
                                       queue_count: u64,
                                       socket_base: &mut ZmqOwn,
                                       endpoint_pair: &mut ZmqEndpointUriPair) {
        let mut cmd = ZmqCommand::new();
        cmd.destination = Some(destination);
        cmd.type_ = ZmqType::pipe_peer_stats;
        cmd.args.pipe_peer_stats.queue_count = queue_count;
        cmd.args.pipe_peer_stats.socket_base = socket_base;
        cmd.args.pipe_peer_stats.endpoint_pair = endpoint_pair;
        self.send_command(&cmd);
    }

    pub unsafe fn send_pipe_stats_publish(&mut self,
                                          destination: &mut ZmqOwn,
                                          outbound_queue_count: u64,
                                          inbound_queue_count: u64,
                                          endpoint_pair: &mut ZmqEndpointUriPair) {
        let mut cmd = ZmqCommand::new();
        cmd.destination = Some(destination);
        cmd.type_ = ZmqType::pipe_stats_publish;
        cmd.args.pipe_stats_publish.outbound_queue_count = outbound_queue_count;
        cmd.args.pipe_stats_publish.inbound_queue_count = inbound_queue_count;
        cmd.args.pipe_stats_publish.endpoint_pair = endpoint_pair;
        self.send_command(&cmd);
    }

    pub unsafe fn send_pipe_term (&mut self, destination: &mut ZmqPipe)
    {
        let mut cmd = ZmqCommand::new();
        cmd.destination = Some(destination);
        cmd.type_ = ZmqType::pipe_term;
        self.send_command (&cmd);
    }

    pub unsafe fn send_pipe_term_ack (&mut self, destination: &mut ZmqPipe)
    {
        let mut cmd = ZmqCommand::new();
        cmd.destination = Some(destination);
        cmd.type_ = ZmqType::pipe_term_ack;
        self.send_command (&cmd);
    }

    pub unsafe fn send_term_endpoint (&mut self, destination: &mut ZmqOwn,
                                      endpoint: &str)
    {
        let mut cmd = ZmqCommand::new();
        cmd.destination = destination;
        cmd.type_ = ZmqType::term_endpoint;
        cmd.args.term_endpoint.endpoint = endpoint.to_string();
        self.send_command (&cmd);
    }

    pub unsafe fn send_reap (&mut self, socket: *mut ZmqSocketBase)
    {
        let mut cmd = ZmqCommand::new();
        cmd.destination = Some(self.context.get_reaper ());
        cmd.type_ = ZmqType::reap;
        cmd.args.reap.socket = socket;
        self.send_command (&cmd);
    }

    pub unsafe fn send_reaped (&mut self)
    {
        let mut cmd = ZmqCommand::new();
        cmd.destination = self.context.get_reaper ();
        cmd.type_ = ZmqType::reaped;
        self.send_command (&cmd);
    }

    pub unsafe fn send_inproc_connected (&mut self, socket: *mut ZmqSocketBase)
    {
        let mut cmd = ZmqCommand::new();
        cmd.destination = socket;
        cmd.type_ = ZmqType::inproc_connected;
        self.send_command (&cmd);
    }

    pub unsafe fn send_done (&mut self)
    {
        let mut cmd = ZmqCommand::new();
        cmd.destination = null_mut();
        cmd.type_ = ZmqType::done;
        self.context.send_command (ZmqContext::term_tid, &mut cmd);
    }

    pub unsafe fn send_command (&mut self, cmd: &ZmqCommand)
    {
        self.context.send_command (cmd.destination.get_tid (), cmd);
    }
}

impl object_ops for ZmqObject {
    fn process_stop(&mut self) {
        todo!()
    }

    fn process_plug(&mut self) {
        todo!()
    }

    fn process_own(&mut self, object_: *mut ZmqOwn) {
        todo!()
    }

    fn process_attach(&mut self, engine_: *mut dyn IEngine) {
        todo!()
    }

    fn process_bind(&mut self, pipe_: *mut ZmqPipe) {
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

    fn process_pipe_peer_stats(&mut self, queue_count_: u64, socket_base: *mut ZmqOwn, endpoint_pair_: *mut ZmqEndpointUriPair) {
        todo!()
    }

    fn process_pipe_stats_publish(&mut self, outbound_queue_count_: u64, inbound_queue_count: u64, endpoint_pair_: *mut ZmqEndpointUriPair) {
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

    fn process_pipe_term_req(&mut self, object_: *mut ZmqOwn) {
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

    fn process_reap(&mut self, socket_: *mut ZmqSocketBase) {
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
    fn process_own(&mut self, object_: *mut ZmqOwn);

    fn process_attach(&mut self, engine_: *mut dyn IEngine);

    fn process_bind(&mut self, pipe_: *mut ZmqPipe);

    fn process_activate_read(&mut self);

    fn process_activate_write(&mut self, msgs_read_: u64);

    fn process_hiccup(&mut self, pipe_: *mut c_void);

    fn process_pipe_peer_stats(&mut self, queue_count_: u64, socket_base: *mut ZmqOwn, endpoint_pair_: *mut ZmqEndpointUriPair);

    fn process_pipe_stats_publish(&mut self, outbound_queue_count_: u64, inbound_queue_count: u64, endpoint_pair_: *mut ZmqEndpointUriPair);

    fn process_pipe_term(&mut self);

    fn process_pipe_term_ack(&mut self);

    fn process_pipe_hwm(&mut self, inhwm_: i32, outhwm_: i32);

    fn process_pipe_term_req(&mut self, object_: *mut ZmqOwn);

    fn process_term(&mut self, linger_: i32);

    fn process_term_ack(&mut self);

    fn process_term_endpoint(&mut self, endpoint_: &str);

    fn process_reap(&mut self, socket_: *mut ZmqSocketBase);

    fn process_reaped(&mut self);

    fn process_conn_failed(&mut self);

    fn process_seqnum(&mut self);
}
