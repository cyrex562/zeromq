use crate::address::Address;
use crate::clock::clock_t;
use crate::command::ZmqCommand;
use crate::context::ZmqContext;
use crate::endpoint::EndpointType::endpoint_type_none;
use crate::endpoint::{make_unconnected_bind_endpoint_pair, EndpointUriPair, ZmqEndpoint};
use crate::i_mailbox::i_mailbox;
use crate::ipc_address::IpcAddress;
use crate::ipc_listener::ipc_listener_t;
use crate::mailbox::mailbox_t;
use crate::mailbox_safe::mailbox_safe_t;
use crate::message::{routing_id, ZmqMessage, ZMQ_MSG_MORE};
use crate::object::object_t;
use crate::options::{get_effective_conflate_option, ZmqOptions};
use crate::out_pipe::out_pipe_t;
use crate::own::own_t;
use crate::pgm_socket::pgm_socket_t;
use crate::pipe::pipe_t;
use crate::session_base::session_base_t;
use crate::signaler::signaler_t;
use crate::tcp_address::TcpAddress;
use crate::tcp_listener::tcp_listener_t;
use crate::tipc_address::TipcAddress;
use crate::tipc_listener::tipc_listener_t;
use crate::udp_address::UdpAddress;
use crate::vmci_address::VmciAddress;
use crate::vmci_listener::vmci_listener_t;
use crate::ws_address::WsAddress;
use crate::ws_listener::ws_listener_t;
use crate::wss_address::WssAddress;
use crate::zmq_hdr::{
    ZMQ_BLOCKY, ZMQ_CONNECT_ROUTING_ID, ZMQ_DEALER, ZMQ_DISH, ZMQ_DONTWAIT, ZMQ_EVENTS,
    ZMQ_EVENT_ACCEPTED, ZMQ_EVENT_ACCEPT_FAILED, ZMQ_EVENT_BIND_FAILED, ZMQ_EVENT_CLOSED,
    ZMQ_EVENT_CLOSE_FAILED, ZMQ_EVENT_CONNECTED, ZMQ_EVENT_CONNECT_DELAYED,
    ZMQ_EVENT_CONNECT_RETRIED, ZMQ_EVENT_DISCONNECTED, ZMQ_EVENT_HANDSHAKE_FAILED_AUTH,
    ZMQ_EVENT_HANDSHAKE_FAILED_NO_DETAIL, ZMQ_EVENT_HANDSHAKE_FAILED_PROTOCOL,
    ZMQ_EVENT_HANDSHAKE_SUCCEEDED, ZMQ_EVENT_LISTENING, ZMQ_EVENT_MONITOR_STOPPED,
    ZMQ_EVENT_PIPES_STATS, ZMQ_FD, ZMQ_IPV6, ZMQ_LAST_ENDPOINT, ZMQ_LINGER, ZMQ_POLLIN,
    ZMQ_POLLOUT, ZMQ_PUB, ZMQ_RADIO, ZMQ_RCVHWM, ZMQ_RCVMORE, ZMQ_RECONNECT_STOP_AFTER_DISCONNECT,
    ZMQ_REQ, ZMQ_SNDHWM, ZMQ_SNDMORE, ZMQ_SUB, ZMQ_THREAD_SAFE, ZMQ_XPUB, ZMQ_XSUB,
    ZMQ_ZERO_COPY_RECV,
};
use crate::zmq_ops::{
    zmq_bind, zmq_close, zmq_errno, zmq_msg_data, zmq_msg_init_size, zmq_msg_send, zmq_setsockopt,
    zmq_socket,
};
use anyhow::bail;
use libc::{c_void, EAGAIN, EINTR, EINVAL};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::mem;
use std::ptr::null_mut;
use std::sync::atomic::Ordering;
use std::sync::Mutex;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ZmqSocketBase {
    pub own: own_t,
    // Mutex for synchronize access to the socket in thread safe mode
    pub sync: Mutex<u8>,
    //  Map of open endpoints.
    // typedef std::pair<own_t *, pipe_t *> endpoint_pipe_t;
    // typedef std::multimap<std::string, endpoint_pipe_t> endpoints_t;
    // endpoints_t _endpoints;
    pub endpoints: HashMap<String, (own_t, pipe_t)>,
    //  Used to check whether the object is a socket.
    pub tag: u32,

    //  If true, associated context was already terminated.
    pub ctx_terminated: bool,

    //  If true, object should have been already destroyed. However,
    //  destruction is delayed while we unwind the stack to the point
    //  where it doesn't intersect the object being destroyed.
    pub destroyed: bool,

    //  Socket's mailbox object.
    // i_mailbox *_mailbox;
    pub mailbox: Option<i_mailbox>,

    //  List of attached pipes.
    // typedef array_t<pipe_t, 3> pipes_t;
    // pipes_t _pipes;
    pub pipes: [pipe_t; 3],

    //  Reaper's poller and handle of this socket within it.
    // poller_t *_poller;
    pub poller: Option<poller_t>,
    // poller_t::handle_t _handle;
    pub handle: Option<handle_t>,

    //  Timestamp of when commands were processed the last time.
    pub last_tsc: u64,

    //  Number of messages received since last command processing.
    pub ticks: i32,

    //  True if the last message received had MORE flag set.
    pub rcvmore: bool,

    //  Improves efficiency of time measurement.
    pub clock: clock_t,

    // Monitor socket;
    pub monitor_socket: Vec<u8>,

    // Bitmask of events being monitored
    pub monitor_events: i64,

    // Last socket endpoint resolved URI
    pub last_endpoint: String,

    // Indicate if the socket is thread safe
    pub thread_safe: bool,

    // Signaler to be used in the reaping stage
    pub reaper_signaler: Option<signaler_t>,

    // Mutex to synchronize access to the monitor Pair socket
    pub monitor_sync: Mutex<u8>,

    // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqSocketBase)

    // Add a flag for mark disconnect action
    pub disconnected: bool,
}

impl ZmqSocketBase {
    // ZmqSocketBase (ZmqContext *parent_,
    //                uint32_t tid_,
    //                sid_: i32,
    //                bool thread_safe_ = false);
    // ~ZmqSocketBase () ZMQ_OVERRIDE;
    // ZmqSocketBase (ZmqContext *parent_,
    //                                    u32 tid_,
    //                                    sid_: i32,
    //                                    thread_safe_: bool) :
    //     own_t (parent_, tid_),
    //     _sync (),
    //     _tag (0xbaddecaf),
    //     _ctx_terminated (false),
    //     _destroyed (false),
    //     _poller (null_mut()),
    //     _handle (static_cast<poller_t::handle_t> (null_mut())),
    //     _last_tsc (0),
    //     _ticks (0),
    //     _rcvmore (false),
    //     _monitor_socket (null_mut()),
    //     _monitor_events (0),
    //     _thread_safe (thread_safe_),
    //     _reaper_signaler (null_mut()),
    //     _monitor_sync ()
    pub fn new(
        parent: &mut ZmqContext,
        options: &mut ZmqOptions,
        tid_: u32,
        sid_: i32,
        thread_safe_: bool,
    ) -> Self {
        let mut out = Self::default();
        out._tag = 0xbaddecafu32;
        out.ctx_terminated = false;
        out.destroyed = false;
        out.last_tsc = 0;
        out.ticks = 0;
        out.rcvmore = false;
        out.monitor_socket = vec![];
        out.monitor_events = 0;
        out.thread_safe = thread_safe_;
        out.reaper_signaler = None;
        out.monitor_sync = Mutex::new(0);
        out.poller = None;
        out.sync = Mutex::new(0);
        out.handle = None;

        options.socket_id = sid_;
        options.ipv6 = (parent_.get(ZMQ_IPV6) != 0);
        options.linger.store(
            if parent.get(ZMQ_BLOCKY) { -1 } else { 0 },
            Ordering::Relaxed,
        );
        options.zero_copy = parent_.get(ZMQ_ZERO_COPY_RECV) != 0;

        if out.thread_safe {
            out.mailbox = mailbox_safe_t::new(&out.sync);
            // zmq_assert (_mailbox);
        } else {
            let mut m = mailbox_t::new();
            // zmq_assert (m);

            if m.get_fd() != retired_fd {
                out.mailbox = m;
            } else {
                // LIBZMQ_DELETE (m);
                out.mailbox = None;
            }
        }
        out
    }

    //  Returns false if object is not a socket.
    // bool check_tag () const;
    pub fn check_tag(&self) -> bool {
        return self._tag == 0xbaddecafu32;
    }

    //  Returns whether the socket is thread-safe.
    // bool is_thread_safe () const;
    pub fn is_thread_safe(&self) -> bool {
        return self.thread_safe;
    }

    //  Create a socket of a specified type.
    // static ZmqSocketBase * create (type_: i32, ZmqContext *parent_, uint32_t tid_, sid_: i32);

    // pub fn create (type_: i32, parent_: &mut ZmqContext, tid_: u32, sid_: i32) -> anyhow::Result<Self>
    // {
    //     let mut s: Self;
    //     // ZmqSocketBase *s = NULL;
    //     match type_ {
    //         ZMQ_PAIR => { s = Self::new(parent_, tid_, sid_); }
    //             // s = new (std::nothrow) pair_t (parent_, tid_, sid_);
    //             // break;
    //         ZMQ_PUB =>{}
    //             // s = new (std::nothrow) pub_t (parent_, tid_, sid_);
    //             // break;
    //         ZMQ_SUB =>
    //             s = new (std::nothrow) sub_t (parent_, tid_, sid_);
    //             break;
    //         ZMQ_REQ =>
    //             s = new (std::nothrow) req_t (parent_, tid_, sid_);
    //             break;
    //         ZMQ_REP =>
    //             s = new (std::nothrow) rep_t (parent_, tid_, sid_);
    //             break;
    //         ZMQ_DEALER =>
    //             s = new (std::nothrow) dealer_t (parent_, tid_, sid_);
    //             break;
    //         ZMQ_ROUTER =>
    //             s = new (std::nothrow) router_t (parent_, tid_, sid_);
    //             break;
    //         ZMQ_PULL =>
    //             s = new (std::nothrow) pull_t (parent_, tid_, sid_);
    //             break;
    //         ZMQ_PUSH =>
    //             s = new (std::nothrow) push_t (parent_, tid_, sid_);
    //             break;
    //         ZMQ_XPUB =>
    //             s = new (std::nothrow) xpub_t (parent_, tid_, sid_);
    //             break;
    //         ZMQ_XSUB =>
    //             s = new (std::nothrow) xsub_t (parent_, tid_, sid_);
    //             break;
    //         ZMQ_STREAM =>
    //             s = new (std::nothrow) stream_t (parent_, tid_, sid_);
    //             break;
    //         ZMQ_SERVER =>
    //             s = new (std::nothrow) server_t (parent_, tid_, sid_);
    //             break;
    //         ZMQ_CLIENT =>
    //             s = new (std::nothrow) client_t (parent_, tid_, sid_);
    //             break;
    //         ZMQ_RADIO =>
    //             s = new (std::nothrow) radio_t (parent_, tid_, sid_);
    //             break;
    //         ZMQ_DISH =>
    //             s = new (std::nothrow) dish_t (parent_, tid_, sid_);
    //             break;
    //         ZMQ_GATHER =>
    //             s = new (std::nothrow) gather_t (parent_, tid_, sid_);
    //             break;
    //         ZMQ_SCATTER =>
    //             s = new (std::nothrow) scatter_t (parent_, tid_, sid_);
    //             break;
    //         ZMQ_DGRAM =>
    //             s = new (std::nothrow) dgram_t (parent_, tid_, sid_);
    //             break;
    //         ZMQ_PEER =>
    //             s = new (std::nothrow) peer_t (parent_, tid_, sid_);
    //             break;
    //         ZMQ_CHANNEL =>
    //             s = new (std::nothrow) channel_t (parent_, tid_, sid_);
    //             break;
    //         default:
    //             errno = EINVAL;
    //             return NULL;
    //     }
    //
    //     // alloc_assert (s);
    //
    //     if (s->_mailbox == NULL) {
    //         s->_destroyed = true;
    //         LIBZMQ_DELETE (s);
    //         return NULL;
    //     }
    //
    //     return s;
    // }

    //  Returns the mailbox associated with this socket.
    // i_mailbox *get_mailbox () const;
    // i_mailbox *get_mailbox () const
    pub fn get_mailbox(&mut self) -> Option<i_mailbox> {
        return self.mailbox.clone();
    }

    //  Interrupt blocking call if the socket is stuck in one.
    //  This function can be called from a different thread!
    // void stop ();
    pub fn stop(&mut self) {
        //  Called by ctx when it is terminated (zmq_ctx_term).
        //  'stop' command is sent from the threads that called zmq_ctx_term to
        //  the thread owning the socket. This way, blocking call in the
        //  owner thread can be interrupted.
        self.send_stop();
    }

    //  Interface for communication with the API layer.
    // int setsockopt (option_: i32, const optval_: *mut c_void, optvallen_: usize);
    pub fn setsockopt(
        &mut self,
        options: &mut ZmqOptions,
        opt_kind: i32,
        opt_val: &mut [u8],
        opt_len: usize,
    ) -> anyhow::Result<()> {
        // scoped_optional_lock_t sync_lock (_thread_safe ? &_sync : NULL);
        // let mut sync_lock = if self._thread_safe {
        //     &mut _sync
        // } else {
        //     null_mut()
        // };

        // if (unlikely (_ctx_terminated)) {
        //     errno = ETERM;
        //     return -1;
        // }

        //  First, check whether specific socket type overloads the option.
        self.xsetsockopt(opt_kind, opt_val, opt_len)?;

        //  If the socket type doesn't support the option, pass it to
        //  the generic option parser.
        options.setsockopt(opt_kind, opt_val, opt_len)?;
        self.update_pipe_options(options, opt_kind)?;
        Ok(())
    }

    // int getsockopt (option_: i32, optval_: *mut c_void, optvallen_: *mut usize);
    pub fn getsockopt(
        &mut self,
        options: &mut ZmqOptions,
        opt_kind: i32,
        opt_val: &mut [u8],
        opt_len: &mut usize,
    ) -> anyhow::Result<()> {
        // scoped_optional_lock_t sync_lock (_thread_safe ? &_sync : NULL);
        let mut sync_lock = if self.thread_safe {
            &mut _sync
        } else {
            null_mut()
        };
        // if (unlikely (_ctx_terminated)) {
        //     errno = ETERM;
        //     return -1;
        // }

        //  First, check whether specific socket type overloads the option.
        let rc = self.xgetsockopt(opt_kind, opt_val, *opt_len);
        if rc == 0 || errno != EINVAL {
            return rc;
        }

        if opt_kind == ZMQ_RCVMORE {
            return do_getsockopt_int(opt_val, opt_len, if self.rcvmore { 1 } else { 0 });
        }

        if opt_kind == ZMQ_FD {
            if self.thread_safe {
                // thread safe socket doesn't provide file descriptor
                // errno = EINVAL;
                // return -1;
                bail!("thread safe socket doenst provide a file descriptor")
            }

            return do_getsockopt_fd(opt_val, opt_len, self.mailbox.get_fd());
        }

        if opt_kind == ZMQ_EVENTS {
            self.process_commands(0, false)?;
            // if rc != 0 && (errno == EINTR || errno == ETERM) {
            //     return -1;
            // }
            // errno_assert (rc == 0);

            return do_getsockopt_int(
                opt_val,
                opt_len,
                (if self.has_out() { ZMQ_POLLOUT } else { 0 })
                    | (if self.has_in() { ZMQ_POLLIN } else { 0 }),
            );
        }

        if opt_kind == ZMQ_LAST_ENDPOINT {
            return do_getsockopt_string(opt_val, opt_len, &self.last_endpoint);
        }

        if opt_kind == ZMQ_THREAD_SAFE {
            return do_getsockopt_int(opt_val, opt_len, if self.thread_safe { 1 } else { 0 });
        }

        options.getsockopt(opt_kind, opt_val, opt_len)
    }

    // int bind (endpoint_uri_: *const c_char);
    pub fn bind(&mut self, options: &mut ZmqOptions, endpoint_uri_: &str) -> anyhow::Result<()> {
        // scoped_optional_lock_t sync_lock (_thread_safe ? &_sync : NULL);

        // if (unlikely (_ctx_terminated)) {
        //     errno = ETERM;
        //     return -1;
        // }

        //  Process pending commands, if any.
        self.process_commands(0, false)?;
        // if (unlikely (rc != 0)) {
        //     return -1;
        // }

        //  Parse endpoint_uri_ string.
        let mut protocol: String = String::new();
        let mut address: String = String::new();
        if self.parse_uri(endpoint_uri_, &mut protocol, &mut address)
            || self.check_protocol(options, &protocol)
        {
            bail!("failed to parse endpoint_uri")
        }

        if protocol == protocol_name::inproc {
            // const ZmqEndpoint endpoint = {this, options};
            let mut endpoint: ZmqEndpoint = ZmqEndpoint::default();
            endpoint.options = options.clone();
            self.register_endpoint(endpoint_uri_, endpoint)?;
            self.connect_pending(endpoint_uri_, this);
            self.last_endpoint.assign(endpoint_uri_);
            options.connected = true;
            return Ok(());
        }

        // #if defined ZMQ_HAVE_OPENPGM || defined ZMQ_HAVE_NORM
        // #if defined ZMQ_HAVE_OPENPGM && defined ZMQ_HAVE_NORM
        if protocol == protocol_name::pgm
            || protocol == protocol_name::epgm
            || protocol == protocol_name::norm
        {
            // #elif defined ZMQ_HAVE_OPENPGM
            //     if (protocol == protocol_name::pgm || protocol == protocol_name::epgm) {
            // // #else // defined ZMQ_HAVE_NORM
            //     if (protocol == protocol_name::norm) {
            // #endif
            //  For convenience's sake, bind can be used interchangeable with
            //  connect for PGM, EPGM, NORM transports.
            self.connect(options, endpoint_uri_)?;
            options.connected = true;
            return Ok(());
            // if rc != -1 {
            //
            // }
            // return rc;
        }
        // #endif

        if protocol == protocol_name::udp {
            if !(options.type_ == ZMQ_DGRAM || options.type_ == ZMQ_DISH) {
                // errno = ENOCOMPATPROTO;
                // return -1;
                bail!("no compatible protocol")
            }

            //  Choose the I/O thread to run the session in.
            let io_thread = self.choose_io_thread(options.affinity);
            if !io_thread {
                // errno = EMTHREAD;
                // return -1;
                bail!("EMTHREAD")
            }

            // Address *paddr =
            //   new (std::nothrow) Address (protocol, address, this->get_ctx ());
            // alloc_assert (paddr);
            let mut paddr = Address::new(&protocol, &address, self.get_ctx());

            paddr.resolved.udp_addr = UdpAddress::new();
            // alloc_assert (paddr.resolved.udp_addr);
            paddr
                .resolved
                .udp_addr
                .resolve(&address, true, options.ipv6)?;
            // if rc != 0 {
            //     // LIBZMQ_DELETE (paddr);
            //     return -1;
            // }

            let session = session_base_t::create(io_thread, true, this, options, paddr);
            // errno_assert (session);

            //  Create a bi-directional pipe.
            let mut parents: [object_t; 2] = [this, session];
            let mut new_pipes: [pipe_t; 2] = [pipe_t::default(), pipe_t::default()];

            let mut hwms: [i32; 2] = [options.sndhwm, options.rcvhwm];
            let mut conflates: [nool; 2] = [false, false];
            self.pipepair(parents, new_pipes, hwms, conflates)?;
            // errno_assert (rc == 0);

            //  Attach local end of the pipe to the socket object.
            self.attach_pipe(&mut new_pipes[0], true, true)?;
            let mut newpipe = new_pipes[0].clone();

            //  Attach remote end of the pipe to the session object later on.
            session.attach_pipe(&mut new_pipes[1]);

            //  Save last endpoint URI
            paddr.to_string(&self.last_endpoint);

            //  TODO shouldn't this use _last_endpoint instead of endpoint_uri_? as in the other cases
            let mut ep = EndpointUriPair::new(endpoint_uri_, "", endpoint_type_none);
            self.add_endpoint(&ep, session, &mut newpipe);

            Ok(())
        }

        //  Remaining transports require to be run in an I/O thread, so at this
        //  point we'll choose one.
        let io_thread = choose_io_thread(options.affinity);
        if !io_thread {
            // errno = EMTHREAD;
            // return -1;
            bail!("EMTHREAD")
        }

        if protocol == protocol_name::tcp {
            let mut listener = tcp_listener_t::new(io_thread, this, options);
            // alloc_assert (listener);
            if listener.set_local_address(address.c_str()).is_err() {
                // LIBZMQ_DELETE (listener);
                event_bind_failed(make_unconnected_bind_endpoint_pair(&address), zmq_errno());
                bail!("failed to set local address")
            }

            // Save last endpoint URI
            listener.get_local_address(&self.last_endpoint);

            add_endpoint(
                make_unconnected_bind_endpoint_pair(_last_endpoint),
                listener,
                null_mut(),
            );
            options.connected = true;
            return Ok(());
        }

        // #ifdef ZMQ_HAVE_WS
        // #ifdef ZMQ_HAVE_WSS
        if protocol == protocol_name::ws || protocol == protocol_name::wss {
            let listener =
                ws_listener_t::new(io_thread, self, options, protocol == protocol_name::wss);
            // #else
            //     if protocol == protocol_name::ws {
            let listener = ws_listener_t::new(io_thread, self, options, false);
            // #endif
            //         alloc_assert (listener);
            if listener.set_local_address(address.c_str()).is_err() {
                // LIBZMQ_DELETE (listener);
                event_bind_failed(make_unconnected_bind_endpoint_pair(&address), zmq_errno());
                bail!("failed to set local address")
            }

            // Save last endpoint URI
            listener.get_local_address(&self.last_endpoint);

            add_endpoint(
                make_unconnected_bind_endpoint_pair(_last_endpoint),
                (listener),
                null_mut(),
            );
            options.connected = true;
            return Ok(());
        }
        // #endif

        // #if defined ZMQ_HAVE_IPC
        if protocol == protocol_name::ipc {
            let listener = ipc_listener_t::new(io_thread, this, options);
            // alloc_assert (listener);
            if listener.set_local_address(address.c_str()).is_err() {
                // LIBZMQ_DELETE (listener);
                event_bind_failed(make_unconnected_bind_endpoint_pair(&address), zmq_errno());
                bail!("failed to set local address");
            }

            // Save last endpoint URI
            listener.get_local_address(_last_endpoint);

            add_endpoint(
                make_unconnected_bind_endpoint_pair(_last_endpoint),
                (listener),
                null_mut(),
            );
            options.connected = true;
            return Ok(());
        }
        // #endif
        // #if defined ZMQ_HAVE_TIPC
        if protocol == protocol_name::tipc {
            listener = tipc_listener_t::new(io_thread, this, options);
            // alloc_assert (listener);
            if listener.set_local_address(address.c_str()).is_err() {
                event_bind_failed(make_unconnected_bind_endpoint_pair(&address), zmq_errno());
                bail!("failed to set local address");
            }

            // Save last endpoint URI
            listener.get_local_address(_last_endpoint);

            // TODO shouldn't this use _last_endpoint as in the other cases?
            add_endpoint(
                make_unconnected_bind_endpoint_pair(endpoint_uri_),
                (listener),
                null_mut(),
            );
            options.connected = true;
            return Ok(());
        }
        // #endif
        // #if defined ZMQ_HAVE_VMCI
        if protocol == protocol_name::vmci {
            let listener = vmci_listener_t::new(io_thread, this, options);
            // alloc_assert (listener);
            if listener.set_local_address(address.c_str()).is_err() {
                // LIBZMQ_DELETE (listener);
                event_bind_failed(make_unconnected_bind_endpoint_pair(&address), zmq_errno());
                bail!("failed to set local address");
            }

            listener.get_local_address(&self.last_endpoint);

            add_endpoint(
                make_unconnected_bind_endpoint_pair(_last_endpoint),
                (listener),
                null_mut(),
            );
            options.connected = true;
            return Ok(());
        }
        // #endif

        // zmq_assert (false);
        bail!("bind failed")
    }

    // int connect (endpoint_uri_: *const c_char);
    pub fn connect(&mut self, options: &mut ZmqOptions, endpoint_uri_: &str) -> anyhow::Result<()> {
        // scoped_optional_lock_t sync_lock (_thread_safe ? &_sync : null_mut());
        self.connect_internal(options, endpoint_uri_)
    }

    // int term_endpoint (endpoint_uri_: *const c_char);
    pub fn term_endpoint(&mut self, options: &mut ZmqOptions, endpoint_uri_: &str) -> i32 {
        // scoped_optional_lock_t sync_lock (_thread_safe ? &_sync : null_mut());

        //  Check whether the context hasn't been shut down yet.
        // if (unlikely (_ctx_terminated)) {
        //     errno = ETERM;
        //     return -1;
        // }

        //  Check whether endpoint address passed to the function is valid.
        // if (unlikely (!endpoint_uri_)) {
        //     errno = EINVAL;
        //     return -1;
        // }

        //  Process pending commands, if any, since there could be pending unprocessed process_own()'s
        //  (from launch_child() for example) we're asked to terminate now.
        let rc: i32 = process_commands(0, false);
        // if (unlikely (rc != 0)) {
        //     return -1;
        // }

        //  Parse endpoint_uri_ string.
        let mut uri_protocol = String::new();
        let mut uri_path = String::new();
        if parse_uri(endpoint_uri_, uri_protocol, uri_path) || check_protocol(&uri_protocol) {
            return -1;
        }

        let mut endpoint_uri_str = String::from(endpoint_uri_);

        // Disconnect an inproc socket
        if uri_protocol == protocol_name::inproc {
            return if unregister_endpoint(endpoint_uri_str, self) == 0 {
                0
            } else {
                self._inprocs.erase_pipes(&endpoint_uri_str)
            };
        }

        let resolved_endpoint_uri = if uri_protocol == protocol_name::tcp {
            resolve_tcp_addr(endpoint_uri_str, uri_path.c_str())
        } else {
            endpoint_uri_str
        };

        //  Find the endpoints range (if any) corresponding to the endpoint_uri_pair_ string.
        // const std::pair<endpoints_t::iterator, endpoints_t::iterator> range =
        //   _endpoints.equal_range (resolved_endpoint_uri);
        // if (range.first == range.second) {
        //     errno = ENOENT;
        //     return -1;
        // }
        if self.endpoints.is_empty() {
            return -1;
        }

        // for (endpoints_t::iterator it = range.first; it != range.second; ++it) {
        //     //  If we have an associated pipe, terminate it.
        //     if (it->second.second != null_mut())
        //         it->second.second->terminate (false);
        //     term_child (it->second.first);
        // }
        for (_, pipe) in self.endpoints.values_mut() {
            pipe.terminate(false);
        }
        // _endpoints.erase (range.first, range.second);
        self.endpoints.clear();

        if options.reconnect_stop & ZMQ_RECONNECT_STOP_AFTER_DISCONNECT {
            self.disconnected = true;
        }

        return 0;
    }

    // int send (msg: &mut ZmqMessage flags: i32);
    pub fn send(
        &mut self,
        msg: &mut ZmqMessage,
        options: &mut ZmqOptions,
        flags: i32,
    ) -> anyhow::Result<()> {
        // scoped_optional_lock_t sync_lock (_thread_safe ? &_sync : null_mut());

        //  Check whether the context hasn't been shut down yet.
        // if (unlikely (_ctx_terminated)) {
        //     errno = ETERM;
        //     return -1;
        // }

        //  Check whether message passed to the function is valid.
        // if (unlikely (!msg || !msg.check ())) {
        //     errno = EFAULT;
        //     return -1;
        // }

        //  Process pending commands, if any.
        self.process_commands(0, true)?;
        // if (unlikely (rc != 0)) {
        //     return -1;
        // }

        //  Clear any user-visible flags that are set on the message.
        msg.reset_flags(ZMQ_MSG_MORE);

        //  At this point we impose the flags on the message.
        if (flags & ZMQ_SNDMORE) {
            msg.set_flags(ZMQ_MSG_MORE);
        }

        msg.reset_metadata();

        //  Try to send the message using method in each socket class
        ops.xsend(msg)?;
        // if (rc == 0) {
        //     return 0;
        // }
        //  Special case for ZMQ_PUSH: -2 means pipe is dead while a
        //  multi-part send is in progress and can't be recovered, so drop
        //  silently when in blocking mode to keep backward compatibility.
        // if (unlikely (rc == -2)) {
        //     if (!((flags & ZMQ_DONTWAIT) || options.sndtimeo == 0)) {
        //         rc = msg.close ();
        //         errno_assert (rc == 0);
        //         rc = msg.init ();
        //         errno_assert (rc == 0);
        //         return 0;
        //     }
        // }
        // if (unlikely (errno != EAGAIN)) {
        //     return -1;
        // }

        //  In case of non-blocking send we'll simply propagate
        //  the error - including EAGAIN - up the stack.
        if (flags & ZMQ_DONTWAIT) != 0 || (options.sndtimeo == 0) {
            bail!("EAGAIN")
        }

        //  Compute the time when the timeout should occur.
        //  If the timeout is infinite, don't care.
        let mut timeout = options.sndtimeo;
        let end = if timeout < 0 {
            0
        } else {
            (self.clock.now_ms() + timeout)
        };

        //  Oops, we couldn't send the message. Wait for the next
        //  command, process it and try to send the message again.
        //  If timeout is reached in the meantime, return EAGAIN.
        loop {
            // if (unlikely (process_commands (timeout, false) != 0)) {
            //     return -1;
            // }
            ops.xsend(msg)?;
            // if (rc == 0)
            //     break;
            // if (unlikely (errno != EAGAIN)) {
            //     return -1;
            // }
            if timeout > 0 {
                timeout = (end - self.clock.now_ms());
                if timeout <= 0 {
                    bail!("EAGAIN")
                }
            }
        }

        Ok(())
    }

    // int recv (msg: &mut ZmqMessage flags: i32);
    pub fn recv(
        &mut self,
        msg: &mut ZmqMessage,
        options: &mut ZmqOptions,
        flags: i32,
    ) -> anyhow::Result<()> {
        // scoped_optional_lock_t sync_lock (_thread_safe ? &_sync : null_mut());

        //  Check whether the context hasn't been shut down yet.
        // if (unlikely (_ctx_terminated)) {
        //     errno = ETERM;
        //     return -1;
        // }

        //  Check whether message passed to the function is valid.
        // if (unlikely (!msg || !msg.check ())) {
        //     errno = EFAULT;
        //     return -1;
        // }

        //  Once every INBOUND_POLL_RATE messages check for signals and process
        //  incoming commands. This happens only if we are not polling altogether
        //  because there are messages available all the time. If poll occurs,
        //  ticks is set to zero and thus we avoid this code.
        //
        //  Note that 'recv' uses different command throttling algorithm (the one
        //  described above) from the one used by 'send'. This is because counting
        //  ticks is more efficient than doing RDTSC all the time.
        self.ticks += 1;
        if self.ticks == inbound_poll_rate {
            // if (unlikely (process_commands (0, false) != 0)) {
            //     return -1;
            // }
            self.ticks = 0;
        }

        //  Get the message.
        ops.xrecv(msg)?;
        // if (unlikely (rc != 0 && errno != EAGAIN)) {
        //     return -1;
        // }

        //  If we have the message, return immediately.
        // if (rc == 0) {
        //     extract_flags (msg);
        //     return 0;
        // }
        self.extract_flags(msg);
        // TODO: find condition to exit if message is processed.

        //  If the message cannot be fetched immediately, there are two scenarios.
        //  For non-blocking recv, commands are processed in case there's an
        //  activate_reader command already waiting in a command pipe.
        //  If it's not, return EAGAIN.
        if (flags & ZMQ_DONTWAIT) != 0 || options.rcvtimeo == 0 {
            // if (unlikely (process_commands (0, false) != 0)) {
            //     return -1;
            // }
            self.ticks = 0;

            ops.xrecv(msg)?;
            // if rc < 0 {
            //     return rc;
            // }
            self.extract_flags(msg);

            // return 0;
            return Ok(());
        }

        //  Compute the time when the timeout should occur.
        //  If the timeout is infinite, don't care.
        let mut timeout = options.rcvtimeo;
        let end = if timeout < 0 {
            0
        } else {
            (self.clock.now_ms() + timeout)
        };

        //  In blocking scenario, commands are processed over and over again until
        //  we are able to fetch a message.
        let mut block = (self.ticks != 0);
        loop {
            // if (unlikely (process_commands (block ? timeout : 0, false) != 0)) {
            //     return -1;
            // }
            xrecv(msg)?;
            if (rc == 0) {
                self.ticks = 0;
                break;
            }
            // if (unlikely (errno != EAGAIN)) {
            //     return -1;
            // }
            block = true;
            if (timeout > 0) {
                timeout = (end - self.clock.now_ms());
                if (timeout <= 0) {
                    // errno = EAGAIN;
                    // return -1;
                    bail!("EAGAIN");
                }
            }
        }

        extract_flags(msg);
        Ok(())
    }

    // void add_signaler (signaler_t *s_);
    pub fn add_signaler(&mut self, s_: &mut signaler_t) {
        // zmq_assert (_thread_safe);

        // scoped_lock_t sync_lock (_sync);
        // (static_cast<mailbox_safe_t *> (_mailbox))->add_signaler (s_);
        self.mailbox.add_signaler(s_);
    }

    // void remove_signaler (signaler_t *s_);
    pub fn remove_signaler(&mut self, s_: &mut signaler_t) {
        // zmq_assert (_thread_safe);

        // scoped_lock_t sync_lock (_sync);
        // (static_cast<mailbox_safe_t *> (_mailbox))->remove_signaler (s_);
        self.mailbox.remove_signaler(s_);
    }

    // int close ();
    pub fn close(&mut self) -> anyhow::Result<()> {
        // scoped_optional_lock_t sync_lock (_thread_safe ? &_sync : null_mut());

        //  Remove all existing signalers for thread safe sockets
        if (self.thread_safe) {
            self.mailbox.clear_signalers();
        }

        //  Mark the socket as dead
        self._tag = 0xdeadbeef;

        //  Transfer the ownership of the socket from this application thread
        //  to the reaper thread which will take care of the rest of shutdown
        //  process.
        self.send_reap(self);

        Ok(())
    }

    //  These functions are used by the polling mechanism to determine
    //  which events are to be reported from this socket.
    // bool has_in ();
    pub fn has_in(&mut self) -> bool {
        return ops.xhas_in();
    }

    // bool has_out ();
    pub fn has_out(&mut self) -> bool {
        return ops.xhas_out();
    }

    //  Joining and leaving groups
    // int join (group_: *const c_char);
    pub fn join(&mut self, group_: &str) -> anyhow::Result<()> {
        // scoped_optional_lock_t sync_lock (_thread_safe ? &_sync : null_mut());

        ops.xjoin(group_)
    }
    // int leave (group_: *const c_char);
    pub fn leave(&mut self, group_: &str) -> anyhow::Result<()> {
        // scoped_optional_lock_t sync_lock (_thread_safe ? &_sync : null_mut());
        ops.xleave(group_)
    }

    //  Using this function reaper thread ask the socket to register with
    //  its poller.
    // void start_reaping (poller_t *poller_);
    pub fn start_reaping(&mut self, poller_: &mut poller_t) {
        //  Plug the socket to the reaper thread.
        self.poller = poller_.clone();

        let fd: fd_t;

        if !self.thread_safe {
            fd = self.mailbox.get_fd();
        } else {
            // scoped_optional_lock_t sync_lock (_thread_safe ? &_sync : null_mut());

            self.reaper_signaler = signaler_t::default(); //new (std::nothrow) signaler_t ();
                                                          // zmq_assert (_reaper_signaler);

            //  Add signaler to the safe mailbox
            fd = _reaper_signaler.get_fd();
            self.mailbox.add_signaler(&self.reaper_signaler);

            //  Send a signal to make sure reaper handle existing commands
            self.reaper_signaler.send();
        }

        self.handle = self.poller.add_fd(fd, self);
        self.poller.set_pollin(self.handle);

        //  Initialise the termination and check whether it can be deallocated
        //  immediately.
        self.terminate();
        self.check_destroy();
    }

    //  i_poll_events implementation. This interface is used when socket
    //  is handled by the poller in the reaper thread.
    // void in_event () ZMQ_FINAL;
    pub fn in_event(&mut self) {
        //  This function is invoked only once the socket is running in the context
        //  of the reaper thread. Process any commands from other threads/sockets
        //  that may be available at the moment. Ultimately, the socket will
        //  be destroyed.
        {
            // scoped_optional_lock_t sync_lock (_thread_safe ? &_sync : null_mut());

            //  If the socket is thread safe we need to unsignal the reaper signaler
            if (self.thread_safe) {
                self.reaper_signaler.recv();
            }

            self.process_commands(0, false);
        }
        self.check_destroy();
    }

    // void out_event () ZMQ_FINAL;
    pub fn out_event(&mut self) {
        unimplemented!()
        // zmq_assert (false);
    }

    // void timer_event (id_: i32) ZMQ_FINAL;
    pub fn timer_event(&mut self, id_: i32) {
        unimplemented!()
    }

    //  i_pipe_events interface implementation.
    // void read_activated (pipe_t *pipe_) ZMQ_FINAL;
    pub fn read_activated(&mut self, pipe_: &mut pipe_t) {
        ops.xread_activated(pipe_);
    }

    // void write_activated (pipe_t *pipe_) ZMQ_FINAL;
    pub fn write_activated(&mut self, pipe_: &mut pipe_t) {
        ops.xwrite_activated(pipe_);
    }

    // void hiccuped (pipe_t *pipe_) ZMQ_FINAL;
    pub fn hiccuped(&mut self, options: &mut ZmqOptions, pipe_: &mut pipe_t) {
        if (options.immediate == 1) {
            pipe_.terminate(false);
        } else {
            // Notify derived sockets of the hiccup
            ops.xhiccuped(pipe_);
        }
    }

    // void pipe_terminated (pipe_t *pipe_) ZMQ_FINAL;
    pub fn pipe_terminated(&mut self, pipe_: &mut pipe_t) {
        //  Notify the specific socket type about the pipe termination.
        ops.xpipe_terminated(pipe_);

        // Remove pipe from inproc pipes
        self._inprocs.erase_pipe(pipe_);

        //  Remove the pipe from the list of attached pipes and confirm its
        //  termination if we are already shutting down.
        self.pipes.erase(pipe_);

        // Remove the pipe from _endpoints (set it to NULL).
        let identifier = pipe_.get_endpoint_pair().identifier();
        if (!identifier.empty()) {
            // std::pair<endpoints_t::iterator, endpoints_t::iterator> range;
            // range = _endpoints.equal_range (identifier);
            // for (endpoints_t::iterator it = range.first; it != range.second; ++it) {
            //     if (it.second.second == pipe_) {
            //         it.second.second = null_mut();
            //         break;
            //     }
            // }
            for it in self.endpoints.iter() {}
        }

        if (self.is_terminating()) {
            self.unregister_term_ack();
        }
    }

    // void lock ();
    // void unlock ();

    // int monitor (endpoint_: *const c_char,
    //              events_: u64,
    //              event_version_: i32,
    //              type_: i32);

    pub fn monitor(
        &mut self,
        options: &mut ZmqOptions,
        endpoint_: &str,
        events_: u64,
        event_version_: i32,
        type_: i32,
    ) -> anyhow::Result<()> {
        // scoped_lock_t lock (_monitor_sync);

        // if (unlikely (_ctx_terminated)) {
        // errno = ETERM;
        // return -1;
        // }

        //  Event version 1 supports only first 16 events.
        // if (unlikely (event_version_ == 1 && events_ >> 16 != 0)) {
        // errno = EINVAL;
        // return -1;
        // }

        //  Support deregistering monitoring endpoints as well
        if (endpoint_ == null_mut()) {
            self.stop_monitor(options, false);
            return Ok(());
        }
        //  Parse endpoint_uri_ string.
        let mut protocol = String::new();
        let mut address = String::new();
        if (self.parse_uri(endpoint_, &mut protocol, &mut address)
            || self.check_protocol(options, &protocol))
        {
            bail!("failed to parse uri and/or protocol");
        }

        //  Event notification only supported over inproc://
        if (protocol != protocol_name::inproc) {
            // errno = EPROTONOSUPPORT;
            // return -1;
            bail!("protocol not supported");
        }

        // already monitoring. Stop previous monitor before starting new one.
        if (self.monitor_socket != null_mut()) {
            self.stop_monitor(options, true);
        }

        // Check if the specified socket type is supported. It must be a
        // one-way socket types that support the SNDMORE flag.
        match type_ {
            ZMQ_PAIR => {}

            ZMQ_PUB => {}

            ZMQ_PUSH => {}

            _ => {
                bail!("invalid socket type")
            } // errno = EINVAL;
              // return -1;
        }

        //  Register events to monitor
        self.monitor_events = events_ as i64;
        options.monitor_event_version = event_version_;
        //  Create a monitor socket of the specified type.
        self.monitor_socket = zmq_socket(get_ctx(), type_);
        if (self.monitor_socket == null_mut()) {
            bail!("failed to create monitor socket")
        }

        //  Never block context termination on pending event messages
        let mut linger = 0i32;
        let linger_bytes: [u8; 4] = linger.to_le_bytes();
        let mut rc = zmq_setsockopt(
            options,
            self.monitor_socket.as_slice(),
            ZMQ_LINGER,
            &linger_bytes,
            mem::size_of::<linger>(),
        );
        if (rc == -1) {
            self.stop_monitor(options, false);
        }

        //  Spawn the monitor socket endpoint
        rc = zmq_bind(_monitor_socket, endpoint_);
        if (rc == -1) {
            self.stop_monitor(options, false);
        }
        Ok(())
    }

    // void event_connected (const EndpointUriPair &endpoint_uri_pair_,
    //                       fd_t fd_);
    pub fn event_connected(
        &mut self,
        options: &mut ZmqOptions,
        endpoint_uri_pair_: &EndpointUriPair,
        fd_: fd_t,
    ) {
        // u64 values[1] = {static_cast<u64> (fd_)};
        let values: [u64; 1] = [fd_];
        self.event(
            options,
            endpoint_uri_pair_,
            &values,
            1,
            ZMQ_EVENT_CONNECTED as u64,
        );
    }

    // void event_connect_delayed (const EndpointUriPair &endpoint_uri_pair_,
    //                             err_: i32);
    pub fn event_connect_delayed(
        &mut self,
        options: &mut ZmqOptions,
        endpoint_uri_pair_: &EndpointUriPair,
        err_: i32,
    ) {
        // u64 values[1] = {static_cast<u64> (err_)};
        let values: [u64; 1] = [err_ as u64];
        self.event(
            options,
            endpoint_uri_pair_,
            &values,
            1,
            ZMQ_EVENT_CONNECT_DELAYED as u64,
        );
    }

    // void event_connect_retried (const EndpointUriPair &endpoint_uri_pair_,
    //                             interval_: i32);
    pub fn event_connect_retried(
        &mut self,
        options: &mut ZmqOptions,
        endpoint_uri_pair_: &EndpointUriPair,
        interval_: i32,
    ) {
        // u64 values[1] = {static_cast<u64> (interval_)};
        let values: [u64; 1] = [interval_ as u64];
        self.event(
            options,
            endpoint_uri_pair_,
            &values,
            1,
            ZMQ_EVENT_CONNECT_RETRIED as u64,
        );
    }

    // void event_listening (const EndpointUriPair &endpoint_uri_pair_,
    //                       fd_t fd_);
    pub fn event_listening(
        &mut self,
        options: &mut ZmqOptions,
        endpoint_uri_pair_: &EndpointUriPair,
        fd_: fd_t,
    ) {
        // u64 values[1] = {static_cast<u64> (fd_)};
        let values: [u64; 1] = [fd_];
        self.event(
            options,
            endpoint_uri_pair_,
            &values,
            1,
            ZMQ_EVENT_LISTENING as u64,
        );
    }

    // void event_bind_failed (const EndpointUriPair &endpoint_uri_pair_,
    //                         err_: i32);
    pub fn event_bind_failed(
        &mut self,
        options: &mut ZmqOptions,
        endpoint_uri_pair_: &EndpointUriPair,
        err_: i32,
    ) {
        // u64 values[1] = {static_cast<u64> (err_)};
        let values: [u64; 1] = [err_ as u64];

        self.event(
            options,
            endpoint_uri_pair_,
            &values,
            1,
            ZMQ_EVENT_BIND_FAILED as u64,
        );
    }

    // void event_accepted (const EndpointUriPair &endpoint_uri_pair_,
    //                      fd_t fd_);
    pub fn event_accepted(
        &mut self,
        options: &mut ZmqOptions,
        endpoint_uri_pair_: &EndpointUriPair,
        fd_: fd_t,
    ) {
        // u64 values[1] = {static_cast<u64> (fd_)};
        let values: [u64; 1] = [fd_];
        self.event(
            options,
            endpoint_uri_pair_,
            &values,
            1,
            ZMQ_EVENT_ACCEPTED as u64,
        );
    }

    // void event_accept_failed (const EndpointUriPair &endpoint_uri_pair_,
    //                           err_: i32);
    pub fn event_accept_failed(
        &mut self,
        options: &mut ZmqOptions,
        endpoint_uri_pair_: &EndpointUriPair,
        err_: i32,
    ) {
        // u64 values[1] = {static_cast<u64> (err_)};
        let values: [u64; 1] = [err_ as u64];
        self.event(
            options,
            endpoint_uri_pair_,
            &values,
            1,
            ZMQ_EVENT_ACCEPT_FAILED as u64,
        );
    }

    // void event_closed (const EndpointUriPair &endpoint_uri_pair_,
    //                    fd_t fd_);
    pub fn event_closed(
        &mut self,
        options: &mut ZmqOptions,
        endpoint_uri_pair_: &EndpointUriPair,
        fd_: fd_t,
    ) {
        // u64 values[1] = {static_cast<u64> (fd_)};
        let values: [u64; 1] = [fd_];
        self.event(
            options,
            endpoint_uri_pair_,
            &values,
            1,
            ZMQ_EVENT_CLOSED as u64,
        );
    }

    // void event_close_failed (const EndpointUriPair &endpoint_uri_pair_,
    //                          err_: i32);
    pub fn event_close_failed(
        &mut self,
        options: &mut ZmqOptions,
        enpoint_uri_pair_: &EndpointUriPair,
        err_: i32,
    ) {
        // u64 values[1] = {static_cast<u64> (err_)};
        let values: [u64; 1] = [err_ as u64];
        self.event(
            options,
            endpoint_uri_pair_,
            &values,
            1,
            ZMQ_EVENT_CLOSE_FAILED as u64,
        );
    }

    // void event_disconnected (const EndpointUriPair &endpoint_uri_pair_,
    //                          fd_t fd_);
    pub fn event_disconnected(
        &mut self,
        options: &mut ZmqOptions,
        endpoint_uri_pair_: &EndpointUriPair,
        fd_: fd_t,
    ) {
        // u64 values[1] = {static_cast<u64> (fd_)};
        let values: [u64; 1] = [fd_];
        self.event(
            options,
            endpoint_uri_pair_,
            &values,
            1,
            ZMQ_EVENT_DISCONNECTED as u64,
        );
    }

    // void event_handshake_failed_no_detail (
    //   const EndpointUriPair &endpoint_uri_pair_, err_: i32);
    pub fn event_handshake_failed_no_detail(
        &mut self,
        options: &mut ZqmOptions,
        endpoint_uri_pair: &EndpointUriPair,
        err_: i32,
    ) {
        // u64 values[1] = {static_cast<u64> (err_)};
        let values: [u64; 1] = [err_ as u64];
        self.event(
            options,
            endpoint_uri_pair_,
            &values,
            1,
            ZMQ_EVENT_HANDSHAKE_FAILED_NO_DETAIL as u64,
        );
    }

    // void event_handshake_failed_protocol (
    //   const EndpointUriPair &endpoint_uri_pair_, err_: i32);
    pub fn event_handshake_failed_protocol(
        &mut self,
        options: &mut ZmqOptions,
        endpoint_uri_pair_: &EndpointUriPair,
        err_: i32,
    ) {
        // u64 values[1] = {static_cast<u64> (err_)};
        let values: [u64; 1] = [err_ as u64];
        self.event(
            options,
            endpoint_uri_pair_,
            &values,
            1,
            ZMQ_EVENT_HANDSHAKE_FAILED_PROTOCOL as u64,
        );
    }

    // void
    // event_handshake_failed_auth (const EndpointUriPair &endpoint_uri_pair_,
    //                              err_: i32);
    pub fn event_handshake_failed_auth(
        &mut self,
        options: &mut ZmqOptions,
        endpoint_uri_pair_: &EndpointUriPair,
        err_: i32,
    ) {
        // u64 values[1] = {static_cast<u64> (err_)};
        let values: [u64; 1] = [err_ as u64];
        self.event(
            options,
            endpoint_uri_pair_,
            &values,
            1,
            ZMQ_EVENT_HANDSHAKE_FAILED_AUTH as u64,
        );
    }

    // void
    // event_handshake_succeeded (const EndpointUriPair &endpoint_uri_pair_,
    //                            err_: i32);
    pub fn event_handshake_succeeded(
        &mut self,
        options: &mut ZmqOptions,
        endpoint_uri_pair_: &EndpointUriPair,
        err_: i32,
    ) {
        // u64 values[1] = {static_cast<u64> (err_)};
        let values: [u64; 1] = [err_ as u64];
        self.event(
            options,
            endpoint_uri_pair_,
            &values,
            1,
            ZMQ_EVENT_HANDSHAKE_SUCCEEDED as u64,
        );
    }

    //  Request for pipes statistics - will generate a ZMQ_EVENT_PIPES_STATS
    //  after gathering the data asynchronously. Requires event monitoring to
    //  be enabled.
    // int query_pipes_stats ();
    /*
     * There are 2 pipes per connection, and the inbound one _must_ be queried from
     * the I/O thread. So ask the outbound pipe, in the application thread, to send
     * a message (pipe_peer_stats) to its peer. The message will carry the outbound
     * pipe stats and endpoint, and the reference to the socket object.
     * The inbound pipe on the I/O thread will then add its own stats and endpoint,
     * and write back a message to the socket object (pipe_stats_publish) which
     * will raise an event with the data.
     */
    pub fn query_pipes_stats(&mut self) -> anyhow::Result<()> {
        {
            // scoped_lock_t lock (_monitor_sync);
            if !(self.monitor_events & ZMQ_EVENT_PIPES_STATS) {
                // errno = EINVAL;
                // return -1;
                bail!("EINVAL!")
            }
        }
        if self.pipes.size() == 0 {
            // errno = EAGAIN;
            // return -1;
            bail!("EAGAIN!")
        }
        // for (pipes_t::size_type i = 0, size = _pipes.size (); i != size; ++i) {
        //     _pipes[i]->send_stats_to_peer (this);
        // }
        for pipe in self.pipes.iter_mut() {
            pipe.send_stats_to_peer(self);
        }

        Ok(())
    }

    // bool is_disconnected () const;
    pub fn is_disconnected(&mut self) -> bool {
        self.disconnected
    }

    //  Delay actual destruction of the socket.
    // void process_destroy () ZMQ_FINAL;
    pub fn process_destroy(&mut self) {
        self.destroyed = true;
    }

    // int connect_internal (endpoint_uri_: *const c_char);
    pub fn connect_internal(
        &mut self,
        options: &mut ZmqOptions,
        endpoint_uri_: &str,
    ) -> anyhow::Result<()> {
        // if (unlikely (_ctx_terminated)) {
        //     errno = ETERM;
        //     return -1;
        // }

        //  Process pending commands, if any.
        self.process_commands(0, false)?;
        // if (unlikely (rc != 0)) {
        //     return -1;
        // }

        //  Parse endpoint_uri_ string.
        let mut protocol = String::new();
        let mut address = String::new();
        if self.parse_uri(endpoint_uri_, &mut protocol, &mut address)
            || self.check_protocol(options, &protocol)
        {
            bail!("failed to parse uri or check protocol");
        }

        if protocol == protocol_name::inproc {
            //  TODO: inproc connect is specific with respect to creating pipes
            //  as there's no 'reconnect' functionality implemented. Once that
            //  is in place we should follow generic pipe creation algorithm.

            //  Find the peer endpoint.
            let mut peer = self.find_endpoint(endpoint_uri_);

            // The total HWM for an inproc connection should be the sum of
            // the binder's HWM and the connector's HWM.
            let sndhwm: i32 = if peer.socket == null_mut() {
                options.sndhwm
            } else if options.sndhwm != 0 && peer.options.rcvhwm != 0 {
                options.sndhwm + peer.options.rcvhwm
            } else {
                0
            };
            let rcvhwm: i32 = if peer.socket == null_mut() {
                options.rcvhwm
            } else if options.rcvhwm != 0 && peer.options.sndhwm != 0 {
                options.rcvhwm + peer.options.sndhwm
            } else {
                0
            };

            //  Create a bi-directional pipe to connect the peers.
            let mut parents: [&mut ZmqSocketBase; 2] = [
                self,
                if peer.socket == null_mut() {
                    self
                } else {
                    peer.socket
                },
            ];
            let mut new_pipes: [pipe_t; 2] = [pipe_t::default(), pipe_t::default()];

            let conflate = get_effective_conflate_option(options);

            let mut hwms: [i32; 2] = [
                if conflate { -1 } else { sndhwm },
                if conflate { -1 } else { rcvhwm },
            ];
            let mut conflates: [bool; 2] = [conflate, conflate];
            rc = pipepair(parents, new_pipes, hwms, conflates);
            if !conflate {
                new_pipes[0].set_hwms_boost(peer.options.sndhwm, peer.options.rcvhwm);
                new_pipes[1].set_hwms_boost(options.sndhwm, options.rcvhwm);
            }

            // errno_assert (rc == 0);

            if !peer.socket {
                //  The peer doesn't exist yet so we don't know whether
                //  to send the routing id message or not. To resolve this,
                //  we always send our routing id and drop it later if
                //  the peer doesn't expect it.
                self.send_routing_id(&mut new_pipes[0], options);

                // #ifdef ZMQ_BUILD_DRAFT_API
                //  If set, send the hello msg of the local socket to the peer.
                if options.can_send_hello_msg && options.hello_msg.size() > 0 {
                    self.send_hello_msg(&mut new_pipes[0], options);
                }
                // #endif

                let mut endpoint: ZmqEndpoint = ZmqEndpoint::new(self, options);
                self.pend_connection(endpoint_uri_, endpoint, &new_pipes);
            } else {
                //  If required, send the routing id of the local socket to the peer.
                if peer.options.recv_routing_id {
                    self.send_routing_id(&new_pipes[0], options);
                }

                //  If required, send the routing id of the peer to the local socket.
                if options.recv_routing_id {
                    self.send_routing_id(&new_pipes[1], peer.options);
                }

                // #ifdef ZMQ_BUILD_DRAFT_API
                //  If set, send the hello msg of the local socket to the peer.
                if options.can_send_hello_msg && options.hello_msg.size() > 0 {
                    self.send_hello_msg(&new_pipes[0], options);
                }

                //  If set, send the hello msg of the peer to the local socket.
                if peer.options.can_send_hello_msg && peer.options.hello_msg.size() > 0 {
                    send_hello_msg(&new_pipes[1], peer.options);
                }

                if peer.options.can_recv_disconnect_msg && peer.options.disconnect_msg.size() > 0 {
                    new_pipes[0].set_disconnect_msg(peer.options.disconnect_msg);
                }
                // #endif

                //  Attach remote end of the pipe to the peer socket. Note that peer's
                //  seqnum was incremented in find_endpoint function. We don't need it
                //  increased here.
                self.send_bind(peer.socket, &new_pipes[1], false);
            }

            //  Attach local end of the pipe to this socket object.
            self.attach_pipe(&mut new_pipes[0], false, true);

            // Save last endpoint URI
            self.last_endpoint.assign(endpoint_uri_);

            // remember inproc connections for disconnect
            self._inprocs.emplace(endpoint_uri_, &new_pipes[0]);

            options.connected = true;
            return Ok(());
        }
        let is_single_connect = (options.type_ == ZMQ_DEALER
            || options.type_ == ZMQ_SUB
            || options.type_ == ZMQ_PUB
            || options.type_ == ZMQ_REQ);
        // if (unlikely (is_single_connect)) {
        //     if (0 != _endpoints.count (endpoint_uri_)) {
        //         // There is no valid use for multiple connects for SUB-PUB nor
        //         // DEALER-ROUTER nor REQ-REP. Multiple connects produces
        //         // nonsensical results.
        //         return 0;
        //     }
        // }

        //  Choose the I/O thread to run the session in.
        let io_thread = self.choose_io_thread(options.affinity)?;
        // if (!io_thread) {
        //     errno = EMTHREAD;
        //     return -1;
        // }

        let mut paddr = Address::new(&protocol, &address, self.get_ctx());
        // alloc_assert (paddr);

        //  Resolve address (if needed by the protocol)
        if protocol == protocol_name::tcp {
            //  Do some basic sanity checks on tcp:// address syntax
            //  - hostname starts with digit or letter, with embedded '-' or '.'
            //  - IPv6 address may contain hex chars and colons.
            //  - IPv6 link local address may contain % followed by interface name / zone_id
            //    (Reference: https://tools.ietf.org/html/rfc4007)
            //  - IPv4 address may contain decimal digits and dots.
            //  - Address must end in ":port" where port is *, or numeric
            //  - Address may contain two parts separated by ':'
            //  Following code is quick and dirty check to catch obvious errors,
            //  without trying to be fully accurate.
            // let check = &address;
            // let check = address.as_mut_ptr();
            // if (isalnum (*check) || isxdigit (*check) || *check == '['
            //     || *check == ':')
            let mut check = address.as_ptr();
            let lbracket_bytes = String::from("[").as_ptr();
            let colon_bytes = String::from(":").as_ptr();
            let dot_bytes = String::from(".").as_ptr();
            let semicolon_bytes = String::from(";").as_ptr();
            let percent_bytes = String::from("%").as_ptr();
            let rbracked_bytes = String::from("]").as_ptr();
            let underscore_bytes = String::from("_").as_ptr();
            let asterisk_bytes = String::from("*").as_ptr();
            let hyphen_bytes = String::from("-").as_ptr();

            // if *check.is_ascii_alphanumeric() || *check.is_ascii_hexdigit() || *check.eq(*('['.encode_utf8().as_ptr())) || *check.eq(':')
            // {
            //     check++;
            //     while (isalnum (*check) || isxdigit (*check) || *check == '.'
            //            || *check == '-' || *check == ':' || *check == '%'
            //            || *check == ';' || *check == '[' || *check == ']'
            //            || *check == '_' || *check == '*') {
            //         check++;
            //     }
            // }
            if *check.is_ascii_alphanumeric()
                || *check.is_ascii_hexdigit()
                || *check.eq(&lbracket_bytes)
                || *check.eq(&colon_bytes)
            {
                check += 1;
                while *check.is_ascii_alphanumeric()
                    || *check.is_ascii_hexdigit()
                    || *check.eq(&dot_bytes)
                    || *check.eq(&hyphen_bytes)
                    || *check.eq(&colon_bytes)
                    || *check.eq(&percent_bytes)
                    || *check.eq(&semicolon_bytes)
                    || *check.eq(&lbracket_bytes)
                    || *check.eq(&rbracked_bytes)
                    || *check.eq(&underscore_bytes)
                    || *check.eq(&asterisk_bytes)
                {
                    check += 1;
                }
            }

            //  Assume the worst, now look for success
            rc = -1;
            //  Did we reach the end of the address safely?
            if *check.is_null() {
                //  Do we have a valid port string? (cannot be '*' in connect
                // check = strrchr (address, ':');
                check = address.as_ptr();
                let index = address.find(":");
                if index.is_some() {
                    check += 1;
                    check += index.unwrap();
                    if *check.is_null() == false && *check.is_ascii_digit() {
                        //  Valid
                        rc = 0;
                    }
                }
            }
            if rc == -1 {
                // errno = EINVAL;
                // LIBZMQ_DELETE (paddr);
                // return -1;
                bail!("EINVAL");
            }
            //  Defer resolution until a socket is opened
            paddr.resolved.tcp_addr = null_mut();
            Ok(())
        }
        // #ifdef ZMQ_HAVE_WS
        // #ifdef ZMQ_HAVE_WSS
        else if protocol == protocol_name::ws || protocol == protocol_name::wss {
            if protocol == protocol_name::wss {
                paddr.resolved.wss_addr = WssAddress::new();
                // alloc_assert (paddr.resolved.wss_addr);
                paddr
                    .resolved
                    .wss_addr
                    .resolve(&address, false, options.ipv6)?;
            }
            // #else
            else if protocol == protocol_name::ws {
                // #endif

                paddr.resolved.ws_addr = WsAddress::new();
                // alloc_assert (paddr.resolved.ws_addr);
                paddr
                    .resolved
                    .ws_addr
                    .resolve(&address, false, options.ipv6)?;
            }
        }
        // #endif

        // #if defined ZMQ_HAVE_IPC
        else if protocol == protocol_name::ipc {
            paddr.resolved.ipc_addr = IpcAddress::new();
            // alloc_assert (paddr.resolved.ipc_addr);
            paddr.resolved.ipc_addr.resolve(&address)?;
        }
        // #endif
        else if protocol == protocol_name::udp {
            if options.type_ != ZMQ_RADIO {
                // errno = ENOCOMPATPROTO;
                // LIBZMQ_DELETE (paddr);
                // return -1;
                bail!("socket type not ZMQ_RADIO");
            }

            paddr.resolved.udp_addr = UdpAddress::new();
            // alloc_assert (paddr.resolved.udp_addr);
            paddr
                .resolved
                .udp_addr
                .resolve(&address, false, options.ipv6)?;
            // if (rc != 0) {
            //     LIBZMQ_DELETE (paddr);
            //     return -1;
            // }
        }
        // TBD - Should we check address for ZMQ_HAVE_NORM???

        // #ifdef ZMQ_HAVE_OPENPGM
        else if protocol == protocol_name::pgm || protocol == protocol_name::epgm {
            let mut res = pgm_addrinfo_t::new();
            let mut port_number = 0u16;
            pgm_socket_t::init_address(&address, &mut res, &port_number)?;
            // if (res != null_mut()) {
            //     pgm_freeaddrinfo(res);
            // }
            // if rc != 0 || port_number == 0 {
            //     baiL!("failed to create PGM socket");
            // }
        }
        // #endif
        // #if defined ZMQ_HAVE_TIPC
        else if protocol == protocol_name::tipc {
            paddr.resolved.tipc_addr = TipcAddress::new();
            alloc_assert(paddr.resolved.tipc_addr);
            paddr.resolved.tipc_addr.resolve(address.c_str())?;
            // if (rc != 0) {
            //     LIBZMQ_DELETE (paddr);
            //     return -1;
            // }
            // const sockaddr_tipc *const saddr =
            //   reinterpret_cast<const sockaddr_tipc *> (
            //     paddr.resolved.tipc_addr->addr ());
            // Cannot connect to random Port Identity
            let saddr = paddr.resolved.tipc_addr as sockaddr_tipc;
            if saddr.addrtype == TIPC_ADDR_ID && paddr.resolved.tipc_addr.is_random() {
                // LIBZMQ_DELETE (paddr);
                // errno = EINVAL;
                // return -1;
                bail!("resolved TIPC address is random!");
            }
        }
        // #endif
        // #if defined ZMQ_HAVE_VMCI
        else if protocol == protocol_name::vmci {
            paddr.resolved.vmci_addr = VmciAddress::new(&self.get_ctx());
            // alloc_assert (paddr.resolved.vmci_addr);
            paddr.resolved.vmci_addr.resolve(&address)?;
            // if (rc != 0) {
            //     LIBZMQ_DELETE (paddr);
            //     return -1;
            // }
        }
        // #endif

        //  Create session.
        let mut session = session_base_t::create(io_thread, true, self, options, paddr);
        // errno_assert (session);

        //  PGM does not support subscription forwarding; ask for all data to be
        //  sent to this pipe. (same for NORM, currently?)
        // #if defined ZMQ_HAVE_OPENPGM && defined ZMQ_HAVE_NORM
        let subscribe_to_all = protocol == protocol_name::pgm
            || protocol == protocol_name::epgm
            || protocol == protocol_name::norm
            || protocol == protocol_name::udp;
        // #elif defined ZMQ_HAVE_OPENPGM
        //     const bool subscribe_to_all = protocol == protocol_name::pgm
        //                                   || protocol == protocol_name::epgm
        //                                   || protocol == protocol_name::udp;
        // #elif defined ZMQ_HAVE_NORM
        //     const bool subscribe_to_all =
        //       protocol == protocol_name::norm || protocol == protocol_name::udp;
        // // #else
        //     const bool subscribe_to_all = protocol == protocol_name::udp;
        // // #endif
        //     pipe_t *newpipe = null_mut();
        let mut newpipe = pipe_t::new();

        if options.immediate != 1 || subscribe_to_all {
            //  Create a bi-directional pipe.
            let mut parents: [&mut ZmqSocketBase; 2] = [self, session];
            let mut new_pipes: [pipe_t; 2] = [pipe_t::default(), pipe_t::default()];

            let conflate = get_effective_conflate_option(options);

            let hwms: [i32; 2] = [
                if conflate { -1 } else { options.sndhwm },
                if conflate { -1 } else { options.rcvhwm },
            ];
            let conflates: [bool; 2] = [conflate, conflate];
            rc = pipepair(parents, new_pipes, hwms, conflates);
            errno_assert(rc == 0);

            //  Attach local end of the pipe to the socket object.
            self.attach_pipe(&mut new_pipes[0], subscribe_to_all, true);
            newpipe = new_pipes[0].clone();

            //  Attach remote end of the pipe to the session object later on.
            session.attach_pipe(&new_pipes[1]);
        }

        //  Save last endpoint URI
        paddr.to_string(_last_endpoint);

        add_endpoint(
            make_unconnected_connect_endpoint_pair(endpoint_uri_),
            session,
            newpipe,
        );
        Ok(())
    }

    // test if event should be sent and then dispatch it
    // void event (const EndpointUriPair &endpoint_uri_pair_,
    //             u64 values_[],
    //             values_count_: u64,
    //             u64 type_);
    pub fn event(
        &mut self,
        options: &mut ZmqOptions,
        endpoint_uri_pair_: &EndpointUriPair,
        values_: &[u64],
        values_count_: u64,
        type_: u64,
    ) {
        // scoped_lock_t lock (_monitor_sync);
        if (self.monitor_events & type_) {
            self.monitor_event(options, type_, values_, values_count_, endpoint_uri_pair_);
        }
    }

    // Socket event data dispatch
    // void monitor_event (event_: u64,
    //                     const u64 values_[],
    //                     values_count_: u64,
    //                     const EndpointUriPair &endpoint_uri_pair_) const;
    pub fn monitor_event(
        &mut self,
        options: &mut ZmqOptions,
        event_: u64,
        values_: &[u64],
        values_count_: u64,
        endpoint_uri_pair_: &EndpointUriPair,
    ) {
        // this is a private method which is only called from
        // contexts where the _monitor_sync mutex has been locked before

        if (_monitor_socket) {
            let mut raw_msg = ZmqMessage::default();

            match (options.monitor_event_version) {
                1 => {
                    //  The API should not allow to activate unsupported events
                    // zmq_assert (event_ <= std::numeric_limits<uint16_t>::max ());
                    //  v1 only allows one value
                    // zmq_assert (values_count_ == 1);
                    // zmq_assert (values_[0]
                    //             <= std::numeric_limits<u32>::max ());

                    //  Send event and value in first frame
                    // const uint16_t event = static_cast<uint16_t> (event_);
                    let event = event_ as u16;
                    // const u32 value = static_cast<u32> (values_[0]);
                    let value = values_[0] as u32;
                    zmq_msg_init_size(
                        &mut raw_msg,
                        mem::size_of::<event>() + mem::size_of::<value>(),
                    );
                    // let mut data = (zmq_msg_data (&mut raw_msg));
                    //  Avoid dereferencing uint32_t on unaligned address
                    // memcpy (data + 0, &event, mem::size_of::<event>());
                    // let event_bytes = event.to_le_bytes();
                    // data.extend_from_slice(&event_bytes);
                    // memcpy (data + mem::size_of::<event>(), &value, mem::size_of::<value>());
                    // let value_bytes = event.to_le_bytes();
                    // data.extend_from_slice(&value_bytes);
                    zmq_msg_send(&mut raw_msg, _monitor_socket, ZMQ_SNDMORE);

                    let endpoint_uri = endpoint_uri_pair_.identifier();

                    //  Send address in second frame
                    zmq_msg_init_size(&mut raw_msg, endpoint_uri.size());
                    // memcpy (zmq_msg_data (&mut msg), endpoint_uri,
                    //         endpoint_uri.size ());
                    zmq_msg_send(&mut raw_msg, _monitor_socket, 0);
                }
                2 => {
                    //  Send event in first frame (64bit unsigned)
                    zmq_msg_init_size(&mut raw_msg, mem::size_of::<event_>());
                    // memcpy (zmq_msg_data (&mut msg), &event_, mem::size_of::<event_>());

                    zmq_msg_send(&mut raw_msg, _monitor_socket, ZMQ_SNDMORE);

                    //  Send number of values that will follow in second frame
                    zmq_msg_init_size(&mut raw_msg, mem::size_of::<values_count_>());
                    // memcpy (zmq_msg_data (&msg), &values_count_,
                    //         mem::size_of::<values_count_>());
                    zmq_msg_data(&mut raw_msg)
                        .extend_from_slice(&values_count_.to_le_bytes().as_slice());
                    zmq_msg_send(&mut raw_msg, _monitor_socket, ZMQ_SNDMORE);

                    //  Send values in third-Nth frames (64bit unsigned)
                    // for (u64 i = 0; i < values_count_; ++i)
                    for i in 0..values_count_ {
                        zmq_msg_init_size(&mut raw_msg, sizeof(values_[i]));
                        // memcpy (zmq_msg_data (&msg), &values_[i],
                        //         sizeof (values_[i]));
                        zmq_msg_data(&mut raw_msg).extend_from_slice(&values_[i]);
                        zmq_msg_send(&mut raw_msg, _monitor_socket, ZMQ_SNDMORE);
                    }

                    //  Send local endpoint URI in second-to-last frame (string)
                    zmq_msg_init_size(&mut raw_msg, endpoint_uri_pair_.local.size());
                    // memcpy (zmq_msg_data (&msg), endpoint_uri_pair_.local,
                    //         endpoint_uri_pair_.local.size ());
                    let epup_bytes = endpoint_uri_pair_.local.as_bytes();
                    zmq_msg_data(&mut raw_msg).extend_from_slice(epup_bytes);
                    zmq_msg_send(&mut raw_msg, _monitor_socket, ZMQ_SNDMORE);

                    //  Send remote endpoint URI in last frame (string)
                    zmq_msg_init_size(&mut raw_msg, endpoint_uri_pair_.remote.size());
                    // memcpy (zmq_msg_data (&msg), endpoint_uri_pair_.remote,
                    //         endpoint_uri_pair_.remote.size ());
                    zmq_msg_data(&mut raw_msg)
                        .extend_from_slice(endpoint_uri_pair_.remote.as_bytes());
                    zmq_msg_send(&mut raw_msg, _monitor_socket, 0);
                }
                _ => {}
            }
        }
    }

    // Monitor socket cleanup
    // void stop_monitor (bool send_monitor_stopped_event_ = true);
    pub fn stop_monitor(&mut self, options: &mut ZmqOptions, send_monitor_stopped_event_: bool) {
        // this is a private method which is only called from
        // contexts where the _monitor_sync mutex has been locked before

        if (self.monitor_socket) {
            if ((self.monitor_events & ZMQ_EVENT_MONITOR_STOPPED) != 0
                && send_monitor_stopped_event_)
            {
                let values: [u64; 1] = [0];
                self.monitor_event(
                    options,
                    ZMQ_EVENT_MONITOR_STOPPED as u64,
                    &values,
                    1,
                    &EndpointUriPair::default(),
                );
            }
            zmq_close(&mut self.monitor_socket);
            self.monitor_socket.clear();
            self.monitor_events = 0;
        }
    }

    //  Creates new endpoint ID and adds the endpoint to the map.
    // void add_endpoint (const EndpointUriPair &endpoint_pair_,
    //                    own_t *endpoint_,
    //                    pipe_t *pipe_);
    pub fn add_endpoint(
        &mut self,
        endpoint_pair_: &EndpointUriPair,
        endpoint_: &mut own_t,
        pipe_: &mut pipe_t,
    ) {
        //  Activate the session. Make it a child of this socket.
        self.launch_child(endpoint_);
        self.endpoints.ZMQ_MAP_INSERT_OR_EMPLACE(
            endpoint_pair_.identifier(),
            endpoint_pipe_t::new(endpoint_, pipe_),
        );

        if pipe_ != null_mut() {
            pipe_.set_endpoint_pair(endpoint_pair_);
        }
    }

    //  To be called after processing commands or invoking any command
    //  handlers explicitly. If required, it will deallocate the socket.
    // void check_destroy ();
    pub fn check_destroy(&mut self) {
        //  If the object was already marked as destroyed, finish the deallocation.
        if (self.destroyed) {
            //  Remove the socket from the reaper's poller.
            self.poller.rm_fd(self.handle);

            //  Remove the socket from the context.
            self.destroy_socket(self);

            //  Notify the reaper about the fact.
            self.send_reaped();

            //  Deallocate.
            // self.own_t::process_destroy ();
        }
    }

    //  Moves the flags from the message to local variables,
    //  to be later retrieved by getsockopt.
    // void extract_flags (const ZmqMessage *msg);
    pub fn extract_flags(&mut self, msg: &mut ZmqMessage) {
        //  Test whether routing_id flag is valid for this socket type.
        // if (unlikely (msg.flags () & ZMQ_MSG_ROUTING_ID)){
        //     zmq_assert (options.recv_routing_id);
        // }

        //  Remove MORE flag.
        self.rcvmore = (msg.flags() & ZMQ_MSG_MORE) != 0;
    }

    //  Parse URI string.
    // static int
    // parse_uri (uri_: *const c_char, std::string &protocol_, std::string &path_);
    // TODO consider renaming protocol_ to scheme_ in conformance with RFC 3986
    // terminology, but this requires extensive changes to be consistent
    pub fn parse_uri(
        &mut self,
        uri: &str,
        protocol: &mut String,
        path: &mut String,
    ) -> anyhow::Result<()> {
        // zmq_assert (uri_ != null_mut());

        // const std::string uri (uri_);
        let pos = uri.find("://");
        if pos.is_none() {
            bail!("invalid uri {}", uri);
        }
        *protocol = String::from(&uri[0..pos.unwrap()]); // uri.substr (0, pos);
        *path = String::from(&uri[pos.unwrap() + 3..]);

        if protocol.is_empty() || path.is_empty() {
            bail!("failed to parse protocol and path from uri string")
        }
        Ok(())
    }

    //  Check whether transport protocol, as specified in connect or
    //  bind, is available and compatible with the socket type.
    // int check_protocol (protocol_: &str) const;

    pub fn check_protocol(
        &mut self,
        options: &mut ZmqOptions,
        protocol_: &str,
    ) -> anyhow::Result<()> {
        //  First check out whether the protocol is something we are aware of.
        if protocol_ != protocol_name::inproc
    // #if defined ZMQ_HAVE_IPC
            && protocol_ != protocol_name::ipc
    // #endif
            && protocol_ != protocol_name::tcp
    // #ifdef ZMQ_HAVE_WS
            && protocol_ != protocol_name::ws
    // #endif
    // #ifdef ZMQ_HAVE_WSS
            && protocol_ != protocol_name::wss
    // #endif
    // #if defined ZMQ_HAVE_OPENPGM
            //  pgm/epgm transports only available if 0MQ is compiled with OpenPGM.
            && protocol_ != protocol_name::pgm
            && protocol_ != protocol_name::epgm
    // #endif
    // #if defined ZMQ_HAVE_TIPC
            // TIPC transport is only available on Linux.
            && protocol_ != protocol_name::tipc
    // #endif
    // #if defined ZMQ_HAVE_NORM
            && protocol_ != protocol_name::norm
    // #endif
    // #if defined ZMQ_HAVE_VMCI
            && protocol_ != protocol_name::vmci
    // #endif
            && protocol_ != protocol_name::udp
        {
            // errno = EPROTONOSUPPORT;
            // return -1;
            bail!("udp protocol not supported")
        }

        //  Check whether socket type and transport protocol match.
        //  Specifically, multicast protocols can't be combined with
        //  bi-directional messaging patterns (socket types).
        // #if defined ZMQ_HAVE_OPENPGM || defined ZMQ_HAVE_NORM
        // #if defined ZMQ_HAVE_OPENPGM && defined ZMQ_HAVE_NORM
        if (protocol_ == protocol_name::pgm || protocol_ == protocol_name::epgm
             || protocol_ == protocol_name::norm)
    // #elif defined ZMQ_HAVE_OPENPGM
    //     if ((protocol_ == protocol_name::pgm || protocol_ == protocol_name::epgm)
    // #else // defined ZMQ_HAVE_NORM
    //     if (protocol_ == protocol_name::norm
    // #endif
            && options.type_ != ZMQ_PUB && options.type_ != ZMQ_SUB
            && options.type_ != ZMQ_XPUB && options.type_ != ZMQ_XSUB
        {
            errno = ENOCOMPATPROTO;
            bail!("no compatible protocol found for pub/xpub/sub/xsub");
        }
        // #endif

        if protocol_ == protocol_name::udp
            && (options.type_ != ZMQ_DISH
                && options.type_ != ZMQ_RADIO
                && options.type_ != ZMQ_DGRAM)
        {
            // errno = ENOCOMPATPROTO;
            // return -1;
            bail!("no compatible protocol found for dish/radio/dgram")
        }

        //  Protocol is available.
        // return 0;
        Ok(())
    }

    //  Register the pipe with this socket.
    // void attach_pipe (pipe_t *pipe_,
    //                   bool subscribe_to_all_ = false,
    //                   bool locally_initiated_ = false);
    pub fn attach_pipe(
        &mut self,
        pipe: &mut pipe_t,
        subscribe_to_all_: bool,
        locally_initiated_: bool,
    ) {
        //  First, register the pipe so that we can terminate it later on.
        pipe.set_event_sink(self);
        self.pipes.push_back(pipe);

        //  Let the derived socket type know about new pipe.
        ops.xattach_pipe(pipe, subscribe_to_all_, locally_initiated_);

        //  If the socket is already being closed, ask any new pipes to terminate
        //  straight away.
        if self.is_terminating() {
            self.register_term_acks(1);
            pipe.terminate(false);
        }
    }

    //  Processes commands sent to this socket (if any). If timeout is -1,
    //  returns only after at least one command was processed.
    //  If throttle argument is true, commands are processed at most once
    //  in a predefined time period.
    // int process_commands (timeout_: i32, bool throttle_);
    pub fn process_commands(&mut self, timeout_: i32, throttle_: bool) -> anyhow::Result<()> {
        if timeout_ == 0 {
            //  If we are asked not to wait, check whether we haven't processed
            //  commands recently, so that we can throttle the new commands.

            //  Get the CPU's tick counter. If 0, the counter is not available.
            let tsc = clock_t::rdtsc();

            //  Optimised version of command processing - it doesn't have to check
            //  for incoming commands each time. It does so only if certain time
            //  elapsed since last command processing. Command delay varies
            //  depending on CPU speed: It's ~1ms on 3GHz CPU, ~2ms on 1.5GHz CPU
            //  etc. The optimisation makes sense only on platforms where getting
            //  a timestamp is a very cheap operation (tens of nanoseconds).
            if tsc && throttle_ {
                //  Check whether TSC haven't jumped backwards (in case of migration
                //  between CPU cores) and whether certain time have elapsed since
                //  last command processing. If it didn't do nothing.
                if tsc >= self.last_tsc && tsc - self.last_tsc <= max_command_delay {
                    return Ok(());
                }
                self.last_tsc = tsc;
            }
        }

        //  Check whether there are any commands pending for this thread.
        let mut cmd: ZmqCommand = ZmqCommand::default();
        self.mailbox.recv(&cmd, timeout_)?;
        // if (rc != 0 && errno == EINTR) {
        //     return -1;
        // }

        //  Process all available commands.
        // while (rc == 0 || errno == EINTR) {
        //     if (rc == 0) {
        //         cmd.destination.process_command(cmd);
        //     }
        //     rc = _mailbox->recv (&cmd, 0);
        // }
        loop {
            cmd.destination.process_command(&cmd);
            match self.mailbox.recv(&mut cmd, 0) {
                Ok(_) => {}
                Err(E) => {
                    break;
                }
            }
        }

        // zmq_assert (errno == EAGAIN);

        if self.ctx_terminated {
            // errno = ETERM;
            // return -1;
            bail!("socket terminated")
        }

        Ok(())
    }

    //  Handlers for incoming commands.
    // void process_stop () ZMQ_FINAL;
    pub fn process_stop(&mut self, options: &mut ZmqOptions) {
        //  Here, someone have called zmq_ctx_term while the socket was still alive.
        //  We'll remember the fact so that any blocking call is interrupted and any
        //  further attempt to use the socket will return ETERM. The user is still
        //  responsible for calling zmq_close on the socket though!
        // scoped_lock_t lock (_monitor_sync);
        self.stop_monitor(options, false);

        self.ctx_terminated = true;
    }

    // void process_bind (pipe_t *pipe_) ZMQ_FINAL;
    pub fn process_bind(&mut self, pipe_: &mut pipe_t) {
        self.attach_pipe(pipe_, false, false);
    }

    // void process_pipe_stats_publish (outbound_queue_count_: u64,
    //                             inbound_queue_count_: u64,
    //                             EndpointUriPair *endpoint_pair_) ZMQ_FINAL;
    pub fn process_pipe_stats_publish(
        &mut self,
        options: &mut ZmqOptions,
        outbound_queue_count_: u64,
        inbound_queue_count_: u64,
        endpoint_pair_: &mut EndpointUriPair,
    ) {
        let mut values: [u64; 2] = [outbound_queue_count_, inbound_queue_count_];
        self.event(
            options,
            endpoint_pair_,
            &values,
            2,
            ZMQ_EVENT_PIPES_STATS as u64,
        );
        // delete endpoint_pair_;
    }

    // void process_term (linger_: i32) ZMQ_FINAL;
    pub fn process_term(&mut self, linger_: i32) {
        //  Unregister all inproc endpoints associated with this socket.
        //  Doing this we make sure that no new pipes from other sockets (inproc)
        //  will be initiated.
        self.unregister_endpoints(self);

        //  Ask all attached pipes to terminate.
        // for (pipes_t::size_type i = 0, size = _pipes.size (); i != size; ++i)
        // {
        //     //  Only inprocs might have a disconnect message set
        //     _pipes[i]->send_disconnect_msg ();
        //     _pipes[i]->terminate (false);
        // }
        for pipe in self.pipes.iter_mut() {
            pipe.send_disconnect_msg();
            pipe.terminate(false);
        }

        self.register_term_acks(self.pipes.len());

        //  Continue the termination process immediately.
        // own_t::process_term (linger_);
    }

    // void process_term_endpoint (std::string *endpoint_) ZMQ_FINAL;
    pub fn process_term_endpoint(&mut self, options: &mut ZmqOptions, endpoint_: &str) {
        self.term_endpoint(options, endpoint_);
        // delete endpoint_;
    }

    // void update_pipe_options (option_: i32);
    pub fn update_pipe_options(&mut self, options: &mut ZmqOptions, option_: i32) {
        if option_ == ZMQ_SNDHWM || option_ == ZMQ_RCVHWM {
            // for (pipes_t::size_type i = 0, size = _pipes.size (); i != size; ++i)
            for pipe in self.pipes.iter_mut() {
                pipe.set_hwms(options.rcvhwm, options.sndhwm);
                pipe.send_hwms_to_peer(options.sndhwm, options.rcvhwm);
            }
        }
    }

    // std::string resolve_tcp_addr (std::string endpoint_uri_,
    //                               tcp_address_: *const c_char);
    pub fn resolve_tcp_addr(
        &mut self,
        options: &mut ZmqOptions,
        endpoint_uri_pair_: &mut String,
        tcp_address_: &mut str,
    ) -> String {
        // The resolved last_endpoint is used as a key in the endpoints map.
        // The address passed by the user might not match in the TCP case due to
        // IPv4-in-IPv6 mapping (EG: tcp://[::ffff:127.0.0.1]:9999), so try to
        // resolve before giving up. Given at this stage we don't know whether a
        // socket is connected or bound, try with both.
        if self.endpoints.find(endpoint_uri_pair_) == self.endpoints.end() {
            // TcpAddress *tcp_addr = new (std::nothrow) TcpAddress ();
            let mut tcp_addr = TcpAddress::default();
            // alloc_assert (tcp_addr);
            let mut rc = tcp_addr.resolve(tcp_address_, false, options.ipv6);

            if rc == 0 {
                tcp_addr.to_string(endpoint_uri_pair_);
                if self.endpoints.find(endpoint_uri_pair_) == self.endpoints.end() {
                    rc = tcp_addr.resolve(tcp_address_, true, options.ipv6);
                    if rc == 0 {
                        tcp_addr.to_string(endpoint_uri_pair_);
                    }
                }
            }
            // LIBZMQ_DELETE (tcp_addr);
        }
        return endpoint_uri_pair_.clone();
    }
} // impl ZmqSocketBase

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct routing_socket_base_t {
    // protected:
    // methods from ZmqSocketBase
    // own methods
    // private:
    //  Outbound pipes indexed by the peer IDs.
    // typedef std::map<Blob, out_pipe_t> out_pipes_t;
    // out_pipes_t _out_pipes;
    pub _out_pipes: HashMap<Blob, out_pipe_t>,
    pub base: ZmqSocketBase,
    // Next assigned name on a zmq_connect() call used by ROUTER and STREAM socket types
    // std::string _connect_routing_id;
    pub _connect_routing_id: String,
}

impl routing_socket_base_t {
    // routing_socket_base_t (class ZmqContext *parent_, u32 tid_, sid_: i32);
    // routing_socket_base_t::routing_socket_base_t (class ZmqContext *parent_,
    // u32 tid_,
    // sid_: i32) :
    // ZmqSocketBase (parent_, tid_, sid_)
    // {
    // }
    pub fn new(parent_: &mut ZmqContext, options: &mut ZmqOptions, tid_: u32, sid_: i32) -> Self {
        Self {
            _out_pipes: HashMap::new(),
            base: ZmqSocketBase::new(parent_, options, tid_, sid_, false),
            _connect_routing_id: String::new(),
        }
    }

    // ~routing_socket_base_t () ZMQ_OVERRIDE;
    // routing_socket_base_t::~routing_socket_base_t ()
    // {
    // zmq_assert (_out_pipes.empty ());
    // }

    // int xsetsockopt (option_: i32, const optval_: *mut c_void, ptvallen_: usize) ZMQ_OVERRIDE;
    pub fn xsetsockopt(&mut self, option_: i32, optval_: &[u8], optvallen_: usize) -> i32 {
        match (option_) {
            ZMQ_CONNECT_ROUTING_ID => {
                // TODO why isn't it possible to set an empty connect_routing_id
                //   (which is the default value)
                if (optvallen_ > 0) {
                    // self._connect_routing_id.assign(static_cast <const char
                    // * > (optval_),
                    // optvallen_);
                    self._connect_routing_id +=
                        String::from_utf8_lossy(optval_).to_string().as_str();
                    return 0;
                }
            }
            _ => {}
        }
        errno = EINVAL;
        return -1;
    }

    // void xwrite_activated (pipe_: &mut pipe_t) ZMQ_FINAL;
    pub fn xwrite_activated(&mut self, pipe_: &mut pipe_t) {
        // const out_pipes_t::iterator end = _out_pipes.end ();
        let (end_blob, end_pipe) = self._out_pipes.last().unwrap();
        // out_pipes_t::iterator it;
        // for (it = _out_pipes.begin (); it != end; ++it)
        // let mut pipe_ref: &mut pipe_t
        for (it_blob, it_pipe) in self._out_pipes.iter_mut() {
            // if (it.second.pipe == pipe_) {
            //     break;
            // }
            if it_pipe == pipe_ && it_pipe != end_pipe && it_pipe.active == false {
                it_pipe.active = true;
                break;
            }
        }

        // zmq_assert (it != end);
        // zmq_assert (!it.second.active);
        // it.second.active = true;
    }

    // std::string extract_connect_routing_id ();
    pub fn extract_connect_routing_id(&mut self) -> String {
        let res = self._connect_routing_id.clone();
        self._connect_routing_id.clear();
        return res;
    }

    // bool connect_routing_id_is_set () const;
    pub fn connect_routing_id_is_set(&mut self) -> bool {
        return !self._connect_routing_id.is_empty();
    }

    // void add_out_pipe (Blob routing_id_, pipe_: &mut pipe_t);
    pub fn add_out_pipe(&mut self, routing_id_: Blob, pipe_: &mut pipe_t) {
        //  Add the record into output pipes lookup table
        let outpipe = out_pipe_t::new(pipe_, true);
        let ok = self
            ._out_pipes
            .ZMQ_MAP_INSERT_OR_EMPLACE(routing_id_, outpipe)
            .second;
        // zmq_assert (ok);
    }

    // bool has_out_pipe (const Blob &routing_id_) const;
    pub fn has_out_pipe(&mut self, routing_id_: &mut Blob) -> bool {
        return 0 != _out_pipes.count(routing_id_);
    }

    // out_pipe_t *lookup_out_pipe (const Blob &routing_id_);
    pub fn lookup_out_pipe(&mut self, routing_id_: &mut Blob) -> Option<out_pipe_t> {
        // TODO we could probably avoid constructor a temporary Blob to call this function
        // out_pipes_t::iterator it = _out_pipes.find (routing_id_);
        // return it == _out_pipes.end () ? null_mut() : &it.second;
        let result = self._out_pipes.iter().find(routing_id_);
        if result.is_some() {
            Some(result.unwrap())
        }
        None
    }

    // const out_pipe_t *lookup_out_pipe (const Blob &routing_id_) const;

    // void erase_out_pipe (const pipe_: &mut pipe_t);
    pub fn erase_out_pipe(&mut self, pipe_: &mut pipe_t) {
        let erased = _out_pipes.erase(pipe_.get_routing_id());
        // zmq_assert (erased);
    }

    // out_pipe_t try_erase_out_pipe (const Blob &routing_id_);
    pub fn try_erase_out_pipe(&mut self, routing_id_: &mut Blob) -> Option<out_pipe_t> {
        // const out_pipes_t::iterator it = _out_pipes.find (routing_id_);
        let result = self._out_pipes.remove(routing_id_);
        //     if result.is_some() {
        //
        //     }
        //     out_pipe_t res = {null_mut(), false};
        // if (it != _out_pipes.end ()) {
        // res = it.second;
        // _out_pipes.erase (it);
        // }
        // return res;
        // }
        result
    }

    // template <typename Func> bool any_of_out_pipes (Func func_)
    pub fn any_of_out_pipes(&mut self) -> bool {
        let mut res = false;
        // for (out_pipes_t::iterator it = _out_pipes.begin (),
        //                            end = _out_pipes.end ();
        //      it != end && !res; ++it) {
        //     res |= func_ (*it->second.pipe);
        // }
        for pipe in self._out_pipes.iter_mut() {}

        return res;
    }
}

//  Send a monitor event
