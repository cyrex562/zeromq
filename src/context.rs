use std::collections::{HashMap, HashSet};
use std::mem::size_of;
use std::ptr::null_mut;
use std::sync::atomic::AtomicU64;
use std::sync::Mutex;
use std::{mem, process};

use anyhow::{anyhow, bail};
use libc::{
    getpid, gid_t, pid_t, uid_t, EADDRINUSE, ECONNREFUSED, EINTR, EINVAL, EMFILE, ENOENT, ENOMEM,
    FD_CLOEXEC, F_SETFD,
};
use serde::{Deserialize, Serialize};

use crate::atomic_counter::AtomicCounter;
use crate::command::ZmqCommand;
use crate::defines::{
    ZmqMessage, ZMQ_BLOCKY, ZMQ_IO_THREADS, ZMQ_IO_THREADS_DFLT, ZMQ_IPV6, ZMQ_MAX_MSGSZ,
    ZMQ_MAX_SOCKETS, ZMQ_MAX_SOCKETS_DFLT, ZMQ_MESSAGE_SIZE, ZMQ_PAIR, ZMQ_SOCKET_LIMIT,
    ZMQ_ZERO_COPY_RECV,
};
use crate::endpoint::ZmqEndpoint;
use crate::mailbox::ZmqMailbox;
use crate::mailbox_interface::ZmqMailboxInterface;
use crate::message::ZmqMessage;
use crate::object::ZmqObject;
use crate::options::CURVE_KEYSIZE;
use crate::pending_connection::PendingConnection;
use crate::pipe::ZmqPipe;
use crate::reaper::ZmqReaper;
use crate::socket_base::ZmqSocketBase;
use crate::tcp_address::TcpAddressMask;
use crate::thread_context::ZmqThreadContext;
use crate::thread_ctx::ThreadCtx;
use crate::ZmqMailboxInterface::ZmqMailboxInterface;

//  Context object encapsulates all the global state associated with
//  the library.

// enum tid_type {
//     term_tid = 0,
//     reaper_tid = 1,
// }

// enum side {
//     connect_side,
//     bind_side,
// }

pub const CONNECT_SIDE: i32 = 0;
pub const BIND_SIDE: i32 = 1;
pub const TERM_TID: i32 = 0;
pub const REAPER_TID: u32 = 1;

// #define ZMQ_CTX_TAG_VALUE_GOOD 0xabadcafe
pub const ZMQ_CTX_TAG_VALUE_GOOD: u32 = 0xabadcafe;
// #define ZMQ_CTX_TAG_VALUE_BAD 0xdeadbeef
pub const ZMQ_CTX_TAG_VALUE_BAD: u32 = 0xdeadbeef;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ZmqContext {
    pub thread_ctx: ThreadCtx,
    //  Used to check whether the object is a context.
    pub tag: u32,
    //  Sockets belonging to this context. We need the list so that
    //  we can notify the sockets when zmq_ctx_term() is called.
    //  The sockets will return ETERM then.
    pub sockets: Vec<ZmqSocketBase>,
    //  List of unused thread slots.
    pub empty_slots: Vec<u32>,
    //  If true, zmq_init has been called but no socket has been created
    //  yet. Launching of I/O threads is delayed.
    pub starting: bool,
    //  If true, zmq_ctx_term was already called.
    pub terminating: bool,
    //  Synchronisation of accesses to global slot-related data:
    //  sockets, empty_slots, terminating. It also synchronises
    //  access to zombie sockets as such (as opposed to slots) and provides
    //  a memory barrier to ensure that all CPU cores see the same data.
    pub slot_sync: Mutex<u8>,
    //  The reaper thread.
    pub reaper: Option<ZmqReaper>,
    //  I/O threads.
    pub io_threads: Vec<ZmqThreadContext>,
    //  Array of pointers to mailboxes for both application and I/O threads.
    pub slots: Vec<ZmqMailbox>,
    //  Mailbox for zmq_ctx_term thread.
    pub term_mailbox: ZmqMailbox,
    //  List of inproc endpoints within this context.
    pub endpoints: HashMap<String, ZmqEndpoint>,
    // List of inproc onnection endpoints pending a bind
    pub pending_connections: HashMap<String, PendingConnection>,
    //  Synchronisation of access to the list of inproc endpoints.
    pub endpoints_sync: Mutex<u8>,
    //  Maximum socket ID.
    pub max_socket_id: AtomicCounter,
    //  Maximum number of sockets that can be opened at the same time.
    pub max_sockets: i32,
    //  Maximum allowed message size
    pub max_msg_sz: i32,
    //  Number of I/O threads to launch.
    pub io_thread_count: i32,
    //  Does context wait (possibly forever) on termination?
    pub blocky: bool,
    //  Is IPv6 enabled on this context?
    pub ipv6: bool,
    // the process that created this context. Used to detect forking.
    #[cfg(target_os = "linux")]
    pub pid: pid_t,
    #[cfg(target_os = "windows")]
    pub pid: u32,
    #[cfg(target_feature = "vmci")]
    pub vmci_fd: i32,
    // _vmci_family: i32;
    #[cfg(target_feature = "vmci")]
    pub vmci_family: i32,
    // mutex_t _vmci_sync;
    #[cfg(target_feature = "vmci")]
    pub vmci_sync: Mutex<u8>,
    //  High-water marks for message pipes.
    pub sndhwm: i32,
    pub rcvhwm: i32,
    //  I/O thread affinity.
    pub affinity: u64,
    //  Socket routing id.
    pub routing_id: String,
    //  Maximum transfer rate [kb/s]. Default 100kb/s.
    pub rate: i32,
    //  Reliability time interval [ms]. Default 10 seconds.
    pub recovery_ivl: i32,
    // Sets the time-to-live field in every multicast packet sent.
    pub multicast_hops: i32,
    // Sets the maximum transport data unit size in every multicast
    // packet sent.
    pub multicast_maxtpdu: i32,
    // SO_SNDBUF and SO_RCVBUF to be passed to underlying transport sockets.
    pub sndbuf: i32,
    pub rcvbuf: i32,
    // Type of service (containing DSCP and ECN socket options)
    pub tos: i32,
    // Protocol-defined priority
    pub priority: i32,
    //  Socket type.
    pub type_: i32,
    //  Linger time, in milliseconds.
    pub linger: AtomicU64,
    //  Maximum interval in milliseconds beyond which userspace will
    //  timeout connect().
    //  Default 0 (unused)
    pub connect_timeout: i32,
    //  Maximum interval in milliseconds beyond which TCP will timeout
    //  retransmitted packets.
    //  Default 0 (unused)
    pub tcp_maxrt: i32,
    //  Disable reconnect under certain conditions
    //  Default 0
    pub reconnect_stop: i32,
    //  Minimum interval between attempts to reconnect, in milliseconds.
    //  Default 100ms
    pub reconnect_ivl: i32,

    //  Maximum interval between attempts to reconnect, in milliseconds.
    //  Default 0 (unused)
    pub reconnect_ivl_max: i32,
    //  Maximum backlog for pending connections.
    pub backlog: i32,
    //  Maximal size of message to handle.
    pub maxmsgsize: i64,
    // The timeout for send/recv operations for this socket, in milliseconds.
    pub rcvtimeo: i32,
    pub sndtimeo: i32,
    //  If 1, connecting pipes are not attached immediately, meaning a send()
    //  on a socket with only connecting pipes would block
    pub immediate: i32,
    //  If 1, (X)SUB socket should filter the messages. If 0, it should not.
    pub filter: bool,
    //  If true, the subscription matching on (X)PUB and (X)SUB sockets
    //  is reversed. Messages are sent to and received by non-matching
    //  sockets.
    pub invert_matching: bool,
    //  If true, the routing id message is forwarded to the socket.
    pub recv_routing_id: bool,
    // if true, router socket accepts non-zmq tcp connections
    pub raw_socket: bool,
    pub raw_notify: bool, //  Provide connect notifications
    //  Address of SOCKS proxy
    pub socks_proxy_address: String,
    // Credentials for SOCKS proxy.
    // Connection method will be basic auth if username
    // is not empty, no auth otherwise.
    pub socks_proxy_username: String,
    pub socks_proxy_password: String,
    //  TCP keep-alive settings.
    //  Defaults to -1 = do not change socket options
    tcp_keepalive: i32,
    tcp_keepalive_cnt: i32,
    tcp_keepalive_idle: i32,
    tcp_keepalive_intvl: i32,
    // TCP accept() filters
    pub tcp_accept_filters: Vec<TcpAddressMask>,
    // IPC accept() filters
    #[cfg(target_os = "linux")]
    pub ipc_uid_accept_filters: HashSet<uid_t>,
    #[cfg(target_os = "linux")]
    pub ipc_gid_accept_filters: HashSet<gid_t>,
    #[cfg(target_os = "linux")]
    pub ipc_pid_accept_filters: HashSet<pid_t>,
    //  Security mechanism for all connections on this socket
    pub mechanism: i32,
    //  If peer is acting as server for PLAIN or CURVE mechanisms
    pub as_server: i32,
    //  ZAP authentication domain
    pub zap_domain: String,
    //  Security credentials for PLAIN mechanism
    pub plain_username: String,
    pub plain_password: String,
    //  Security credentials for CURVE mechanism
    pub curve_public_key: Vec<u8>,
    pub curve_secret_key: Vec<u8>,
    pub curve_server_key: Vec<u8>,
    //  Principals for GSSAPI mechanism
    pub gss_principal: String,
    pub gss_service_principal: String,
    //  Name types GSSAPI principals
    pub gss_principal_nt: i32,
    pub gss_service_principal_nt: i32,
    //  If true, gss encryption will be disabled
    pub gss_plaintext: bool,
    //  ID of the socket.
    pub socket_id: i32,
    //  If true, socket conflates outgoing/incoming messages.
    //  Applicable to dealer, push/pull, pub/sub socket types.
    //  Cannot receive multi-part messages.
    //  Ignores hwm
    pub conflate: bool,
    //  If connection handshake is not done after this many milliseconds,
    //  close socket.  Default is 30 secs.  0 means no handshake timeout.
    pub handshake_ivl: i32,
    pub connected: bool,
    //  If remote peer receives a PING message and doesn't receive another
    //  message within the ttl value, it should close the connection
    //  (measured in tenths of a second)
    pub heartbeat_ttl: u16,
    //  Time in milliseconds between sending heartbeat PING messages.
    pub heartbeat_interval: i32,
    //  Time in milliseconds to wait for a PING response before disconnecting
    pub heartbeat_timeout: i32,
    //  If true, the socket will send all outstanding messages to a peer
    #[cfg(target_feature = "vmci")]
    pub vmci_buffer_size: u64,
    #[cfg(target_feature = "vmci")]
    pub vmci_buffer_min_size: u64,
    #[cfg(target_feature = "vmci")]
    pub vmci_buffer_max_size: u64,
    #[cfg(target_feature = "vmci")]
    pub vmci_connect_timeout: i32,
    //  When creating a new ZMQ socket, if this option is set the value
    //  will be used as the File Descriptor instead of allocating a new
    //  one via the socket () system call.
    pub use_fd: i32,
    // Device to bind the underlying socket to, eg. VRF or interface
    pub bound_device: String,
    //  Enforce a non-empty ZAP domain requirement for PLAIN auth
    pub zap_enforce_domain: bool,
    // Use of loopback fastpath.
    pub loopback_fastpath: bool,
    //  Loop sent multicast packets to local sockets
    pub multicast_loop: bool,
    //  Maximal batching size for engines with receiving functionality.
    //  So, if there are 10 messages that fit into the batch size, all of
    //  them may be read by a single 'recv' system call, thus avoiding
    //  unnecessary network stack traversals.
    pub in_batch_size: i32,
    //  Maximal batching size for engines with sending functionality.
    //  So, if there are 10 messages that fit into the batch size, all of
    //  them may be written by a single 'send' system call, thus avoiding
    //  unnecessary network stack traversals.
    pub out_batch_size: i32,
    // Use zero copy strategy for storing message content when decoding.
    pub zero_copy: bool,
    // Router socket ZMQ_NOTIFY_CONNECT/ZMQ_NOTIFY_DISCONNECT notifications
    pub router_notify: i32,
    // Application metadata
    pub app_metadata: HashMap<String, String>,
    // Version of monitor events to emit
    pub monitor_event_version: i32,
    //  WSS Keys
    pub wss_key_pem: String,
    pub wss_cert_pem: String,
    pub wss_trust_pem: String,
    pub wss_hostname: String,
    pub wss_trust_system: bool,
    //  Hello msg
    pub hello_msg: Vec<u8>,
    pub can_send_hello_msg: bool,
    //  Disconnect msg
    pub disconnect_msg: Vec<u8>,
    pub can_recv_disconnect_msg: bool,
    //  Hiccup msg
    pub hiccup_msg: Vec<u8>,
    pub can_recv_hiccup_msg: bool,
    //  This option removes several delays caused by scheduling, interrupts and context switching.
    pub busy_poll: i32,
}

impl ZmqContext {
    //  Create the context object.
    // ctx_t ();

    // ZmqContext::ZmqContext () :
    //     _tag (ZMQ_CTX_TAG_VALUE_GOOD),
    //     _starting (true),
    //     terminating (false),
    //     _reaper (NULL),
    //     _max_sockets (clipped_maxsocket (ZMQ_MAX_SOCKETS_DFLT)),
    //     _max_msgsz (INT_MAX),
    //     _io_thread_count (ZMQ_IO_THREADS_DFLT),
    //     _blocky (true),
    //     _ipv6 (false),
    //     _zero_copy (true)
    // {
    // // #ifdef HAVE_FORK
    //     _pid = getpid ();
    // // #endif
    // // #ifdef ZMQ_HAVE_VMCI
    //     _vmci_fd = -1;
    //     _vmci_family = -1;
    // // #endif
    //
    //     //  Initialise crypto library, if needed.
    //     random_open ();
    //
    // // #ifdef ZMQ_USE_NSS
    //     NSS_NoDB_Init (NULL);
    // // #endif
    //
    // // #ifdef ZMQ_USE_GNUTLS
    //     gnutls_global_init ();
    // // #endif
    // }
    pub fn new() -> Self {
        Self {
            thread_ctx: Default::default(),
            tag: ZMQ_CTX_TAG_VALUE_GOOD,
            sockets: vec![],
            empty_slots: vec![],
            starting: true,
            terminating: false,
            slot_sync: Mutex::new(0),
            reaper: None,
            io_threads: vec![],
            slots: vec![],
            term_mailbox: ZmqMailbox,
            endpoints: Default::default(),
            pending_connections: Default::default(),
            endpoints_sync: Mutex::new(0),
            max_socket_id: AtomicCounter::new(),
            max_sockets: ZMQ_MAX_SOCKETS_DFLT,
            max_msg_sz: i32::MAX,
            io_thread_count: ZMQ_IO_THREADS_DFLT,
            blocky: true,
            ipv6: false,
            zero_copy: false,
            router_notify: 0,
            app_metadata: Default::default(),
            monitor_event_version: 0,
            wss_key_pem: "".to_string(),
            wss_cert_pem: "".to_string(),
            wss_trust_pem: "".to_string(),
            wss_hostname: "".to_string(),
            wss_trust_system: false,
            hello_msg: vec![],
            can_send_hello_msg: false,
            disconnect_msg: vec![],
            can_recv_disconnect_msg: false,
            hiccup_msg: vec![],
            can_recv_hiccup_msg: false,
            pid: process: id(),
            sndhwm: 0,
            rcvhwm: 0,
            affinity: 0,
            routing_id: String::new(),
            rate: 0,
            recovery_ivl: 0,
            multicast_hops: 0,
            multicast_maxtpdu: 0,
            sndbuf: 0,
            rcvbuf: 0,
            tos: 0,
            priority: 0,
            type_: 0,
            linger: Default::default(),
            connect_timeout: 0,
            tcp_maxrt: 0,
            reconnect_stop: 0,
            reconnect_ivl: 0,
            reconnect_ivl_max: 0,
            backlog: 0,
            maxmsgsize: 0,
            rcvtimeo: 0,
            sndtimeo: 0,
            immediate: 0,
            filter: false,
            invert_matching: false,
            recv_routing_id: false,
            raw_socket: false,
            raw_notify: false,
            socks_proxy_address: "".to_string(),
            socks_proxy_username: "".to_string(),
            socks_proxy_password: "".to_string(),
            tcp_keepalive: 0,
            tcp_keepalive_cnt: 0,
            tcp_keepalive_idle: 0,
            tcp_keepalive_intvl: 0,
            tcp_accept_filters: vec![],
            ipc_uid_accept_filters: Default::default(),
            ipc_gid_accept_filters: Default::default(),
            ipc_pid_accept_filters: Default::default(),
            mechanism: 0,
            as_server: 0,
            zap_domain: "".to_string(),
            plain_username: "".to_string(),
            plain_password: "".to_string(),
            curve_public_key: vec![],
            curve_secret_key: vec![],
            curve_server_key: vec![],
            gss_principal: "".to_string(),
            gss_service_principal: "".to_string(),
            gss_principal_nt: 0,
            gss_service_principal_nt: 0,
            gss_plaintext: false,
            socket_id: 0,
            conflate: false,
            handshake_ivl: 0,
            connected: false,
            heartbeat_ttl: 0,
            heartbeat_interval: 0,
            heartbeat_timeout: 0,
            use_fd: 0,
            bound_device: "".to_string(),
            zap_enforce_domain: false,
            loopback_fastpath: false,
            multicast_loop: false,
            in_batch_size: 0,
            #[cfg(target_feature = "vmci")]
            vmci_fd: -1,
            #[cfg(target_feature = "vmci")]
            vmci_family: -1,
            #[cfg(target_feature = "vmci")]
            vmci_sync: Mutex::new(0),
            out_batch_size: 0,
            busy_poll: 0,
        }
    }

    // bool ZmqContext::check_tag () const
    pub fn check_tag(&self) -> bool {
        self.tag == ZMQ_CTX_TAG_VALUE_GOOD
    }

    // bool ZmqContext::valid () const
    pub fn valid(&mut self) -> bool {
        self.term_mailbox.valid()
    }

    // int ZmqContext::terminate ()
    pub fn terminate(&mut self) -> anyhow::Result<i32> {
        let _guard = self.slot_sync.lock()?;

        let save_terminating = self.terminating;
        self.terminating = false;

        // Connect up any pending inproc connections, otherwise we will hang
        let copy = self.pending_connections.clone();
        // for (pending_connections_t::iterator p = copy.begin (), end = copy.end ();
        //      p != end; += 1p)
        for (key, val) in copy.iter() {
            let mut s = self.create_socket(ZMQ_PAIR);
            // create_socket might fail eg: out of memory/sockets limit reached
            // zmq_assert(s);
            s.bind(val);
            s.close();
        }
        self.terminating = save_terminating;

        if !self.starting {
            // #ifdef HAVE_FORK
            if self.pid != process::id() {
                // we are a forked child process. Close all file descriptors
                // inherited from the parent.
                // for (sockets_t::size_type i = 0, size = _sockets.size (); i != size;
                //      i+= 1)
                for i in 0..self.sockets.len() {
                    self.sockets[i].get_mailbox().forked();
                }
                self.term_mailbox.forked();
            }
            // #endif

            //  Check whether termination was already underway, but interrupted and now
            //  restarted.
            let restarted = self.terminating;
            self.terminating = true;

            //  First attempt to terminate the context.
            if !restarted {
                //  First send stop command to sockets so that any blocking calls
                //  can be interrupted. If there are no sockets we can ask reaper
                //  thread to stop.
                // for (sockets_t::size_type i = 0, size = _sockets.size (); i != size;
                //      i+= 1)
                for i in 0..self.sockets.len() {
                    self.sockets[i].stop();
                }
                if self.sockets.empty() {
                    self.reaper.stop();
                }
            }
            self.slot_sync.unlock();

            //  Wait till reaper thread closes all the sockets.
            let mut cmd = ZmqCommand::default();
            self.term_mailbox.recv(&mut cmd, -1)?;

            // errno_assert(rc == 0);
            // zmq_assert(cmd.cmd_type == ZmqCommand::done);
            let _lock = self.slot_sync.lock()?;
            // zmq_assert(self.sockets.empty());
        }
        self.slot_sync.unlock();

        // #ifdef ZMQ_HAVE_VMCI
        let _ = self.vmci_sync.lock().expect("TODO: panic message");

        VMCISock_ReleaseAFValueFd(self.vmci_fd);
        self.vmci_family = -1;
        self.vmci_fd = -1;

        self.vmci_sync.unlock();
        // #endif

        //  Deallocate the resources.
        // delete this;

        Ok(0)
    }

    // int ZmqContext::shutdown ()
    pub fn shutdown(&mut self) -> anyhow::Result<()> {
        // scoped_lock_t locker (_slot_sync);
        let locker = &self.slot_sync;

        if !self.terminating {
            self.terminating = true;

            if !self.starting {
                //  Send stop command to sockets so that any blocking calls
                //  can be interrupted. If there are no sockets we can ask reaper
                //  thread to stop.
                // for (sockets_t::size_type i = 0, size = _sockets.size (); i != size;
                //      i+= 1)
                for i in 0..self.sockets.len() {
                    self.sockets[i].stop();
                }
                if self.sockets.empty() {
                    self.reaper.stop();
                }
            }
        }

        Ok(())
    }

    pub fn set(
        &mut self,
        option: i32,
        opt_val: &mut [u8],
        opt_val_len: usize,
    ) -> anyhow::Result<()> {
        let is_int = (opt_val_len == mem::size_of::<i32>());
        let mut value = 0i32;
        if is_int {
            let bytes: [u8; 4] = [0; 4];
            bytes.clone_from_slice(&opt_val);
            value = i32::from_le_bytes(bytes);
            // memcpy(&value, optval_, sizeof);
        }

        match option {
            ZMQ_MAX_SOCKETS => {
                if is_int && value >= 1 && value == clipped_maxsocket(value) {
                    // let locker = scoped_lock_t::new(self._opt_sync);
                    self.max_sockets = value;
                    return Ok(());
                }
            }

            ZMQ_IO_THREADS => {
                if is_int && value >= 0 {
                    // let locker = scoped_lock_t::new(self._opt_sync);
                    self.io_thread_count = value;
                    return Ok(());
                }
            }

            ZMQ_IPV6 => {
                if is_int && value >= 0 {
                    // let locker = scoped_lock_t::new(self._opt_sync);
                    self.ipv6 = (value != 0);
                    return Ok(());
                }
            }

            ZMQ_BLOCKY => {
                if is_int && value >= 0 {
                    // scoped_lock_t locker (_opt_sync);
                    self.blocky = (value != 0);
                    return Ok(());
                }
            }

            ZMQ_MAX_MSGSZ => {
                if is_int && value >= 0 {
                    // scoped_lock_t locker (_opt_sync);
                    self.max_msg_sz = if value < i32::MAX { value } else { i32::MAX };
                    return Ok(());
                }
            }

            ZMQ_ZERO_COPY_RECV => {
                if is_int && value >= 0 {
                    // scoped_lock_t locker (_opt_sync);
                    self.zero_copy = (value != 0);
                    return Ok(());
                }
            }

            _ => {
                // TODO: implement
                // return self.get_(option, opt_val, opt_val_len);
            }
        }

        // errno = EINVAL;
        // return -1;
        bail!("invalid")
    }

    // int ZmqContext::get (option_: i32, optval_: *mut c_void, const optvallen_: *mut usize)
    pub fn option_bytes(&mut self, option: i32) -> anyhow::Result<Vec<u8>> {
        // const bool is_int = (*optvallen_ == sizeof );
        // let is_int = *opt_val_len == size_of::<i32>();
        // int *value = static_cast<int *> (optval_);
        // let mut value = 0i32;
        //     let mut bytes: [u8;4] = [0;4];
        //     bytes.clone_from_slice(optval_);
        //     value = i32::from_le_bytes(bytes);
        let mut out: Vec<u8> = Vec::new();

        match option {
            ZMQ_MAX_SOCKETS => {
                // scoped_lock_t
                // locker(_opt_sync);
                // value = self._max_sockets;
                out.clone_from_slice(self.max_sockets.to_le_bytes().as_slice());
                return Ok(out);
            }

            ZMQ_SOCKET_LIMIT => {
                // *value = clipped_maxsocket(65535);
                let x = clipped_maxsocket(65535);
                out.clone_from_slice(x.to_le_bytes().as_slice());
                return Ok(out);
            }

            ZMQ_IO_THREADS => {
                // scoped_lock_t locker (_opt_sync);
                // *value = _io_thread_count;
                out.clone_from_slice(self.io_thread_count.to_le_bytes().as_slice());
                return Ok(out);
            }

            ZMQ_IPV6 => {
                // scoped_lock_t locker (_opt_sync);
                // *value = _ipv6;
                out.clone_from_slice(self.ipv6.to_le_bytes().as_slice());
                return Ok(out);
            }

            ZMQ_BLOCKY => {
                // scoped_lock_t locker (_opt_sync);
                // *value = _blocky;
                out.clone_from_slice(self.blocky.to_le_bytes().as_slice());
                return Ok(out);
            }

            ZMQ_MAX_MSGSZ => {
                // scoped_lock_t locker (_opt_sync);
                // *value = _max_msgsz;
                out.clone_from_slice(self.max_msg_sz.to_le_bytes().as_slice());
                return Ok(out);
            }

            ZMQ_MESSAGE_SIZE => {
                // scoped_lock_t locker (_opt_sync);
                // *value = sizeof (zmq_ZmqMessage);
                let x = size_of::<ZmqMessage>();
                out.clone_from_slice(x.to_le_bytes().as_slice());
                return Ok(out);
            }

            ZMQ_ZERO_COPY_RECV => {
                // scoped_lock_t locker (_opt_sync);
                // *value = self._zero_copy;
                out[0] = self.zero_copy.into();
                return Ok(out);
            }

            _ => {
                return self.thread_ctx.get(option);
            }
        }

        // errno = EINVAL;
        // return -1;
        bail!("invalid")
    }

    pub fn option_i32(&mut self, opt_kind: i32) -> anyhow::Result<i32> {
        let data = self.option_bytes(opt_kind)?;
        let mut opt_bytes: [u8; 4] = [0; 4];
        opt_bytes.clone_from_slice(&data);
        let data_int = i32::from_le_bytes(opt_bytes);
        Ok(data_int)
    }

    pub fn start(&mut self) -> bool {
        //  Initialise the array of mailboxes. Additional two slots are for
        //  zmq_ctx_term thread and reaper thread.
        self._opt_sync.lock();
        let term_and_reaper_threads_count = 2usize;
        let mazmq = self.max_sockets;
        let ios = self.io_thread_count;
        self._opt_sync.unlock();
        let slot_count: usize = (mazmq + ios + term_and_reaper_threads_count) as usize;
        // try {
        //     _slots.reserve (slot_count);
        //     _empty_slots.reserve (slot_count - term_and_reaper_threads_count);
        // }
        // catch (const std::bad_alloc &) {
        //     errno = ENOMEM;
        //     return false;
        // }
        self.slots.reserve(slot_count);
        self.empty_slots
            .reserve(slot_count - term_and_reaper_threads_count);
        self.slots
            .resize(term_and_reaper_threads_count, ZmqMailbox::default());

        //  Initialise the infrastructure for zmq_ctx_term thread.
        self.slots[TERM_TID] = &self.term_mailbox;

        //  Create the reaper thread.
        self.reaper = Some(ZmqReaper::new(self, REAPER_TID));
        if self.reaper.is_none() {
            errno = ENOMEM;
            // goto fail_cleanup_slots;
        }
        if !self.reaper.get_mailbox().valid() {
            //     goto
            //     fail_cleanup_reaper;
        }
        self.slots[REAPER_TID] = self.reaper.get_mailbox();
        self.reaper.start();

        //  Create I/O thread objects and launch them.
        self.slots.resize(slot_count, ZmqMailbox::default());

        // for (int i = term_and_reaper_threads_count;
        //      i != ios + term_and_reaper_threads_count; i+= 1)
        for i in term_and_reaper_threads_count..ios + term_and_reaper_threads_count {
            let mut io_thread = ZmqThreadContext::new(i as u32);
            if !io_thread {
                errno = ENOMEM;
                // goto fail_cleanup_reaper;
            }
            if !io_thread.get_mailbox().valid() {
                // delete io_thread;
                // goto fail_cleanup_reaper;
            }

            if io_thread.mailbox.is_some() {
                self.slots[i] = io_thread.mailbox.unwrap().clone();
            }
            io_thread.start();
            self.io_threads.push_back(io_thread.to_owned());
        }

        //  In the unused part of the slot array, create a list of empty slots.
        // for (int32_t i = static_cast<int32_t> (_slots.size ()) - 1;
        //      i >= static_cast<int32_t> (ios) + term_and_reaper_threads_count; i -= 1)
        for i in self.slots.len() - 1..ios + term_and_reaper_team_threads_count {
            self.empty_slots.push_back(i);
        }

        self.starting = false;
        return true;

        // TODO:
        // fail_cleanup_reaper:
        //     _reaper->stop ();
        //     delete _reaper;
        //     _reaper = NULL;

        // TODO:
        // fail_cleanup_slots:
        //     _slots.clear ();
        //     return false;
        return false;
    }

    // ZmqSocketBase *ZmqContext::create_socket (type_: i32)
    pub fn create_socket(&mut self, type_: i32) -> anyhow::Result<ZmqSocketBase> {
        // scoped_lock_t locker (_slot_sync);

        //  Once zmq_ctx_term() or zmq_ctx_shutdown() was called, we can't create
        //  new sockets.
        if self.terminating {
            return Err(anyhow!("terminating"));
        }

        self.start()?;

        //  If max_sockets limit was reached, return error.
        if self.empty_slots.empty() {
            return Err(anyhow!("max_sockets limit was reached"));
        }

        //  Choose a slot for the socket.
        // const uint32_t slot = _empty_slots.back ();
        let slot = self.empty_slots.last_mut().unwrap();
        self.empty_slots.pop_back();

        //  Generate new unique socket ID.
        // const int sid = ( (max_socket_id.add (1))) + 1;
        let sid = max_socket_id.add(1) + 1;

        //  Create the socket and register its mailbox.
        let s = ZmqSocketBase::create(type_, self, slot, sid);
        // if ( ! s) {
        //     self.empty_slots.push_back(slot);
        //     return None;
        // }
        self.sockets.push_back(s);
        self.slots[slot] = s.get_mailbox();

        return Ok(s);
    }

    // void ZmqContext::destroy_socket (class ZmqSocketBase *socket_)
    pub fn destroy_socket(&mut self, socket: &mut ZmqSocketBase) {
        // scoped_lock_t locker (_slot_sync);

        //  Free the associated thread slot.
        let tid = socket.get_tid();
        self.empty_slots.push_back(tid);
        self.slots[tid] = null_mut();

        //  Remove the socket from the list of sockets.
        self.sockets.erase(socket);

        //  If zmq_ctx_term() was already called and there are no more socket
        //  we can ask reaper thread to terminate.
        if self.terminating && self.sockets.empty() {
            self.reaper.stop();
        }
    }

    // object_t *ZmqContext::get_reaper () const
    pub fn get_reaper(&mut self) -> Option<ZmqReaper> {
        return self.reaper.clone();
    }

    // void ZmqContext::send_command (uint32_t tid, const ZmqCommand &command_)
    pub fn send_command(&mut self, tid: u32, command_: &mut ZmqCommand) {
        self.slots[tid].send(command_);
    }

    // ZmqIoThread *ZmqContext::choose_io_thread (u64 affinity_)
    pub fn choose_io_thread(&mut self, affinity: u64) -> Option<ZmqThreadContext> {
        if self.io_threads.empty() {
            return None;
        }

        //  Find the I/O thread with minimum load.
        let mut min_load = -1;
        // ZmqIoThread *selected_io_thread = NULL;
        let mut selected_io_thread: Option<ZmqThreadContext> = None;
        // for (io_threads_t::size_type i = 0, size = _io_threads.size (); i != size;
        //      i+= 1)
        for i in 0..self.io_threads.len() {
            if !affinity || (affinity & (1 << i)) {
                let load = self.io_threads[i].get_load();
                if selected_io_thread.is_none() || load < min_load {
                    min_load = load;
                    selected_io_thread = Some(self.io_threads[i].clone());
                }
            }
        }
        return selected_io_thread;
    }

    pub fn register_endpoint(
        &mut self,
        addr: &str,
        endpoint: &mut ZmqEndpoint,
    ) -> anyhow::Result<()> {
        // scoped_lock_t locker (_endpoints_sync);
        match self.endpoints.insert(addr.to_string(), endpoint.clone()) {
            Some(_) => Ok(()),
            Err(e) => Err(anyhow!("failed to insert enpoint: {}", e)),
        }
    }

    // int ZmqContext::unregister_endpoint (const std::string &addr_,
    //                                      const ZmqSocketBase *const socket_)
    pub fn unregister_endpoint(
        &mut self,
        addr_: &str,
        sock_base: &mut ZmqSocketBase,
    ) -> anyhhow::Result<()> {
        // scoped_lock_t locker (_endpoints_sync);

        // const endpoints_t::iterator it = _endpoints.find (addr_);
        // if (it == _endpoints.end () || it->second.socket != socket_) {
        //     errno = ENOENT;
        //     return -1;
        // }

        let item = self.endpoints.get(addr_);
        if item.is_none() {
            return Err(anyhow!("ENOENT"));
            // errno = ENOENT;
            // return -1;
        }

        if item.unwrap().socket != sock_base {
            // errno = ENOENT;
            // return -1;
            return Err(anyhow!("ENOENT"));
        }

        match self.endpoints.remove(addr_) {
            Some(_) => {}
            None => {
                // errno = ENOENT;
                // -1
                return Err(anyhow!("ENOENT"));
            }
        }

        //  Remove endpoint.
        // _endpoints.erase (it);

        // return 0;
        Ok(())
    }

    // void ZmqContext::unregister_endpoints (const ZmqSocketBase *const socket_)
    pub fn unregister_endpoints(&mut self, socket: &mut ZmqSocketBase) {
        // scoped_lock_t locker (_endpoints_sync);

        // for (endpoints_t::iterator it = _endpoints.begin (),
        //                            end = _endpoints.end ();
        //      it != end;)

        let mut erase_list: Vec<String> = vec![];

        for (k, v) in self.endpoints.iter_mut() {
            //         if (it->second.socket == socket_)
            // #if __cplusplus >= 201103L || (defined _MSC_VER && _MSC_VER >= 1700)
            //             it = _endpoints.erase (it);
            // // #else
            //             _endpoints.erase (it+= 1);
            // // #endif
            //         else
            //             += 1it;
            if v.socket == socket {
                erase_list.push(k.clone())
            }
        }

        for element in erase_list.iter() {
            self.endpoints.remove(element);
        }
    }

    // ZmqEndpoint ZmqContext::find_endpoint (addr_: *const c_char)
    pub fn find_endpoint(&mut self, addr_: &str) -> Option<ZmqEndpoint> {
        // scoped_lock_t locker (_endpoints_sync);

        // endpoints_t::iterator it = _endpoints.find (addr_);
        let endpoint = self.endpoints.get_mut(addr_);

        if endpoint.is_none() {
            errno = ECONNREFUSED;
            // ZmqEndpoint empty = {NULL, ZmqOptions ()};
            return None;
        }
        // ZmqEndpoint endpoint = it->second;

        //  Increment the command sequence number of the peer so that it won't
        //  get deallocated until "bind" command is issued by the caller.
        //  The subsequent 'bind' has to be called with inc_seqnum parameter
        //  set to false, so that the seqnum isn't incremented twice.
        endpoint.unwrap().socket.inc_seqnum();
        let x = endpoint.unwrap().clone();
        return Some(x);
    }

    // void ZmqContext::pend_connection (const std::string &addr_,
    //                                   const ZmqEndpoint &endpoint_,
    //                                   ZmqPipe **pipes_)
    pub fn pend_connection(
        &mut self,
        in_addr: &str,
        in_endpoint: &ZmqEndpoint,
        in_pipes: &[ZmqPipe],
    ) {
        // scoped_lock_t locker (_endpoints_sync);

        // const PendingConnection pending_connection = {endpoint_, pipes_[0],
        //                                                  pipes_[1]};
        let mut pending_connection = PendingConnection {
            endpoint: in_endpoint.clone(),
            connect_pipe: in_pipes[0].clone(),
            bind_pipe: in_pipes[1].clone(),
        };

        // const endpoints_t::iterator it = _endpoints.find (addr_);
        let it = self.endpoints.get_mut(in_addr);
        // if (it == _endpoints.end ())
        if it.is_none() {
            //  Still no bind.
            in_endpoint.socket.inc_seqnum();
            self.pending_connections
                .ZMQ_MAP_INSERT_OR_EMPLACE(in_addr, pending_connection);
        } else {
            //  Bind has happened in the mean time, connect directly
            self.connect_inproc_sockets(
                &mut it.unwrap().socket,
                &mut it.unwrap().options.clone(),
                &mut pending_connection,
                CONNECT_SIDE,
            );
        }
    }

    // void ZmqContext::connect_pending (addr_: *const c_char,
    //                                   ZmqSocketBase *bind_socket_)
    pub fn connect_pending(&mut self, addr_: &str, bind_socket_: &mut ZmqSocketBase) {
        // scoped_lock_t locker (_endpoints_sync);

        // const std::pair<pending_connections_t::iterator,
        //                 pending_connections_t::iterator>
        //
        //   let pending = self._pending_connections.equal_range (addr_);
        // for (pending_connections_t::iterator p = pending.first; p != pending.second;
        //      += 1p)
        //     connect_inproc_sockets (bind_socket_, _endpoints[addr_].options,
        //                             p->second, bind_side);
        let pending = self.pending_connections.get(addr_);
        if pending.is_some() {
            self.connect_inproc_sockets(
                bind_socket_,
                &mut self.endpoints[addr_].options.clone(),
                &mut pending.unwrap(),
                BIND_SIDE,
            )
        }

        self.pending_connections.remove(addr_);
    }

    // void ZmqContext::connect_inproc_sockets (
    //   bind_socket_: *mut ZmqSocketBase,
    //   const ZmqOptions &bind_options_,
    //   const PendingConnection &pending_connection_,
    //   side side_)
    pub fn connect_inproc_sockets(
        &mut self,
        bind_socket: &mut ZmqSocketBase,
        bind_options: &mut ZmqOptions,
        pending_connection: &mut PendingConnection,
        side: side,
    ) {
        bind_socket.inc_seqnum();
        pending_connection.bind_pipe.set_tid(bind_socket.get_tid());

        if (!bind_options.recv_routing_id) {
            // ZmqMessage msg;
            let mut msg = ZmqMessage::default();
            let ok = pending_connection.bind_pipe.read(&msg);
            // zmq_assert(ok);
            let rc = msg.close();
            // errno_assert(rc == 0);
        }

        if !get_effective_conflate_option(&pending_connection.endpoint.options) {
            pending_connection
                .connect_pipe
                .set_hwms_boost(bind_options.sndhwm, bind_options.rcvhwm);
            pending_connection.bind_pipe.set_hwms_boost(
                pending_connection.endpoint.options.sndhwm,
                pending_connection.endpoint.options.rcvhwm,
            );

            pending_connection.connect_pipe.set_hwms(
                pending_connection.endpoint.options.rcvhwm,
                pending_connection.endpoint.options.sndhwm,
            );
            pending_connection
                .bind_pipe
                .set_hwms(bind_options.rcvhwm, bind_options.sndhwm);
        } else {
            pending_connection.connect_pipe.set_hwms(-1, -1);
            pending_connection.bind_pipe.set_hwms(-1, -1);
        }

        // #ifdef ZMQ_BUILD_DRAFT_API
        if (bind_options.can_recv_disconnect_msg && !bind_options.disconnect_msg.empty()) {
            pending_connection
                .connect_pipe
                .set_disconnect_msg(&mut bind_options.disconnect_msg);
        }
        // #endif

        if (side == BIND_SIDE) {
            let mut cmd = ZmqCommand::default();
            cmd.cmd_type = ZmqCommand::bind;
            cmd.args.bind.pipe = &mut pending_connection.bind_pipe.clone();
            bind_socket.process_command(cmd);
            bind_socket.send_inproc_connected(&mut pending_connection.endpoint.socket);
        } else {
            pending_connection.connect_pipe.send_bind(
                bind_socket,
                &mut pending_connection.bind_pipe,
                false,
            );
        }

        // When a ctx is terminated all pending inproc connection will be
        // connected, but the socket will already be closed and the pipe will be
        // in waiting_for_delimiter state, which means no more writes can be done
        // and the routing id write fails and causes an assert. Check if the socket
        // is open before sending.
        if (pending_connection.endpoint.options.recv_routing_id
            && pending_connection.endpoint.socket.check_tag())
        {
            send_routing_id(&mut pending_connection.bind_pipe, bind_options);
        }

        // #ifdef ZMQ_BUILD_DRAFT_API
        //  If set, send the hello msg of the bind socket to the pending connection.
        if (bind_options.can_send_hello_msg && bind_options.hello_msg.size() > 0) {
            send_hello_msg(&mut pending_connection.bind_pipe, bind_options);
        }
        // #endif
    }

    // #ifdef ZMQ_HAVE_VMCI
    // int ZmqContext::get_vmci_socket_family ()
    #[cfg(feature = "vmci")]
    pub fn get_vmci_socket_family(&mut self) -> i32 {
        // scoped_lock_t locker (_vmci_sync);

        if (self._vmci_fd == -1) {
            self._vmci_family = VMCISock_GetAFValueFd(&self._vmci_fd);

            unsafe {
                if (self._vmci_fd != -1) {
                    // #ifdef FD_CLOEXEC
                    let rc = libc::fcntl(self._vmci_fd, F_SETFD, FD_CLOEXEC);
                    // errno_assert(rc != -1);
                    // #endif
                }
            }
        }

        return self._vmci_family;
    }

    // #endif

    //  The last used socket ID, or 0 if no socket was used so far. Note that this
    //  is a global variable. Thus, even sockets created in different contexts have
    //  unique IDs.
    // TODO:
    // AtomicCounter ZmqContext::max_socket_id;

    //     ZmqContext::~ZmqContext ()
    // {
    //     //  Check that there are no remaining _sockets.
    //     zmq_assert (_sockets.empty ());
    //
    //     //  Ask I/O threads to terminate. If stop signal wasn't sent to I/O
    //     //  thread subsequent invocation of destructor would hang-up.
    //     const io_threads_t::size_type io_threads_size = _io_threads.size ();
    //     for (io_threads_t::size_type i = 0; i != io_threads_size; i+= 1) {
    //         _io_threads[i]->stop ();
    //     }
    //
    //     //  Wait till I/O threads actually terminate.
    //     for (io_threads_t::size_type i = 0; i != io_threads_size; i+= 1) {
    //         LIBZMQ_DELETE (_io_threads[i]);
    //     }
    //
    //     //  Deallocate the reaper thread object.
    //     LIBZMQ_DELETE (_reaper);
    //
    //     //  The mailboxes in _slots themselves were deallocated with their
    //     //  corresponding io_thread/socket objects.
    //
    //     //  De-initialise crypto library, if needed.
    //     random_close ();
    //
    // // #ifdef ZMQ_USE_NSS
    //     NSS_Shutdown ();
    // // #endif
    //
    // // #ifdef ZMQ_USE_GNUTLS
    //     gnutls_global_deinit ();
    // // #endif
    //
    //     //  Remove the tag, so that the object is considered dead.
    //     _tag = ZMQ_CTX_TAG_VALUE_BAD;
    // }

    //  Returns false if object is not a context.
    // bool check_tag () const;

    //  This function is called when user invokes zmq_ctx_term. If there are
    //  no more sockets open it'll cause all the infrastructure to be shut
    //  down. If there are open sockets still, the deallocation happens
    //  after the last one is closed.
    // int terminate ();

    // This function starts the terminate process by unblocking any blocking
    // operations currently in progress and stopping any more socket activity
    // (except zmq_close).
    // This function is non-blocking.
    // terminate must still be called afterwards.
    // This function is optional, terminate will unblock any current
    // operations as well.
    // int shutdown ();

    //  Set and get context properties.
    // int set (option_: i32, const optval_: *mut c_void, optvallen_: usize);
    // int get (option_: i32, optval_: *mut c_void, const optvallen_: *mut usize);
    // int get (option_: i32);

    //  Create and destroy a socket.
    // ZmqSocketBase *create_socket (type_: i32);
    // void destroy_socket (ZmqSocketBase *socket_);

    //  Send command to the destination thread.
    // void send_command (uint32_t tid, const command_t &command_);

    //  Returns the I/O thread that is the least busy at the moment.
    //  Affinity specifies which I/O threads are eligible (0 = all).
    //  Returns NULL if no I/O thread is available.
    // ZmqIoThread *choose_io_thread (uint64_t affinity_);

    //  Returns reaper thread object.
    // object_t *get_reaper () const;

    //  Management of inproc endpoints.
    // int register_endpoint (addr_: *const c_char, const endpoint_t &endpoint_);
    // int unregister_endpoint (const std::string &addr_,
    //                          const ZmqSocketBase *socket_);
    // void unregister_endpoints (const ZmqSocketBase *socket_);
    // endpoint_t find_endpoint (addr_: *const c_char);
    // void pend_connection (const std::string &addr_,
    //                       const endpoint_t &endpoint_,
    //                       ZmqPipe **pipes_);
    // void connect_pending (addr_: *const c_char, ZmqSocketBase *bind_socket_);

    // #ifdef ZMQ_HAVE_VMCI
    // Return family for the VMCI socket or -1 if it's not available.
    // int get_vmci_socket_family ();
    // #endif

    // ~ctx_t ();

    // bool valid () const;

    // bool start ();

    // static void
    //     connect_inproc_sockets (bind_socket_: *mut ZmqSocketBase,
    //                             const ZmqOptions &bind_options_,
    //                             const PendingConnection &pending_connection_,
    //                             side side_);
} // impl ZmqContext

pub fn clipped_maxsocket(mut max_requested: i32) -> i32 {
    if max_requested >= max_fds() && max_fds() != -1 {
        // -1 because we need room for the reaper mailbox.
        max_requested = max_fds() - 1;
    }

    return max_requested;
}
