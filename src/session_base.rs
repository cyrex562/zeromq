/*
    Copyright (c) 2007-2016 Contributors as noted in the AUTHORS file

    This file is part of libzmq, the ZeroMQ core engine in C+= 1.

    libzmq is free software; you can redistribute it and/or modify it under
    the terms of the GNU Lesser General Public License (LGPL) as published
    by the Free Software Foundation; either version 3 of the License, or
    (at your option) any later version.

    As a special exception, the Contributors give you permission to link
    this library with independent modules to produce an executable,
    regardless of the license terms of these independent modules, and to
    copy and distribute the resulting executable under terms of your choice,
    provided that you also meet, for each linked independent module, the
    terms and conditions of the license of that module. An independent
    module is a module which is not derived from or based on this library.
    If you modify this library, you must extend this exception to your
    version of the library.

    libzmq is distributed in the hope that it will be useful, but WITHOUT
    ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
    FITNESS FOR A PARTICULAR PURPOSE. See the GNU Lesser General Public
    License for more details.

    You should have received a copy of the GNU Lesser General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

// #include "precompiled.hpp"
// #include "macros.hpp"
// #include "session_base.hpp"
// #include "ZmqIEngine.hpp"
// #include "err.hpp"
// #include "pipe.hpp"
// #include "likely.hpp"
// #include "tcp_connecter.hpp"
// #include "ws_connecter.hpp"
// #include "ipc_connecter.hpp"
// #include "tipc_connecter.hpp"
// #include "socks_connecter.hpp"
// #include "vmci_connecter.hpp"
// #include "pgm_sender.hpp"
// #include "pgm_receiver.hpp"
// #include "address.hpp"
// #include "norm_engine.hpp"
// #include "udp_engine.hpp"

use std::collections::HashSet;
use std::process::id;
use std::ptr::null_mut;

use anyhow::anyhow;
use bincode::options;
use libc::{pipe, ECONNREFUSED};
use windows::Win32::Networking::WinSock::{recv, send};

use crate::address::ZmqAddress;
use crate::context::{choose_io_thread, find_endpoint, get_effective_conflate_option, ZmqContext};
use crate::defines::{
    ZMQ_CHANNEL, ZMQ_CLIENT, ZMQ_DEALER, ZMQ_DGRAM, ZMQ_DISH, ZMQ_GATHER, ZMQ_NULL, ZMQ_PAIR,
    ZMQ_PEER, ZMQ_PUB, ZMQ_PULL, ZMQ_PUSH, ZMQ_RADIO, ZMQ_REP, ZMQ_REQ, ZMQ_ROUTER, ZMQ_SCATTER,
    ZMQ_SERVER, ZMQ_STREAM, ZMQ_SUB, ZMQ_XPUB, ZMQ_XSUB,
};
use crate::dish::DishSession;
use crate::endpoint::{EndpointUriPair, ZmqEndpoint};
use crate::engine_interface::ZmqEngineInterface;
use crate::io_object::ZmqIoObject;
use crate::ipc_connecter::IpcConnecter;
use crate::message::{ZmqMessage, ZMQ_MSG_COMMAND, ZMQ_MSG_MORE, ZMQ_MSG_ROUTING_ID};
use crate::norm_engine::NormEngine;
use crate::object::ZmqObject;
use crate::own::ZmqOwn;
use crate::pgm_receiver::ZmqPgmReceiver;
use crate::pgm_sender::pgm_sender_t;
use crate::pipe::PipeState::active;
use crate::pipe::ZmqPipe;
use crate::proxy::ZmqSocket;
use crate::radio::RadioSession;
use crate::req::ReqSession;
use crate::socks_connecter::ZmqSocksConnector;
use crate::tcp_connecter::ZmqTcpConnector;
use crate::thread_context::ZmqThreadContext;
use crate::tipc_connecter::ZmqTipcConnecter;
use crate::transport::ZmqTransport;
use crate::udp_engine::ZmqUdpEngine;
use crate::vmci_connecter::ZmqVmciConnecter;
use crate::ws_connecter::ZmqWsConnecter;

// enum
// {
//     LINGER_TIMER_ID = 0x20
// };
pub const LINGER_TIMER_ID: i32 = 0x20;

// #include "ctx.hpp"
// #include "req.hpp"
// #include "radio.hpp"
// #include "dish.hpp"
// pub struct ZmqSessionBase : public ZmqOwn, public io_object_t, public i_pipe_events
#[derive(Default, Debug, Clone)]
pub struct ZmqSessionBase {
    //  If true, this session (re)connects to the peer. Otherwise, it's
    //  a transient session created by the listener.
    pub active: bool,
    //  Pipe connecting the session to its socket.
    // ZmqPipe *pipe;
    pub pipe: Option<ZmqPipe>,
    //  Pipe used to exchange messages with ZAP socket.
    // ZmqPipe *_zap_pipe;
    pub zap_pipe: Option<ZmqPipe>,
    //  This set is added to with pipes we are disconnecting, but haven't yet completed
    // std::set<ZmqPipe *> _terminating_pipes;
    pub terminating_pipes: HashSet<ZmqPipe>,
    //  This flag is true if the remainder of the message being processed
    //  is still in the in pipe.
    pub incomplete_in: bool,
    //  True if termination have been suspended to push the pending
    //  messages to the network.
    pub pending: bool,
    //  The protocol I/O engine connected to the session.
    // ZmqIEngine *_engine;
    pub engine: Option<ZmqEngineInterface>,
    //  The socket the session belongs to.
    // ZmqSocketBase *_socket;
    pub socket: ZmqSocket,
    //  I/O thread the session is living in. It will be used to Plug in
    //  the engines into the same thread.
    // ZmqIoThread *_io_thread;
    pub io_thread: ZmqThreadContext,
    //  ID of the linger timer
    //  True is linger timer is running.
    pub has_linger_timer: bool,
    //  Protocol and address to use when connecting.
    // Address *_addr;
    pub addr: ZmqAddress,
    // #ifdef ZMQ_HAVE_WSS
    //  TLS handshake, we need to take a copy when the session is created,
    //  in order to maintain the value at the creation time
    // const _wss_hostname: String;
    pub wss_hostname: String,
    // #endif
    // // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqSessionBase)
    pub reset_fn: Option<fn()>,
}

impl ZmqSessionBase {
    // ZmqSessionBase::ZmqSessionBase (class ZmqIoThread *io_thread_,
    //                                  active_: bool,
    // pub struct ZmqSocketBase *socket_,
    //                                      options: &ZmqOptions,
    //                                      Address *addr_) :
    //     ZmqOwn (io_thread_, options_),
    //     io_object_t (io_thread_),
    //     active (active_),
    //     pipe (null_mut()),
    //     _zap_pipe (null_mut()),
    //     _incomplete_in (false),
    //     _pending (false),
    //     _engine (null_mut()),
    //     _socket (socket_),
    //     _io_thread (io_thread_),
    //     _has_linger_timer (false),
    //     _addr (addr_)
    // // #ifdef ZMQ_HAVE_WSS
    //     ,
    //     _wss_hostname (options_.wss_hostname)
    // // #endif
    // {
    // }
    pub fn new(
        zmq_ctx: &mut ZmqContext,
        io_thread: &mut ZmqThreadContext,
        active_: bool,
        socket: &mut ZmqSocket,
        addr: &mut ZmqAddress,
    ) -> Self {
        let mut own = ZmqOwn::new(options, zmq_ctx, io_thread.tid);
        let mut io_object = ZmqIoObject::new(Some(io_thread.clone()));
        Self {
            active: active_,
            pipe: None,
            zap_pipe: None,
            terminating_pipes: HashSet::new(),
            incomplete_in: false,
            pending: false,
            engine: None,
            socket: socket.clone(),
            io_thread: io_thread.clone(),
            has_linger_timer: false,
            addr: addr.clone(),
            wss_hostname: options.wss_hostname.clone(),
            reset_fn: None,
        }
    }

    //
    //  Create a session of the particular type.
    // static ZmqSessionBase *create (ZmqIoThread *io_thread_,
    // active_: bool,
    // socket_: *mut ZmqSocketBase,
    // options: &ZmqOptions,
    // Address *addr_);
    pub fn create(
        ctx: &mut ZmqContext,
        io_thread: &mut ZmqThreadContext,
        active_: bool,
        socket: &mut ZmqSocket,
        addr: Option<&mut ZmqAddress>,
    ) -> anyhow::Result<Self> {
        // ZmqSessionBase *s = null_mut();
        let mut s = ZmqSessionBase::default();
        match (options.type_) {
            ZMQ_REQ => s = ReqSession::new(io_thread_, active_, socket, options_, addr_),
            ZMQ_RADIO => s = RadioSession::new(io_thread_, active_, socket, options_, addr_),
            ZMQ_DISH => s = DishSession(io_thread_, active_, socket, options_, addr_),
            ZMQ_DEALER | ZMQ_ROUTER | ZMQ_XPUB | ZMQ_XSUB | ZMQ_REP | ZMQ_PUB | ZMQ_SUB | ZMQ_PUSH | ZMQ_PULL | ZMQ_PAIR | ZMQ_STREAM | ZMQ_SERVER | ZMQ_CLIENT | ZMQ_GATHER | ZMQ_SCATTER | ZMQ_DGRAM | ZMQ_PEER | ZMQ_CHANNEL => {
                // #ifdef ZMQ_BUILD_DRAFT_API
                if (options.can_send_hello_msg && options.hello_msg.size() > 0) {
                    // TODO
                    // s = ZmqHelloMsgSession::new(ctx, io_thread, active_, socket, options, addr);
                }
                // hello_msg_session_t(
                //     io_thread_, active_, socket_, options_, addr_);
                else {
                    s = Self::new(ctx, io_thread, active_, socket, options, addr);

                    // ZmqSessionBase(
                    //     io_thread_, active_, socket_, options_, addr_);

                    // break;
                    // #else
                    //             s = new(std::nothrow)
                    //             ZmqSessionBase(io_thread_, active_, socket_, options_, addr_);
                    //             break;
                    // #endif
                }
            }
            _ => {
                // errno = EINVAL;
                // return null_mut();
                return Err(anyhow!("EINVAL"));
            }
        }

        // alloc_assert (s);
        // return s;
        Ok(s)
    }

    //  To be used once only, when creating the session.
    // void attach_pipe (pipe_: &mut ZmqPipe);
    pub fn attach_pipe(&mut self, pipe: &mut ZmqPipe) {
        // zmq_assert (!is_terminating ());
        // zmq_assert (!pipe);
        // zmq_assert (pipe_);
        self.pipe = Some(pipe.clone());
        self.pipe.set_event_sink(this);
    }

    //  Fetches a message. Returns 0 if successful; -1 otherwise.
    //  The caller is responsible for freeing the message when no
    //  longer used.
    // virtual int pull_msg (msg: &mut ZmqMessage);
    pub fn pull_msg(&mut self, msg: &mut ZmqMessage) -> anyhow::Result<()> {
        // TODO: enable override by checking for filled in fn ptr
        if (self.pipe.is_none() || self.pipe.unwrap().read(msg)) {
            // errno = EAGAIN;
            // return -1;
            return Err(anyhow!("EAGAIN"));
        }

        self.incomplete_in = (msg.flags() & ZMQ_MSG_MORE) != 0;

        Ok(())
    }

    pub fn push_msg(&mut self, msg: &mut ZmqMessage) -> anyhow::Result<()> {
        //  pass subscribe/cancel to the sockets
        if ((msg.flags() & ZMQ_MSG_COMMAND) && !msg.is_subscribe() && !msg.is_cancel()) {
            Ok(())
        }
        if (self.pipe.is_some() && self.pipe.unwrap().write(msg)) {
            msg.init2()?;
            // errno_assert (rc == 0);
            // return 0;
            Ok(())
        }

        // errno = EAGAIN;
        // return -1;
        Err(anyhow!("EAGAIN"))
    }

    // void flush ();
    pub fn flush(&mut self) -> anyhow::Result<()> {
        // if (pipe)
        // pipe.flush ();
        if self.pipe.is_some() {
            self.pipe.unwrap().flush()?;
        }
        Ok(())
    }

    // void rollback ();
    pub fn rollback(&mut self) -> anyhow::Result<()> {
        // if (pipe)
        // pipe.rollback ();
        if self.pipe.is_some() {
            self.pipe.unwrap().rollback()?;
        }
        Ok(())
    }

    // void engine_error (handshaked_: bool, ZmqIEngine::ZmqErrorReason reason_);

    // void engine_ready ();

    //  i_pipe_events interface implementation.
    // void read_activated (pipe_: &mut ZmqPipe) ;

    pub fn read_activated(&mut self, pipe: &mut ZmqPipe) {
        // Skip activating if we're detaching this pipe
        if (pipe != self.pipe && pipe != self.zap_pipe) {
            // zmq_assert (_terminating_pipes.count (pipe_) == 1);
            return;
        }

        if (self.engine.is_none()) {
            if (self.pipe.is_some()) {
                self.pipe.unwrap().check_read();
            }
            return;
        }

        if (pipe == self.pipe) {
            self.engine.restart_output();
        } else {
            // i.e. pipe_ == zap_pipe
            self.engine.zap_msg_available();
        }
    }

    // void write_activated (pipe_: &mut ZmqPipe) ;

    // void hiccuped (pipe_: &mut ZmqPipe) ;

    // void pipe_terminated (pipe_: &mut ZmqPipe) ;
    pub fn pipe_terminated(&mut self, pipe: &mut ZmqPipe) -> anyhow::Result<()> {
        // Drop the reference to the deallocated pipe if required.
        // zmq_assert (pipe_ == pipe || pipe_ == _zap_pipe
        // || _terminating_pipes.count (pipe_) == 1);

        if (pipe == self.pipe) {
            // If this is our current pipe, remove it
            self.pipe = None;
            if (self.has_linger_timer) {
                cancel_timer(LINGER_TIMER_ID);
                self.has_linger_timer = false;
            }
        } else if (pipe == self.zap_pipe) {
            self.zap_pipe = None;
        } else {
            // Remove the pipe from the detached pipes set
            self.terminating_pipes.erase(pipe);
        }

        if (!self.is_terminating() && self.options.raw_socket) {
            if (_engine) {
                self.engine.unwrap().terminate();
                self.engine = None;
            }
            self.terminate();
        }

        //  If we are waiting for pending messages to be sent, at this point
        //  we are sure that there will be no more messages and we can proceed
        //  with termination safely.
        if (self.pending && self.pipe.is_none() && self.zap_pipe.is_none() && self.terminating_pipes.empty()) {
            self.pending = false;
            self.process_term(0);
        }

        Ok(())
    }

    // int zap_connect ();

    // bool zap_enabled () const;

    //  Sends message to ZAP socket.
    //  Returns 0 on success; -1 otherwise.
    //  The function takes ownership of the message.
    // int write_zap_msg (msg: &mut ZmqMessage);
    pub fn write_zap_msg(&mut self, msg: &mut ZmqMessage) -> anyhow::Result<()> {
        if (self.zap_pipe.is_none() || !self.zap_pipe.unwrap().write(msg)) {
            // errno = ENOTCONN;
            // return -1;
            Err(anyhow!("ENOTCONN"))
        }

        if ((msg.flags() & ZMQ_MSG_MORE) == 0) {
            _zap_pipe.flush();
        }

        msg.init2()?;
        // errno_assert (rc == 0);
        // return 0;
        Ok(())
    }

    // ZmqSocketBase *get_socket () const;

    // const EndpointUriPair &get_endpoint () const;
    pub fn get_endpoint(&self) -> &EndpointUriPair {
        self.engine.get_endpoint()
    }

    //  Receives message from ZAP socket.
    //  Returns 0 on success; -1 otherwise.
    //  The caller is responsible for freeing the message.
    // int read_zap_msg (msg: &mut ZmqMessage);
    pub fn read_zap_msg(&mut self, msg: &mut ZmqMessage) -> anyhow::Result<()> {
        if (self.zap_pipe.is_none()) {
            // errno = ENOTCONN;
            // return -1;
            Err(anyhow!("ENOTCONN"))
        }

        if (!self.zap_pipe.unwrap().read(msg)) {
            // errno = EAGAIN;
            // return -1;
            Err(anyhow!("EAGAIN"))
        }

        Ok(())
    }

    //
    // ZmqSessionBase (ZmqIoThread *io_thread_,
    // active_: bool,
    // socket_: *mut ZmqSocketBase,
    // options: &ZmqOptions,
    // Address *addr_);

    // ~ZmqSessionBase () ;
    // ZmqSessionBase::~ZmqSessionBase ()
    // {
    // zmq_assert (!pipe);
    // zmq_assert (!_zap_pipe);
    //
    // //  If there's still a pending linger timer, remove it.
    // if (_has_linger_timer) {
    // cancel_timer (LINGER_TIMER_ID);
    // _has_linger_timer = false;
    // }
    //
    // //  Close the engine.
    // if (_engine)
    // _engine.terminate ();
    //
    // LIBZMQ_DELETE (_addr);
    // }

    //
    // void start_connecting (wait_: bool);

    // void reconnect ();

    //  Handlers for incoming commands.
    // void process_plug () ;

    // void process_attach (ZmqIEngine *engine_) ;

    // void process_term (linger_: i32) ;

    // void process_conn_failed () ;

    //  i_poll_events handlers.
    // void timer_event (id_: i32) ;

    //  Remove any half processed messages. Flush unflushed messages.
    //  Call this function when engine disconnect to get rid of leftovers.
    // void clean_pipes ();
    pub fn clean_pipes(&mut self) -> anyhow::Result<()> {
        // zmq_assert (pipe != null_mut());

        //  Get rid of half-processed messages in the out pipe. Flush any
        //  unflushed messages upstream.
        self.pipe.unwrap().rollback()?;
        self.pipe.unwrap().flush();

        //  Remove any half-read message from the in pipe.
        while (self.incomplete_in) {
            // ZmqMessage msg;
            let mut msg = ZmqMessage::default();
            msg.init2()?;
            // errno_assert (rc == 0);
            self.pull_msg(&mut msg)?;
            // errno_assert (rc == 0);
            msg.close()?;
            // errno_assert (rc == 0);
        }
        Ok(())
    }

    //  Following functions are the interface exposed towards the engine.
    // virtual void reset ();
    pub fn reset(&mut self) {
        if self.reset_fn.is_some() {
            self.reset_fn.unwrap()();
        }
    }

    pub fn write_activated(&mut self, pipe: &mut ZmqPipe) {
        // Skip activating if we're detaching this pipe
        if (pipe != pipe) {
            // zmq_assert (_terminating_pipes.count (pipe) == 1);
            return;
        }

        if (self._engine) {
            self._engine.restart_input();
        }
    }

    pub fn hiccuped(&mut self, pipe: &mut ZmqPipe) {
        //  Hiccups are always sent from session to socket, not the other
        //  way round.
        // zmq_assert (false);
    }

    pub fn get_socket(&mut self) -> &mut ZmqSocket {
        return self._socket;
    }

    pub fn process_plug(&mut self, options: &mut ZmqContext) {
        if (self.active) {
            self.start_connecting(options, false);
        }
    }

    //  This functions can return 0 on success or -1 and errno=ECONNREFUSED if ZAP
    //  is not setup (IE: inproc://zeromq.zap.01 does not exist in the same context)
    //  or it aborts on any other error. In other words, either ZAP is not
    //  configured or if it is configured it MUST be configured correctly and it
    //  MUST work, otherwise authentication cannot be guaranteed and it would be a
    //  security flaw.
    pub fn zap_connect(&mut self) -> i32 {
        if (self._zap_pipe != null_mut()) {
            return 0;
        }

        let peer = self.find_endpoint("inproc://zeromq.zap.01");
        if (peer.socket == null_mut()) {
            errno = ECONNREFUSED;
            return -1;
        }
        // zmq_assert (peer.options.type == ZMQ_REP || peer.options.type == ZMQ_ROUTER
        //             || peer.options.type == ZMQ_SERVER);

        //  Create a bi-directional pipe that will connect
        //  session with zap socket.
        let mut parents: [ZmqObject; 2] = [self, peer.socket];
        let mut new_pipes: [ZmqPipe; 2] = [ZmqPipe::default(); 2];
        let mut hwms: [i32; 2] = [0; 2];
        let mut conflates: [bool; 2] = [bool; 2];
        let rc = pipepair(parents, new_pipes, hwms, conflates);
        // errno_assert (rc == 0);

        //  Attach local end of the pipe to this socket object.
        self._zap_pipe = new_pipes[0].clone();
        self._zap_pipe.set_nodelay();
        self._zap_pipe.set_event_sink(this);

        send_bind(peer.socket, new_pipes[1].clone(), false);

        //  Send empty routing id if required by the peer.
        if (peer.options.recv_routing_id) {
            let mut id: ZmqMessage = ZmqMesssage::default();
            rc = id.init2();
            // errno_assert (rc == 0);
            id.set_flags(ZMQ_MSG_ROUTING_ID);
            let ok = self._zap_pipe.write(&id);
            // zmq_assert (ok);
            self._zap_pipe.flush();
        }

        return 0;
    }

    pub fn zap_enabled(&mut self) -> bool {
        return (self.options.mechanism != ZMQ_NULL || !self.options.zap_domain.empty());
    }

    pub fn process_attach(&mut self, engine: &mut ZmqEngineInterface) {
        // zmq_assert (engine_ != null_mut());
        // zmq_assert (!_engine);
        self._engine = engine_;

        if (!engine_.has_handshake_stage()) {
            engine_ready();
        }

        //  Plug in the engine.
        self._engine.plug(self._io_thread, self);
    }

    pub fn engine_ready(&mut self) {
        //  Create the pipe if it does not exist yet.
        if (!pipe && !is_terminating()) {
            ZmqObject * parents[2] = [self, self._socket];
            ZmqPipe * pipes[2] = [null_mut(), null_mut()];

            let conflate = get_effective_conflate_option(self.options);

            let hwms: [i32; 2] = [
                if conflate { -1 } else { options.rcvhwm },
                if conflate { -1 } else { options.sndhwm },
            ];
            let conflates: [bool; 2] = [conflate, conflate];
            let rc: i32 = pipepair(parents, pipes, hwms, conflates);
            // errno_assert (rc == 0);

            //  Plug the local end of the pipe.
            pipes[0].set_event_sink(this);

            //  Remember the local end of the pipe.
            // zmq_assert (!pipe);
            pipe = pipes[0];

            //  The endpoints strings are not set on Bind, set them here so that
            //  events can use them.
            pipes[0].set_endpoint_pair(_engine.get_endpoint());
            pipes[1].set_endpoint_pair(_engine.get_endpoint());

            //  Ask socket to Plug into the remote end of the pipe.
            send_bind(self._socket, pipes[1]);
        }
    }

    pub fn engine_error(&mut self, handshaked_: bool, reason_: ZmqEngineInterface::ZmqErrorReason) {
        //  Engine is dead. Let's forget about it.
        _engine = null_mut();

        //  Remove any half-Done messages from the pipes.
        if (pipe) {
            clean_pipes();

            //  Only send disconnect message if socket was accepted and handshake was completed
            if (!active_ && handshaked_ && options.can_recv_disconnect_msg && !options.disconnect_msg.empty()) {
                pipe.set_disconnect_msg(options.disconnect_msg);
                pipe.send_disconnect_msg();
            }

            //  Only send Hiccup message if socket was connected and handshake was completed
            if (active_ && handshaked_ && options.can_recv_hiccup_msg && !options.hiccup_msg.empty()) {
                pipe.send_hiccup_msg(options.hiccup_msg);
            }
        }

        // zmq_assert (reason_ == ZmqIEngine::connection_error
        // || reason_ == ZmqEngineInterface::timeout_error
        //     || reason_ == ZmqEngineInterface::protocol_error);

        match (reason_) {
            ZmqEngineInterface::timeout_error | /* FALLTHROUGH */
            ZmqEngineInterface::connection_error => {
                if (active_) {
                    reconnect();
                }
            }

            ZmqEngineInterface::protocol_error => {
                if (_pending) {
                    if (pipe) {
                        pipe.terminate(false);
                    }
                    if (_zap_pipe) {
                        _zap_pipe.terminate(false);
                    }
                } else {
                    terminate();
                }
            }
        }

        //  Just in case there's only a delimiter in the pipe.
        if (pipe) {
            pipe.check_read();
        }

        if (_zap_pipe) {
            _zap_pipe.check_read();
        }
    }

    pub fn process_term(&mut self, linger: i32) {
        // zmq_assert (!_pending);

        //  If the termination of the pipe happens before the Term command is
        //  delivered there's nothing much to do. We can proceed with the
        //  standard termination immediately.
        if (!pipe && !_zap_pipe && _terminating_pipes.empty()) {
            self.process_term(0);
            return;
        }

        self._pending = true;

        if (pipe != null_mut()) {
            //  If there's finite linger value, delay the termination.
            //  If linger is infinite (negative) we don't even have to set
            //  the timer.
            if (linger > 0) {
                // zmq_assert (!_has_linger_timer);
                add_timer(linger, LINGER_TIMER_ID);
                self._has_linger_timer = true;
            }

            //  Start pipe termination process. Delay the termination till all messages
            //  are processed in case the linger time is non-zero.
            pipe.terminate(linger != 0);

            //  TODO: Should this go into ZmqPipe::terminate ?
            //  In case there's no engine and there's only delimiter in the
            //  pipe it wouldn't be ever read. Thus we check for it explicitly.
            if (!self._engine) {
                pipe.check_read();
            }
        }

        if (self._zap_pipe != null_mut()) {
            self._zap_pipe.terminate(false);
        }
    }

    pub fn timer_event(&mut self, id_: i32) {
        //  Linger period expired. We can proceed with termination even though
        //  there are still pending messages to be sent.
        // zmq_assert (id_ == LINGER_TIMER_ID);
        self._has_linger_timer = false;

        //  Ask pipe to terminate even though there may be pending messages in it.
        // zmq_assert (pipe);
        pipe.terminate(false);
    }

    pub fn process_conn_failed(&mut self) {
        let mut ep = String::new();
        self._addr.to_string(&mut ep);
        send_term_endpoint(self._socket, ep);
    }

    pub fn reconnect(&mut self) {
        //  For delayed connect situations, terminate the pipe
        //  and reestablish later on
        if (self.pipe.is_some() && self.options.immediate == 1) {
            // #ifdef ZMQ_HAVE_OPENPGM && self._addr.protocol != protocol_name::pgm && self._addr.protocol != protocol_name::epgm
            // #endif
            // #ifdef ZMQ_HAVE_NORM && self._addr.protocol != protocol_name::norm
            // #endif && self._addr.protocol != protocol_name::udp) {
            self.pipe.unwrap().hiccup();
            self.pipe.terminate(false);
            self._terminating_pipes.insert(&mut self.pipe);
            self.pipe = None;

            if (self._has_linger_timer) {
                cancel_timer(LINGER_TIMER_ID);
                self._has_linger_timer = false;
            }
        }

        reset();

        //  Reconnect.
        if (self.options.reconnect_ivl > 0) {
            start_connecting(true);
        } else {
            std::string * ep = new(std::string);
            self._addr.to_string(*ep);
            send_term_endpoint(self._socket, ep);
        }

        //  For subscriber sockets we Hiccup the inbound pipe, which will cause
        //  the socket object to resend all the subscriptions.
        if (self.pipe.is_some() && (self.options.type_ == ZMQ_SUB || self.options.type_ == ZMQ_XSUB || self.options.type_ == ZMQ_DISH)) {
            pipe.hiccup();
        }
    }

    pub fn start_connecting(&mut self, options: &mut ZmqContext, wait_: bool) {
        // zmq_assert (active);

        //  Choose I/O thread to run connecter in. Given that we are already
        //  running in an I/O thread, there must be at least one available.
        let io_thread = self.choose_io_thread(self.options.affinity);
        // zmq_assert (io_thread);

        //  Create the connecter object.
        ZmqOwn * connecter = null_mut();
        if (_addr.protocol == ZmqTransport::ZmqTcp) {
            if (!options.socks_proxy_address.empty()) {
                let mut proxy_address = ZmqAddress::new(
                    ZmqTransport::ZmqTcp,
                    self.options.socks_proxy_address,
                    this.get_ctx(),
                );
                // alloc_assert (proxy_address);
                connecter = ZmqSocksConnector::new(io_thread, this, options, _addr, proxy_address, wait_);
                // alloc_assert (connecter);
                if (!options.socks_proxy_username.empty()) {
                    (connecter).set_auth_method_basic(
                        &mut options.socks_proxy_username,
                        &mut options.socks_proxy_password,
                    );
                }
            } else {
                connecter = ZmqTcpConnector::new(io_thread, this, options, _addr, wait_);
            }
        }
        // #if defined ZMQ_HAVE_IPC
        else if (_addr.protocol == ZmqTransport::ZmqIpc) {
            connecter = IpcConnecter::new(options, io_thread, this, _addr, wait_);
        }
        // #endif
        // #if defined ZMQ_HAVE_TIPC
        else if (_addr.protocol == ZmqTransport::ZmqTipc) {
            connecter = ZmqTipcConnecter::new(io_thread, this, options, _addr, wait_);
        }
        // #endif
        // #if defined ZMQ_HAVE_VMCI
        else if (_addr.protocol == ZmqTransport::ZmqVmci) {
            connecter = ZmqVmciConnecter::new(io_thread, this, options, _addr, wait_);
        }
        // #endif
        // #if defined ZMQ_HAVE_WS
        else if (_addr.protocol == ZmqTransport::ZmqWs) {
            connecter = ZmqWsConnecter::new(io_thread, this, options, _addr, wait_, false, std::string());
        }
        // #endif
        // #if defined ZMQ_HAVE_WSS
        else if (_addr.protocol == ZmqTransport::ZmqWss) {
            connecter = ZmqWsConnecter::new(io_thread, this, options, _addr, wait_, true, _wss_hostname);
        }
        // #endif
        if (connecter != null_mut()) {
            // alloc_assert (connecter);
            launch_child(connecter);
            return;
        }

        if (_addr.protocol == protocol_name::udp) {
            // zmq_assert (options.type == ZMQ_DISH || options.type == ZMQ_RADIO
            // || options.type_ == ZMQ_DGRAM);

            ZmqUdpEngine * engine = ZmqUdpEngine::new(options);
            // alloc_assert (engine);

            let mut recv = false;
            let mut send = false;

            if (options.type_ == ZMQ_RADIO) {
                send = true;
                recv = false;
            } else if (options.type_ == ZMQ_DISH) {
                send = false;
                recv = true;
            } else if (options.type_ == ZMQ_DGRAM) {
                send = true;
                recv = true;
            }

            let rc = engine.init(_addr, send, recv);
            // errno_assert (rc == 0);

            send_attach(this, engine);

            return;
        }

        // #ifdef ZMQ_HAVE_OPENPGM

        //  Both PGM and EPGM transports are using the same infrastructure.
        if (self._addr.protocol == "pgm" || self._addr.protocol == "epgm") {
            // zmq_assert (options.type == ZMQ_PUB || options.type == ZMQ_XPUB
            // || options.type_ == ZMQ_SUB || options.type_ == ZMQ_XSUB);

            //  For EPGM transport with UDP encapsulation of PGM is used.
            let udp_encapsulation = self._addr.protocol == "epgm";

            //  At this point we'll create message pipes to the session straight
            //  away. There's no point in delaying it as no concept of 'connect'
            //  exists with PGM anyway.
            if (options.type_ == ZMQ_PUB || options.type_ == ZMQ_XPUB) {
                //  PGM sender.
                let pgm_sender = pgm_sender_t::new(io_thread, options);
                // alloc_assert (pgm_sender);

                let rc = pgm_sender.init(udp_encapsulation, _addr.address.c_str());
                // errno_assert (rc == 0);

                send_attach(this, pgm_sender);
            } else {
                //  PGM receiver.
                let pgm_receiver = ZmqPgmReceiver::new(io_thread, options);
                // alloc_assert (pgm_receiver);

                let rc = pgm_receiver.init(udp_encapsulation, _addr.address.c_str());
                // errno_assert (rc == 0);

                send_attach(this, pgm_receiver);
            }

            return;
        }
        // #endif

        // #ifdef ZMQ_HAVE_NORM
        if (self._addr.protocol == "norm") {
            //  At this point we'll create message pipes to the session straight
            //  away. There's no point in delaying it as no concept of 'connect'
            //  exists with NORM anyway.
            if (options.type_ == ZMQ_PUB || options.type_ == ZMQ_XPUB) {
                //  NORM sender.
                let norm_sender = NormEngine::new(options, io_thread);
                // alloc_assert (norm_sender);

                let rc = norm_sender.init(_addr.address, true, false);
                // errno_assert (rc == 0);

                send_attach(this, norm_sender);
            } else {
                // ZMQ_SUB or ZMQ_XSUB

                //  NORM receiver.
                let norm_receiver = NormEngine::new(options, io_thread);
                // alloc_assert (norm_receiver);

                let rc = norm_receiver.init(_addr.address, false, true);
                // errno_assert (rc == 0);

                send_attach(this, norm_receiver);
            }
            return;
        }
        // #endif // ZMQ_HAVE_NORM

        // zmq_assert (false);
    }
} // end of impl SessionBase

// pub struct hello_msg_session_t  : public ZmqSessionBase
#[derive(Default, Debug, Clone)]
pub struct ZmqHelloMsgSession {
    //
    pub new_pipe: bool,
    pub session_base: ZmqSessionBase,
    // // ZMQ_NON_COPYABLE_NOR_MOVABLE (hello_msg_session_t)
}

impl ZmqHelloMsgSession {
    //
    // hello_msg_session_t (ZmqIoThread *io_thread_,
    // connect_: bool,
    // socket_: *mut ZmqSocketBase,
    // options: &ZmqOptions,
    // Address *addr_);

    // ~hello_msg_session_t ();

    //  Overrides of the functions from ZmqSessionBase.
    // int pull_msg (msg: &mut ZmqMessage);

    // void reset ();
    pub fn new(
        ctx: &mut ZmqContext,
        io_thread_: &mut ZmqThreadContext,
        connect_: bool,
        socket: &mut ZmqSocket,
        options: &mut ZmqContext,
        addr_: &mut ZmqAddress,
    ) -> Self {
        //  :
        //     ZmqSessionBase (io_thread_, connect_, socket, options_, addr_),
        //     _new_pipe (true)
        Self {
            new_pipe: true,
            session_base: ZmqSessionBase::new(ctx, io_thread_, connect_, socket, options, addr_),
        }
    }

    // hello_msg_session_t::~hello_msg_session_t ()
    // {
    // }

    pub fn new_pull_msg(&mut self, msg: &mut ZmqMessage) -> i32 {
        if (_new_pipe) {
            _new_pipe = false;

            let rc: i32 = msg.init_buffer(&mut options.hello_msg[0], options.hello_msg.size());
            // errno_assert (rc == 0);

            return 0;
        }

        return self.pull_msg(msg);
    }

    pub fn reset(&mut self) {
        self.session_base.reset();
        self._new_pipe = true;
    }
}
