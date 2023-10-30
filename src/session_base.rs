use crate::address::ZmqAddress;
use crate::defines::{
    MSG_COMMAND, MSG_MORE, ZMQ_DGRAM, ZMQ_DISH, ZMQ_NULL, ZMQ_RADIO, ZMQ_SUB, ZMQ_XSUB,
};
use crate::endpoint::ZmqEndpointUriPair;
use crate::i_engine::ErrorReason::TimeoutError;
use crate::i_engine::{ErrorReason, IEngine};
use crate::io_object::IoObject;
use crate::io_thread::ZmqIoThread;
use crate::msg::ZmqMsg;
use crate::object::ZmqObject;
use crate::options::{get_effective_conflate_option, ZmqOptions};
use crate::own::ZmqOwn;
use crate::pipe::{pipepair, IPipeEvents, ZmqPipe};
use crate::socket_base::ZmqSocket;
use std::collections::HashSet;
use std::ptr::null_mut;

pub struct ZmqSessionBase<'a> {
    pub own: ZmqOwn<'a>,
    pub io_object: IoObject,
    pub _active: bool,
    pub _pipe: Option<&'a mut ZmqPipe<'a>>,
    pub _zap_pipe: Option<&'a mut ZmqPipe<'a>>,
    pub _terminating_pipes: HashSet<&'a mut ZmqPipe<'a>>,
    pub _incomplete_in: bool,
    pub _pending: bool,
    pub _socket: &'a mut ZmqSocket<'a>,
    pub _io_thread: &'a mut ZmqIoThread,
    pub _has_linger_timer: bool,
    pub _addr: ZmqAddress,
    pub _engine: Option<&'a mut dyn IEngine>,
}

pub const _linger_timer_id: i32 = 0x20;

impl IPipeEvents for ZmqSessionBase {
    fn read_activated(&self, pipe: &ZmqPipe) {
        unimplemented!()
    }
    fn write_activated(&self, pipe: &ZmqPipe) {
        unimplemented!()
    }
    fn hiccuped(&self, pipe: &ZmqPipe) {
        unimplemented!()
    }
    fn pipe_terminated(&self, pipe: &ZmqPipe) {
        unimplemented!()
    }
}

// impl i_engine for session_base_t
// {
//     fn has_handshake_stage(&mut self) -> bool {
//         todo!()
//     }
//
//     fn Plug(&mut self, io_thread_: *mut io_thread_t, session_: *mut session_base_t) {
//         todo!()
//     }
//
//     fn terminate(&mut self) {
//         todo!()
//     }
//
//     fn restart_input(&mut self) {
//         todo!()
//     }
//
//     fn restart_output(&mut self) {
//         todo!()
//     }
//
//     fn zap_msg_available(&mut self) {
//         todo!()
//     }
//
//     fn get_endpoint(&mut self) -> &mut endpoint_uri_pair_t {
//         todo!()
//     }
// }

impl ZmqSessionBase {
    pub unsafe fn create(
        io_thread_: &mut ZmqIoThread,
        active_: bool,
        socket_: &mut ZmqSocket,
        options_: &ZmqOptions,
        addr_: ZmqAddress,
    ) -> ZmqSessionBase {
        // let mut s: *mut session_base_t = null_mut();
        let mut s = ZmqSessionBase::default();
        match options_.type_ {
            ZMQ_REQ => {
                // s = &mut req_session_t::new(io_thread_, active_, socket_, options_, addr_);
            }
            ZMQ_RADIO => {
                // s = &mut radio_session_t::new(io_thread_, active_, socket_, options_, addr_);
            }
            ZMQ_DISH => {
                // s = &mut dish_session_t::new(io_thread_, active_, socket_, options_, addr_);
            }
            _ => {
                if options_.can_send_hello_msg && options_.hello_msg.len() > 0 {
                    // s = &mut hello_session_t::new(io_thread_, active_, socket_, options_, addr_);
                } else {
                    s = ZmqSessionBase::new(io_thread_, active_, socket_, options_, addr_);
                }
            }
        }
        return s;
    }

    pub unsafe fn new(
        io_thread_: &mut ZmqIoThread,
        active_: bool,
        socket_: &mut ZmqSocket,
        options_: &ZmqOptions,
        addr_: ZmqAddress,
    ) -> Self {
        Self {
            own: ZmqOwn::from_io_thread(io_thread_, options_),
            io_object: IoObject::new(io_thread_),
            _active: active_,
            _pipe: None,
            _zap_pipe: None,
            _terminating_pipes: HashSet::new(),
            _incomplete_in: false,
            _pending: false,
            _socket: socket_,
            _io_thread: io_thread_,
            _engine: None,
            _addr: addr_,
            _has_linger_timer: false,
        }
    }

    pub fn get_endpoint(&mut self) -> &mut ZmqEndpointUriPair {
        return self.get_endpoint();
    }

    pub fn attach_pipe(&mut self, pipe_: &mut ZmqPipe) {
        self._pipe = Some(pipe_);
        self._pipe.set_event_risk(self)
    }

    pub unsafe fn pull_msg(&mut self, msg_: &mut ZmqMsg) -> i32 {
        if self._pipe == null_mut() || !(self._pipe).read(msg_) {
            return -1;
        }

        self._incomplete_in = msg_.flags() & MSG_MORE != 0;
        return 0;
    }

    pub unsafe fn push_msg(&mut self, msg_: &mut ZmqMsg) -> i32 {
        if (msg_).flags() & MSG_COMMAND != 0 && !msg_.is_subscribe() && !msg_.is_cancel() {
            return 0;
        }
        if self._pipe != null_mut() && (self._pipe).write(msg_) {
            let mut rc = (msg_).init2();
            return 0;
        }

        return -1;
    }

    pub unsafe fn read_zap_msg(&mut self, msg_: &mut ZmqMsg) -> i32 {
        if self._zap_pipe == null_mut() || !(self._zap_pipe).read(msg_) {
            return -1;
        }
        return 0;
    }

    pub unsafe fn write_zap_msg(&mut self, msg_: &mut ZmqMsg) -> i32 {
        if self._zap_pipe == null_mut() && !(self._zap_pipe).write(msg_) {
            return -1;
        }

        if msg_.flags() & MSG_MORE == 0 {
            self._zap_pipe.flush()
        }

        let rc = (msg_).init2();

        return 0;
    }

    pub fn reset(&mut self) {
        unimplemented!()
    }

    pub unsafe fn flush(&mut self) {
        if self._pipe != null_mut() {
            self._pipe.flush();
        }
    }

    pub unsafe fn rollback(&mut self) {
        if self._pipe != null_mut() {
            self._pipe.rollback();
        }
    }

    pub unsafe fn clean_pipes(&mut self) {
        self._pipe.rollback();
        self._pipe.flush();

        while (self._incomplete_in) {
            let mut msg = ZmqMsg::new();
            self.pull_msg(&mut msg);
            msg.close();
        }
    }

    pub unsafe fn pipe_terminated(&mut self, pipe_: &mut ZmqPipe) {
        if pipe_ == self._pipe {
            self._pipe = None;
            if self._has_linger_timer {
                self.io_object.cancel_timer(_linger_timer_id);
                self._has_linger_timer = false;
            }
        } else if pipe_ == self._zap_pipe {
            self._zap_pipe = None;
        } else {
            self._terminating_pipes.insert(pipe_);
        }

        if !self.is_terminating() && self.own.options.raw_socket {
            // if self._engine != null_mut() {
            //     self._engine.terminate();
            //     self._engine = null_mut();
            // }
            self.terminate();
        }

        if self._pending
            && self._pipe == null_mut()
            && self._zap_pipe == null_mut()
            && self._terminating_pipes.len() == 0
        {
            self._pending = false;
            self.io_object.signal();
        }
    }

    pub fn read_activated(&mut self, pipe_: &mut ZmqPipe) {
        if pipe_ != self._pipe.unwrap() && pipe_ != self._zap_pipe.unwrap() {
            return;
        }

        if self._engine == null_mut() {
            if self._pipe {
                self._pipe.check_read()
            }
            return;
        }

        if pipe_ == self._pipe {
            self._engine.restart_input();
        } else {
            self._engine.zap_msg_available();
        }
    }

    pub fn write_activated(&mut self, pipe_: *mut ZmqPipe) {
        if self._pipe != pipe_ {
            return;
        }

        if self._engine != null_mut() {
            self._engine.restart_output();
        }
    }

    pub fn hiccuped(&mut self, pipe_: *mut ZmqPipe) {
        unimplemented!()
    }

    pub fn get_socket(&mut self) -> &mut ZmqSocket {
        return self._socket;
    }

    pub unsafe fn process_plug(&mut self) {
        if self._active {
            self.start_connecting(false)
        }
    }

    pub unsafe fn zap_connect(&mut self) -> i32 {
        if self._zap_pipe != null_mut() {
            return 0;
        }

        let mut peer = self.find_endpoint("inproc://zeromq.zap.01");
        if (peer.socket == null_mut()) {
            // errno = ECONNREFUSED;
            return -1;
        }
        // zmq_assert (peer.options.type == ZMQ_REP || peer.options.type == ZMQ_ROUTER
        //             || peer.options.type == ZMQ_SERVER);

        //  Create a bi-directional pipe that will connect
        //  session with zap socket.
        // let mut parents: [*mut object_t;2] = [self, peer.socket];
        let mut new_pipes: [Option<&mut ZmqPipe>; 2] = [None, None];
        let mut hwms: [i32; 2] = [0, 0];
        let mut conflates: [bool; 2] = [false, false];
        // let rc = pipepair (parents, &mut new_pipes, hwms, conflates);
        // errno_assert (rc == 0);

        //  Attach local end of the pipe to this socket object.
        self._zap_pipe = new_pipes[0];
        self._zap_pipe.set_nodelay();
        self._zap_pipe.set_event_sink(self);

        self.send_bind(peer.socket, new_pipes[1], false);

        //  Send empty routing id if required by the peer.
        if (peer.options.recv_routing_id) {
            let mut id = ZmqMsg::default();
            let rc = id.init();
            // errno_assert (rc == 0);
            id.set_flags(ZmqMsg::routing_id);
            let ok = (*self._zap_pipe).write(id);
            // zmq_assert (ok);
            self._zap_pipe.flush();
        }

        return 0;
    }

    pub fn zap_enabled(&mut self) -> bool {
        return self.own.options.mechanism != ZMQ_NULL || !self.own.options.zap_domain.is_empty();
    }

    pub unsafe fn process_attach(&mut self, engine_: &mut dyn IEngine) {
        self._engine = Some(engine_);

        if !((*engine_).has_handshake_stage()) {
            self.engine_ready();
        }

        self._engine.plug(self._io_thread, self);
    }

    pub unsafe fn engine_ready(&mut self) {
        //  Create the pipe if it does not exist yet.
        if (self._pipe.is_none() && !self.is_terminating()) {
            // object_t *parents[2] = {this, _socket};
            let parents: [&mut ZmqObject; 2] =
                [self as &mut ZmqObject, self._socket as &mut ZmqObject];
            // pipe_t *pipes[2] = {NULL, NULL};
            let mut pipes: [Option<&mut ZmqPipe>; 2] = [None, None];

            let conflate = get_effective_conflate_option(&self.own.options);

            // int hwms[2] = {conflate ? -1 : options.rcvhwm,
            //                conflate ? -1 : options.sndhwm};
            let hwms: [i32; 2] = [
                if conflate {
                    -1
                } else {
                    self.own.options.rcvhwm
                },
                if conflate {
                    -1
                } else {
                    self.own.options.sndhwm
                },
            ];

            // bool conflates[2] = {conflate, conflate};
            let conflates: [bool; 2] = [conflate, conflate];
            let mut rc = pipepair(parents, &mut pipes.unwrap(), hwms, conflates);
            // errno_assert (rc == 0);

            //  Plug the local end of the pipe.
            pipes[0].set_event_sink(self);

            //  Remember the local end of the pipe.
            // zmq_assert (!_pipe);
            self._pipe = pipes[0];

            //  The endpoints strings are not set on Bind, set them here so that
            //  events can use them.
            pipes[0].set_endpoint_pair(self._engine.get_endpoint());
            pipes[1].set_endpoint_pair(self._engine.get_endpoint());

            //  Ask socket to Plug into the remote end of the pipe.
            self.send_bind(self._socket, pipes[1]);
        }
    }

    pub unsafe fn engine_error(&mut self, handshaked_: bool, reason_: ErrorReason) {
        //  Engine is dead. Let's forget about it.
        self._engine = None;

        //  Remove any half-Done messages from the pipes.
        if (self._pipe != null_mut()) {
            self.clean_pipes();

            //  Only send disconnect message if socket was accepted and handshake was completed
            if (!self._active
                && handshaked_
                && self.own.options.can_recv_disconnect_msg
                && !self.own.options.disconnect_msg.empty())
            {
                self._pipe
                    .set_disconnect_msg(&mut self.own.options.disconnect_msg);
                self._pipe.send_disconnect_msg();
            }

            //  Only send Hiccup message if socket was connected and handshake was completed
            if (self._active
                && handshaked_
                && self.own.options.can_recv_hiccup_msg
                && !self.own.options.hiccup_msg.empty())
            {
                self._pipe.send_hiccup_msg(&mut self.own.options.hiccup_msg);
            }
        }

        // zmq_assert (reason_ == i_engine::ConnectionError
        //             || reason_ == i_engine::TimeoutError
        //             || reason_ == i_engine::ProtocolError);

        match (reason_) {
            TimeoutError => {}
            /* FALLTHROUGH */
            connection_error => {
                if self._active {
                    self.reconnect();
                    // break;
                }
            }
            protocol_error => {
                if self._pending {
                    if self._pipe {
                        self._pipe.terminate(false);
                    }
                    if self._zap_pipe {
                        self._zap_pipe.terminate(false);
                    }
                } else {
                    self.terminate();
                }
            } // break;
        }

        //  Just in case there's only a delimiter in the pipe.
        if (self._pipe) {
            self._pipe.check_read();
        }

        if (self._zap_pipe) {
            self._zap_pipe.check_read();
        }
    }

    pub unsafe fn process_term(&mut self, linger_: i32) {
        //  If the termination of the pipe happens before the Term command is
        //  delivered there's nothing much to do. We can proceed with the
        //  standard termination immediately.
        if (!self._pipe && !self._zap_pipe && self._terminating_pipes.empty()) {
            // own_t::process_term (0);
            self.own.process_term(0);
            return;
        }

        self._pending = true;

        if (self._pipe != null_mut()) {
            //  If there's finite linger value, delay the termination.
            //  If linger is infinite (negative) we don't even have to set
            //  the timer.
            if (linger_ > 0) {
                // zmq_assert (!_has_linger_timer);
                self.add_timer(linger_, _linger_timer_id);
                self._has_linger_timer = true;
            }

            //  Start pipe termination process. Delay the termination till all messages
            //  are processed in case the linger time is non-zero.
            self._pipe.terminate(linger_ != 0);

            //  TODO: Should this go into pipe_t::terminate ?
            //  In case there's no engine and there's only delimiter in the
            //  pipe it wouldn't be ever read. Thus we check for it explicitly.
            if (!self._engine) {
                self._pipe.check_read();
            }
        }

        if (self._zap_pipe != null_mut()) {
            self._zap_pipe.terminate(false);
        }
    }

    pub unsafe fn timer_event(&mut self, id_: i32) {
        //  Linger period expired. We can proceed with termination even though
        //  there are still pending messages to be sent.
        // zmq_assert (id_ == linger_timer_id);
        self._has_linger_timer = false;

        //  Ask pipe to terminate even though there may be pending messages in it.
        // zmq_assert (_pipe);
        self._pipe.terminate(false);
    }

    pub unsafe fn process_conn_failed(&mut self) {
        // std::string *ep = new (std::string);
        let mut ep = String::new();
        self._addr.to_string(&mut ep);
        self.send_term_endpoint(self._socket, ep);
    }

    pub unsafe fn reconnect(&mut self) {
        //  For delayed connect situations, terminate the pipe
        //  and reestablish later on
        if (self._pipe != null_mut() && self.own.options.immediate == 1
            // #ifdef ZMQ_HAVE_OPENPGM
            //         && _addr->protocol != protocol_name::pgm
            //         && _addr->protocol != protocol_name::epgm
            // #endif
            // #ifdef ZMQ_HAVE_NORM
            //         && _addr->protocol != protocol_name::norm
            // #endif
        && (*self._addr).protocol != "udp")
        {
            self._pipe.hiccup();
            self._pipe.terminate(false);
            self._terminating_pipes.insert(self._pipe.unwrap());
            self._pipe = None;

            if (self._has_linger_timer) {
                self.cancel_timer(_linger_timer_id);
                self._has_linger_timer = false;
            }
        }

        self.reset();

        //  Reconnect.
        if (self.own.options.reconnect_ivl > 0) {
            self.start_connecting(true);
        } else {
            // std::string *ep = new (std::string);
            let mut ep = String::new();
            self._addr.to_string(&mut ep);
            self.send_term_endpoint(self._socket, ep);
        }

        //  For subscriber sockets we Hiccup the inbound pipe, which will cause
        //  the socket object to resend all the subscriptions.
        if (self._pipe.is_some()
            && (self.own.options.type_ == ZMQ_SUB
                || self.own.options.type_ == ZMQ_XSUB
                || self.own.options.type_ == ZMQ_DISH))
        {
            self._pipe.hiccup();
        }
    }

    pub unsafe fn start_connecting(&mut self, wait_: bool) {
        //  Choose I/O thread to run connecter in. Given that we are already
        //  running in an I/O thread, there must be at least one available.
        let io_thread = self.choose_io_thread(self.own.options.affinity);
        // zmq_assert (io_thread);

        //  Create the connecter object.
        // own_t *connecter = NULL;
        let mut connecter: *mut ZmqOwn = null_mut();
        if ((*self._addr).protocol == tcp) {
            if (!self.own.options.socks_proxy_address.empty()) {
                // address_t *proxy_address = new (std::nothrow)
                //   address_t (protocol_name::tcp, options.socks_proxy_address,
                //              this->get_ctx ());
                let proxy_address = ZmqAddress::new2(
                    "tcp",
                    &mut self.own.options.socks_proxy_address,
                    self.get_ctx(),
                );
                // alloc_assert (proxy_address);
                // connecter = new (std::nothrow) socks_connecter_t (
                //   io_thread, this, options, _addr, proxy_address, wait_);
                // connecter = socks_connecter_t::new2 (io_thread, self, self.Own.options, self._addr, proxy_address, wait_);
                // alloc_assert (connecter);
                if (!self.own.options.socks_proxy_username.empty()) {
                    // reinterpret_cast<socks_connecter_t *> (connecter)
                    //   ->set_auth_method_basic (options.socks_proxy_username,
                    //                            options.socks_proxy_password);
                }
            } else {
                // connecter = new (std::nothrow)
                //   tcp_connecter_t (io_thread, this, options, _addr, wait_);
                connecter =
                    tcp_connecter_t::new2(io_thread, self, &self.own.options, self._addr, wait_);
            }
        }
        // #if defined ZMQ_HAVE_IPC
        //     else if (_addr->protocol == protocol_name::ipc) {
        //         connecter = new (std::nothrow)
        //           ipc_connecter_t (io_thread, this, options, _addr, wait_);
        //     }
        // #endif
        // #if defined ZMQ_HAVE_TIPC
        //     else if (_addr->protocol == protocol_name::tipc) {
        //         connecter = new (std::nothrow)
        //           tipc_connecter_t (io_thread, this, options, _addr, wait_);
        //     }
        // #endif
        // #if defined ZMQ_HAVE_VMCI
        //     else if (_addr->protocol == protocol_name::vmci) {
        //         connecter = new (std::nothrow)
        //           vmci_connecter_t (io_thread, this, options, _addr, wait_);
        //     }
        // #endif
        // #if defined ZMQ_HAVE_WS
        //     else if (_addr->protocol == protocol_name::ws) {
        //         connecter = new (std::nothrow) ws_connecter_t (
        //           io_thread, this, options, _addr, wait_, false, std::string ());
        //     }
        // #endif
        // #if defined ZMQ_HAVE_WSS
        //     else if (_addr->protocol == protocol_name::wss) {
        //         connecter = new (std::nothrow) ws_connecter_t (
        //           io_thread, this, options, _addr, wait_, true, _wss_hostname);
        //     }
        // #endif
        if (connecter != null_mut()) {
            // alloc_assert (connecter);
            self.launch_child(connecter);
            return;
        }

        if (self._addr.protocol == "udp") {
            // zmq_assert (options.type == ZMQ_DISH || options.type == ZMQ_RADIO
            //             || options.type == ZMQ_DGRAM);

            // udp_engine_t *engine = new (std::nothrow) udp_engine_t (options);
            let engine = udp_engine_t::new2(&self.own.options);
            // alloc_assert (engine);

            let mut recv = false;
            let mut send = false;

            if (self.own.options.type_ == ZMQ_RADIO) {
                send = true;
                recv = false;
            } else if (self.own.options.type_ == ZMQ_DISH) {
                send = false;
                recv = true;
            } else if (self.own.options.type_ == ZMQ_DGRAM) {
                send = true;
                recv = true;
            }

            let rc = engine.init(self._addr, send, recv);
            // errno_assert (rc == 0);

            self.send_attach(self, engine);

            return;
        }

        // #ifdef ZMQ_HAVE_OPENPGM
        //
        //     //  Both PGM and EPGM transports are using the same infrastructure.
        //     if (_addr->protocol == "pgm" || _addr->protocol == "epgm") {
        //         zmq_assert (options.type == ZMQ_PUB || options.type == ZMQ_XPUB
        //                     || options.type == ZMQ_SUB || options.type == ZMQ_XSUB);
        //
        //         //  For EPGM transport with UDP encapsulation of PGM is used.
        //         bool const udp_encapsulation = _addr->protocol == "epgm";
        //
        //         //  At this point we'll create message pipes to the session straight
        //         //  away. There's no point in delaying it as no concept of 'connect'
        //         //  exists with PGM anyway.
        //         if (options.type == ZMQ_PUB || options.type == ZMQ_XPUB) {
        //             //  PGM sender.
        //             pgm_sender_t *pgm_sender =
        //               new (std::nothrow) pgm_sender_t (io_thread, options);
        //             alloc_assert (pgm_sender);
        //
        //             int rc =
        //               pgm_sender->init (udp_encapsulation, _addr->address.c_str ());
        //             errno_assert (rc == 0);
        //
        //             send_attach (this, pgm_sender);
        //         } else {
        //             //  PGM receiver.
        //             pgm_receiver_t *pgm_receiver =
        //               new (std::nothrow) pgm_receiver_t (io_thread, options);
        //             alloc_assert (pgm_receiver);
        //
        //             int rc =
        //               pgm_receiver->init (udp_encapsulation, _addr->address.c_str ());
        //             errno_assert (rc == 0);
        //
        //             send_attach (this, pgm_receiver);
        //         }
        //
        //         return;
        //     }
        // #endif

        // #ifdef ZMQ_HAVE_NORM
        //     if (_addr->protocol == "norm") {
        //         //  At this point we'll create message pipes to the session straight
        //         //  away. There's no point in delaying it as no concept of 'connect'
        //         //  exists with NORM anyway.
        //         if (options.type == ZMQ_PUB || options.type == ZMQ_XPUB) {
        //             //  NORM sender.
        //             norm_engine_t *norm_sender =
        //               new (std::nothrow) norm_engine_t (io_thread, options);
        //             alloc_assert (norm_sender);
        //
        //             int rc = norm_sender->init (_addr->address.c_str (), true, false);
        //             errno_assert (rc == 0);
        //
        //             send_attach (this, norm_sender);
        //         } else { // ZMQ_SUB or ZMQ_XSUB
        //
        //             //  NORM receiver.
        //             norm_engine_t *norm_receiver =
        //               new (std::nothrow) norm_engine_t (io_thread, options);
        //             alloc_assert (norm_receiver);
        //
        //             int rc = norm_receiver->init (_addr->address.c_str (), false, true);
        //             errno_assert (rc == 0);
        //
        //             send_attach (this, norm_receiver);
        //         }
        //         return;
        //     }
        // #endif // ZMQ_HAVE_NORM

        // zmq_assert (false);
    }
}

pub struct hello_msg_session_t<'a> {
    pub session_base_t: ZmqSessionBase<'a>,
    pub _hello_sent: bool,
    pub _hello_received: bool,
    pub _new_pipe: bool,
}

impl hello_msg_session_t {
    pub unsafe fn new(
        io_thread_: &mut ZmqIoThread,
        connect_: bool,
        socket_: &mut ZmqSocket,
        options: &ZmqOptions,
        addr_: ZmqAddress,
    ) -> Self {
        Self {
            session_base_t: ZmqSessionBase::new(io_thread_, connect_, socket_, options, addr_),
            _hello_sent: false,
            _hello_received: false,
            _new_pipe: true,
        }
    }

    pub unsafe fn pull_msg(&mut self, msg_: &mut ZmqMsg) -> i32 {
        if self._new_pipe != null_mut() {
            self._new_pipe = false;
            // let rc = init_buffer(&self.options.hello_msg[0], self.options.hello_msg.len();
            return 0;
        }
        self.session_base_t.pull_msg(msg_)
    }

    pub unsafe fn reset(&mut self) {
        self.session_base_t.reset();
        self._new_pipe = true;
    }
}
