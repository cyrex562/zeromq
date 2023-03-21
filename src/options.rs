use crate::tcp_address::TcpAddressMask;
use crate::utils::copy_bytes;
use crate::zmq_hdr::{
    ZMQ_AFFINITY, ZMQ_BACKLOG, ZMQ_BINDTODEVICE, ZMQ_BUSY_POLL, ZMQ_CONFLATE, ZMQ_CONNECT_TIMEOUT,
    ZMQ_CURVE, ZMQ_CURVE_PUBLICKEY, ZMQ_CURVE_SECRETKEY, ZMQ_CURVE_SERVER, ZMQ_CURVE_SERVERKEY,
    ZMQ_DEALER, ZMQ_DISCONNECT_MSG, ZMQ_GSSAPI, ZMQ_GSSAPI_NT_HOSTBASED,
    ZMQ_GSSAPI_NT_KRB5_PRINCIPAL, ZMQ_GSSAPI_NT_USER_NAME, ZMQ_GSSAPI_PLAINTEXT,
    ZMQ_GSSAPI_PRINCIPAL, ZMQ_GSSAPI_PRINCIPAL_NAMETYPE, ZMQ_GSSAPI_SERVER,
    ZMQ_GSSAPI_SERVICE_PRINCIPAL, ZMQ_GSSAPI_SERVICE_PRINCIPAL_NAMETYPE, ZMQ_HANDSHAKE_IVL,
    ZMQ_HEARTBEAT_IVL, ZMQ_HEARTBEAT_TIMEOUT, ZMQ_HEARTBEAT_TTL, ZMQ_HELLO_MSG, ZMQ_HICCUP_MSG,
    ZMQ_IMMEDIATE, ZMQ_INVERT_MATCHING, ZMQ_IN_BATCH_SIZE, ZMQ_IPC_FILTER_GID, ZMQ_IPC_FILTER_PID,
    ZMQ_IPC_FILTER_UID, ZMQ_IPV4ONLY, ZMQ_IPV6, ZMQ_LINGER, ZMQ_LOOPBACK_FASTPATH, ZMQ_MAXMSGSIZE,
    ZMQ_MECHANISM, ZMQ_METADATA, ZMQ_MULTICAST_HOPS, ZMQ_MULTICAST_LOOP, ZMQ_MULTICAST_MAXTPDU,
    ZMQ_NULL, ZMQ_OUT_BATCH_SIZE, ZMQ_PLAIN, ZMQ_PLAIN_PASSWORD, ZMQ_PLAIN_SERVER,
    ZMQ_PLAIN_USERNAME, ZMQ_PRIORITY, ZMQ_PUB, ZMQ_PULL, ZMQ_PUSH, ZMQ_RATE, ZMQ_RCVBUF,
    ZMQ_RCVHWM, ZMQ_RCVTIMEO, ZMQ_RECONNECT_IVL, ZMQ_RECONNECT_IVL_MAX, ZMQ_RECONNECT_STOP,
    ZMQ_RECOVERY_IVL, ZMQ_ROUTER_NOTIFY, ZMQ_ROUTING_ID, ZMQ_SNDBUF, ZMQ_SNDHWM, ZMQ_SNDTIMEO,
    ZMQ_SOCKS_PASSWORD, ZMQ_SOCKS_PROXY, ZMQ_SOCKS_USERNAME, ZMQ_SUB, ZMQ_TCP_ACCEPT_FILTER,
    ZMQ_TCP_KEEPALIVE, ZMQ_TCP_KEEPALIVE_CNT, ZMQ_TCP_KEEPALIVE_IDLE, ZMQ_TCP_KEEPALIVE_INTVL,
    ZMQ_TCP_MAXRT, ZMQ_TOS, ZMQ_TYPE, ZMQ_USE_FD, ZMQ_VMCI_BUFFER_MAX_SIZE,
    ZMQ_VMCI_BUFFER_MIN_SIZE, ZMQ_VMCI_BUFFER_SIZE, ZMQ_VMCI_CONNECT_TIMEOUT, ZMQ_WSS_CERT_PEM,
    ZMQ_WSS_HOSTNAME, ZMQ_WSS_KEY_PEM, ZMQ_WSS_TRUST_PEM, ZMQ_WSS_TRUST_SYSTEM, ZMQ_ZAP_DOMAIN,
    ZMQ_ZAP_ENFORCE_DOMAIN,
};
use crate::zmq_utils::zmq_z85_encode;
use anyhow::{anyhow, bail};
use libc::{c_void, gid_t, memcpy, pid_t, uid_t, EINVAL};
use std::collections::{HashMap, HashSet};
use std::ffi::{CStr, CString};
use std::mem;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Default, Debug, Clone)]
pub struct ZmqOptions {
    //  High-water marks for message pipes.
    pub sndhwm: i32,
    pub rcvhwm: i32,

    //  I/O thread affinity.
    pub affinity: u64,

    //  Socket routing id.
    // unsigned char routing_id_size,
    pub routing_id_size: usize,
    pub routing_id: [u8; 256],

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

    //  If true, IPv6 is enabled (as well as IPv4)
    pub ipv6: bool,

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
    // typedef std::vector<TcpAddressMask> tcp_accept_filters_t,
    // tcp_accept_filters_t tcp_accept_filters,
    pub tcp_accept_filters: Vec<TcpAddressMask>,

    // IPC accept() filters
    // #if defined ZMQ_HAVE_SO_PEERCRED || defined ZMQ_HAVE_LOCAL_PEERCRED
    //     typedef std::set<uid_t> ipc_uid_accept_filters_t,
    //     ipc_uid_accept_filters_t ipc_uid_accept_filters,
    #[cfg(target_os = "linux")]
    pub ipc_uid_accept_filters: HashSet<uid_t>,

    // typedef std::set<gid_t> ipc_gid_accept_filters_t,
    // ipc_gid_accept_filters_t ipc_gid_accept_filters,
    #[cfg(target_os = "linux")]
    pub ipc_gid_accept_filters: HashSet<gid_t>,
    // #endif
    // #if defined ZMQ_HAVE_SO_PEERCRED
    //     typedef std::set<pid_t> ipc_pid_accept_filters_t,
    //     ipc_pid_accept_filters_t ipc_pid_accept_filters,
    #[cfg(target_os = "linux")]
    pub ipc_pid_accept_filters: HashSet<pid_t>,
    // #endif

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
    pub curve_public_key: [u8; CURVE_KEYSIZE],
    pub curve_secret_key: [u8; CURVE_KEYSIZE],
    pub curve_server_key: [u8; CURVE_KEYSIZE],

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

    // #if defined ZMQ_HAVE_VMCI
    pub vmci_buffer_size: u64,
    pub vmci_buffer_min_size: u64,
    pub vmci_buffer_max_size: u64,
    pub vmci_connect_timeout: i32,
    // #endif

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

impl ZmqOptions {
    // ZmqOptions ();
    pub fn new() -> Self {
        Self {
            sndhwm: DEFAULT_HWM as i32,
            rcvhwm: DEFAULT_HWM as i32,
            affinity: 0,
            routing_id_size: 0,
            routing_id: [0; 256],
            rate: 0,
            recovery_ivl: 0,
            multicast_hops: 0,
            multicast_maxtpdu: 0,
            sndbuf: -1,
            rcvbuf: -1,
            tos: 0,
            priority: 0,
            type_: -1,
            linger: AtomicU64::from(0),
            connect_timeout: 0,
            tcp_maxrt: 0,
            reconnect_stop: 0,
            reconnect_ivl: 100,
            reconnect_ivl_max: 0,
            backlog: 100,
            maxmsgsize: -1,
            rcvtimeo: -1,
            sndtimeo: -1,
            ipv6: false,
            immediate: 0,
            filter: false,
            invert_matching: false,
            recv_routing_id: false,
            raw_socket: false,
            raw_notify: true,
            socks_proxy_address: "".to_string(),
            socks_proxy_username: "".to_string(),
            socks_proxy_password: "".to_string(),
            tcp_keepalive: -1,
            tcp_keepalive_cnt: -1,
            tcp_keepalive_idle: -1,
            tcp_keepalive_intvl: -1,
            tcp_accept_filters: vec![],
            #[cfg(target_os = "linux")]
            ipc_uid_accept_filters: Default::default(),
            #[cfg(target_os = "linux")]
            ipc_gid_accept_filters: Default::default(),
            #[cfg(target_os = "linux")]
            ipc_pid_accept_filters: Default::default(),
            mechanism: ZMQ_NULL as i32,
            as_server: 0,
            zap_domain: "".to_string(),
            plain_username: "".to_string(),
            plain_password: "".to_string(),
            curve_public_key: [0; 128],
            curve_secret_key: [0; 128],
            curve_server_key: [0; 128],
            gss_principal: "".to_string(),
            gss_service_principal: "".to_string(),
            gss_principal_nt: ZMQ_GSSAPI_NT_HOSTBASED as i32,
            gss_service_principal_nt: ZMQ_GSSAPI_NT_HOSTBASED as i32,
            gss_plaintext: false,
            socket_id: 0,
            conflate: false,
            handshake_ivl: 30000,
            connected: false,
            heartbeat_ttl: 0,
            heartbeat_interval: 0,
            heartbeat_timeout: -1,
            vmci_buffer_size: 0,
            vmci_buffer_min_size: 0,
            vmci_buffer_max_size: 0,
            vmci_connect_timeout: 0,
            use_fd: -1,
            bound_device: "".to_string(),
            zap_enforce_domain: false,
            loopback_fastpath: false,
            multicast_loop: true,
            in_batch_size: 8192,
            out_batch_size: 8192,
            zero_copy: true,
            router_notify: 0,
            app_metadata: Default::default(),
            monitor_event_version: 1,
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
            busy_poll: 0,
        }
    }

    // int set_curve_key (uint8_t *destination_,
    //                    const opt_val: *mut c_void,
    //                    opt_val_len: usize);
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
                    if zmq_z85_decode(destination, z85_key) {
                        self.mechanism = ZMQ_CURVE as i32;
                        return 0;
                    }
                }
                _ => {}
            }
        }
        return -1;
    }

    // int setsockopt (option_: i32, const opt_val: *mut c_void, opt_val_len: usize);

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
                if opt_val_len > 0 && opt_val_len <= UCHAR_MAX {
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
                    rcvbuf = value;
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
                        let mut mask = TcpAddressMask::default();
                        rc = mask.resolve(&filter_str, ipv6);
                        if rc == 0 {
                            self.tcp_accept_filters.push_back(mask);
                        }
                    }
                }
                return rc;
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
                } else if opt_val_len > 0 && opt_val_len <= UCHAR_MAX && opt_val.is_null() == false
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
                    && opt_val_len <= UCHAR_MAX
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
                if opt_val_len > 0 && opt_val_len <= UCHAR_MAX && opt_val != null_mut() {
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
                if opt_val_len > 0 && opt_val_len <= UCHAR_MAX && opt_val.is_empty() == false {
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
                if is_int && value >= 0 && value <= UINT16_MAX {
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
                return set_ops_u64(opt_val, &mut vmci_connect_timeout);
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
                            if key[0..2] == "X-" && key.len() <= UCHAR_MAX {
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
                // wss_key_pem = std::string((char *) opt_val, opt_val_len);
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
                    priority = value;
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

            // default:
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
            zmq_assert(false);
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
                let mut opt_val: Vec<u8> = Vec::with_capacity(self.routing_id_size);
                do_getsockopt2(&mut opt_val, &self.routing_id);
                return Ok(opt_val);
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
                if (is_int) {
                    // *value = self.connect_timeout;
                    // value.clone_from_slice(self.connect_timeout.to_le_bytes().as_slice());
                    return Ok(self.connect_timeout.to_le_bytes().to_vec());
                }
            }

            ZMQ_TCP_MAXRT => {
                if (is_int) {
                    // *value = self.tcp_maxrt;
                    // value.clone_from_slice(self.tcp_maxrt.to_le_bytes().as_slice());
                    return Ok(self.tcp_maxrt.to_le_bytes().to_vec());
                }
            }

            ZMQ_RECONNECT_STOP => {
                if (is_int) {
                    // *value = self.reconnect_stop;
                    //value.clone_from_slice(self.reconnect_stop.to_le_bytes().as_slice());
                    return Ok(self.reconnect_stop.to_le_bytes().to_vec());
                }
            }

            ZMQ_RECONNECT_IVL => {
                if (is_int) {
                    // *value = self.reconnect_ivl;
                    // value.clone_from_slice(self.reconnect_ivl.to_le_bytes().as_slice());
                    return Ok(self.reconnect_ivl.to_le_bytes().to_vec());
                }
            }

            ZMQ_RECONNECT_IVL_MAX => {
                if (is_int) {
                    // *value = self.reconnect_ivl_max;
                    // value.clone_from_slice(self.reconnect_ivl_max.to_le_bytes().as_slice());
                    return Ok(self.reconnect_ivl_max.to_le_bytes().to_vec());
                }
            }

            ZMQ_BACKLOG => {
                if (is_int) {
                    // *value = self.backlog;
                    // value.clone_from_slice(self.backlog.to_le_bytes().as_slice());
                    return Ok(self.backlog.to_le_bytes().to_vec());
                }
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
} // impl ZmqOptions

pub const CURVE_KEYSIZE: usize = 128;
pub const CURVE_KEYSIZE_Z85: usize = 256;
pub const CURVE_KEYSIZE_Z85_P1: usize = CURVE_KEYSIZE_Z85 + 1;

pub fn get_effective_conflate_option(options: &ZmqOptions) -> bool {
    // conflate is only effective for some socket types
    return options.conflate
        && (options.type_ == ZMQ_DEALER
            || options.type_ == ZMQ_PULL
            || options.type_ == ZMQ_PUSH
            || options.type_ == ZMQ_PUB
            || options.type_ == ZMQ_SUB);
}

// int do_getsockopt (opt_val: *mut c_void,
//                    size_t *opt_val_len,
//                    const value_: *mut c_void,
//                    value_len_: usize);

// template <typename T>
// int do_getsockopt (opt_val: *const c_void, size_t *const opt_val_len, T value_)
// {
// #if __cplusplus >= 201103L && (!defined(__GNUC__) || __GNUC__ > 5)
//     static_assert (std::is_trivially_copyable<T>::value,
//                    "invalid use of do_getsockopt");
// // #endif
//     return do_getsockopt (opt_val, opt_val_len, &value_, mem::size_of::<T>());
// }

// int do_getsockopt (opt_val: *mut c_void,
//                    size_t *opt_val_len,
//                    const std::string &value_);
//
// int do_setsockopt_int_as_bool_strict (const opt_val: *mut c_void,
//                                       opt_val_len: usize,
//                                       bool *out_value_);
//
// int do_setsockopt_int_as_bool_relaxed (const opt_val: *mut c_void,
//                                        opt_val_len: usize,
//                                        bool *out_value_);
// }

pub fn sockopt_invalid() -> i32 {
    // #if defined(ZMQ_ACT_MILITANT)
    //     zmq_assert (false);
    // #endif
    errno = EINVAL;
    return -1;
}

// pub fn do_getsockopt(
//     opt_val: &mut [u8],
//     opt_val_len: &mut usize,
//     val_in: &str,
// ) -> anyhow::Result<()> {
//     // let mut str_ptr = CString::from(val_in);
//     return do_getsockopt2(opt_val, opt_val_len, val_in.as_bytes(), val_in.len());
// }

// pub fn do_getsockopt2(opt_val: &mut [u8], val_in: &[u8]) -> anyhow::Result<()> {
//     // TODO behaviour is inconsistent with ZmqOptions::getsockopt; there, an
//     // *exact* length match is required except for string-like (but not the
//     // CURVE keys!) (and therefore null-ing remaining memory is a no-op, see
//     // comment below)
//     if *opt_val.len() < val_in.len() {
//         // return sockopt_invalid();
//         return Err(anyhow!("sockopt invalid"));
//     }
//     // unsafe { libc::memcpy(opt_val, value_, value_len_); }
//     // opt_val.clone_from_slice(&val_in[0..val_in_len]);
//     // TODO why is the remaining memory null-ed?
//     // libc::memset (static_cast<char *> (opt_val) + value_len_, 0,
//     //         *opt_val_len - value_len_);
//     // *opt_val_len = val_in_len;
//     copy_bytes(opt_val, 0, val_in, 0, val_in.len());
//     Ok(())
// }

// #ifdef ZMQ_HAVE_CURVE
// pub fn do_getsockopt_curve_key(
//     opt_val: &mut [u8],
//     opt_val_len: &mut usize,
//     curve_key_: [u8; CURVE_KEYSIZE],
// ) -> anyhow::Result<()> {
//     if *opt_val_len == CURVE_KEYSIZE {
//         // unsafe { libc::memcpy(opt_val, curve_key_.as_ptr() as *const c_void, CURVE_KEYSIZE); }
//         opt_val.clone_from_slice(&curve_key_[0..CURVE_KEYSIZE]);
//         Ok(())
//     }
//     if *opt_val_len == CURVE_KEYSIZE_Z85 + 1 {
//         zmq_z85_encode(opt_val, &curve_key_, CURVE_KEYSIZE);
//         Ok(())
//     }
//     bail!("sockopt invalid")
// }
// #endif

// template <typename T>
// static int do_setsockopt (const opt_val: *const c_void,
//                           const opt_val_len: usize,
//                           T *const out_value_)
// pub fn do_setsockopt<T>(opt_val: &mut [u8], opt_val_len: usize, out_value: &mut T) -> i32 {
//     if opt_val_len == mem::size_of::<T>() {
//         // unsafe { memcpy(out_value as *mut c_void, optval, mem::size_of::<T>()); }
//         *out_value = T::from(opt_val);
//         return 0;
//     }
//     return sockopt_invalid();
// }

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

#[cfg(target_os = "linux")]
pub fn set_opt_uid_hash_set(
    in_opt_bytes: &[u8],
    out_val: &mut HashSet<libc::uid_t>,
) -> anyhow::Result<()> {
    unimplemented!()
}

#[cfg(target_os = "linux")]
pub fn set_opt_gid_hash_set(
    in_opt_bytes: &[u8],
    out_val: &mut HashSet<libc::gid_t>,
) -> anyhow::Result<()> {
    unimplemented!()
}

#[cfg(target_os = "linux")]
pub fn set_opt_pid_hash_set(
    in_opt_bytes: &[u8],
    out_val: &mut HashSet<libc::pid_t>,
) -> anyhow::Result<()> {
    unimplemented!()
}

// pub fn do_setsockopt_int_as_bool_strict(opt_val: &[u8],
//                                         out_value: &mut bool) -> anyhow::Result<()> {
//     // TODO handling of values other than 0 or 1 is not consistent,
//     // here it is disallowed, but for other options such as
//     // ZMQ_ROUTER_RAW any positive value is accepted
//     // let mut value = false;
//     // if do_setsockopt(opt_val, opt_val_len, &mut value) == -1 {
//     //     return -1;
//     // }
//     set_opt_bool(opt_val, out_value)
//     // if value == 0 || value == 1 {
//     //     *out_value = (value != 0);
//     //     return 0;
//     // }
//     // return sockopt_invalid();
// }

// pub fn do_setsockopt_int_as_bool_relaxed(opt_val: &mut [u8],
//                                          opt_val_len: usize,
//                                          out_value_: &mut bool) -> anyhow::Result<()> {
//     // let mut value = -1;
//     // if do_setsockopt(opt_val, opt_val_len, &mut value) == -1 {
//     //     return -1;
//     // }
//     // *out_value_ = (value != 0);
//     // return 0;
//     set_opt_bool(opt_val, out_value)
// }

// pub fn do_setsockopt_string_allow_empty_strict(opt_val: &mut [u8],
//                                                opt_val_len: usize,
//                                                out_value_: &mut String,
//                                                max_len_: usize) -> i32 {
//     // TODO why is opt_val != NULL not allowed in case of opt_val_len== 0?
//     // TODO why are empty strings allowed for some socket options, but not for others?
//     if opt_val.is_null() && opt_val_len == 0 {
//         out_value_.clear();
//         return 0;
//     }
//     if opt_val.is_null() == false && opt_val_len > 0 && opt_val_len <= max_len_ {
//         unsafe { *out_value_ = String::from_raw_parts(opt_value_ as *mut u8, opt_val_len, max_len_); }
//         return 0;
//     }
//     return sockopt_invalid();
// }

// pub fn do_setsockopt_string_allow_empty_relaxed(
//     opt_val: &mut [u8],
//     opt_val_len: usize,
//     out_value: &mut String,
//     max_len: usize,
// ) -> i32 {
//     // TODO use either do_setsockopt_string_allow_empty_relaxed or
//     // do_setsockopt_string_allow_empty_strict everywhere
//     if opt_val_len > 0 && opt_val_len <= max_len {
//         // out_value_->assign (static_cast<const char *> (opt_val), opt_val_len);
//         unsafe {
//             *out_value = String::from_raw_parts(opt_val as *mut u8, opt_val_len, max_len);
//         };
//         return 0;
//     }
//     return sockopt_invalid();
// }

// pub fn do_setsockopt_set<T>(opt_val: &mut [u8], opt_val_len: usize, set_: &mut HashSet<T>) -> i32 {
//     if opt_val_len == 0 && opt_val == null_mut() {
//         set_.clear();
//         return 0;
//     }
//     if (opt_val_len == mem::size_of::<T>()) && (opt_val.is_null() == false) {
//         // set_->insert (*(static_cast<const T *> (opt_val)));
//         unsafe { set_.insert(opt_val as *mut T) }
//         return 0;
//     }
//     return sockopt_invalid();
// }

// TODO why is 1000 a sensible default?
pub const DEFAULT_HWM: u32 = 1000;
//
// ZmqOptions::ZmqOptions () :
//     sndhwm (default_hwm),
//     rcvhwm (default_hwm),
//     affinity (0),
//     routing_id_size (0),
//     rate (100),
//     recovery_ivl (10000),
//     multicast_hops (1),
//     multicast_maxtpdu (1500),
//     sndbuf (-1),
//     rcvbuf (-1),
//     tos (0),
//     priority (0),
//     type (-1),
//     linger (-1),
//     connect_timeout (0),
//     tcp_maxrt (0),
//     reconnect_stop (0),
//     reconnect_ivl (100),
//     reconnect_ivl_max (0),
//     backlog (100),
//     maxmsgsize (-1),
//     rcvtimeo (-1),
//     sndtimeo (-1),
//     ipv6 (false),
//     immediate (0),
//     filter (false),
//     invert_matching (false),
//     recv_routing_id (false),
//     raw_socket (false),
//     raw_notify (true),
//     tcp_keepalive (-1),
//     tcp_keepalive_cnt (-1),
//     tcp_keepalive_idle (-1),
//     tcp_keepalive_intvl (-1),
//     mechanism (ZMQ_NULL),
//     as_server (0),
//     gss_principal_nt (ZMQ_GSSAPI_NT_HOSTBASED),
//     gss_service_principal_nt (ZMQ_GSSAPI_NT_HOSTBASED),
//     gss_plaintext (false),
//     socket_id (0),
//     conflate (false),
//     handshake_ivl (30000),
//     connected (false),
//     heartbeat_ttl (0),
//     heartbeat_interval (0),
//     heartbeat_timeout (-1),
//     use_fd (-1),
//     zap_enforce_domain (false),
//     loopback_fastpath (false),
//     multicast_loop (true),
//     in_batch_size (8192),
//     out_batch_size (8192),
//     zero_copy (true),
//     router_notify (0),
//     monitor_event_version (1),
//     wss_trust_system (false),
//     hello_msg (),
//     can_send_hello_msg (false),
//     disconnect_msg (),
//     can_recv_disconnect_msg (false),
//     hiccup_msg (),
//     can_recv_hiccup_msg (false),
//     busy_poll (0)
// {
//     memset (curve_public_key, 0, CURVE_KEYSIZE);
//     memset (curve_secret_key, 0, CURVE_KEYSIZE);
//     memset (curve_server_key, 0, CURVE_KEYSIZE);
// // #if defined ZMQ_HAVE_VMCI
//     vmci_buffer_size = 0;
//     vmci_buffer_min_size = 0;
//     vmci_buffer_max_size = 0;
//     vmci_connect_timeout = -1;
// // #endif
// }

pub const DECISECONDS_PER_MILLISECOND: u32 = 100;
