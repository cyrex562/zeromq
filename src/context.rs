use std::collections::{HashMap, HashSet};
use std::mem::size_of;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::{mem, process};

use anyhow::{anyhow, bail};
use libc::{
    getpid, EADDRINUSE, ECONNREFUSED, EINTR, EINVAL, EMFILE, ENOENT, ENOMEM,
};

#[cfg(not(windows))]
use libc::{getgid, getuid, setgid, setuid, gid_t, pid_t, uid_t, F_SETFD, FD_CLOEXEC};

use serde::{Deserialize, Serialize};
use crate::address::ZmqAddress;

use crate::command::{CommandType, ZmqCommand};
use crate::defines::{ZMQ_BLOCKY, ZMQ_CURVE, ZMQ_DEALER, ZMQ_GSSAPI, ZMQ_GSSAPI_NT_HOSTBASED, ZMQ_GSSAPI_NT_KRB5_PRINCIPAL, ZMQ_GSSAPI_NT_USER_NAME, ZMQ_IO_THREADS, ZMQ_IO_THREADS_DFLT, ZMQ_IPV6, ZMQ_MAX_MSGSZ, ZMQ_MAX_SOCKETS, ZMQ_MAX_SOCKETS_DFLT, ZMQ_MESSAGE_SIZE, ZMQ_NULL, ZMQ_PAIR, ZMQ_PLAIN, ZMQ_PUB, ZMQ_PULL, ZMQ_PUSH, ZMQ_SOCKET_LIMIT, ZMQ_SUB, ZMQ_ZERO_COPY_RECV};
use crate::endpoint::{EndpointUriPair, ZmqEndpoint};
use crate::engine::ZmqEngine;
use crate::engine_interface::ZmqEngineInterface;
use crate::mailbox::ZmqMailbox;
use crate::mailbox_interface::ZmqMailboxInterface;
use crate::message::ZmqMessage;
use crate::object::ZmqObject;
use crate::own::ZmqOwn;
use crate::pending_connection::PendingConnection;
use crate::pipe::{send_hello_msg, ZmqPipe};
use crate::reaper::ZmqReaper;
use crate::session_base::ZmqSessionBase;
use crate::socket::ZmqSocket;
use crate::thread_context::ZmqThreadContext;
use crate::utils::zmq_z85_decode;


pub const CONNECT_SIDE: i32 = 0;
pub const BIND_SIDE: i32 = 1;
pub const TERM_TID: i32 = 0;
pub const REAPER_TID: u32 = 1;
pub const CURVE_KEYSIZE: usize = 128;
pub const CURVE_KEYSIZE_Z85: usize = 256;
pub const CURVE_KEYSIZE_Z85_P1: usize = CURVE_KEYSIZE_Z85 + 1;
pub const DEFAULT_HWM: u32 = 1000;
pub const DECISECONDS_PER_MILLISECOND: u32 = 100;

// #define ZMQ_CTX_TAG_VALUE_GOOD 0xabadcafe
pub const ZMQ_CTX_TAG_VALUE_GOOD: u32 = 0xabadcafe;
// #define ZMQ_CTX_TAG_VALUE_BAD 0xdeadbeef
pub const ZMQ_CTX_TAG_VALUE_BAD: u32 = 0xdeadbeef;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ZmqContext<'a> {
    pub thread_ctx: ZmqThreadContext,
    //  Used to check whether the object is a context.
    pub tag: u32,
    //  Sockets belonging to this context. We need the list so that
    //  we can notify the sockets when zmq_ctx_term() is called.
    //  The sockets will return ETERM then.
    pub sockets: Vec<ZmqSocket<'a>>,
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
    pub threads: Vec<ZmqThreadContext>,
    //  Array of pointers to mailboxes for both application and I/O threads.
    pub slots: Vec<ZmqMailbox>,
    //  Mailbox for zmq_ctx_term thread.
    pub term_mailbox: ZmqMailbox,
    //  List of inproc endpoints within this context.
    pub endpoints: HashMap<String, ZmqEndpoint<'a>>,
    // List of inproc onnection endpoints pending a Bind
    pub pending_connections: HashMap<String, PendingConnection>,
    //  Synchronisation of access to the list of inproc endpoints.
    pub endpoints_sync: Mutex<u8>,
    //  Maximum socket ID.
    pub max_socket_id: AtomicU64,
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
    pub tcp_accept_filters: Vec<ZmqAddress>,
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
    //  If connection handshake is not Done after this many milliseconds,
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
    // Device to Bind the underlying socket to, eg. VRF or interface
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

impl <'a> ZmqContext<'a> {
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
            threads: vec![],
            slots: vec![],
            term_mailbox: Default::default(),
            endpoints: Default::default(),
            pending_connections: Default::default(),
            endpoints_sync: Mutex::new(0),
            max_socket_id: AtomicU64::new(0),
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
            #[cfg(target_os="linux")]
            ipc_uid_accept_filters: Default::default(),
            #[cfg(target_os="linux")]
            ipc_gid_accept_filters: Default::default(),
            #[cfg(target_os="linux")]
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
            pid: 0,
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
                //  First send Stop command to sockets so that any blocking calls
                //  can be interrupted. If there are no sockets we can ask reaper
                //  thread to Stop.
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
            // zmq_assert(cmd.cmd_type == ZmqCommand::Done);
            let _lock = self.slot_sync.lock()?;
            // zmq_assert(self.sockets.empty());
        }
        self.slot_sync.unlock();

        // #ifdef ZMQ_HAVE_VMCI
        #[cfg(target_feature="vmci")]
        {
        let _ = self.vmci_sync.lock().expect("TODO: panic message");

        VMCISock_ReleaseAFValueFd(self.vmci_fd);
        self.vmci_family = -1;
        self.vmci_fd = -1;

        self.vmci_sync.unlock();}
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
                //  Send Stop command to sockets so that any blocking calls
                //  can be interrupted. If there are no sockets we can ask reaper
                //  thread to Stop.
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
            if !io_thread.get_mailbox().valid() {
                // delete io_thread;
                // goto fail_cleanup_reaper;
                self._reaper.stop();
                bail!("invalid mailbox");
            }

            if io_thread.mailbox.is_some() {
                self.slots[i] = io_thread.mailbox.unwrap().clone();
            }
            io_thread.start();
            self.threads.push_back(io_thread.to_owned());
        }

        //  In the unused part of the slot array, create a list of empty slots.
        // for (int32_t i = static_cast<int32_t> (_slots.size ()) - 1;
        //      i >= static_cast<int32_t> (ios) + term_and_reaper_threads_count; i -= 1)
        for i in self.slots.len() - 1..ios + term_and_reaper_threads_count {
            self.empty_slots.push_back(i);
        }

        self.starting = false;
        return true;

        // TODO:
        // fail_cleanup_reaper:
        //     _reaper->Stop ();
        //     delete _reaper;
        //     _reaper = NULL;

        // TODO:
        // fail_cleanup_slots:
        //     _slots.clear ();
        //     return false;
        return false;
    }

    // ZmqSocketBase *ZmqContext::create_socket (type_: i32)
    pub fn create_socket(&mut self, type_: i32) -> anyhow::Result<ZmqSocket> {
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
        let sid = self.max_socket_id.add(1) + 1;

        //  Create the socket and register its mailbox.
        let s = ZmqSocket::create(type_, self, slot, sid);
        // if ( ! s) {
        //     self.empty_slots.push_back(slot);
        //     return None;
        // }
        self.sockets.push_back(s);
        self.slots[slot] = s.get_mailbox();

        return Ok(s);
    }

    // void ZmqContext::destroy_socket (class ZmqSocketBase *socket_)
    pub fn destroy_socket(&mut self, socket: &mut ZmqSocket) {
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
        if self.threads.empty() {
            return None;
        }

        //  Find the I/O thread with minimum load.
        let mut min_load = -1;
        // ZmqIoThread *selected_io_thread = NULL;
        let mut selected_io_thread: Option<ZmqThreadContext> = None;
        // for (io_threads_t::size_type i = 0, size = _io_threads.size (); i != size;
        //      i+= 1)
        for i in 0..self.threads.len() {
            if !affinity || (affinity & (1 << i)) {
                let load = self.threads[i].get_load();
                if selected_io_thread.is_none() || load < min_load {
                    min_load = load;
                    selected_io_thread = Some(self.threads[i].clone());
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
        sock_base: &mut ZmqSocket,
    ) -> anyhow::Result<()> {
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
    pub fn unregister_endpoints(&mut self, socket: &mut ZmqSocket) {
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
    pub fn find_endpoint(&mut self, addr_: &str) -> anyhow::Result<ZmqEndpoint> {
        // scoped_lock_t locker (_endpoints_sync);

        // endpoints_t::iterator it = _endpoints.find (addr_);
        let endpoint = self.endpoints.get_mut(addr_);

        if endpoint.is_none() {
            // errno = ECONNREFUSED;
            // ZmqEndpoint empty = {NULL, ZmqOptions ()};
            bail!("ECONNREFUSED");
        }
        // ZmqEndpoint endpoint = it->second;

        //  Increment the command sequence number of the peer so that it won't
        //  get deallocated until "Bind" command is issued by the caller.
        //  The subsequent 'Bind' has to be called with inc_seqnum parameter
        //  set to false, so that the seqnum isn't incremented twice.
        endpoint.unwrap().socket.inc_seqnum();
        let x = endpoint.unwrap().clone();
        return Ok(x);
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
            //  Still no Bind.
            in_endpoint.socket.inc_seqnum();
            self.pending_connections
                .ZMQ_MAP_INSERT_OR_EMPLACE(in_addr, pending_connection);
        } else {
            //  Bind has happened in the mean time, connect directly
            self.connect_inproc_sockets(
                &mut it.unwrap().socket,
                &mut it.unwrap().context.clone(),
                &mut pending_connection,
                CONNECT_SIDE,
            );
        }
    }

    // void ZmqContext::connect_pending (addr_: *const c_char,
    //                                   ZmqSocketBase *bind_socket_)
    pub fn connect_pending(&mut self, addr_: &str, bind_socket_: &mut ZmqSocket) {
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
                &mut self.endpoints[addr_].context.clone(),
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
        bind_socket: &mut ZmqSocket,
        bind_context: &mut ZmqContext,
        pending_connection: &mut PendingConnection,
        side: i32,
    ) {
        bind_socket.inc_seqnum();
        pending_connection.bind_pipe.set_tid(bind_socket.get_tid());

        if (!bind_context.recv_routing_id) {
            // ZmqMessage msg;
            let mut msg = ZmqMessage::default();
            let ok = pending_connection.bind_pipe.read(&mut msg);
            // zmq_assert(ok);
            let rc = msg.close();
            // errno_assert(rc == 0);
        }

        if !get_effective_conflate_option(&pending_connection.endpoint.context) {
            pending_connection
                .connect_pipe
                .set_hwms_boost(bind_context.sndhwm, bind_context.rcvhwm);
            pending_connection.bind_pipe.set_hwms_boost(
                pending_connection.endpoint.context.sndhwm,
                pending_connection.endpoint.context.rcvhwm,
            );

            pending_connection.connect_pipe.set_hwms(
                pending_connection.endpoint.context.rcvhwm as u32,
                pending_connection.endpoint.context.sndhwm as u32,
            );
            pending_connection
                .bind_pipe
                .set_hwms(bind_context.rcvhwm as u32, bind_context.sndhwm as u32);
        } else {
            pending_connection.connect_pipe.set_hwms(-1, -1);
            pending_connection.bind_pipe.set_hwms(-1, -1);
        }

        // #ifdef ZMQ_BUILD_DRAFT_API
        if (bind_context.can_recv_disconnect_msg && !bind_context.disconnect_msg.empty()) {
            pending_connection
                .connect_pipe
                .set_disconnect_msg(&mut bind_context.disconnect_msg);
        }
        // #endif

        if (side == BIND_SIDE) {
            let mut cmd = ZmqCommand::default();
            cmd.cmd_type = ZmqCommand::bind;
            cmd.args.bind.pipe = &mut pending_connection.bind_pipe.clone();
            bind_socket.process_command(&cmd);
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
        // in waiting_for_delimiter state, which means no more writes can be Done
        // and the routing id write fails and causes an assert. Check if the socket
        // is open before sending.
        if (pending_connection.endpoint.context.recv_routing_id
            && pending_connection.endpoint.socket.check_tag())
        {
            bind_socket.send_routing_id(&mut pending_connection.bind_pipe, bind_context);
        }

        // #ifdef ZMQ_BUILD_DRAFT_API
        //  If set, send the hello msg of the Bind socket to the pending connection.
        if (bind_context.can_send_hello_msg && bind_context.hello_msg.size() > 0) {
            send_hello_msg(&mut pending_connection.bind_pipe, bind_context);
        }
        // #endif
    }

    // #ifdef ZMQ_HAVE_VMCI
    // int ZmqContext::get_vmci_socket_family ()
    #[cfg(target_feature = "vmci")]
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
    //     //  Ask I/O threads to terminate. If Stop signal wasn't sent to I/O
    //     //  thread subsequent invocation of destructor would hang-up.
    //     const io_threads_t::size_type io_threads_size = _io_threads.size ();
    //     for (io_threads_t::size_type i = 0; i != io_threads_size; i+= 1) {
    //         _io_threads[i]->Stop ();
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

    pub fn set_curve_key(
        &mut self,
        destination: &mut [u8],
        opt_val: &[u8],
        opt_val_len: usize,
    ) -> i32 {
        unsafe {
            match opt_val_len {
                CURVE_KEYSIZE => {
                    // unsafe { memcpy(destination_ as *mut c_void, opt_val, opt_val_len); }
                    destination.clone_from_slice(opt_val);
                    self.mechanism = ZMQ_CURVE as i32;
                    return 0;
                }
                CURVE_KEYSIZE_Z85_P1 => {
                    // const std::string s (static_cast<const char *> (opt_val),
                    //                      opt_val_len);
                    let mut s =
                        String::from_raw_parts(opt_val as *mut u8, opt_val_len, opt_val_len + 1);

                    if zmq_z85_decode(destination, s.c_str()) {
                        self.mechanism = ZMQ_CURVE as i32;
                        return 0;
                    }
                }

                CURVE_KEYSIZE_Z85 => {
                    let mut z85_key: [u8; CURVE_KEYSIZE_Z85_P1] = [0; CURVE_KEYSIZE_Z85_P1];
                    // memcpy(z85_key as *mut c_void, opt_val, opt_val_len);
                    z85_key.clone_from_slice(opt_val);
                    z85_key[CURVE_KEYSIZE_Z85] = 0;
                    if zmq_z85_decode(destination, &z85_key) {
                        self.mechanism = ZMQ_CURVE as i32;
                        return 0;
                    }
                }
                _ => {}
            }
        }
        return -1;
    }

    pub fn setsockopt(
        &mut self,
        opt: i32,
        opt_val: &[u8],
        opt_val_len: usize,
    ) -> anyhow::Result<()> {
        let mut i32_size = mem::size_of::<i32>();
        let is_int = opt_val_len == i32_size;
        let mut value = 0i32;
        if is_int {
            // unsafe { memcpy(&mut value as *mut c_void, opt_val, i32_size); }
            let val_bytes: [u8; 4] = [0; 4];
            val_bytes.clone_from_slice(opt_val);
            value = i32::from_le_bytes(val_bytes);
        }
        // #if defined(ZMQ_ACT_MILITANT)
        //     let mut malformed = true; //  Did caller pass a bad option value?
        // #endif

        match opt {
            ZMQ_SNDHWM => {
                if is_int && value >= 0 {
                    self.sndhwm = value;
                    return Ok(());
                }
            }
            ZMQ_RCVHWM => {
                if is_int && value >= 0 {
                    self.rcvhwm = value;
                    return Ok(());
                }
            }
            ZMQ_AFFINITY => {
                // return do_setsockopt(opt_val, opt_val_len, &mut self.affinity);
                return set_opt_u64(opt_val, &mut self.affinity);
            }
            ZMQ_ROUTING_ID => unsafe {
                //  Routing id is any binary string from 1 to 255 octets
                if opt_val_len > 0 && opt_val_len <= u8::MAX as usize {
                    self.routing_id_size = opt_val_len;
                    // memcpy(&mut self.routing_id as *mut c_void, opt_val, self.routing_id_size);
                    self.routing_id.clone_from_slice(opt_val);
                    return Ok(());
                }
            },
            ZMQ_RATE => {
                if is_int && value > 0 {
                    self.rate = value;
                    return Ok(());
                }
            }
            ZMQ_RECOVERY_IVL => {
                if is_int && value >= 0 {
                    self.recovery_ivl = value;
                    return Ok(());
                }
            }
            ZMQ_SNDBUF => {
                if is_int && value >= -1 {
                    self.sndbuf = value;
                    return Ok(());
                }
            }
            ZMQ_RCVBUF => {
                if (is_int && value >= -1) {
                    self.rcvbuf = value;
                    return Ok(());
                }
            }
            ZMQ_TOS => {
                if (is_int && value >= 0) {
                    self.tos = value;
                    return Ok(());
                }
            }
            ZMQ_LINGER => {
                if is_int && value >= -1 {
                    self.linger.store(value as u64, Ordering::Relaxed);
                    return Ok(());
                }
            }

            ZMQ_CONNECT_TIMEOUT => {
                if is_int && value >= 0 {
                    self.connect_timeout = value;
                    return Ok(());
                }
            }

            ZMQ_TCP_MAXRT => {
                if is_int && value >= 0 {
                    self.tcp_maxrt = value;
                    return Ok(());
                }
            }

            ZMQ_RECONNECT_STOP => {
                if (is_int) {
                    self.reconnect_stop = value;
                    return Ok(());
                }
            }

            ZMQ_RECONNECT_IVL => {
                if is_int && value >= -1 {
                    self.reconnect_ivl = value;
                    return Ok(());
                }
            }

            ZMQ_RECONNECT_IVL_MAX => {
                if is_int && value >= 0 {
                    self.reconnect_ivl_max = value;
                    return Ok(());
                }
            }

            ZMQ_BACKLOG => {
                if is_int && value >= 0 {
                    self.backlog = value;
                    return Ok(());
                }
            }

            ZMQ_MAXMSGSIZE => {
                // return do_setsockopt(opt_val, opt_val_len, &mut self.maxmsgsize);
                return set_opt_i64(opt_val, &mut self.maxmsgsize);
            }

            ZMQ_MULTICAST_HOPS => {
                if is_int && value > 0 {
                    self.multicast_hops = value;
                    return Ok(());
                }
            }

            ZMQ_MULTICAST_MAXTPDU => {
                if is_int && value > 0 {
                    self.multicast_maxtpdu = value;
                    return Ok(());
                }
            }

            ZMQ_RCVTIMEO => {
                if is_int && value >= -1 {
                    self.rcvtimeo = value;
                    return Ok(());
                }
            }

            ZMQ_SNDTIMEO => {
                if is_int && value >= -1 {
                    self.sndtimeo = value;
                    return Ok(());
                }
            }

            /*  Deprecated in favor of ZMQ_IPV6  */
            ZMQ_IPV4ONLY => {
                let mut value = false;
                // let rc = do_setsockopt_int_as_bool_strict(opt_val, opt_val_len, &mut value);
                if set_opt_bool(opt_val, &mut value).is_ok() {
                    self.ipv6 = !value;
                }
                return Ok(());
            }

            /*  To replace the somewhat surprising IPV4ONLY */
            ZMQ_IPV6 => {
                // return do_setsockopt_int_as_bool_strict(opt_val, opt_val_len,
                //                                         &mut self.ipv6);
                return set_opt_bool(opt_val, &mut self.ipv6);
            }

            ZMQ_SOCKS_PROXY => {
                return set_opt_string(opt_val, &mut self.socks_proxy_address);
            }

            ZMQ_SOCKS_USERNAME => {
                /* Make empty string or NULL equivalent. */
                return if opt_val.is_empty() {
                    self.socks_proxy_username.clear();
                    return Ok(());
                } else {
                    return set_opt_string(opt_val, &mut self.socks_proxy_username);
                };
            }
            ZMQ_SOCKS_PASSWORD => {
                /* Make empty string or NULL equivalent. */
                return if opt_val.is_empty() {
                    self.socks_proxy_password.clear();
                    return Ok(());
                } else {
                    return set_opt_string(opt_val, &mut self.socks_proxy_password);
                };
            }
            ZMQ_TCP_KEEPALIVE => {
                if is_int && (value == -1 || value == 0 || value == 1) {
                    self.tcp_keepalive = value;
                    return Ok(());
                }
            }

            ZMQ_TCP_KEEPALIVE_CNT => {
                if is_int && (value == -1 || value >= 0) {
                    self.tcp_keepalive_cnt = value;
                    return Ok(());
                }
            }

            ZMQ_TCP_KEEPALIVE_IDLE => {
                if is_int && (value == -1 || value >= 0) {
                    self.tcp_keepalive_idle = value;
                    return Ok(());
                }
            }

            ZMQ_TCP_KEEPALIVE_INTVL => {
                if is_int && (value == -1 || value >= 0) {
                    self.tcp_keepalive_intvl = value;
                    return Ok(());
                }
            }

            ZMQ_IMMEDIATE => {
                // TODO why is immediate not bool (and called non_immediate, as its meaning appears to be reversed)
                if (is_int && (value == 0 || value == 1)) {
                    self.immediate = value;
                    return Ok(());
                }
            }

            ZMQ_TCP_ACCEPT_FILTER => {
                let mut filter_str = String::new();
                // let mut rc = do_setsockopt_string_allow_empty_strict(
                //     opt_val, opt_val_len, &mut filter_str, UCHAR_MAX);
                if set_opt_string(opt_val, &mut filter_str).is_ok() {
                    if filter_str.empty() {
                        self.tcp_accept_filters.clear();
                    } else {
                        // TODO substitute with ZmqAddress in filter capacity
                        // let mut mask = TcpAddressMask::default();
                        // let mut rc = mask.resolve(&filter_str, ipv6);
                        // if rc == 0 {
                        //     self.tcp_accept_filters.push_back(mask);
                        // }
                    }
                }
                return Ok(());
            }

            // #if defined ZMQ_HAVE_SO_PEERCRED || defined ZMQ_HAVE_LOCAL_PEERCRED
            #[cfg(target_os = "linux")]
            ZMQ_IPC_FILTER_UID => {
                return set_opt_uid_hash_set(opt_val, &mut self.ipc_uid_accept_filters);
                // if cfg!(target_os = "linux") {
                //     return set_opt_h(opt_val, opt_val_len,
                //                              &mut self.ipc_uid_accept_filters);
                // } else {
                //     return Err(anyhow!("IPC/UID not supported"));
                // }
            }
            #[cfg(target_os = "linux")]
            ZMQ_IPC_FILTER_GID => {
                return set_opt_gid_hash_set(opt_val, &mut self.ipc_gid_accept_filters);
            }
            // #endif

            // #if defined ZMQ_HAVE_SO_PEERCRED
            #[cfg(target_os = "linux")]
            ZMQ_IPC_FILTER_PID => {
                return set_opt_pid_hash_set(opt_val, &mut self.ipc_pid_accept_filters);
            }
            // #endif
            ZMQ_PLAIN_SERVER => {
                if is_int && (value == 0 || value == 1) {
                    self.as_server = value;
                    self.mechanism = if value { ZMQ_PLAIN } else { ZMQ_NULL } as i32;
                    return Ok(());
                }
            }

            ZMQ_PLAIN_USERNAME => {
                if opt_val_len == 0 && opt_val.is_null() {
                    self.mechanism = ZMQ_NULL as i32;
                    return Ok(());
                } else if opt_val_len > 0 && opt_val_len <= u8::MAX as usize && opt_val.is_null() == false
                {
                    // self.plain_username.assign(static_cast <const char
                    // * > (opt_val),
                    // opt_val_len);
                    unsafe {
                        self.plain_username = String::from_raw_parts(
                            opt_val as *mut u8,
                            opt_val_len,
                            opt_val_len + 1,
                        );
                    }
                    self.as_server = 0;
                    self.mechanism = ZMQ_PLAIN as i32;
                    return Ok(());
                }
            }

            ZMQ_PLAIN_PASSWORD => {
                if opt_val_len == 0 && opt_val.is_null() {
                    self.mechanism = ZMQ_NULL as i32;
                    return Ok(());
                } else if (opt_val_len > 0
                    && opt_val_len <= u8::MAX as usize
                    && opt_val.is_null() == false)
                {
                    // plain_password.assign(static_cast <const char
                    // * > (opt_val),
                    // opt_val_len);
                    unsafe {
                        self.plain_password = String::from_raw_parts(
                            opt_val as *mut u8,
                            opt_val_len,
                            opt_val_len + 1,
                        );
                    }
                    self.as_server = 0;
                    self.mechanism = ZMQ_PLAIN as i32;
                    return Ok(());
                }
            }

            ZMQ_ZAP_DOMAIN => {
                return set_opt_string(opt_val, &mut self.zap_domain);
            }

            //  If curve encryption isn't built, these options provoke EINVAL
            // #ifdef ZMQ_HAVE_CURVE
            ZMQ_CURVE_SERVER => {
                if is_int && (value == 0 || value == 1) {
                    self.as_server = value;
                    self.mechanism = if value { ZMQ_CURVE } else { ZMQ_NULL } as i32;
                    return Ok(());
                }
            }

            ZMQ_CURVE_PUBLICKEY => {
                if 0 == self.set_curve_key(&mut self.curve_public_key, opt_val, opt_val_len) {
                    return Ok(());
                }
            }

            ZMQ_CURVE_SECRETKEY => {
                if 0 == self.set_curve_key(&mut self.curve_secret_key, opt_val, opt_val_len) {
                    return Ok(());
                }
            }

            ZMQ_CURVE_SERVERKEY => {
                if 0 == self.set_curve_key(&mut self.curve_server_key, opt_val, opt_val_len) {
                    self.as_server = 0;
                    return Ok(());
                }
            }

            // #endif
            ZMQ_CONFLATE => {
                // return do_setsockopt_int_as_bool_strict(opt_val, opt_val_len,
                //                                         &mut self.conflate);
                return set_opt_bool(opt_val, &mut self.conflate);
            }

            //  If libgssapi isn't installed, these options provoke EINVAL
            // #ifdef HAVE_LIBGSSAPI_KRB5
            ZMQ_GSSAPI_SERVER => {
                if is_int && (value == 0 || value == 1) {
                    self.as_server = value;
                    self.mechanism = ZMQ_GSSAPI as i32;
                    return Ok(());
                }
            }

            ZMQ_GSSAPI_PRINCIPAL => {
                if opt_val_len > 0 && opt_val_len <= u8::MAX as usize && opt_val != null_mut() {
                    // self.gss_principal.assign((const char
                    // *) opt_val, opt_val_len);
                    unsafe {
                        self.gss_principal = String::from_raw_parts(
                            opt_val as *mut u8,
                            opt_val_len,
                            opt_val_len + 1,
                        );
                    }
                    self.mechanism = ZMQ_GSSAPI as i32;
                    return Ok(());
                }
            }

            ZMQ_GSSAPI_SERVICE_PRINCIPAL => {
                if opt_val_len > 0 && opt_val_len <= u8::MAX as usize && opt_val.is_empty() == false {
                    // gss_service_principal.assign((const char
                    // *) opt_val,
                    // opt_val_len);
                    unsafe {
                        self.gss_service_principal = String::from_raw_parts(
                            opt_val as *mut u8,
                            opt_val_len,
                            opt_val_len + 1,
                        );
                    }
                    self.mechanism = ZMQ_GSSAPI as i32;
                    self.as_server = 0;
                    return Ok(());
                }
            }

            ZMQ_GSSAPI_PLAINTEXT => {
                // return do_setsockopt_int_as_bool_strict(opt_val, opt_val_len,
                //                                         &mut self.gss_plaintext);
                return set_opt_bool(opt_val, &mut self.gss_plaintext);
            }

            ZMQ_GSSAPI_PRINCIPAL_NAMETYPE => {
                if is_int
                    && (value == ZMQ_GSSAPI_NT_HOSTBASED
                    || value == ZMQ_GSSAPI_NT_USER_NAME
                    || value == ZMQ_GSSAPI_NT_KRB5_PRINCIPAL)
                {
                    self.gss_principal_nt = value;
                    return Ok(());
                }
            }

            ZMQ_GSSAPI_SERVICE_PRINCIPAL_NAMETYPE => {
                if is_int
                    && (value == ZMQ_GSSAPI_NT_HOSTBASED
                    || value == ZMQ_GSSAPI_NT_USER_NAME
                    || value == ZMQ_GSSAPI_NT_KRB5_PRINCIPAL)
                {
                    self.gss_service_principal_nt = value;
                    return Ok(());
                }
            }

            // #endif
            ZMQ_HANDSHAKE_IVL => {
                if is_int && value >= 0 {
                    self.handshake_ivl = value;
                    return Ok(());
                }
            }

            ZMQ_INVERT_MATCHING => {
                return set_opt_bool(opt_val, &mut self.invert_matching);
            }

            ZMQ_HEARTBEAT_IVL => {
                if is_int && value >= 0 {
                    self.heartbeat_interval = value;
                    return Ok(());
                }
            }

            ZMQ_HEARTBEAT_TTL => {
                // Convert this to deciseconds from milliseconds
                value = value / DECISECONDS_PER_MILLISECOND;
                if is_int && value >= 0 && value <= u16::MAX as i32 {
                    self.heartbeat_ttl = value as u16;
                    return Ok(());
                }
            }

            ZMQ_HEARTBEAT_TIMEOUT => {
                if is_int && value >= 0 {
                    self.heartbeat_timeout = value;
                    return Ok(());
                }
            }

            // #ifdef ZMQ_HAVE_VMCI
            ZMQ_VMCI_BUFFER_SIZE => {
                return set_opt_u64(opt_val, &mut self.vmci_buffer_size);
            }

            ZMQ_VMCI_BUFFER_MIN_SIZE => {
                return set_opt_u64(opt_val, &mut self.vmci_buffer_min_size);
            }

            ZMQ_VMCI_BUFFER_MAX_SIZE => {
                return set_opt_u64(opt_val, &mut self.vmci_buffer_max_size);
            }

            ZMQ_VMCI_CONNECT_TIMEOUT => {
                return set_opt_u64(opt_val, &mut self.vmci_connect_timeout);
            }
            // #endif
            ZMQ_USE_FD => {
                if (is_int && value >= -1) {
                    self.use_fd = value;
                    return Ok(());
                }
            }

            ZMQ_BINDTODEVICE => {
                return set_opt_string(opt_val, &mut self.bound_device);
            }

            ZMQ_ZAP_ENFORCE_DOMAIN => {
                return set_opt_bool(opt_val, &mut self.zap_enforce_domain);
            }

            ZMQ_LOOPBACK_FASTPATH => {
                return set_opt_bool(opt_val, &mut self.loopback_fastpath);
            }

            ZMQ_METADATA => {
                unsafe {
                    if opt_val_len > 0 && !is_int {
                        // const std
                        // ::string
                        // s(static_cast <const char
                        // * > (opt_val),
                        // opt_val_len);
                        let mut s = String::from_raw_parts(
                            opt_val as *mut u8,
                            opt_val_len,
                            opt_val_len + 1,
                        );
                        // const size_t
                        // pos = s.find(':');
                        let mut pos = s.find(':');
                        if pos.is_some() && pos.unwrap() != 0 && pos.unwrap() != s.length() - 1 {
                            let mut key = &s[0..pos.unwrap()];
                            if key[0..2] == "X-" && key.len() <= u8::MAX as usize {
                                let val = &s[pos.unwrap() + 1..s.len()];
                                self.app_metadata
                                    .insert(String::from(key), String::from(val));
                                return Ok(());
                            }
                        }
                    }
                }
                // errno = EINVAL;
                // return -1;
                return Err(anyhow!("EINVAL"));
            }

            ZMQ_MULTICAST_LOOP => {
                return set_opt_bool(opt_val, &mut self.multicast_loop);
            }

            // #ifdef ZMQ_BUILD_DRAFT_API
            ZMQ_IN_BATCH_SIZE => {
                if (is_int && value > 0) {
                    self.in_batch_size = value;
                    return Ok(());
                }
            }

            ZMQ_OUT_BATCH_SIZE => {
                if (is_int && value > 0) {
                    self.out_batch_size = value;
                    return Ok(());
                }
            }

            ZMQ_BUSY_POLL => {
                if (is_int) {
                    self.busy_poll = value;
                    return Ok(());
                }
            }

            // #ifdef ZMQ_HAVE_WSS
            ZMQ_WSS_KEY_PEM => {
                // TODO: check if valid certificate
                // wss_key_pem = std::string( opt_val, opt_val_len);
                unsafe {
                    self.wss_key_pem =
                        String::from_raw_parts(opt_val as *mut u8, opt_val_len, opt_val_len + 1);
                }
                return Ok(());
            }
            ZMQ_WSS_CERT_PEM => {
                // TODO: check if valid certificate
                unsafe {
                    self.wss_cert_pem =
                        String::from_raw_parts(opt_val as *mut u8, opt_val_len, opt_val_len + 1);
                }
                return Ok(());
            }
            ZMQ_WSS_TRUST_PEM => {
                // TODO: check if valid certificate
                unsafe {
                    self.wss_trust_pem =
                        String::from_raw_parts(opt_val as *mut u8, opt_val_len, opt_val_len + 1);
                };
                return Ok(());
            }
            ZMQ_WSS_HOSTNAME => {
                unsafe {
                    self.wss_hostname =
                        String::from_raw_parts(opt_val as *mut u8, opt_val_len, opt_val_len + 1);
                }
                return Ok(());
            }
            ZMQ_WSS_TRUST_SYSTEM => {
                // return do_setsockopt_int_as_bool_strict(opt_val, opt_val_len,
                //                                         &mut self.wss_trust_system);
                return set_opt_bool(opt_val, &mut self.wss_trust_system);
            }
            // #endif
            ZMQ_HELLO_MSG => {
                unsafe {
                    if opt_val_len > 0 {
                        // unsigned
                        // char * bytes = (unsigned
                        // char *) opt_val;
                        // hello_msg = std::vector < unsigned
                        // char > (bytes, bytes + opt_val_len);
                        let bytes: Vec<u8> =
                            Vec::from_raw_parts(opt_val as *mut u8, opt_val_len, opt_val_len + 1);
                        self.hello_msg = bytes;
                    } else {
                        self.hello_msg.clear();
                    }
                }

                return Ok(());
            }

            ZMQ_DISCONNECT_MSG => unsafe {
                if opt_val_len > 0 {
                    // unsigned
                    // char * bytes = (unsigned
                    // char *) opt_val;
                    // disconnect_msg = std::vector < unsigned
                    // char > (bytes, bytes + opt_val_len);
                    let bytes: Vec<u8> =
                        Vec::from_raw_parts(opt_val as *mut u8, opt_val_len, opt_val_len + 1);
                    self.disconnect_msg = bytes;
                } else {
                    self.hello_msg.clear();
                }

                return Ok(());
            },

            ZMQ_PRIORITY => {
                if is_int && value >= 0 {
                    self.priority = value;
                    return Ok(());
                }
            }

            ZMQ_HICCUP_MSG => {
                unsafe {
                    if opt_val_len > 0 {
                        // unsigned
                        // char * bytes = (unsigned
                        // char *) opt_val;
                        // hiccup_msg = std::vector < unsigned
                        // char > (bytes, bytes + opt_val_len);
                        let bytes: Vec<u8> =
                            Vec::from_raw_parts(opt_val as *mut u8, opt_val_len, opt_val_len + 1);
                        self.hiccup_msg = bytes;
                    } else {
                        // hiccup_msg = std::vector < unsigned
                        // char > ();
                        self.hiccup_msg.clear();
                    }
                }

                return Ok(());
            }

            // #endif

            // _ =>
            _ => {
                // #if defined(ZMQ_ACT_MILITANT)
                //  There are valid scenarios for probing with unknown socket option
                //  values, e.g. to check if security is enabled or not. This will not
                //  provoke a militant assert. However, passing bad values to a valid
                //  socket option will, if ZMQ_ACT_MILITANT is defined.
                self.malformed = false;
                // #endif
                //
            }
        }

        // TODO mechanism should either be set explicitly, or determined when
        // connecting. currently, it depends on the order of setsockopt calls
        // if there is some inconsistency, which is confusing. in addition,
        // the assumed or set mechanism should be queryable (as a socket option)

        // #if defined(ZMQ_ACT_MILITANT)
        //  There is no valid use case for passing an error back to the application
        //  when it sent malformed arguments to a socket option. Use ./configure
        //  --with-militant to enable this checking.
        if self.malformed {
            // zmq_assert(false);
        }
        // #endif
        // errno = EINVAL;
        // return -1;
        return Err(anyhow!("EINVAL"));
    }

    // int getsockopt (option_: i32, opt_val: *mut c_void, opt_val_len: *mut usize) const;
    pub fn getsockopt(&mut self, opt_kind: i32) -> anyhow::Result<Vec<u8>> {
        // let is_int = *opt_val_len == mem::size_of::<i32>();
        // let mut value = opt_val;
        // #if defined(ZMQ_ACT_MILITANT)
        //         let mut malformed = true; //  Did caller pass a bad option value?
        // #endif

        match opt_kind {
            ZMQ_SNDHWM => {
                return Ok(self.sndhwm.to_le_bytes().to_vec());
            }

            ZMQ_RCVHWM => {
                return Ok(self.sndhwm.to_le_bytes().to_vec());
            }

            ZMQ_AFFINITY => {
                return Ok(self.affinity.to_le_bytes().to_vec());
            }

            ZMQ_ROUTING_ID => {
                // let mut opt_val: Vec<u8> = Vec::with_capacity(self.routing_id_size);
                // do_getsockopt2(&mut opt_val, &self.routing_id);
                // return Ok(opt_val);
                return Ok( self.routing_id.into_bytes())
            }

            ZMQ_RATE => {
                return Ok(self.rate.to_le_bytes().to_vec());
            }

            ZMQ_RECOVERY_IVL => {
                return Ok(self.recovery_ivl.to_le_bytes().to_vec());
            }

            ZMQ_SNDBUF => {
                return Ok(self.sndbuf.to_le_bytes().to_vec());
            }

            ZMQ_RCVBUF => {
                return Ok(self.rcvbuf.to_le_bytes().to_vec());
            }

            ZMQ_TOS => {
                return Ok(self.tos.to_le_bytes().to_vec());
            }

            ZMQ_TYPE => {
                return Ok(self.type_.to_le_bytes().to_vec());
            }

            ZMQ_LINGER => {
                return Ok(self.linger.load(Ordering::Relaxed).to_le_bytes().to_vec());
            }

            ZMQ_CONNECT_TIMEOUT => {
                return Ok(self.connect_timeout.to_le_bytes().to_vec());
            }

            ZMQ_TCP_MAXRT => {
                return Ok(self.tcp_maxrt.to_le_bytes().to_vec());
            }

            ZMQ_RECONNECT_STOP => {
                return Ok(self.reconnect_stop.to_le_bytes().to_vec());
            }

            ZMQ_RECONNECT_IVL => {
                return Ok(self.reconnect_ivl.to_le_bytes().to_vec());
            }

            ZMQ_RECONNECT_IVL_MAX => {
                return Ok(self.reconnect_ivl_max.to_le_bytes().to_vec());
            }

            ZMQ_BACKLOG => {
                return Ok(self.backlog.to_le_bytes().to_vec());
            }

            ZMQ_MAXMSGSIZE => {
                // if (*opt_val_len == mem::size_of::<i64>()) {
                //     // *opt_val = self.maxmsgsize;
                //     // value.clone_from_slice(self.maxmsgsize.to_le_bytes().as_slice());
                //     *opt_val_len = mem::size_of::<i64>();
                //     return Ok(());
                // }
                return Ok(self.maxmsgsize.to_le_bytes().to_vec());
            }

            ZMQ_MULTICAST_HOPS => {
                // if (is_int) {
                //     // *value = self.multicast_hops;
                //     value.clone_from_slice(self.multicast_hops.to_le_bytes().as_slice());
                //     return Ok(());
                // }
                return Ok((self.multicast_hops.to_le_bytes().to_vec()));
            }

            ZMQ_MULTICAST_MAXTPDU => {
                // if (is_int) {
                //     // *value = self.multicast_maxtpdu;
                //     value.clone_from_slice(self.multicast_maxtpdu.to_le_bytes().as_slice());
                //     return Ok(());
                // }
                return Ok(self.multicast_maxtpdu.to_le_bytes().to_vec());
            }

            ZMQ_RCVTIMEO => {
                // if (is_int) {
                //     // *value = self.rcvtimeo;
                //     value.clone_from_slice(self.rcvtimeo.to_le_bytes().as_slice());
                //     return Ok(());
                // }
                return Ok(self.rcvtimeo.to_le_bytes().to_vec());
            }

            ZMQ_SNDTIMEO => {
                // if (is_int) {
                //     // *value = self.sndtimeo;
                //     value.clone_from_slice(self.sndtimeo.to_le_bytes().as_slice());
                //     return Ok(());
                // }
                return Ok(self.sndtimeo.to_le_bytes().to_vec());
            }

            ZMQ_IPV4ONLY => {
                // if (is_int) {
                //     // *value = 1 - self.ipv6;
                //     let x: u8 = self.ipv6.into();
                //     value[0] = x;
                //     return Ok(());
                // }
                match self.ipv6 {
                    true => {
                        let truth_val = 0i32;
                        return Ok(truth_val.to_le_bytes().to_vec());
                    }
                    false => {
                        let truth_val = 1i32;
                        return Ok(truth_val.to_le_bytes().to_vec());
                    }
                }
            }

            ZMQ_IPV6 => {
                // if (is_int) {
                //     // *value = self.ipv6;
                //     let x: u8 = self.ipv6.into();
                //     value[0] = x;
                //     return Ok(());
                // }
                match self.ipv6 {
                    true => {
                        let truth_val = 1i32;
                        return Ok(truth_val.to_le_bytes().to_vec());
                    }
                    false => {
                        let truth_val = 0i32;
                        return Ok(truth_val.to_le_bytes().to_vec());
                    }
                }
            }

            ZMQ_IMMEDIATE => {
                // if (is_int) {
                //     // *value = self.immediate;
                //     value.clone_from_slice(self.immediate.to_le_bytes().as_slice());
                //     return Ok(());
                // }
                return Ok(self.immediate.to_le_bytes().to_vec());
            }

            ZMQ_SOCKS_PROXY => {
                // return do_getsockopt(opt_val, opt_val_len, &self.socks_proxy_address);
                return str_to_vec(&self.socks_proxy_address);
            }

            ZMQ_SOCKS_USERNAME => {
                // return do_getsockopt(opt_val, opt_val_len, &self.socks_proxy_username);
                return str_to_vec(&self.scoks_proxy_username);
            }

            ZMQ_SOCKS_PASSWORD => {
                // return do_getsockopt(opt_val, opt_val_len, &self.socks_proxy_password);
                return str_to_vec(&self.socks_proxy_password);
            }

            ZMQ_TCP_KEEPALIVE => {
                // if (is_int) {
                //     // *value = self.tcp_keepalive;
                //     value.clone_from_slice(self.tcp_keepalive.to_le_bytes().as_slice());
                //     return Ok(());
                // }
                return Ok(self.tcp_keepalive.to_le_bytes().to_vec());
            }

            ZMQ_TCP_KEEPALIVE_CNT => {
                // if (is_int) {
                //     // *value = self.tcp_keepalive_cnt;
                //     value.clone_from_slice(self.tcp_keepalive_cnt.to_le_bytes().as_slice());
                //     return Ok(());
                // }
                return Ok(self.tcp_keepalive_cnt.to_le_bytes().to_vec());
            }

            ZMQ_TCP_KEEPALIVE_IDLE => {
                // if (is_int) {
                //     // *value = self.tcp_keepalive_idle;
                //     value.clone_from_slice(self.tcp_keepalive_idle.to_le_bytes().as_slice());
                //     return Ok(());
                // }
                return Ok(self.tcp_keepalive_idle.to_le_bytes().to_vec());
            }

            ZMQ_TCP_KEEPALIVE_INTVL => {
                // if (is_int) {
                //     // *value = self.tcp_keepalive_intvl;
                //     value.clone_from_slice(self.tcp_keepalive_intvl.to_le_bytes().as_slice());
                //     return Ok(());
                // }
                return Ok(self.tcp_keepalive_intvl.to_le_bytes().to_vec());
            }

            ZMQ_MECHANISM => {
                // if (is_int) {
                //     // *value = self.mechanism;
                //     value.clone_from_slice(self.mechanism.to_le_bytes().as_slice());
                //     return Ok(());
                // }
                return Ok(self.mechanism.to_le_bytes().to_vec());
            }

            ZMQ_PLAIN_SERVER => {
                // if (is_int) {
                //     // *value = self.as_server && self.mechanism == ZMQ_PLAIN;
                //     let x = self.as_server != 0 && self.mechanism == ZMQ_PLAIN;
                //     value[0] = x.into();
                //     return Ok(());
                // }
                return bool_to_vec(self.as_server != 0 && self.mechanism == ZMQ_PLAIN);
            }

            ZMQ_PLAIN_USERNAME => {
                // return do_getsockopt(opt_val, opt_val_len, &self.plain_username);
                return str_to_vec(&self.plain_username);
            }

            ZMQ_PLAIN_PASSWORD => {
                // return do_getsockopt(opt_val, opt_val_len, &self.plain_password);
                return str_to_vec(&self.plain_password);
            }

            ZMQ_ZAP_DOMAIN => {
                // return do_getsockopt(opt_val, opt_val_len, &self.zap_domain);
                return str_to_vec(&self.zap_domain);
            }

            //  If curve encryption isn't built, these options provoke EINVAL
            // #ifdef ZMQ_HAVE_CURVE
            ZMQ_CURVE_SERVER => {
                // if (is_int) {
                //     // *value = self.as_server && self.mechanism == ZMQ_CURVE;
                //     let x = self.as_server != 0 && self.mechanism == ZMQ_CURVE;
                //     value[0] = x.into();
                //     return Ok(());
                // }
                return bool_to_vec(self.as_server != 0 && self.mechanism == ZMQ_CURVE);
            }

            ZMQ_CURVE_PUBLICKEY => {
                // return do_getsockopt_curve_key(opt_val, opt_val_len, self.curve_public_key);
                return Ok(self.curve_public_key.to_vec());
            }

            ZMQ_CURVE_SECRETKEY => {
                // return do_getsockopt_curve_key(opt_val, opt_val_len, self.curve_secret_key);
                return Ok(self.curve_secret_key.to_vec());
            }

            ZMQ_CURVE_SERVERKEY => {
                // return do_getsockopt_curve_key(opt_val, opt_val_len, self.curve_server_key);
                return Ok(self.curve_server_key.to_vec());
            }
            // #endif
            ZMQ_CONFLATE => {
                // if (is_int) {
                //     // *value = self.conflate;
                //     let x = self.conflate;
                //     value[0] = x.into();
                //     return Ok(());
                // }
                return bool_to_vec(self.conflate);
            }

            //  If libgssapi isn't installed, these options provoke EINVAL
            // #ifdef HAVE_LIBGSSAPI_KRB5
            ZMQ_GSSAPI_SERVER => {
                // if (is_int) {
                //     // *value = self.as_server && self.mechanism == ZMQ_GSSAPI;
                //     let x = self.as_server != 0 && self.mechanism == ZMQ_GSSAPI;
                //     value[0] = x.into();
                //     return Ok(());
                // }
                return bool_to_vec(self.as_server != 0 && self.mechanism == ZMQ_GSSAPI);
            }

            ZMQ_GSSAPI_PRINCIPAL => {
                // return do_getsockopt(opt_val, opt_val_len, &self.gss_principal);
                return str_to_vec(&self.gss_principal);
            }

            ZMQ_GSSAPI_SERVICE_PRINCIPAL => {
                // return do_getsockopt(opt_val, opt_val_len, &self.gss_service_principal);
                return str_to_vec(&self.gss_service_principal);
            }
            ZMQ_GSSAPI_PLAINTEXT => {
                // if (is_int) {
                //     // *value = self.gss_plaintext;
                //     value[0] = self.gss_plaintext.into();
                //     return Ok(());
                // }
                return bool_to_vec(self.gss_plaintext);
            }

            ZMQ_GSSAPI_PRINCIPAL_NAMETYPE => {
                // if (is_int) {
                //     // *value = self.gss_principal_nt;
                //     value.clone_from_slice(self.gss_principal_nt.to_le_bytes().as_slice());
                //     return Ok(());
                // }
                return i32_to_vec(self.gss_principal_nt);
            }

            ZMQ_GSSAPI_SERVICE_PRINCIPAL_NAMETYPE => {
                // if (is_int) {
                //     // *value = self.gss_service_principal_nt;
                //     value.clone_from_slice(self.gss_service_principal_nt.to_le_bytes().as_slice());
                //     return Ok(());
                // }
                return i32_to_vec(self.gss_service_principal_nt);
            }

            // #endif
            ZMQ_HANDSHAKE_IVL => {
                // if (is_int) {
                //     // *value = self.handshake_ivl;
                //     value.clone_from_slice(self.handshake_ivl.to_le_bytes().as_slice());
                //     return Ok(());
                // }
                return i32_to_vec(self.handshake_ivl);
            }

            ZMQ_INVERT_MATCHING => {
                // if (is_int) {
                //     // *value = self.invert_matching;
                //     value[0] = self.invert_matching.into();
                //     return Ok(());
                // }
                return bool_to_vec(self.invert_matching);
            }

            ZMQ_HEARTBEAT_IVL => {
                // if (is_int) {
                //     // *value = self.heartbeat_interval;
                //     value.clone_from_slice(self.heartbeat_interval.to_le_bytes().as_slice());
                //     return Ok(());
                // }
                return i32_to_vec(self.heartbeat_interval);
            }

            ZMQ_HEARTBEAT_TTL => {
                // if (is_int) {
                //     // Convert the internal deciseconds value to milliseconds
                //     // *value = self.heartbeat_ttl * 100;
                //     let x = self.heartbeat_ttl * 100;
                //     value.clone_from_slice(x.to_le_bytes().as_slice());
                //     return Ok(());
                // }
                return i32_to_vec((self.heartbeat_ttl * 100) as i32);
            }

            ZMQ_HEARTBEAT_TIMEOUT => {
                // if (is_int) {
                //     // *value = self.heartbeat_timeout;
                //     value.clone_from_slice(self.heartbeat_timeout.to_le_bytes().as_slice());
                //     return Ok(());
                // }
                return i32_to_vec(self.heartbeat_timeout);
            }

            ZMQ_USE_FD => {
                // if (is_int) {
                //     // *value = self.use_fd;
                //     value.clone_from_slice(self.use_fd.to_le_bytes().as_slice());
                //     return Ok(());
                // }
                return i32_to_vec(self.use_fd);
            }

            ZMQ_BINDTODEVICE => {
                // return do_getsockopt(opt_val, opt_val_len, &self.bound_device);
                return str_to_vec(&self.bound_device);
            }

            ZMQ_ZAP_ENFORCE_DOMAIN => {
                // if (is_int) {
                //     // *value = self.zap_enforce_domain;
                //     value[0] = self.zap_enforce_domain.into();
                //     return Ok(());
                // }
                return bool_to_vec(self.zap_enforce_domain);
            }

            ZMQ_LOOPBACK_FASTPATH => {
                // if (is_int) {
                //     // *value = self.loopback_fastpath;
                //     value[0] = self.loopback_fastpath.into();
                //     return Ok(());
                // }
                return bool_to_vec(self.loopback_fastpath);
            }

            ZMQ_MULTICAST_LOOP => {
                // if (is_int) {
                //     // *value = self.multicast_loop;
                //     value[0] = self.multicast_loop.into();
                //     return Ok(());
                // }
                return bool_to_vec(self.multicast_loop);
            }

            // #ifdef ZMQ_BUILD_DRAFT_API
            ZMQ_ROUTER_NOTIFY => {
                // if (is_int) {
                //     // *value = self.router_notify;
                //     value.clone_from_slice(self.router_notify.to_le_bytes().as_slice());
                //     return Ok(());
                // }
                return i32_to_vec(self.router_notify);
            }

            ZMQ_IN_BATCH_SIZE => {
                // if (is_int) {
                //     // *value = self.in_batch_size;
                //     value.clone_from_slice(self.in_batch_size.to_le_bytes().as_slice());
                //     return Ok(());
                // }
                return i32_to_vec(self.in_batch_size);
            }

            ZMQ_OUT_BATCH_SIZE => {
                // if (is_int) {
                //     // *value = self.out_batch_size;
                //     value.clone_from_slice(self.out_batch_size.to_le_bytes().as_slice());
                //     return Ok(());
                // }
                return i32_to_vec(self.out_batch_size);
            }

            ZMQ_PRIORITY => {
                // if (is_int) {
                //     // *value = self.priority;
                //     value.clone_from_slice(self.priority.to_le_bytes().as_slice());
                //     return Ok(());
                // }
                return i32_to_vec(self.priority);
            }

            ZMQ_BUSY_POLL => {
                // if (is_int) {
                //     // *value = self.busy_poll;
                //     value.clone_from_slice(self.priority.to_le_bytes().as_slice());
                // }
                return i32_to_vec(self.priority);
            }

            // #endif
            _ => {
                // #if defined(ZMQ_ACT_MILITANT)
                // malformed = false;
            } // #endif
        }
        // #if defined(ZMQ_ACT_MILITANT)
        // if self.malformed {
        //     zmq_assert(false);
        // }
        // #endif
        //         errno = EINVAL;
        //         return -1;
        Err(anyhow!("EINVAL"))
    }

    pub fn send_inproc_connected(&mut self, tid: u32, socket: &mut ZmqSocket) {
        // ZmqCommand cmd;
        let mut cmd = ZmqCommand::default();
        cmd.destination =socket.destination.clone();
        cmd.cmd_type = CommandType::InprocConnected;
        self.send_command(tid, &mut cmd);
    }

    fn send_bind(&mut self, tid: u32, destination: ZmqAddress, pipe: &mut ZmqPipe, inc_seqnum: bool) {
        if (inc_seqnum) {
            destination.inc_seqnum();
        }

        let mut cmd = ZmqCommand::default();

        cmd.destination = destination.clone();
        cmd.cmd_type = ZmqCommand::bind;
        cmd.args.bind.pipe = pipe.clone();
        self.send_command(tid, &mut cmd);
    }

    pub fn send_stop(&mut self, destination: ZmqAddress, tid: i32) {
        //  'Stop' command goes always from administrative thread to
        //  the current object.
        let mut cmd = ZmqCommand::default();
        cmd.destination = destination;
        cmd.cmd_type = ZmqCommand::stop;
        self.get_ctx().send_command(tid, &mut cmd);
    }

    fn send_plug(&mut self, tid: u32, destination: ZmqAddress, inc_seqnum: bool) {
        // TODO
        // if (self.inc_seqnum_) {
        //     destination.inc_seqnum();
        // }

        let mut cmd = ZmqCommand::default();
        cmd.destination = destination;
        cmd.cmd_type = CommandType::Plug;
        self.send_command(tid, &mut cmd);
    }

    fn send_own(&mut self, tid: u32, destination: ZmqAddress, object: &mut ZmqOwn) {
        destination.inc_seqnum();
        let mut cmd = ZmqCommand::default();
        cmd.destination = destination;
        cmd.cmd_type = ZmqCommand::own;
        cmd.args.own.object = object;
        self.send_command(tid, &mut cmd);
    }

    fn send_attach(
        &mut self,
        tid: u32,
        destination: ZmqAddress,
        engine: &mut ZmqEngine,
        inc_seqnum: bool,
    ) {
        if (inc_seqnum) {
            destination.inc_seqnum();
        }

        let mut cmd = ZmqCommand::default();
        cmd.destination = destination;
        cmd.cmd_type = ZmqCommand::attach;
        cmd.args.attach.engine = engine;
        self.send_command(tid, &mut cmd);
    }

    fn send_activate_read(&mut self, tid: u32, destination: ZmqAddress) {
        let mut cmd = ZmqCommand::default();
        cmd.destination = destination;
        cmd.cmd_type = CommandType::ActivateRead;
        self.send_command(tid, &mut cmd);
    }

    fn send_activate_write(&mut self, tid: u32, destination: ZmqAddress, msgs_read: u64) {
        let mut cmd = ZmqCommand::default();
        cmd.destination = destination;
        cmd.cmd_type = CommandType::ActivateWrite;
        cmd.args.activate_write.msgs_read = msgs_read;
        self.send_command(tid, &mut cmd);
    }

    fn send_hiccup(&mut self, tid: u32, destination: ZmqAddress, pipe: &mut [u8]) {
        let mut cmd = ZmqCommand::default();
        cmd.destination = destination;
        cmd.cmd_type = CommandType::Hiccup;
        cmd.args.hiccup.pipe = pipe;
        self.send_command(tid, &mut cmd);
    }

    fn send_pipe_peer_stats(
        &mut self,
        tid: u32,
        destination: ZmqAddress,
        queue_count: u64,
        socket_base: &mut ZmqOwn,
        endpoint_pair: &mut EndpointUriPair,
    ) {
        let mut cmd = ZmqCommand::default();
        cmd.destination = destination;
        cmd.cmd_type = CommandType::PipePeerStats;
        cmd.args.pipe_peer_stats.queue_count = queue_count;
        cmd.args.pipe_peer_stats.socket_base = socket_base;
        cmd.args.pipe_peer_stats.endpoint_pair = endpoint_pair;
        self.send_command(tid, &mut cmd);
    }

    fn send_pipe_stats_publish(
        &mut self,
        tid: u32,
        destination: ZmqAddress,
        outbound_queue_count: u64,
        inbound_queue_count: u64,
        endpoint_pair: &mut EndpointUriPair,
    ) {
        let mut cmd = ZmqCommand::default();
        cmd.destination = destination;
        cmd.cmd_type = CommandType::PipeStatsPublish;
        cmd.args.pipe_stats_publish.outbound_queue_count = outbound_queue_count;
        cmd.args.pipe_stats_publish.inbound_queue_count = inbound_queue_count;
        cmd.args.pipe_stats_publish.endpoint_pair = endpoint_pair;
        self.send_command(tid, &mut cmd);
    }

    fn send_pipe_term(&mut self, tid: u32, destination: ZmqAddress) {
        let mut cmd = ZmqCommand::default();
        cmd.destination = destination;
        cmd.cmd_type = CommandType::PipeTerm;
        self.send_command(tid, &mut cmd);
    }

    fn send_pipe_term_ack(&mut self, tid: u32, destination: ZmqAddress) {
        let mut cmd = ZmqCommand::default();
        cmd.destination = destination;
        cmd.cmd_type = CommandType::PipeTermAck;
        self.send_command(tid, &mut cmd);
    }

    fn send_pipe_hwm(&mut self, tid: u32, destination: ZmqAddress, inhwm: i32, outhwm: i32) {
        let mut cmd = ZmqCommand::default();
        cmd.destination = destination;
        cmd.cmd_type = CommandType::PipeHwm;
        cmd.args.pipe_hwm.inhwm = inhwm;
        cmd.args.pipe_hwm.outhwm = outhwm;
        self.send_command(tid, &mut cmd);
    }

    fn send_term_req(&mut self, tid: u32, destination: ZmqAddress, object: &mut ZmqOwn) {
        let mut cmd = ZmqCommand::default();
        cmd.destination = destination;
        cmd.cmd_type = CommandType::TermReq;
        cmd.args.term_req.object = object;
        self.send_command(tid, &mut cmd);
    }

    fn send_term(&mut self, tid: u32, destination: ZmqAddress, linger: i32) {
        let mut cmd = ZmqCommand::default();
        cmd.destination = destination;
        cmd.cmd_type = CommandType::Term;
        cmd.args.term.linger = linger;
        self.send_command(tid, &mut cmd);
    }

    fn send_term_ack(&mut self, tid: u32, destination: ZmqAddress) {
        let mut cmd = ZmqCommand::default();
        cmd.destination = destination;
        cmd.cmd_type = CommandType::TermAck;
        self.send_command(tid, &mut cmd);
    }

    fn send_term_endpoint(&mut self, tid: u32, destination: ZmqAddress, endpoint: &str) {
        let mut cmd = ZmqCommand::default();
        cmd.destination = destination;
        cmd.cmd_type = CommandType::TermEndpoint;
        cmd.args.term_endpoint.endpoint = endpoint.into_string();
        self.send_command(tid, &mut cmd);
    }

    fn send_reap(&mut self, tid: u32, socket: &mut ZmqSocket) {
        let mut cmd = ZmqCommand::default();
        cmd.destination = self.get_ctx().get_reaper().unwrap();
        cmd.cmd_type = CommandType::Reap;
        cmd.args.reap.socket = socket;
        self.send_command(tid, &mut cmd);
    }

    fn send_reaped(&mut self, tid: u32, ) {
        let mut cmd = ZmqCommand::default();
        cmd.destination = self.ctx.get_reaper().unwrap();
        cmd.cmd_type = CommandType::Reaped;
        self.send_command(tid, &mut cmd);
    }

    fn send_done(&mut self) {
        let mut cmd = ZmqCommand::default();
        cmd.destination = ZmqAddress::default();
        cmd.cmd_type = CommandType::Done;
        self.ctx.send_command(ZmqContext::TERM_TID, cmd);
    }

    fn send_conn_failed(&mut self, tid: u32, destination: &mut ZmqSessionBase) {
        let mut cmd = ZmqCommand::default();
        cmd.destination = ZmqAddress::default();
        cmd.cmd_type = CommandType::ConnFailed;
        self.send_command(tid, &mut cmd);
    }

} // impl ZmqContext

pub fn clipped_maxsocket(mut max_requested: i32) -> i32 {
    // TODO
    // if max_requested >= max_fds() && max_fds() != -1 {
    //     // -1 because we need room for the reaper mailbox.
    //     max_requested = max_fds() - 1;
    // }
    //
    // return max_requested;
    todo!("clipped_maxsocket")
}

pub fn get_effective_conflate_option(ctx: &ZmqContext) -> bool {
    // conflate is only effective for some socket types
    return ctx.conflate
        && (ctx.type_ == ZMQ_DEALER
        || ctx.type_ == ZMQ_PULL
        || ctx.type_ == ZMQ_PUSH
        || ctx.type_ == ZMQ_PUB
        || ctx.type_ == ZMQ_SUB);
}

pub fn sockopt_invalid() -> i32 {
    // #if defined(ZMQ_ACT_MILITANT)
    //     zmq_assert (false);
    // #endif
    // errno = EINVAL;
    return -1;
}

pub fn set_opt_u64(in_opt_bytes: &[u8], out_val: &mut u64) -> anyhow::Result<()> {
    let in_opt_int: u64 = u64::from_le_bytes([
        in_opt_bytes[0],
        in_opt_bytes[1],
        in_opt_bytes[2],
        in_opt_bytes[3],
        in_opt_bytes[4],
        in_opt_bytes[5],
        in_opt_bytes[6],
        in_opt_bytes[7],
    ]);
    *out_val = in_opt_int;
    Ok(())
}

pub fn set_opt_i64(in_opt_bytes: &[u8], out_val: &mut i64) -> anyhow::Result<()> {
    let in_opt_int: i64 = i64::from_le_bytes([
        in_opt_bytes[0],
        in_opt_bytes[1],
        in_opt_bytes[2],
        in_opt_bytes[3],
        in_opt_bytes[4],
        in_opt_bytes[5],
        in_opt_bytes[6],
        in_opt_bytes[7],
    ]);
    *out_val = in_opt_int;
    Ok(())
}

pub fn i32_to_vec(in_i32: i32) -> anyhow::Result<Vec<u8>> {
    Ok(in_i32.to_le_bytes().to_vec())
}

pub fn bool_to_vec(in_val: bool) -> anyhow::Result<Vec<u8>> {
    let mut out_int = if in_val { 1 } else { 0 };
    Ok(out_int.to_le_bytes().to_vec())
}

pub fn str_to_vec(in_val: &str) -> anyhow::Result<Vec<u8>> {
    Ok(in_val.into_string().into_bytes())
}

pub fn set_opt_bool(in_opt_bytes: &[u8], out_val: &mut bool) -> anyhow::Result<()> {
    if in_opt_bytes.len() != 4 {
        return Err(anyhow!(
            "looking for a byte sequence of 4 bytes not {}",
            in_opt_bytes.len()
        ));
    }
    let in_opt_i32 = u32::from_le_bytes([
        in_opt_bytes[0],
        in_opt_bytes[1],
        in_opt_bytes[2],
        in_opt_bytes[3],
    ]);
    if in_opt_i32 == 0 {
        *out_val = false;
        Ok(())
    } else if in_opt_i32 >= 0 {
        *out_val = true;
        Ok(())
    }
}

pub fn set_opt_string(in_opt_bytes: &[u8], out_val: &mut String) -> anyhow::Result<()> {
    let out_str = String::from_utf8_lossy(in_opt_bytes).to_string();
    *out_val = out_str;
    Ok(())
}
