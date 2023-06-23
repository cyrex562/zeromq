use std::collections::HashMap;
use std::mem;
use std::ptr::null_mut;
use std::sync::atomic::Ordering;
use std::sync::Mutex;

use anyhow::{anyhow, bail};
use bincode::options;
use libc::{c_char, c_int, c_void, EAGAIN, EINTR, EINVAL};
use serde::{Deserialize, Serialize};
use windows::Win32::Networking::WinSock::SOL_SOCKET;
use crate::address::{sockaddr_tipc, ZmqAddress};
use crate::channel::channel_xrecv;
use crate::client::client_xrecv;

use crate::command::ZmqCommand;
use crate::context::{bool_to_vec, get_effective_conflate_option, i32_to_vec, str_to_vec, ZmqContext};
use crate::cpu_time::get_cpu_tick_counter;
use crate::dealer::dealer_xrecv;
use crate::defines::{retired_fd, TIPC_ADDR_ID, ZMQ_BLOCKY, ZMQ_CONNECT_ROUTING_ID, ZMQ_DEALER, ZMQ_DGRAM, ZMQ_DISH, ZMQ_DONTWAIT, ZMQ_EVENT_ACCEPT_FAILED, ZMQ_EVENT_ACCEPTED, ZMQ_EVENT_BIND_FAILED, ZMQ_EVENT_CLOSE_FAILED, ZMQ_EVENT_CLOSED, ZMQ_EVENT_CONNECT_DELAYED, ZMQ_EVENT_CONNECT_RETRIED, ZMQ_EVENT_CONNECTED, ZMQ_EVENT_DISCONNECTED, ZMQ_EVENT_HANDSHAKE_FAILED_AUTH, ZMQ_EVENT_HANDSHAKE_FAILED_NO_DETAIL, ZMQ_EVENT_HANDSHAKE_FAILED_PROTOCOL, ZMQ_EVENT_HANDSHAKE_SUCCEEDED, ZMQ_EVENT_LISTENING, ZMQ_EVENT_MONITOR_STOPPED, ZMQ_EVENT_PIPES_STATS, ZMQ_EVENTS, ZMQ_FD, ZMQ_IPV6, ZMQ_LAST_ENDPOINT, ZMQ_LINGER, ZMQ_POLLIN, ZMQ_POLLOUT, ZMQ_PUB, ZMQ_RADIO, ZMQ_RCVHWM, ZMQ_RCVMORE, ZMQ_RECONNECT_STOP_AFTER_DISCONNECT, ZMQ_REQ, ZMQ_SNDHWM, ZMQ_SNDMORE, ZMQ_SUB, ZMQ_THREAD_SAFE, ZMQ_XPUB, ZMQ_XSUB, ZMQ_ZERO_COPY_RECV, ZmqHandle};
use crate::devpoll::ZmqPoller;
use crate::endpoint::EndpointType::endpoint_type_none;
use crate::endpoint::{make_unconnected_bind_endpoint_pair, ZmqEndpoint};
use crate::engine_interface::ZmqEngineInterface;
use crate::defines::ZmqFileDesc;
use crate::dgram::dgram_xrecv;
use crate::dish::{dish_xrecv};
use crate::endpoint_uri::EndpointUriPair;
use crate::mailbox::ZmqMailbox;
use crate::mailbox_interface::ZmqMailboxInterface;
use crate::mailbox_safe::ZmqMailboxSafe;
use crate::message::{ZMQ_MSG_MORE, ZmqMessage};
use crate::ops::{
    zmq_bind, zmq_close, zmq_msg_init_size, zmq_msg_send, zmq_setsockopt,
    zmq_socket,
};
use crate::out_pipe::ZmqOutPipe;
use crate::own::ZmqOwn;
use crate::pair::pair_xrecv;
use crate::pgm_socket::PgmSocket;
use crate::pipe::{send_hello_msg, ZmqPipe};
use crate::pull::pull_xrecv;
use crate::zmq_pub::{pub_xrecv};
use crate::radio::radio_xrecv;
use crate::rep::rep_xrecv;
use crate::req::req_xrecv;
use crate::router::router_xrecv;
use crate::server::server_xrecv;
use crate::session_base::ZmqSessionBase;
use crate::signaler::ZmqSignaler;
use crate::socket_base_ops::ZmqSocketBaseOps;
use crate::socket_option::ZmqSocketOption;
use crate::stream::{stream_xrecv, xrecv};
use crate::thread_context::ZmqThreadContext;
use crate::transport::ZmqTransport;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ZmqSocketType {
    ZmqPair,
    ZmqPub,
    ZmqSub,
    ZmqReq,
    ZmqRep,
    ZmqDealer,
    ZmqRouter,
    ZmqPull,
    ZmqPush,
    ZmqXPub,
    ZmqXSub,
    ZmqServer,
    ZmqClient,
    ZmqRadio,
    ZmqDish,
    ZmqGather,
    ZmqScatter,
    ZmqChannel
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ZmqSocket<'a> {
    // the type of socket
    pub socket_type: ZmqSocketType,
    // reference to the socket's parent context
    pub context: &'a mut ZmqContext<'a>,
    //  Map of open endpoints.
    pub endpoints: HashMap<String, ZmqEndpoint<'a>>,
    // id of the parent context's thread that performs I/O for this socket
    pub thread_id: i32,
    pub sent_seqnum: u64,
    pub term_acks: u32,
    pub fd: ZmqFileDesc,
    pub destination: ZmqAddress,
    //  Used to check whether the object is a socket.
    // pub tag: u32,
    //  If true, associated context was already terminated.
    pub ctx_terminated: bool,
    //  If true, object should have been already destroyed. However,
    //  destruction is delayed while we unwind the stack to the point
    //  where it doesn't intersect the object being destroyed.
    pub destroyed: bool,

    //  Socket's mailbox object.
    // ZmqMailboxInterface *mailbox;
    // pub mailbox: Option<ZmqMailboxInterface>,

    //  List of attached pipes.
    // pub pipes: Vec<ZmqPipe>,

    //  Reaper's poller and handle of this socket within it.
    // Poller *poller;
    pub poller: Option<ZmqPoller>,
    // Poller::handle_t _handle;
    pub handle: Option<ZmqHandle>,

    //  Timestamp of when commands were processed the last time.
    pub last_tsc: u64,

    //  Number of messages received since last command processing.
    pub ticks: i32,

    //  True if the last message received had MORE flag set.
    pub rcvmore: bool,

    //  Improves efficiency of time measurement.
    // pub clock: clock_t,

    // Monitor socket;
    pub monitor_socket: Vec<u8>,

    // Bitmask of events being monitored
    pub monitor_events: i64,

    // Last socket endpoint resolved URI
    pub last_endpoint: String,

    // Indicate if the socket is thread safe
    pub thread_safe: bool,

    // Signaler to be used in the reaping stage
    pub reaper_signaler: Option<ZmqSignaler>,

    // Mutex to synchronize access to the monitor Pair socket
    pub monitor_sync: Mutex<u8>,

    // // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqSocketBase)

    // Add a flag for mark disconnect action
    pub disconnected: bool,
}

impl<'a> ZmqSocket<'a> {
    pub fn new(parent: &mut ZmqContext, thread_id: i32, sock_id: i32, thread_safe: bool) -> Self {
        let mut out = Self::default();
        // out.tag = 0xbaddecafu32;
        out.ctx_terminated = false;
        out.destroyed = false;
        out.last_tsc = 0;
        out.ticks = 0;
        out.rcvmore = false;
        out.monitor_socket = vec![];
        out.monitor_events = 0;
        out.thread_safe = thread_safe;
        out.reaper_signaler = None;
        out.monitor_sync = Mutex::new(0);
        out.poller = None;
        out.sync = Mutex::new(0);
        out.handle = None;
        out.context = parent;
        out.thread_id = thread_id;

        parent.socket_id = sock_id;
        parent.ipv6 = (parent.get(ZMQ_IPV6) != 0);
        parent.linger.store(
            if parent.get(ZMQ_BLOCKY) { -1 } else { 0 },
            Ordering::Relaxed,
        );
        parent.zero_copy = parent.get(ZMQ_ZERO_COPY_RECV) != 0;

        if out.thread_safe {
            out.mailbox = Some(ZmqMailboxSafe::new(&mut out.sync));
            // zmq_assert (mailbox);
        } else {
            let mut m = ZmqMailbox::new();
            // zmq_assert (m);

            if m.get_fd() != retired_fd as usize {
                out.mailbox = Some(m);
            } else {
                // LIBZMQ_DELETE (m);
                out.mailbox = None;
            }
        }
        out
    }

    //  Returns false if object is not a socket.
    // pub fn check_tag(&self) -> bool {
    //     return self._tag == 0xbaddecafu32;
    // }

    //  Returns whether the socket is thread-safe.
    pub fn is_thread_safe(&self) -> bool {
        return self.thread_safe;
    }

    //  Create a socket of a specified type.
    // static ZmqSocketBase * create (type_: i32, ZmqContext *parent_, uint32_t tid, sid_: i32);

    // pub fn create (type_: i32, parent_: &mut ZmqContext, tid: u32, sid_: i32) -> anyhow::Result<Self>
    // {
    //     let mut s: Self;
    //     // ZmqSocketBase *s = NULL;
    //     match type_ {
    //         ZMQ_PAIR => { s = Self::new(parent_, tid, sid_); }
    //             // s = new (std::nothrow) ZmqPair (parent_, tid, sid_);
    //             // break;
    //         ZMQ_PUB =>{}
    //             // s = new (std::nothrow) pub_t (parent_, tid, sid_);
    //             // break;
    //         ZMQ_SUB =>
    //             s = new (std::nothrow) ZmqSub (parent_, tid, sid_);
    //             break;
    //         ZMQ_REQ =>
    //             s = new (std::nothrow) req_t (parent_, tid, sid_);
    //             break;
    //         ZMQ_REP =>
    //             s = new (std::nothrow) rep_t (parent_, tid, sid_);
    //             break;
    //         ZMQ_DEALER =>
    //             s = new (std::nothrow) ZmqDealer (parent_, tid, sid_);
    //             break;
    //         ZMQ_ROUTER =>
    //             s = new (std::nothrow) router_t (parent_, tid, sid_);
    //             break;
    //         ZMQ_PULL =>
    //             s = new (std::nothrow) ZmqPull (parent_, tid, sid_);
    //             break;
    //         ZMQ_PUSH =>
    //             s = new (std::nothrow) ZmqPush (parent_, tid, sid_);
    //             break;
    //         ZMQ_XPUB =>
    //             s = new (std::nothrow) XPub (parent_, tid, sid_);
    //             break;
    //         ZMQ_XSUB =>
    //             s = new (std::nothrow) XSub (parent_, tid, sid_);
    //             break;
    //         ZMQ_STREAM =>
    //             s = new (std::nothrow) ZmqStream (parent_, tid, sid_);
    //             break;
    //         ZMQ_SERVER =>
    //             s = new (std::nothrow) ZmqServer (parent_, tid, sid_);
    //             break;
    //         ZMQ_CLIENT =>
    //             s = new (std::nothrow) client_t (parent_, tid, sid_);
    //             break;
    //         ZMQ_RADIO =>
    //             s = new (std::nothrow) ZmqRadio (parent_, tid, sid_);
    //             break;
    //         ZMQ_DISH =>
    //             s = new (std::nothrow) ZmqDish (parent_, tid, sid_);
    //             break;
    //         ZMQ_GATHER =>
    //             s = new (std::nothrow) ZmqGather (parent_, tid, sid_);
    //             break;
    //         ZMQ_SCATTER =>
    //             s = new (std::nothrow) ZmqScatter (parent_, tid, sid_);
    //             break;
    //         ZMQ_DGRAM =>
    //             s = new (std::nothrow) ZmqDgram (parent_, tid, sid_);
    //             break;
    //         ZMQ_PEER =>
    //             s = new (std::nothrow) ZmqPeer (parent_, tid, sid_);
    //             break;
    //         ZMQ_CHANNEL =>
    //             s = new (std::nothrow) channel_t (parent_, tid, sid_);
    //             break;
    //         _ =>
    //             errno = EINVAL;
    //             return NULL;
    //     }
    //
    //     // alloc_assert (s);
    //
    //     if (s->mailbox == NULL) {
    //         s->_destroyed = true;
    //         LIBZMQ_DELETE (s);
    //         return NULL;
    //     }
    //
    //     return s;
    // }

    //  Interrupt blocking call if the socket is stuck in one.
    //  This function can be called from a different thread!
    // void Stop ();
    pub fn stop(&mut self) {
        //  Called by ctx when it is terminated (zmq_ctx_term).
        //  'Stop' command is sent from the threads that called zmq_ctx_term to
        //  the thread owning the socket. This way, blocking call in the
        //  owner thread can be interrupted.
        // TODO
        // self.context.send_stop(self.thread_id);
    }

    //  Interface for communication with the API layer.
    // int setsockopt (option_: i32, const optval_: *mut c_void, optvallen_: usize);
    pub fn setsockopt(
        &mut self,
        options: &mut ZmqContext,
        opt_kind: i32,
        opt_val: &[u8],
        opt_len: usize,
    ) -> anyhow::Result<()> {
        // scoped_optional_lock_t sync_lock (_thread_safe ? &sync : NULL);
        // let mut sync_lock = if self._thread_safe {
        //     &mut sync
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
        opt_kind: ZmqSocketOption,
    ) -> anyhow::Result<Vec<u8>> {
        // scoped_optional_lock_t sync_lock (_thread_safe ? &sync : NULL);
        // let mut sync_lock = if self.thread_safe {
        //     &mut sync
        // } else {
        //     null_mut()
        // };
        // if (unlikely (_ctx_terminated)) {
        //     errno = ETERM;
        //     return -1;
        // }

        //  First, check whether specific socket type overloads the option.
        let rc = self.xgetsockopt(self, opt_kind);
        // if rc == 0 || errno != EINVAL {
        //     return rc;
        // }

        if rc.is_ok() {
            return Ok(rc.unwrap());
        }

        match opt_kind {
            ZmqSocketOption::ZMQ_RCVMORE => return bool_to_vec(self.rcvmore),
            ZmqSocketOption::ZMQ_FD => {
                if self.thread_safe {
                    // thread safe socket doesn't provide file descriptor
                    return Err(anyhow!(
                        "thread safe socket doenst provide a file descriptor"
                    ));
                }
                return Ok(self.fd.to_le_bytes().to_vec());
            }
            ZmqSocketOption::ZMQ_EVENTS => {
                match self.process_commands(0, false) {
                    Ok(_) => {}
                    Err(e) => {
                        return Err(anyhow!("failed to process comands: {}", e));
                    }
                }
                return if self.has_out() {
                    i32_to_vec(ZMQ_POLLOUT)
                } else if self.has_in {
                    i32_to_vec(ZMQ_POLLIN)
                } else {
                    i32_to_vec(0)
                };
            }
            ZmqSocketOption::ZMQ_LAST_ENDPOINT => {
                return str_to_vec(&self.last_endpoint);
            }
            ZmqSocketOption::ZMQ_THREAD_SAFE => {
                return bool_to_vec(self.thread_safe);
            }

            _ => {
                return bail!("invalid option: {:?}", opt_kind);
            }
        }
    }

    pub fn lower_getsockopt_i32(&mut self, optkind: i32) -> anyhow::Result<i32> {
        let mut rc = 0i32;
        let mut out_val: i32 = 0i32;
        let mut out_len: c_int = mem::size_of::<i32>() as c_int;
        rc = unsafe {
            libc::getsockopt(
                self.fd,
                SOL_SOCKET,
                optkind,
                &mut out_val as *mut i32 as *mut c_char,
                &mut out_len as *mut c_int,
            )
        };
        if rc != 0 {
            bail!("failed to getsockopt");
        }
        Ok(out_val)
    }

    // int Bind (endpoint_uri_: *const c_char);
    pub fn bind(
        &mut self,
        endpoint_uri: &str,
    ) -> anyhow::Result<()> {
        // scoped_optional_lock_t sync_lock (_thread_safe ? &sync : NULL);

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
        if self.parse_uri(endpoint_uri, &mut protocol, &mut address) || self.check_protocol(self.context, &protocol) {
            bail!("failed to parse endpoint_uri")
        }

        if protocol == ZmqTransport::ZmqInproc {
            // const ZmqEndpoint endpoint = {this, options};
            let mut endpoint = ZmqEndpoint::new(self);
            self.context.register_endpoint(endpoint_uri, &mut endpoint)?;
            self.connect_pending(endpoint_uri, self);
            self.last_endpoint.assign(endpoint_uri);
            options.connected = true;
            return Ok(());
        }

        // #if defined ZMQ_HAVE_OPENPGM || defined ZMQ_HAVE_NORM
        // #if defined ZMQ_HAVE_OPENPGM && defined ZMQ_HAVE_NORM
        if protocol == ZmqTransport::ZmqPgm || protocol == ZmqTransport::ZmqEpgm || protocol == ZmqTransport::ZmqNorm {
            // #elif defined ZMQ_HAVE_OPENPGM
            //     if (protocol == protocol_name::pgm || protocol == protocol_name::epgm) {
            // // #else // defined ZMQ_HAVE_NORM
            //     if (protocol == protocol_name::norm) {
            // #endif
            //  For convenience's sake, Bind can be used interchangeable with
            //  connect for PGM, EPGM, NORM transports.
            self.connect(endpoint_uri)?;
            options.connected = true;
            return Ok(());
            // if rc != -1 {
            //
            // }
            // return rc;
        }
        // #endif

        if protocol == ZmqTransport::ZmqUdp {
            if !(options.type_ == ZMQ_DGRAM || options.type_ == ZMQ_DISH) {
                bail!("no compatible protocol")
            }

            //  Choose the I/O thread to run the session in.
            let io_thread = self.choose_io_thread(options.affinity).unwrap();
            if !io_thread {
                bail!("EMTHREAD")
            }

            let mut paddr = ZmqAddress::default();
            paddr.address = address;
            paddr.resolve()?;

            let mut session = ZmqSessionBase::create(

                &mut io_thread,
                true,
                self,
                Some(&mut paddr),
            )?;
            // errno_assert (session);

            //  Create a bi-directional pipe.
            // let mut parents: [Box<dyn ZmqObject>; 2] = [Box::new(self), Box::new(session)];
            let mut new_pipes: [ZmqPipe; 2] = [ZmqPipe::default(), ZmqPipe::default()];

            let mut hwms: [i32; 2] = [options.sndhwm, options.rcvhwm];
            let mut conflates: [bool; 2] = [false, false];
            self.pipepair((session, self), new_pipes, hwms, conflates)?;
            // errno_assert (rc == 0);

            //  Attach local end of the pipe to the socket object.
            self.attach_pipe(&mut new_pipes[0], true, true)?;
            let mut newpipe = new_pipes[0].clone();

            //  Attach remote end of the pipe to the session object later on.
            session.attach_pipe(&mut new_pipes[1]);

            //  Save last endpoint URI
            paddr.to_string();

            //  TODO shouldn't this use _last_endpoint instead of endpoint_uri_? as in the other cases
            let mut ep = EndpointUriPair::new(endpoint_uri, "", endpoint_type_none);
            // todo add trait ZmqOwn to ZmqSessionBase
            self.add_endpoint(&ep, &mut session, &mut newpipe);

            Ok(())
        }

        //  Remaining transports require to be run in an I/O thread, so at this
        //  point we'll choose one.
        let mut io_thread = self.choose_io_thread(options.affinity).expect("EMTHREAD");

        if protocol == ZmqTransport::ZmqTcp {
            let mut listener = TcpListener::new(&mut io_thread, this, options);
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
        if protocol == ZmqTransport::ZmqWs || protocol == ZmqTransport::ZmqWss {
            let listener = ZmqWsListener::new(
                &mut io_thread,
                self,
                options,
                protocol == ZmqTransport::ZmqWss,
            );
            // #else
            //     if protocol == protocol_name::ws {
            let mut listener = ZmqWsListener::new(&mut io_thread, self, options, false);
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
        if protocol == ZmqTransport::ZmqIpc {
            let mut listener = IpcListener::new(options, &mut io_thread.unwrap(), this);
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
        if protocol == ZmqTransport::ZmqTipc {
            listener = ZmqTipcListener::new(&mut io_thread, this, options);
            // alloc_assert (listener);
            if listener.set_local_address(address.c_str()).is_err() {
                event_bind_failed(make_unconnected_bind_endpoint_pair(&address), zmq_errno());
                bail!("failed to set local address");
            }

            // Save last endpoint URI
            listener.get_local_address(_last_endpoint);

            // TODO shouldn't this use _last_endpoint as in the other cases?
            add_endpoint(
                make_unconnected_bind_endpoint_pair(endpoint_uri),
                (listener),
                null_mut(),
            );
            options.connected = true;
            return Ok(());
        }
        // #endif
        // #if defined ZMQ_HAVE_VMCI
        if protocol == ZmqTransport::ZmqVmci {
            let mut listener = ZmqVmciListener::new(&mut io_thread, this, options);
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
        bail!("Bind failed")
    }

    // int connect (endpoint_uri_: *const c_char);
    pub fn connect(&mut self, endpoint_uri: &str) -> anyhow::Result<()> {
        // scoped_optional_lock_t sync_lock (_thread_safe ? &sync : null_mut());
        self.connect_internal(endpoint_uri)
    }

    // int TermEndpoint (endpoint_uri_: *const c_char);
    pub fn term_endpoint(
        &mut self,
        options: &mut ZmqContext,
        endpoint_uri_: &str,
    ) -> anyhow::Result<()> {
        // scoped_optional_lock_t sync_lock (_thread_safe ? &sync : null_mut());

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
        self.process_commands(0, false)?;
        // if (unlikely (rc != 0)) {
        //     return -1;
        // }

        //  Parse endpoint_uri_ string.
        let mut uri_protocol = String::new();
        let mut uri_path = String::new();
        parse_uri(endpoint_uri_, uri_protocol, uri_path)?;
        check_protocol(&uri_protocol)?;

        let mut endpoint_uri_str = String::from(endpoint_uri_);

        // Disconnect an inproc socket
        if uri_protocol == ZmqTransport::ZmqInproc {
            self.unregister_endpoint(&endpoint_uri_str, self)?;
            self._inprocs.erase_pipes(&endpoint_uri_str)?;
        }

        let resolved_endpoint_uri = if uri_protocol == ZmqTransport::ZmqTcp {
            self.resolve_tcp_addr(options, &mut endpoint_uri_str, uri_path.c_str())?;
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
            bail!("no endpoints");
        }

        // for (endpoints_t::iterator it = range.first; it != range.second; += 1it) {
        //     //  If we have an associated pipe, terminate it.
        //     if (it->second.second != null_mut())
        //         it->second.second->terminate (false);
        //     term_child (it->second.first);
        // }
        for (_, pipe) in self.endpoints.values_mut() {
            pipe.terminate(false)?;
        }
        // _endpoints.erase (range.first, range.second);
        self.endpoints.clear()?;

        if options.reconnect_stop & ZMQ_RECONNECT_STOP_AFTER_DISCONNECT {
            self.disconnected = true;
        }

        Ok(())
    }

    // int send (msg: &mut ZmqMessage flags: i32);
    pub fn send(
        &mut self,
        msg: &mut ZmqMessage,
        options: &mut ZmqContext,
        flags: i32,
    ) -> anyhow::Result<()> {
        // scoped_optional_lock_t sync_lock (_thread_safe ? &sync : null_mut());

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
        flags: i32,
    ) -> anyhow::Result<()> {
        // scoped_optional_lock_t sync_lock (_thread_safe ? &sync : null_mut());

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

        // TODO: figure out where to get the correct pipe from for recv. Check each pipe in the enxpoints?
        let pipe = self.get_pipe(0);

        match self.socket_type {
            ZmqSocketType::ZmqChannel => channel_xrecv(self, msg)?,
            ZmqSocketType::ZmqClient => client_xrecv(self, msg)?,
            ZmqSocketType::ZmqDealer => dealer_xrecv(self, msg)?,
            ZmqSocketType::ZmqDgram => dgram_xrecv(self, msg)?,
            ZmqSocketType::ZmqDish => dish_xrecv(self, msg)?,
            ZmqSocketType::ZmqPair => pair_xrecv(self, msg)?,
            ZmqSocketType::ZmqPub => pub_xrecv(self, msg)?,
            ZmqSocketType::ZmqPull => pull_xrecv(self, msg)?,
            ZmqSocketType::ZmqRadio => radio_xrecv(self, msg)?,
            ZmqSocketType::ZmqRep => rep_xrecv(self, msg)?,
            ZmqSocketType::ZmqReq => req_xrecv(self, msg)?,
            ZmqSocketType::ZmqRouter => router_xrecv(self, msg)?,
            ZmqSocketType::ZmqServer => server_xrecv(self, msg)?,
            ZmqSocketType::ZmqStream => stream_xrecv(self, msg)?,
            _ => {
                bail!("unsupported socket type: {:?}", self.socket_type)
            }
        }


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

    // void add_signaler (ZmqSignaler *s_);
    pub fn add_signaler(&mut self, s_: &mut ZmqSignaler) {
        // zmq_assert (_thread_safe);

        // scoped_lock_t sync_lock (sync);
        // (static_cast<ZmqMailboxSafe *> (mailbox))->add_signaler (s_);
        self.mailbox.add_signaler(s_);
    }

    // void remove_signaler (ZmqSignaler *s_);
    pub fn remove_signaler(&mut self, s_: &mut ZmqSignaler) {
        // zmq_assert (_thread_safe);

        // scoped_lock_t sync_lock (sync);
        // (static_cast<ZmqMailboxSafe *> (mailbox))->remove_signaler (s_);
        self.mailbox.remove_signaler(s_);
    }

    // int close ();
    pub fn close(&mut self) -> anyhow::Result<()> {
        // scoped_optional_lock_t sync_lock (_thread_safe ? &sync : null_mut());

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
        // scoped_optional_lock_t sync_lock (_thread_safe ? &sync : null_mut());

        ops.xjoin(group_)
    }
    // int leave (group_: *const c_char);
    pub fn leave(&mut self, group_: &str) -> anyhow::Result<()> {
        // scoped_optional_lock_t sync_lock (_thread_safe ? &sync : null_mut());
        ops.xleave(group_)
    }

    //  Using this function reaper thread ask the socket to register with
    //  its poller.
    // void start_reaping (Poller *poller_);
    pub fn start_reaping(&mut self, poller_: &mut ZmqPoller) {
        //  Plug the socket to the reaper thread.
        self.poller = Some(poller_.clone());

        let fd: ZmqFileDesc;

        if !self.thread_safe {
            fd = self.mailbox.get_fd();
        } else {
            // scoped_optional_lock_t sync_lock (_thread_safe ? &sync : null_mut());

            self.reaper_signaler = Some(ZmqSignaler::default()); //new (std::nothrow) ZmqSignaler ();
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
    // void in_event () ;
    pub fn in_event(&mut self) {
        //  This function is invoked only once the socket is running in the context
        //  of the reaper thread. Process any commands from other threads/sockets
        //  that may be available at the moment. Ultimately, the socket will
        //  be destroyed.
        {
            // scoped_optional_lock_t sync_lock (_thread_safe ? &sync : null_mut());

            //  If the socket is thread safe we need to unsignal the reaper signaler
            if (self.thread_safe) {
                self.reaper_signaler.recv();
            }

            self.process_commands(0, false);
        }
        self.check_destroy();
    }

    // void out_event () ;
    pub fn out_event(&mut self) {
        unimplemented!()
        // zmq_assert (false);
    }

    // void timer_event (id_: i32) ;
    pub fn timer_event(&mut self, id_: i32) {
        unimplemented!()
    }

    //  i_pipe_events interface implementation.
    // void read_activated (ZmqPipe *pipe_) ;
    pub fn read_activated(&mut self, pipe: &mut ZmqPipe) {
        ops.xread_activated(pipe);
    }

    // void write_activated (ZmqPipe *pipe_) ;
    pub fn write_activated(&mut self, pipe: &mut ZmqPipe) {
        ops.xwrite_activated(pipe);
    }

    // void hiccuped (ZmqPipe *pipe_) ;
    pub fn hiccuped(&mut self, options: &mut ZmqContext, pipe: &mut ZmqPipe) {
        if (options.immediate == 1) {
            pipe.terminate(false);
        } else {
            // Notify derived sockets of the Hiccup
            ops.xhiccuped(pipe);
        }
    }

    // void pipe_terminated (ZmqPipe *pipe_) ;
    pub fn pipe_terminated(&mut self, pipe: &mut ZmqPipe) {
        //  Notify the specific socket type about the pipe termination.
        ops.xpipe_terminated(pipe);

        // Remove pipe from inproc pipes
        self._inprocs.erase_pipe(pipe);

        //  Remove the pipe from the list of attached pipes and confirm its
        //  termination if we are already shutting down.
        self.pipes.erase(pipe);

        // Remove the pipe from _endpoints (set it to NULL).
        let identifier = pipe.get_endpoint_pair().identifier();
        if (!identifier.empty()) {
            // std::pair<endpoints_t::iterator, endpoints_t::iterator> range;
            // range = _endpoints.equal_range (identifier);
            // for (endpoints_t::iterator it = range.first; it != range.second; += 1it) {
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
        options: &mut ZmqContext,
        endpoint: &str,
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
        if (endpoint == null_mut()) {
            self.stop_monitor(options, false);
            return Ok(());
        }
        //  Parse endpoint_uri_ string.
        let mut protocol = String::new();
        let mut address = String::new();
        if (self.parse_uri(endpoint, &mut protocol, &mut address) || self.check_protocol(options, &protocol)) {
            bail!("failed to parse uri and/or protocol");
        }

        //  Event notification only supported over inproc://
        if (protocol != ZmqTransport::ZmqInproc) {
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
        self.monitor_socket = zmq_socket(get_ctx(), type_)?;

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
        rc = zmq_bind(ctx, self._monitor_socket, endpoint);
        if (rc == -1) {
            self.stop_monitor(options, false);
        }
        Ok(())
    }

    // void event_connected (const EndpointUriPair &endpoint_uri_pair_,
    //                       ZmqFileDesc fd);
    pub fn event_connected(
        &mut self,
        options: &mut ZmqContext,
        endpoint_uri_pair_: &EndpointUriPair,
        fd: ZmqFileDesc,
    ) {
        // u64 values[1] = { (fd)};
        let values: [u64; 1] = [fd as u64];
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
        options: &mut ZmqContext,
        endpoint_uri_pair_: &EndpointUriPair,
        err_: i32,
    ) {
        // u64 values[1] = { (err_)};
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
        options: &mut ZmqContext,
        endpoint_uri_pair_: &EndpointUriPair,
        interval_: i32,
    ) {
        // u64 values[1] = { (interval_)};
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
    //                       ZmqFileDesc fd);
    pub fn event_listening(
        &mut self,
        endpoint_uri_pair_: &EndpointUriPair,
        fd: ZmqFileDesc,
    ) {
        // u64 values[1] = { (fd)};
        let values: [u64; 1] = [fd as u64];
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
        options: &mut ZmqContext,
        endpoint_uri_pair_: &EndpointUriPair,
        err_: i32,
    ) {
        // u64 values[1] = { (err_)};
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
    //                      ZmqFileDesc fd);
    pub fn event_accepted(
        &mut self,
        endpoint_uri_pair_: &EndpointUriPair,
        fd: ZmqFileDesc,
    ) {
        // u64 values[1] = { (fd)};
        let values: [u64; 1] = [fd as u64];
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
        endpoint_uri_pair_: &EndpointUriPair,
        err_: i32,
    ) {
        // u64 values[1] = { (err_)};
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
    //                    ZmqFileDesc fd);
    pub fn event_closed(
        &mut self,
        endpoint_uri_pair_: &EndpointUriPair,
        fd: ZmqFileDesc,
    ) {
        // u64 values[1] = { (fd)};
        let values: [u64; 1] = [fd as u64];
        self.event(
            self.context,
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
        enpoint_uri_pair_: &EndpointUriPair,
        err_: i32,
    ) {
        // u64 values[1] = { (err_)};
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
    //                          ZmqFileDesc fd);
    pub fn event_disconnected(
        &mut self,
        options: &mut ZmqContext,
        endpoint_uri_pair_: &EndpointUriPair,
        fd: ZmqFileDesc,
    ) {
        // u64 values[1] = { (fd)};
        let values: [u64; 1] = [fd as u64];
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
        // u64 values[1] = { (err_)};
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
        options: &mut ZmqContext,
        endpoint_uri_pair_: &EndpointUriPair,
        err_: i32,
    ) {
        // u64 values[1] = { (err_)};
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
        options: &mut ZmqContext,
        endpoint_uri_pair_: &EndpointUriPair,
        err_: i32,
    ) {
        // u64 values[1] = { (err_)};
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
        options: &mut ZmqContext,
        endpoint_uri_pair_: &EndpointUriPair,
        err_: i32,
    ) {
        // u64 values[1] = { (err_)};
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
     * a message (PipePeerStats) to its peer. The message will carry the outbound
     * pipe stats and endpoint, and the reference to the socket object.
     * The inbound pipe on the I/O thread will then add its Own stats and endpoint,
     * and write back a message to the socket object (PipeStatsPublish) which
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
        // for (pipes_t::size_type i = 0, size = pipes.size (); i != size; += 1i) {
        //     pipes[i]->send_stats_to_peer (this);
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
    // void process_destroy () ;
    pub fn process_destroy(&mut self) {
        self.destroyed = true;
    }

    // int connect_internal (endpoint_uri_: *const c_char);
    pub fn connect_internal(
        &mut self,
        endpoint_uri: &str,
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
        if self.parse_uri(endpoint_uri, &mut protocol, &mut address) || self.check_protocol(options, &protocol) {
            bail!("failed to parse uri or check protocol");
        }

        if protocol == ZmqTransport::ZmqInproc {
            //  TODO: inproc connect is specific with respect to creating pipes
            //  as there's no 'reconnect' functionality implemented. Once that
            //  is in place we should follow generic pipe creation algorithm.

            //  Find the peer endpoint.
            let mut peer = self.find_endpoint(endpoint_uri);

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
            let mut parents: [&mut ZmqSocket; 2] = [
                self,
                if peer.socket == null_mut() {
                    self
                } else {
                    peer.socket
                },
            ];
            let mut new_pipes: [ZmqPipe; 2] = [ZmqPipe::default(), ZmqPipe::default()];

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
                self.pend_connection(endpoint_uri, &endpoint, &new_pipes);
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
                    send_hello_msg(&mut new_pipes[1], peer.options);
                }

                if peer.options.can_recv_disconnect_msg && peer.options.disconnect_msg.size() > 0 {
                    new_pipes[0].set_disconnect_msg(peer.options.disconnect_msg);
                }
                // #endif

                //  Attach remote end of the pipe to the peer socket. Note that peer's
                //  seqnum was incremented in find_endpoint function. We don't need it
                //  increased here.
                self.send_bind(peer.socket, &mut new_pipes[1], false);
            }

            //  Attach local end of the pipe to this socket object.
            self.attach_pipe(&mut new_pipes[0], false, true);

            // Save last endpoint URI
            self.last_endpoint.assign(endpoint_uri);

            // remember inproc connections for disconnect
            self._inprocs.emplace(endpoint_uri, &new_pipes[0]);

            options.connected = true;
            return Ok(());
        }
        let is_single_connect = (options.type_ == ZMQ_DEALER || options.type_ == ZMQ_SUB || options.type_ == ZMQ_PUB || options.type_ == ZMQ_REQ);
        // if (unlikely (is_single_connect)) {
        //     if (0 != _endpoints.count (endpoint_uri_)) {
        //         // There is no valid use for multiple connects for SUB-PUB nor
        //         // DEALER-ROUTER nor REQ-REP. Multiple connects produces
        //         // nonsensical results.
        //         return 0;
        //     }
        // }

        //  Choose the I/O thread to run the session in.
        let mut io_thread = self.choose_io_thread(options.affinity)?;
        // if (!io_thread) {
        //     errno = EMTHREAD;
        //     return -1;
        // }

        let mut paddr = ZmqAddress::new(&protocol, &address, self.get_ctx_mut());
        // alloc_assert (paddr);

        //  Resolve address (if needed by the protocol)
        if protocol == ZmqTransport::ZmqTcp {
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
            //     check+= 1;
            //     while (isalnum (*check) || isxdigit (*check) || *check == '.'
            //            || *check == '-' || *check == ':' || *check == '%'
            //            || *check == ';' || *check == '[' || *check == ']'
            //            || *check == '_' || *check == '*') {
            //         check+= 1;
            //     }
            // }
            if *check.is_ascii_alphanumeric() || *check.is_ascii_hexdigit() || *check.eq(&lbracket_bytes) || *check.eq(&colon_bytes) {
                check += 1;
                while *check.is_ascii_alphanumeric() || *check.is_ascii_hexdigit() || *check.eq(&dot_bytes) || *check.eq(&hyphen_bytes) || *check.eq(&colon_bytes) || *check.eq(&percent_bytes) || *check.eq(&semicolon_bytes) || *check.eq(&lbracket_bytes) || *check.eq(&rbracked_bytes) || *check.eq(&underscore_bytes) || *check.eq(&asterisk_bytes) {
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
        else if protocol == ZmqTransport::ZmqWs || protocol == ZmqTransport::ZmqWss {
            if protocol == ZmqTransport::ZmqWss {
                paddr.resolved.wss_addr = WssAddress::new();
                // alloc_assert (paddr.resolved.wss_addr);
                paddr.resolved.wss_addr.resolve(&address, false, options.ipv6)?;
            }
            // #else
            else if protocol == ZmqTransport::ZmqWs {
                // #endif

                paddr.resolved.ws_addr = WsAddress::new();
                // alloc_assert (paddr.resolved.ws_addr);
                paddr.resolved.ws_addr.resolve(&address, false, options.ipv6)?;
            }
        }
        // #endif

        // #if defined ZMQ_HAVE_IPC
        else if protocol == ZmqTransport::ZmqIpc {
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
            paddr.resolved.udp_addr.resolve(&address, false, options.ipv6)?;
            // if (rc != 0) {
            //     LIBZMQ_DELETE (paddr);
            //     return -1;
            // }
        }
        // TBD - Should we check address for ZMQ_HAVE_NORM???

        // #ifdef ZMQ_HAVE_OPENPGM
        else if protocol == ZmqTransport::ZmqPgm || protocol == ZmqTransport::ZmqEpgm {
            let mut res = pgm_addrinfo_t::new();
            let mut port_number = 0u16;
            // TODO
            // PgmSocket::init_address(&address, &mut res, &port_number)?;
            // if (res != null_mut()) {
            //     pgm_freeaddrinfo(res);
            // }
            // if rc != 0 || port_number == 0 {
            //     baiL!("failed to create PGM socket");
            // }
        }
        // #endif
        // #if defined ZMQ_HAVE_TIPC
        else if protocol == ZmqTransport::ZmqTipc {
            paddr.resolved.tipc_addr = ZmqTipcAddress::new();
            // alloc_assert(paddr.resolved.tipc_addr);
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
        else if protocol == ZmqTransport::ZmqVmci {
            paddr.resolved.vmci_addr = ZmqVmciAddress::new2(&mut self.get_ctx());
            // alloc_assert (paddr.resolved.vmci_addr);
            paddr.resolved.vmci_addr.resolve(&address)?;
            // if (rc != 0) {
            //     LIBZMQ_DELETE (paddr);
            //     return -1;
            // }
        }
        // #endif

        //  Create session.
        let mut session = ZmqSessionBase::create(ctx, &mut io_thread, true, self, options, Some(&mut paddr));
        // errno_assert (session);

        //  PGM does not support subscription forwarding; ask for all data to be
        //  sent to this pipe. (same for NORM, currently?)
        // #if defined ZMQ_HAVE_OPENPGM && defined ZMQ_HAVE_NORM
        let subscribe_to_all = protocol == ZmqTransport::ZmqPgm || protocol == ZmqTransport::ZmqEpgm || protocol == ZmqTransport::ZmqNorm || protocol == protocol_name::udp;
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
        //     ZmqPipe *newpipe = null_mut();
        let mut newpipe = ZmqPipe::default();

        if options.immediate != 1 || subscribe_to_all {
            //  Create a bi-directional pipe.
            let mut parents: [&mut ZmqSocket; 2] = [self, session];
            let mut new_pipes: [ZmqPipe; 2] = [ZmqPipe::default(), ZmqPipe::default()];

            let conflate = get_effective_conflate_option(options);

            let hwms: [i32; 2] = [
                if conflate { -1 } else { options.sndhwm },
                if conflate { -1 } else { options.rcvhwm },
            ];
            let conflates: [bool; 2] = [conflate, conflate];
            rc = pipepair(parents, new_pipes, hwms, conflates);
            // errno_assert(rc == 0);

            //  Attach local end of the pipe to the socket object.
            self.attach_pipe(&mut new_pipes[0], subscribe_to_all, true);
            newpipe = new_pipes[0].clone();

            //  Attach remote end of the pipe to the session object later on.
            session.attach_pipe(&new_pipes[1]);
        }

        //  Save last endpoint URI
        paddr.to_string(_last_endpoint);

        add_endpoint(
            make_unconnected_connect_endpoint_pair(endpoint_uri),
            &session,
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
        endpoint_uri_pair_: &EndpointUriPair,
        values_: &[u64],
        values_count_: u64,
        type_: u64,
    ) {
        // scoped_lock_t lock (_monitor_sync);
        if (self.monitor_events & type_) {
            self.monitor_event( type_, values_, values_count_, endpoint_uri_pair_);
        }
    }

    // Socket event data dispatch
    // void monitor_event (event_: u64,
    //                     const u64 values_[],
    //                     values_count_: u64,
    //                     const EndpointUriPair &endpoint_uri_pair_) const;
    pub fn monitor_event(
        &mut self,
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
                    // const u32 value =  (values_[0]);
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
                    zmq_msg_data(&mut raw_msg).extend_from_slice(&values_count_.to_le_bytes().as_slice());
                    zmq_msg_send(&mut raw_msg, _monitor_socket, ZMQ_SNDMORE);

                    //  Send values in third-Nth frames (64bit unsigned)
                    // for (u64 i = 0; i < values_count_; += 1i)
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
                    zmq_msg_data(&mut raw_msg).extend_from_slice(endpoint_uri_pair_.remote.as_bytes());
                    zmq_msg_send(&mut raw_msg, _monitor_socket, 0);
                }
                _ => {}
            }
        }
    }

    // Monitor socket cleanup
    // void stop_monitor (bool send_monitor_stopped_event_ = true);
    pub fn stop_monitor(&mut self, options: &mut ZmqContext, send_monitor_stopped_event_: bool) {
        // this is a private method which is only called from
        // contexts where the _monitor_sync mutex has been locked before

        if (self.monitor_socket) {
            if ((self.monitor_events & ZMQ_EVENT_MONITOR_STOPPED) != 0 && send_monitor_stopped_event_) {
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
    //                    ZmqOwn *endpoint_,
    //                    ZmqPipe *pipe_);
    pub fn add_endpoint(
        &mut self,
        endpoint_pair: &EndpointUriPair,
        endpoint: &mut ZmqSessionBase,
        pipe: &mut ZmqPipe,
    ) {
        //  Activate the session. Make it a child of this socket.
        self.launch_child(endpoint);
        self.endpoints.ZMQ_MAP_INSERT_OR_EMPLACE(
            endpoint_pair.identifier(),
            endpoint_pipe_t::new(endpoint, pipe),
        );

        if pipe != null_mut() {
            pipe.set_endpoint_pair(endpoint_pair.clone());
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
            // self.ZmqOwn::process_destroy ();
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
    //  Bind, is available and compatible with the socket type.
    // int check_protocol (protocol_: &str) const;

    pub fn check_protocol(
        &mut self,
        options: &mut ZmqContext,
        protocol_: &str,
    ) -> anyhow::Result<()> {
        //  First check out whether the protocol is something we are aware of.
        if protocol_ != ZmqTransport::ZmqInproc {
            // #if defined ZMQ_HAVE_IPC && protocol_ != protocol_name::ipc
            // #endif && protocol_ != protocol_name::tcp
            // #ifdef ZMQ_HAVE_WS && protocol_ != protocol_name::ws
            // #endif
            // #ifdef ZMQ_HAVE_WSS && protocol_ != protocol_name::wss
            // #endif
            // #if defined ZMQ_HAVE_OPENPGM
            //  pgm/epgm transports only available if 0MQ is compiled with OpenPGM. && protocol_ != protocol_name::pgm && protocol_ != protocol_name::epgm
            // #endif
            // #if defined ZMQ_HAVE_TIPC
            // TIPC transport is only available on Linux. && protocol_ != protocol_name::tipc
            // #endif
            // #if defined ZMQ_HAVE_NORM && protocol_ != protocol_name::norm
            // #endif
            // #if defined ZMQ_HAVE_VMCI && protocol_ != protocol_name::vmci
            // #endif && protocol_ != protocol_name::udp {
            // errno = EPROTONOSUPPORT;
            // return -1;
            bail!("udp protocol not supported")
        }

        //  Check whether socket type and transport protocol match.
        //  Specifically, multicast protocols can't be combined with
        //  bi-directional messaging patterns (socket types).
        // #if defined ZMQ_HAVE_OPENPGM || defined ZMQ_HAVE_NORM
        // #if defined ZMQ_HAVE_OPENPGM && defined ZMQ_HAVE_NORM
        if (protocol_ == ZmqTransport::ZmqPgm || protocol_ == ZmqTransport::ZmqEpgm || protocol_ == ZmqTransport::ZmqNorm) {
            // #elif defined ZMQ_HAVE_OPENPGM
            //     if ((protocol_ == protocol_name::pgm || protocol_ == protocol_name::epgm)
            // #else // defined ZMQ_HAVE_NORM
            //     if (protocol_ == protocol_name::norm
            // #endif && options.type_ != ZMQ_PUB && options.type_ != ZMQ_SUB && options.type_ != ZMQ_XPUB && options.type_ != ZMQ_XSUB {
            errno = ENOCOMPATPROTO;
            bail!("no compatible protocol found for pub/xpub/sub/xsub");
        }
        // #endif

        if protocol_ == protocol_name::udp && (options.type_ != ZMQ_DISH && options.type_ != ZMQ_RADIO && options.type_ != ZMQ_DGRAM) {
            // errno = ENOCOMPATPROTO;
            // return -1;
            bail!("no compatible protocol found for dish/radio/dgram")
        }

        //  Protocol is available.
        // return 0;
        Ok(())
    }

    //  Register the pipe with this socket.
    // void attach_pipe (ZmqPipe *pipe_,
    //                   bool subscribe_to_all_ = false,
    //                   bool locally_initiated_ = false);
    pub fn attach_pipe(
        &mut self,
        pipe: &mut ZmqPipe,
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
    // int process_commands (timeout: i32, bool throttle_);
    pub fn process_commands(&mut self, timeout: i32, throttle_: bool) -> anyhow::Result<()> {
        if timeout == 0 {
            //  If we are asked not to wait, check whether we haven't processed
            //  commands recently, so that we can throttle the new commands.

            //  Get the CPU's tick counter. If 0, the counter is not available.
            // let tsc = clock_t::rdtsc();
            let tsc = get_cpu_tick_counter()?;

            //  Optimised version of command processing - it doesn't have to check
            //  for incoming commands each time. It does so only if certain time
            //  elapsed since last command processing. Command delay varies
            //  depending on CPU speed: It's ~1ms on 3GHz CPU, ~2ms on 1.5GHz CPU
            //  etc. The optimisation makes sense only on platforms where getting
            //  a timestamp is a very cheap operation (tens of nanoseconds).
            if tsc != 0 && throttle_ {
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
        self.mailbox.recv(&cmd, timeout)?;
        // if (rc != 0 && errno == EINTR) {
        //     return -1;
        // }

        //  Process all available commands.
        // while (rc == 0 || errno == EINTR) {
        //     if (rc == 0) {
        //         cmd.destination.process_command(cmd);
        //     }
        //     rc = mailbox->recv (&cmd, 0);
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
    // void process_stop () ;
    pub fn process_stop(&mut self, options: &mut ZmqContext) {
        //  Here, someone have called zmq_ctx_term while the socket was still alive.
        //  We'll remember the fact so that any blocking call is interrupted and any
        //  further attempt to use the socket will return ETERM. The user is still
        //  responsible for calling zmq_close on the socket though!
        // scoped_lock_t lock (_monitor_sync);
        self.stop_monitor(options, false);

        self.ctx_terminated = true;
    }

    // void process_bind (ZmqPipe *pipe_) ;
    pub fn process_bind(&mut self, pipe: &mut ZmqPipe) {
        self.attach_pipe(pipe, false, false);
    }

    // void process_pipe_stats_publish (outbound_queue_count_: u64,
    //                             inbound_queue_count_: u64,
    //                             EndpointUriPair *endpoint_pair_) ;
    pub fn process_pipe_stats_publish(
        &mut self,
        options: &mut ZmqContext,
        outbound_queue_count: u64,
        inbound_queue_count: u64,
        endpoint_pair: &mut EndpointUriPair,
    ) {
        let mut values: [u64; 2] = [outbound_queue_count, inbound_queue_count];
        self.event(
            options,
            endpoint_pair,
            &values,
            2,
            ZMQ_EVENT_PIPES_STATS as u64,
        );
        // delete endpoint_pair_;
    }

    // void process_term (linger_: i32) ;
    pub fn process_term(&mut self, linger: i32) {
        //  Unregister all inproc endpoints associated with this socket.
        //  Doing this we make sure that no new pipes from other sockets (inproc)
        //  will be initiated.
        self.unregister_endpoints(self);

        //  Ask all attached pipes to terminate.
        // for (pipes_t::size_type i = 0, size = pipes.size (); i != size; += 1i)
        // {
        //     //  Only inprocs might have a disconnect message set
        //     pipes[i]->send_disconnect_msg ();
        //     pipes[i]->terminate (false);
        // }
        for pipe in self.pipes.iter_mut() {
            pipe.send_disconnect_msg();
            pipe.terminate(false);
        }

        self.register_term_acks(self.pipes.len());

        //  Continue the termination process immediately.
        // ZmqOwn::process_term (linger_);
    }

    // void process_term_endpoint (std::string *endpoint_) ;
    pub fn process_term_endpoint(&mut self, options: &mut ZmqContext, endpoint: &str) {
        self.term_endpoint(options, endpoint);
        // delete endpoint_;
    }

    // void update_pipe_options (option_: i32);
    pub fn update_pipe_options(&mut self, options: &mut ZmqContext, option_: i32) {
        if option_ == ZMQ_SNDHWM || option_ == ZMQ_RCVHWM {
            // for (pipes_t::size_type i = 0, size = pipes.size (); i != size; += 1i)
            for pipe in self.pipes.iter_mut() {
                pipe.set_hwms(options.rcvhwm as u32, options.sndhwm as u32);
                pipe.send_hwms_to_peer(options.sndhwm, options.rcvhwm);
            }
        }
    }

    // std::string resolve_tcp_addr (std::string endpoint_uri_,
    //                               tcp_address_: *const c_char);
    pub fn resolve_tcp_addr(
        &mut self,
        options: &mut ZmqContext,
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

    pub fn register_endpoint(&mut self, ctx: &mut ZmqContext, addr: &str, endpoint: &mut ZmqEndpoint) -> anyhow::Result<()> {}
} // impl ZmqSocketBase

//  Send a monitor event

pub fn get_sock_opt_zmq_fd(sock: &mut ZmqSocket) -> anyhow::Result<ZmqFileDesc> {
    let mut result_raw = sock.getsockopt(ZmqSocketOption::ZMQ_FD)?;
    let mut result_usize = usize::from_le_bytes([
        result_raw[0], result_raw[1], result_raw[2], result_raw[3], result_raw[4], result_raw[5],
        result_raw[6], result_raw[7],
    ]);
    let result = ZmqFileDesc::from_raw_fd(result_usize);
    Ok(result)
}

pub fn get_sock_opt_zmq_events(sock: &mut ZmqSocket) -> anyhow::Result<u32> {
    let mut result_raw = sock.getsockopt(ZmqSocketOption::ZMQ_EVENTS)?;
    let mut result_u32 = u32::from_le_bytes([
        result_raw[0], result_raw[1], result_raw[2], result_raw[3]
    ]);
    // let result = ZmqFileDesc::from_raw_fd(result_usize);
    Ok(result_u32)
}

pub fn get_sock_opt_zmq_routing_id(sock: &mut ZmqSocket) -> anyhow::Result<String> {
    let mut result_raw = sock.getsockopt(ZmqSocketOption::ZMQ_ROUTING_ID)?;
    let result = String::from_utf8(result_raw)?;
    Ok(result)
}