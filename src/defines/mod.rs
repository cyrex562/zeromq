use std::collections::HashSet;
use std::ffi::c_void;
use std::sync::Mutex;

#[cfg(not(target_os = "windows"))]
use libc::{pollfd, pid_t};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Networking::WinSock::WSAPOLLFD;

pub mod array;
pub mod atomic_counter;
pub mod atomic_ptr;
pub mod blob;
pub mod clock;
pub mod dbuffer;
pub mod fair_queue;
pub mod generic_mtrie;
pub mod load_balancer;
pub mod mtrie;
pub mod mutex;
pub mod radix_tree;
pub mod trie;
pub mod yqueue;
pub mod zmq_draft;
pub mod tcp;
pub mod err;
// #[cfg(target_os="windows")]
// use windows::Win32::Networking::WinSock::sa_family_t;

#[cfg(target_os = "windows")]
#[cfg(target_arch = "x86_64")]
pub type ZmqFd = usize;
#[cfg(target_os = "windows")]
#[cfg(target_arch = "x86")]
pub type ZmqFd = u32;
#[allow(non_camel_case_types)]
#[cfg(target_os = "linux")]
pub type ZmqFd = i32;

/*  Socket options.                                                           */
// pub const ZMQ_AFFINITY: u32 = 4;
// pub const ZMQ_ROUTING_ID: u32 = 5;
// pub const ZMQ_SUBSCRIBE: u32 = 6;
// pub const ZMQ_UNSUBSCRIBE: u32 = 7;
// pub const ZMQ_RATE: u32 = 8;
// pub const ZMQ_RECOVERY_IVL: u32 = 9;
// pub const ZMQ_SNDBUF: u32 = 11;
// pub const ZMQ_RCVBUF: u32 = 12;
// pub const ZMQ_RCVMORE: u32 = 13;
// pub const ZMQ_FD: u32 = 14;
// pub const ZMQ_EVENTS: u32 = 15;
// pub const ZMQ_TYPE: u32 = 16;
// pub const ZMQ_LINGER: u32 = 17;
// pub const ZMQ_RECONNECT_IVL: u32 = 18;
// pub const ZMQ_BACKLOG: u32 = 19;
// pub const ZMQ_RECONNECT_IVL_MAX: u32 = 21;
// pub const ZMQ_MAXMSGSIZE: u32 = 22;
// pub const ZMQ_SNDHWM: u32 = 23;
// pub const ZMQ_RCVHWM: u32 = 24;
// pub const ZMQ_MULTICAST_HOPS: u32 = 25;
// pub const ZMQ_RCVTIMEO: u32 = 27;
// pub const ZMQ_SNDTIMEO: u32 = 28;
// pub const ZMQ_LAST_ENDPOINT: u32 = 32;
// pub const ZMQ_ROUTER_MANDATORY: u32 = 33;
// pub const ZMQ_TCP_KEEPALIVE: u32 = 34;
// pub const ZMQ_TCP_KEEPALIVE_CNT: u32 = 35;
// pub const ZMQ_TCP_KEEPALIVE_IDLE: u32 = 36;
// pub const ZMQ_TCP_KEEPALIVE_INTVL: u32 = 37;
// pub const ZMQ_IMMEDIATE: u32 = 39;
// pub const ZMQ_XPUB_VERBOSE: u32 = 40;
// pub const ZMQ_ROUTER_RAW: u32 = 41;
// pub const ZMQ_IPV6: u32 = 42;
// pub const ZMQ_MECHANISM: u32 = 43;
// pub const ZMQ_PLAIN_SERVER: u32 = 44;
// pub const ZMQ_PLAIN_USERNAME: u32 = 45;
// pub const ZMQ_PLAIN_PASSWORD: u32 = 46;
// pub const ZMQ_CURVE_SERVER: u32 = 47;
// pub const ZMQ_CURVE_PUBLICKEY: u32 = 48;
// pub const ZMQ_CURVE_SECRETKEY: u32 = 49;
// pub const ZMQ_CURVE_SERVERKEY: u32 = 50;
// pub const ZMQ_PROBE_ROUTER: u32 = 51;
// pub const ZMQ_REQ_CORRELATE: u32 = 52;
// pub const ZMQ_REQ_RELAXED: u32 = 53;
// pub const ZMQ_CONFLATE: u32 = 54;
// pub const ZMQ_ZAP_DOMAIN: u32 = 55;
// pub const ZMQ_ROUTER_HANDOVER: u32 = 56;
// pub const ZMQ_TOS: u32 = 57;
// pub const ZMQ_CONNECT_ROUTING_ID: u32 = 61;
// pub const ZMQ_GSSAPI_SERVER: u32 = 62;
// pub const ZMQ_GSSAPI_PRINCIPAL: u32 = 63;
// pub const ZMQ_GSSAPI_SERVICE_PRINCIPAL: u32 = 64;
// pub const ZMQ_GSSAPI_PLAINTEXT: u32 = 65;
// pub const ZMQ_HANDSHAKE_IVL: u32 = 66;
// pub const ZMQ_SOCKS_PROXY: u32 = 68;
// pub const ZMQ_XPUB_NODROP: u32 = 69;
// pub const ZMQ_BLOCKY: u32 = 70;
// pub const ZMQ_XPUB_MANUAL: u32 = 71;
// pub const ZMQ_XPUB_WELCOME_MSG: u32 = 72;
// pub const ZMQ_STREAM_NOTIFY: u32 = 73;
// pub const ZMQ_INVERT_MATCHING: u32 = 74;
// pub const ZMQ_HEARTBEAT_IVL: u32 = 75;
// pub const ZMQ_HEARTBEAT_TTL: u32 = 76;
// pub const ZMQ_HEARTBEAT_TIMEOUT: u32 = 77;
// pub const ZMQ_XPUB_VERBOSER: u32 = 78;
// pub const ZMQ_CONNECT_TIMEOUT: u32 = 79;
// pub const ZMQ_TCP_MAXRT: u32 = 80;
// pub const ZMQ_THREAD_SAFE: u32 = 81;
// pub const ZMQ_MULTICAST_MAXTPDU: u32 = 84;
// pub const ZMQ_VMCI_BUFFER_SIZE: u32 = 85;
// pub const ZMQ_VMCI_BUFFER_MIN_SIZE: u32 = 86;
// pub const ZMQ_VMCI_BUFFER_MAX_SIZE: u32 = 87;
// pub const ZMQ_VMCI_CONNECT_TIMEOUT: u32 = 88;
// pub const ZMQ_USE_FD: u32 = 89;
// pub const ZMQ_GSSAPI_PRINCIPAL_NAMETYPE: u32 = 90;
// pub const ZMQ_GSSAPI_SERVICE_PRINCIPAL_NAMETYPE: u32 = 91;
// pub const ZMQ_BINDTODEVICE: u32 = 92;

// pub const ZMQ_PAIR: u32 = 0;
// pub const ZMQ_PUB: u32 = 1;
// pub const ZMQ_SUB: u32 = 2;
// pub const ZMQ_REQ: u32 = 3;
// pub const ZMQ_REP: u32 = 4;
// pub const ZMQ_DEALER: u32 = 5;
// pub const ZMQ_ROUTER: u32 = 6;
// pub const ZMQ_PULL: u32 = 7;
// pub const ZMQ_PUSH: u32 = 8;
// pub const ZMQ_XPUB: u32 = 9;
// pub const ZMQ_XSUB: u32 = 10;
// pub const ZMQ_STREAM: u32 = 11;

/*  Message options                                                           */
// pub const ZMQ_MORE: u32 = 1;
// pub const ZMQ_SHARED: u32 = 3;

/*  Send/recv options.                                                        */
// pub const ZMQ_DONTWAIT: u32 = 1;
// pub const ZMQ_SNDMORE: u32 = 2;

/*  Security mechanisms                                                       */
// pub const ZMQ_NULL: i32 = 0;
// pub const ZMQ_PLAIN: u32 = 1;
// pub const ZMQ_CURVE: u32 = 2;
// pub const ZMQ_GSSAPI: u32 = 3;

/*  RADIO-DISH protocol                                                       */
// pub const ZMQ_GROUP_MAX_LENGTH: usize = 255;

/*  Deprecated options and aliases                                            */
// pub const ZMQ_IDENTITY: u32 = ZMQ_ROUTING_ID;
// pub const ZMQ_CONNECT_RID: u32 = ZMQ_CONNECT_ROUTING_ID;
// pub const ZMQ_TCP_ACCEPT_FILTER: u32 = 38;
// pub const ZMQ_IPC_FILTER_PID: u32 = 58;
// pub const ZMQ_IPC_FILTER_UID: u32 = 59;
// pub const ZMQ_IPC_FILTER_GID: u32 = 60;
// pub const ZMQ_IPV4ONLY: u32 = 31;
// pub const ZMQ_DELAY_ATTACH_ON_CONNECT: u32 = ZMQ_IMMEDIATE;
// pub const ZMQ_NOBLOCK: u32 = ZMQ_DONTWAIT;
// pub const ZMQ_FAIL_UNROUTABLE: u32 = ZMQ_ROUTER_MANDATORY;
// pub const ZMQ_ROUTER_BEHAVIOR: u32 = ZMQ_ROUTER_MANDATORY;

/*  Deprecated Message options                                                */
// pub const ZMQ_SRCFD: u32 = 2;

// pub const ZMQ_GSSAPI_NT_HOSTBASED: i32 = 0;
// pub const ZMQ_GSSAPI_NT_USER_NAME: u32 = 1;
// pub const ZMQ_GSSAPI_NT_KRB5_PRINCIPAL: u32 = 2;

// pub const ZMQ_EVENT_CONNECTED: u32 = 0x0001;
// pub const ZMQ_EVENT_CONNECT_DELAYED: u32 = 0x0002;
// pub const ZMQ_EVENT_CONNECT_RETRIED: u32 = 0x0004;
// pub const ZMQ_EVENT_LISTENING: u32 = 0x0008;
// pub const ZMQ_EVENT_BIND_FAILED: u32 = 0x0010;
// pub const ZMQ_EVENT_ACCEPTED: u32 = 0x0020;
// pub const ZMQ_EVENT_ACCEPT_FAILED: u32 = 0x0040;
// pub const ZMQ_EVENT_CLOSED: u32 = 0x0080;
// pub const ZMQ_EVENT_CLOSE_FAILED: u32 = 0x0100;
// pub const ZMQ_EVENT_DISCONNECTED: u32 = 0x0200;
// pub const ZMQ_EVENT_MONITOR_STOPPED: u32 = 0x0400;
// pub const ZMQ_EVENT_ALL: u32 = 0xFFFF;
/*  Unspecified system errors during handshake. Event value is an errno.      */
// pub const ZMQ_EVENT_HANDSHAKE_FAILED_NO_DETAIL: u32 = 0x0800;
/*  Handshake complete successfully with successful authentication (if        *
 *  enabled). Event value is unused.                                          */
// pub const ZMQ_EVENT_HANDSHAKE_SUCCEEDED: u32 = 0x1000;
/*  Protocol errors between ZMTP peers or between server and ZAP handler.     *
 *  Event value is one of ZMQ_PROTOCOL_ERROR_*                                */
// pub const ZMQ_EVENT_HANDSHAKE_FAILED_PROTOCOL: u32 = 0x2000;
/*  Failed authentication requests. Event value is the numeric ZAP status     *
 *  code, i.e. 300, 400 or 500.                                               */
// pub const ZMQ_EVENT_HANDSHAKE_FAILED_AUTH: u32 = 0x4000;
// pub const ZMQ_PROTOCOL_ERROR_ZMTP_UNSPECIFIED: u32 = 0x10000000;
// pub const ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND: u32 = 0x10000001;
// pub const ZMQ_PROTOCOL_ERROR_ZMTP_INVALID_SEQUENCE: u32 = 0x10000002;
// pub const ZMQ_PROTOCOL_ERROR_ZMTP_KEY_EXCHANGE: u32 = 0x10000003;
// pub const ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_UNSPECIFIED: u32 = 0x10000011;
// pub const ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_MESSAGE: u32 = 0x10000012;
// pub const ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_HELLO: u32 = 0x10000013;
// pub const ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_INITIATE: u32 = 0x10000014;
// pub const ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_ERROR: u32 = 0x10000015;
// pub const ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_READY: u32 = 0x10000016;
// pub const ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_WELCOME: u32 = 0x10000017;
// pub const ZMQ_PROTOCOL_ERROR_ZMTP_INVALID_METADATA: u32 = 0x10000018;
// the following two may be due to erroneous configuration of a peer
// pub const ZMQ_PROTOCOL_ERROR_ZMTP_CRYPTOGRAPHIC: u32 = 0x11000001;
// pub const ZMQ_PROTOCOL_ERROR_ZMTP_MECHANISM_MISMATCH: u32 = 0x11000002;
// pub const ZMQ_PROTOCOL_ERROR_ZAP_UNSPECIFIED: u32 = 0x20000000;
// pub const ZMQ_PROTOCOL_ERROR_ZAP_MALFORMED_REPLY: u32 = 0x20000001;
// pub const ZMQ_PROTOCOL_ERROR_ZAP_BAD_REQUEST_ID: u32 = 0x20000002;
// pub const ZMQ_PROTOCOL_ERROR_ZAP_BAD_VERSION: u32 = 0x20000003;
// pub const ZMQ_PROTOCOL_ERROR_ZAP_INVALID_STATUS_CODE: u32 = 0x20000004;
// pub const ZMQ_PROTOCOL_ERROR_ZAP_INVALID_METADATA: u32 = 0x20000005;
// pub const ZMQ_PROTOCOL_ERROR_WS_UNSPECIFIED: u32 = 0x30000000;

// pub const ZMQ_POLLIN: u32 = 1;
// pub const ZMQ_POLLOUT: u32 = 2;
// pub const ZMQ_POLLERR: u32 = 4;
// pub const ZMQ_POLLPRI: u32 = 8;

// pub const ZMQ_HAS_CAPABILITIES: u32 = 1;

// pub const ZMQ_STREAMER: u32 = 1;
// pub const ZMQ_FORWARDER: u32 = 2;
// pub const ZMQ_QUEUE: u32 = 3;

/*  DRAFT Socket types.                                                       */
// pub const ZMQ_SERVER: u32 = 12;
// pub const ZMQ_CLIENT: u32 = 13;
// pub const ZMQ_RADIO: u32 = 14;
// pub const ZMQ_DISH: u32 = 15;
// pub const ZMQ_GATHER: u32 = 16;
// pub const ZMQ_SCATTER: u32 = 17;
// pub const ZMQ_DGRAM: u32 = 18;
// pub const ZMQ_PEER: u32 = 19;
// pub const ZMQ_CHANNEL: i8 = 20;

/*  DRAFT Socket options.                                                     */
// pub const ZMQ_ZAP_ENFORCE_DOMAIN: u32 = 93;
// pub const ZMQ_LOOPBACK_FASTPATH: u32 = 94;
// pub const ZMQ_METADATA: u32 = 95;
// pub const ZMQ_MULTICAST_LOOP: u32 = 96;
// pub const ZMQ_ROUTER_NOTIFY: u32 = 97;
// pub const ZMQ_XPUB_MANUAL_LAST_VALUE: u32 = 98;
// pub const ZMQ_SOCKS_USERNAME: u32 = 99;
// pub const ZMQ_SOCKS_PASSWORD: u32 = 100;
// pub const ZMQ_IN_BATCH_SIZE: u32 = 101;
// pub const ZMQ_OUT_BATCH_SIZE: u32 = 102;
// pub const ZMQ_WSS_KEY_PEM: u32 = 103;
// pub const ZMQ_WSS_CERT_PEM: u32 = 104;
// pub const ZMQ_WSS_TRUST_PEM: u32 = 105;
// pub const ZMQ_WSS_HOSTNAME: u32 = 106;
// pub const ZMQ_WSS_TRUST_SYSTEM: u32 = 107;
// pub const ZMQ_ONLY_FIRST_SUBSCRIBE: u32 = 108;
// pub const ZMQ_RECONNECT_STOP: u32 = 109;
// pub const ZMQ_HELLO_MSG: u32 = 110;
// pub const ZMQ_DISCONNECT_MSG: u32 = 111;
// pub const ZMQ_PRIORITY: u32 = 112;
// pub const ZMQ_BUSY_POLL: u32 = 113;
// pub const ZMQ_HICCUP_MSG: u32 = 114;
// pub const ZMQ_XSUB_VERBOSE_UNSUBSCRIBE: u32 = 115;
// pub const ZMQ_TOPICS_COUNT: u32 = 116;
// pub const ZMQ_NORM_MODE: u32 = 117;
// pub const ZMQ_NORM_UNICAST_NACK: u32 = 118;
// pub const ZMQ_NORM_BUFFER_SIZE: u32 = 119;
// pub const ZMQ_NORM_SEGMENT_SIZE: u32 = 120;
// pub const ZMQ_NORM_BLOCK_SIZE: u32 = 121;
// pub const ZMQ_NORM_NUM_PARITY: u32 = 122;
// pub const ZMQ_NORM_NUM_AUTOPARITY: u32 = 123;
// pub const ZMQ_NORM_PUSH: u32 = 124;

/*  DRAFT ZMQ_NORM_MODE options                                               */
// pub const ZMQ_NORM_FIXED: u32 = 0;
// pub const ZMQ_NORM_CC: u32 = 1;
// pub const ZMQ_NORM_CCL: u32 = 2;
// pub const ZMQ_NORM_CCE: u32 = 3;
// pub const ZMQ_NORM_CCE_ECNONLY: u32 = 4;

/*  DRAFT ZMQ_RECONNECT_STOP options                                          */
// pub const ZMQ_RECONNECT_STOP_CONN_REFUSED: u32 = 0x1;
// pub const ZMQ_RECONNECT_STOP_HANDSHAKE_FAILED: u32 = 0x2;
// pub const ZMQ_RECONNECT_STOP_AFTER_DISCONNECT: u32 = 0x4;

/*  DRAFT Context options                                                     */
// pub const ZMQ_ZERO_COPY_RECV: u32 = 10;

//  Number of new messages in message pipe needed to trigger new memory
//  allocation. Setting this parameter to 256 decreases the impact of
//  memory allocation by approximately 99.6%
// pub const MESSAGE_PIPE_GRANULARITY: usize = 256;

//  Commands in pipe per allocation event.
// pub const COMMAND_PIPE_GRANULARITY: u32 = 16;

//  Determines how often does socket poll for new commands when it
//  still has unprocessed messages to handle. Thus, if it is set to 100,
//  socket will process 100 inbound messages before doing the poll.
//  If there are no unprocessed messages available, poll is Done
//  immediately. Decreasing the value trades overall latency for more
//  real-time behaviour (less latency peaks).
// pub const INBOUND_POLL_RATE: u32 = 100;

//  Maximal delta between high and low watermark.
// pub const MAX_WM_DELTA: u32 = 1024;

//  Maximum number of events the I/O thread can process in one go.
// pub const MAX_IO_EVENTS: u32 = 256;

//  Maximal batch size of packets forwarded by a ZMQ proxy.
//  Increasing this value improves throughput at the expense of
//  latency and fairness.
// pub const PROXY_BURST_SIZE: u32 = 1000;

//  Maximal delay to process command in API thread (in CPU ticks).
//  3,000,000 ticks equals to 1 - 2 milliseconds on current CPUs.
//  Note that delay is only applied when there is continuous stream of
//  messages to process. If not so, commands are processed immediately.
// pub const MAX_COMMAND_DELAY: u32 = 3000000;

//  Low-precision clock precision in CPU ticks. 1ms. Value of 1000000
//  should be OK for CPU frequencies above 1GHz. If should work
//  reasonably well for CPU frequencies above 500MHz. For lower CPU
//  frequencies you may consider lowering this value to get best
//  possible latencies.
// pub const CLOCK_PRECISION: u32 = 1000000;

//  On some OSes the signaler has to be emulated using a TCP
//  connection. In such cases following port is used.
//  If 0, it lets the OS choose a free port without requiring use of a
//  global mutex. The original implementation of a Windows signaler
//  socket used port 5905 instead of letting the OS choose a free port.
//  https://github.com/zeromq/libzmq/issues/1542
// pub const SIGNALER_PORT: u32 = 0;

pub type ZmqHandle = *mut c_void;

// pub type ZmqFd = ZmqHandle;

#[cfg(target_os = "windows")]
pub type WSAEVENT = HANDLE;

//     9 #define _NSIG       64
pub const _NSIG: u32 = 64;

//    10 #define _NSIG_BPW   32v
pub const _NSIG_BPW: u32 = 32;
//    11 #define _NSIG_WORDS (_NSIG / _NSIG_BPW)
pub const _NSIG_WORDS: u32 = _NSIG / _NSIG_BPW;

//    12
//    13 typedef unsigned long old_sigset_t;     /* at least 32 bits */
pub type OldSigset = u32;

//    14
//    15 typedef struct {
//    16     unsigned long sig[_NSIG_WORDS];
//    17 } sigset_t;
pub struct Sigset {
    pub sig: [u32; _NSIG_WORDS as usize],
}
//    18
//    19 struct old_sigaction {
//    20     __sighandler_t sa_handler;
//    21     old_sigset_t sa_mask;
//    22     unsigned long sa_flags;
//    23     __sigrestore_t sa_restorer;
//    24 };

pub const ZMQ_CTX_TAG_VALUE_GOOD: u32 = 0xCAFEBABE;
pub const ZMQ_CTX_TAG_VALUE_BAD: u32 = 0xDEADBEEF;

pub const ZMQ_VERSION_MAJOR: u32 = 4;
pub const ZMQ_VERSION_MINOR: u32 = 3;
pub const ZMQ_VERSION_PATCH: u32 = 5;
pub const ZMQ_VERSION: u32 = 0x040305;

pub const ZMQ_HAUSNUMERO: u32 = 156384712;

pub const ENOTSUP: u32 = (ZMQ_HAUSNUMERO + 1);
pub const EPROTONOSUPPORT: u32 = (ZMQ_HAUSNUMERO + 2);
pub const ENOBUFS: u32 = (ZMQ_HAUSNUMERO + 3);
pub const ENETDOWN: u32 = (ZMQ_HAUSNUMERO + 4);
pub const EADDRINUSE: u32 = (ZMQ_HAUSNUMERO + 5);
pub const EADDRNOTAVAILABLE: u32 = (ZMQ_HAUSNUMERO + 6);
pub const ECONNREFUSED: u32 = (ZMQ_HAUSNUMERO + 7);
pub const EINPROGRESS: u32 = (ZMQ_HAUSNUMERO + 8);
pub const ENOTSOCK: u32 = (ZMQ_HAUSNUMERO + 9);
pub const EMSGSIZE: u32 = (ZMQ_HAUSNUMERO + 10);
pub const EAFNOSUPPORT: u32 = (ZMQ_HAUSNUMERO + 11);
pub const ENETUNREACH: u32 = (ZMQ_HAUSNUMERO + 12);
pub const ECONNABORTED: u32 = (ZMQ_HAUSNUMERO + 13);
pub const ECONNRESET: u32 = (ZMQ_HAUSNUMERO + 14);
pub const ENOTCONN: u32 = (ZMQ_HAUSNUMERO + 15);
pub const ETIMEDOUT: u32 = (ZMQ_HAUSNUMERO + 16);
pub const EHOSTUNREACH: u32 = (ZMQ_HAUSNUMERO + 17);
pub const ENETRESET: u32 = (ZMQ_HAUSNUMERO + 18);
pub const EFSM: u32 = (ZMQ_HAUSNUMERO + 51);
pub const ENOCOMPATPROTO: u32 = (ZMQ_HAUSNUMERO + 52);
pub const ETERM: u32 = (ZMQ_HAUSNUMERO + 53);
pub const EMTHREAD: u32 = (ZMQ_HAUSNUMERO + 54);

// /*  Context options                                                           */
pub const ZMQ_IO_THREADS: u32 = 1;
pub const ZMQ_MAX_SOCKETS: u32 = 2;
pub const ZMQ_SOCKET_LIMIT: u32 = 3;
pub const ZMQ_THREAD_PRIORITY: u32 = 3;
pub const ZMQ_THREAD_SCHED_POLICY: u32 = 4;
pub const ZMQ_MAX_MSGSZ: u32 = 5;
pub const ZMQ_MSG_T_SIZE: u32 = 6;
pub const ZMQ_THREAD_AFFINITY_CPU_ADD: u32 = 7;
pub const ZMQ_THREAD_AFFINITY_CPU_REMOVE: u32 = 8;
pub const ZMQ_THREAD_NAME_PREFIX: u32 = 9;

// /*  Default for new contexts                                                  */
pub const ZMQ_IO_THREADS_DFLT: u32 = 1;
pub const ZMQ_MAX_SOCKETS_DFLT: u32 = 1023;
pub const ZMQ_THREAD_PRIORITY_DFLT: i32 = -1;
pub const ZMQ_THREAD_SCHED_POLICY_DFLT: i32 = -1;
pub const ZMQ_PAIR: u32 = 0;
pub const ZMQ_PUB: u32 = 1;
pub const ZMQ_SUB: u32 = 2;
pub const ZMQ_REQ: u32 = 3;
pub const ZMQ_REP: u32 = 4;
pub const ZMQ_DEALER: u32 = 5;
pub const ZMQ_ROUTER: u32 = 6;
pub const ZMQ_PULL: u32 = 7;
pub const ZMQ_PUSH: u32 = 8;
pub const ZMQ_XPUB: u32 = 9;
pub const ZMQ_XSUB: u32 = 10;
pub const ZMQ_STREAM: u32 = 11;
pub const ZMQ_XREQ: u32 = ZMQ_DEALER;
pub const ZMQ_XREP: u32 = ZMQ_ROUTER;
pub const ZMQ_AFFINITY: u32 = 4;
pub const ZMQ_ROUTING_ID: u32 = 5;
pub const ZMQ_SUBSCRIBE: u32 = 6;
pub const ZMQ_UNSUBSCRIBE: u32 = 7;
pub const ZMQ_RATE: u32 = 8;
pub const ZMQ_RECOVERY_IVL: u32 = 9;
pub const ZMQ_SNDBUF: u32 = 11;
pub const ZMQ_RCVBUF: u32 = 12;
pub const ZMQ_RCVMORE: u32 = 13;
pub const ZMQ_FD: u32 = 14;
pub const ZMQ_EVENTS: u32 = 15;
pub const ZMQ_TYPE: u32 = 16;
pub const ZMQ_LINGER: u32 = 17;
pub const ZMQ_RECONNECT_IVL: u32 = 18;
pub const ZMQ_BACKLOG: u32 = 19;
pub const ZMQ_RECONNECT_IVL_MAX: u32 = 21;
pub const ZMQ_MAXMSGSIZE: u32 = 22;
pub const ZMQ_SNDHWM: u32 = 23;
pub const ZMQ_RCVHWM: u32 = 24;
pub const ZMQ_MULTICAST_HOPS: u32 = 25;
pub const ZMQ_RCVTIMEO: u32 = 27;
pub const ZMQ_SNDTIMEO: u32 = 28;
pub const ZMQ_LAST_ENDPOINT: u32 = 32;
pub const ZMQ_ROUTER_MANDATORY: u32 = 33;
pub const ZMQ_TCP_KEEPALIVE: u32 = 34;
pub const ZMQ_TCP_KEEPALIVE_CNT: u32 = 35;
pub const ZMQ_TCP_KEEPALIVE_IDLE: u32 = 36;
pub const ZMQ_TCP_KEEPALIVE_INTVL: u32 = 37;
pub const ZMQ_IMMEDIATE: u32 = 39;
pub const ZMQ_XPUB_VERBOSE: u32 = 40;
pub const ZMQ_ROUTER_RAW: u32 = 41;
pub const ZMQ_IPV6: u32 = 42;
pub const ZMQ_MECHANISM: u32 = 43;
pub const ZMQ_PLAIN_SERVER: u32 = 44;
pub const ZMQ_PLAIN_USERNAME: u32 = 45;
pub const ZMQ_PLAIN_PASSWORD: u32 = 46;
pub const ZMQ_CURVE_SERVER: u32 = 47;
pub const ZMQ_CURVE_PUBLICKEY: u32 = 48;
pub const ZMQ_CURVE_SECRETKEY: u32 = 49;
pub const ZMQ_CURVE_SERVERKEY: u32 = 50;
pub const ZMQ_PROBE_ROUTER: u32 = 51;
pub const ZMQ_REQ_CORRELATE: u32 = 52;
pub const ZMQ_REQ_RELAXED: u32 = 53;
pub const ZMQ_CONFLATE: u32 = 54;
pub const ZMQ_ZAP_DOMAIN: u32 = 55;
pub const ZMQ_ROUTER_HANDOVER: u32 = 56;
pub const ZMQ_TOS: u32 = 57;
pub const ZMQ_CONNECT_ROUTING_ID: u32 = 61;
pub const ZMQ_GSSAPI_SERVER: u32 = 62;
pub const ZMQ_GSSAPI_PRINCIPAL: u32 = 63;
pub const ZMQ_GSSAPI_SERVICE_PRINCIPAL: u32 = 64;
pub const ZMQ_GSSAPI_PLAINTEXT: u32 = 65;
pub const ZMQ_HANDSHAKE_IVL: u32 = 66;
pub const ZMQ_SOCKS_PROXY: u32 = 68;
pub const ZMQ_XPUB_NODROP: u32 = 69;
pub const ZMQ_BLOCKY: u32 = 70;
pub const ZMQ_XPUB_MANUAL: u32 = 71;
pub const ZMQ_XPUB_WELCOME_MSG: u32 = 72;
pub const ZMQ_STREAM_NOTIFY: u32 = 73;
pub const ZMQ_INVERT_MATCHING: u32 = 74;
pub const ZMQ_HEARTBEAT_IVL: u32 = 75;
pub const ZMQ_HEARTBEAT_TTL: u32 = 76;
pub const ZMQ_HEARTBEAT_TIMEOUT: u32 = 77;
pub const ZMQ_XPUB_VERBOSER: u32 = 78;
pub const ZMQ_CONNECT_TIMEOUT: u32 = 79;
pub const ZMQ_TCP_MAXRT: u32 = 80;
pub const ZMQ_THREAD_SAFE: u32 = 81;
pub const ZMQ_MULTICAST_MAXTPDU: u32 = 84;
pub const ZMQ_VMCI_BUFFER_SIZE: u32 = 85;
pub const ZMQ_VMCI_BUFFER_MIN_SIZE: u32 = 86;
pub const ZMQ_VMCI_BUFFER_MAX_SIZE: u32 = 87;
pub const ZMQ_VMCI_CONNECT_TIMEOUT: u32 = 88;
pub const ZMQ_USE_FD: u32 = 89;
pub const ZMQ_GSSAPI_PRINCIPAL_NAMETYPE: u32 = 90;
pub const ZMQ_GSSAPI_SERVICE_PRINCIPAL_NAMETYPE: u32 = 91;
pub const ZMQ_BINDTODEVICE: u32 = 92;
pub const ZMQ_MORE: u32 = 1;
pub const ZMQ_SHARED: u32 = 3;
pub const ZMQ_DONTWAIT: u32 = 1;
pub const ZMQ_SNDMORE: u32 = 2;
pub const ZMQ_NULL: u32 = 0;
pub const ZMQ_PLAIN: u32 = 1;
pub const ZMQ_CURVE: u32 = 2;
pub const ZMQ_GSSAPI: u32 = 3;
pub const ZMQ_GROUP_MAX_LENGTH: usize = 255;
pub const ZMQ_IDENTITY: u32 = ZMQ_ROUTING_ID;
pub const ZMQ_CONNECT_RID: u32 = ZMQ_CONNECT_ROUTING_ID;
pub const ZMQ_TCP_ACCEPT_FILTER: u32 = 38;
pub const ZMQ_IPC_FILTER_PID: u32 = 58;
pub const ZMQ_IPC_FILTER_UID: u32 = 59;
pub const ZMQ_IPC_FILTER_GID: u32 = 60;
pub const ZMQ_IPV4ONLY: u32 = 31;
pub const ZMQ_DELAY_ATTACH_ON_CONNECT: u32 = ZMQ_IMMEDIATE;
pub const ZMQ_NOBLOCK: u32 = ZMQ_DONTWAIT;
pub const ZMQ_FAIL_UNROUTABLE: u32 = ZMQ_ROUTER_MANDATORY;
pub const ZMQ_ROUTER_BEHAVIOR: u32 = ZMQ_ROUTER_MANDATORY;
pub const ZMQ_SRCFD: u32 = 2;
pub const ZMQ_GSSAPI_NT_HOSTBASED: u32 = 0;
pub const ZMQ_GSSAPI_NT_USER_NAME: u32 = 1;
pub const ZMQ_GSSAPI_NT_KRB5_PRINCIPAL: u32 = 2;

pub const ZMQ_EVENT_CONNECTED: u32 = 0x0001;
pub const ZMQ_EVENT_CONNECT_DELAYED: u32 = 0x0002;
pub const ZMQ_EVENT_CONNECT_RETRIED: u32 = 0x0004;
pub const ZMQ_EVENT_LISTENING: u32 = 0x0008;
pub const ZMQ_EVENT_BIND_FAILED: u32 = 0x0010;
pub const ZMQ_EVENT_ACCEPTED: u32 = 0x0020;
pub const ZMQ_EVENT_ACCEPT_FAILED: u32 = 0x0040;
pub const ZMQ_EVENT_CLOSED: u32 = 0x0080;
pub const ZMQ_EVENT_CLOSE_FAILED: u32 = 0x0100;
pub const ZMQ_EVENT_DISCONNECTED: u32 = 0x0200;
pub const ZMQ_EVENT_MONITOR_STOPPED: u32 = 0x0400;
pub const ZMQ_EVENT_ALL: u32 = 0xFFFF;
pub const ZMQ_EVENT_HANDSHAKE_FAILED_NO_DETAIL: u32 = 0x0800;
pub const ZMQ_EVENT_HANDSHAKE_SUCCEEDED: u32 = 0x1000;
pub const ZMQ_EVENT_HANDSHAKE_FAILED_PROTOCOL: u32 = 0x2000;
pub const ZMQ_EVENT_HANDSHAKE_FAILED_AUTH: u32 = 0x4000;
pub const ZMQ_PROTOCOL_ERROR_ZMTP_UNSPECIFIED: u32 = 0x10000000;
pub const ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND: u32 = 0x10000001;
pub const ZMQ_PROTOCOL_ERROR_ZMTP_INVALID_SEQUENCE: u32 = 0x10000002;
pub const ZMQ_PROTOCOL_ERROR_ZMTP_KEY_EXCHANGE: u32 = 0x10000003;
pub const ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_UNSPECIFIED: u32 = 0x10000011;
pub const ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_MESSAGE: u32 = 0x10000012;
pub const ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_HELLO: u32 = 0x10000013;
pub const ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_INITIATE: u32 = 0x10000014;
pub const ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_ERROR: u32 = 0x10000015;
pub const ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_READY: u32 = 0x10000016;
pub const ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_WELCOME: u32 = 0x10000017;
pub const ZMQ_PROTOCOL_ERROR_ZMTP_INVALID_METADATA: u32 = 0x10000018;
// the following two may be due to erroneous configuration of a peer
pub const ZMQ_PROTOCOL_ERROR_ZMTP_CRYPTOGRAPHIC: u32 = 0x11000001;
pub const ZMQ_PROTOCOL_ERROR_ZMTP_MECHANISM_MISMATCH: u32 = 0x11000002;
pub const ZMQ_PROTOCOL_ERROR_ZAP_UNSPECIFIED: u32 = 0x20000000;
pub const ZMQ_PROTOCOL_ERROR_ZAP_MALFORMED_REPLY: u32 = 0x20000001;
pub const ZMQ_PROTOCOL_ERROR_ZAP_BAD_REQUEST_ID: u32 = 0x20000002;
pub const ZMQ_PROTOCOL_ERROR_ZAP_BAD_VERSION: u32 = 0x20000003;
pub const ZMQ_PROTOCOL_ERROR_ZAP_INVALID_STATUS_CODE: u32 = 0x20000004;
pub const ZMQ_PROTOCOL_ERROR_ZAP_INVALID_METADATA: u32 = 0x20000005;
pub const ZMQ_PROTOCOL_ERROR_WS_UNSPECIFIED: u32 = 0x30000000;

pub const ZMQ_POLLIN: u32 = 1;
pub const ZMQ_POLLOUT: u32 = 2;
pub const ZMQ_POLLERR: u32 = 4;
pub const ZMQ_POLLPRI: u32 = 8;

pub const ZMQ_POLLITEMS_DFLT: u32 = 16;

pub const ZMQ_HAS_CAPABILITIES: u32 = 1;
pub const ZMQ_STREAMER: u32 = 1;
pub const ZMQ_FORWARDER: u32 = 2;
pub const ZMQ_QUEUE: u32 = 3;
pub const ZMQ_SERVER: u32 = 12;
pub const ZMQ_CLIENT: u32 = 13;
pub const ZMQ_RADIO: u32 = 14;
pub const ZMQ_DISH: u32 = 15;
pub const ZMQ_GATHER: u32 = 16;
pub const ZMQ_SCATTER: u32 = 17;
pub const ZMQ_DGRAM: u32 = 18;
pub const ZMQ_PEER: u32 = 19;
pub const ZMQ_CHANNEL: u32 = 20;
pub const ZMQ_ZAP_ENFORCE_DOMAIN: u32 = 93;
pub const ZMQ_LOOPBACK_FASTPATH: u32 = 94;
pub const ZMQ_METADATA: u32 = 95;
pub const ZMQ_MULTICAST_LOOP: u32 = 96;
pub const ZMQ_ROUTER_NOTIFY: u32 = 97;
pub const ZMQ_XPUB_MANUAL_LAST_VALUE: u32 = 98;
pub const ZMQ_SOCKS_USERNAME: u32 = 99;
pub const ZMQ_SOCKS_PASSWORD: u32 = 100;
pub const ZMQ_IN_BATCH_SIZE: u32 = 101;
pub const ZMQ_OUT_BATCH_SIZE: u32 = 102;
pub const ZMQ_WSS_KEY_PEM: u32 = 103;
pub const ZMQ_WSS_CERT_PEM: u32 = 104;
pub const ZMQ_WSS_TRUST_PEM: u32 = 105;
pub const ZMQ_WSS_HOSTNAME: u32 = 106;
pub const ZMQ_WSS_TRUST_SYSTEM: u32 = 107;
pub const ZMQ_ONLY_FIRST_SUBSCRIBE: u32 = 108;
pub const ZMQ_RECONNECT_STOP: u32 = 109;
pub const ZMQ_HELLO_MSG: u32 = 110;
pub const ZMQ_DISCONNECT_MSG: u32 = 111;
pub const ZMQ_PRIORITY: u32 = 112;
pub const ZMQ_BUSY_POLL: u32 = 113;
pub const ZMQ_HICCUP_MSG: u32 = 114;
pub const ZMQ_XSUB_VERBOSE_UNSUBSCRIBE: u32 = 115;
pub const ZMQ_TOPICS_COUNT: u32 = 116;
pub const ZMQ_NORM_MODE: u32 = 117;
pub const ZMQ_NORM_UNICAST_NACK: u32 = 118;
pub const ZMQ_NORM_BUFFER_SIZE: u32 = 119;
pub const ZMQ_NORM_SEGMENT_SIZE: u32 = 120;
pub const ZMQ_NORM_BLOCK_SIZE: u32 = 121;
pub const ZMQ_NORM_NUM_PARITY: u32 = 122;
pub const ZMQ_NORM_NUM_AUTOPARITY: u32 = 123;
pub const ZMQ_NORM_PUSH: u32 = 124;
pub const ZMQ_NORM_FIXED: u32 = 0;
pub const ZMQ_NORM_CC: u32 = 1;
pub const ZMQ_NORM_CCL: u32 = 2;
pub const ZMQ_NORM_CCE: u32 = 3;
pub const ZMQ_NORM_CCE_ECNONLY: u32 = 4;
pub const ZMQ_RECONNECT_STOP_CONN_REFUSED: u32 = 0x1;
pub const ZMQ_RECONNECT_STOP_HANDSHAKE_FAILED: u32 = 0x2;
pub const ZMQ_RECONNECT_STOP_AFTER_DISCONNECT: u32 = 0x4;
pub const ZMQ_ZERO_COPY_RECV: u32 = 10;
pub const ZMQ_MSG_PROPERTY_ROUTING_ID: &'static str = "Routing-Id";
pub const ZMQ_MSG_PROPERTY_SOCKET_TYPE: &'static str = "Socket-Type";
pub const ZMQ_MSG_PROPERTY_USER_ID: &'static str = "User-Id";
pub const ZMQ_MSG_PROPERTY_PEER_ADDRESS: &'static str = "Peer-Address";
pub const ZMQ_NOTIFY_CONNECT: u32 = 1;
pub const ZMQ_NOTIFY_DISCONNECT: u32 = 2;
pub const ZMQ_EVENT_PIPES_STATS: u32 = 0x10000;

pub const ZMQ_CURRENT_EVENT_VERSION: u32 = 1;
pub const ZMQ_CURRENT_EVENT_VERSION_DRAFT: u32 = 2;

pub const ZMQ_EVENT_ALL_V1: u32 = ZMQ_EVENT_ALL;
pub const ZMQ_EVENT_ALL_V2: u32 = ZMQ_EVENT_ALL_V1 | ZMQ_EVENT_PIPES_STATS;

#[cfg(not(target_os = "windows"))]
pub const RETIRED_FD: i32 = -1;
#[cfg(target_os = "windows")]
pub const RETIRED_FD: usize = usize::MAX;

#[cfg(target_os = "windows")]
pub type ZmqPollFd = WSAPOLLFD;
#[cfg(not(target_os = "windows"))]
pub type ZmqPollFd = pollfd;

pub const CANCEL_CMD_NAME: &'static str = "\x06CANCEL";
pub const SUB_CMD_NAME: &'static str = "\x09SUBSCRIBE";

pub const CMD_TYPE_MASK: u8 = 0x1c;

pub const ZMQ_MSG_MORE: u8 = 1;
pub const ZMQ_MSG_COMMAND: u8 = 2;
pub const ZMQ_MSG_PING: u8 = 4;
pub const ZMQ_MSG_PONG: u8 = 8;
pub const ZMQ_MSG_SUBSCRIBE: u8 = 12;
pub const ZMQ_MSG_CANCEL: u8 = 16;
pub const ZMQ_MSG_CLOSE_CMD: u8 = 20;
pub const ZMQ_MSG_CREDENTIAL: u8 = 32;
pub const ZMQ_MSG_ROUTING_ID: u8 = 64;
pub const ZMQ_MSG_SHARED: u8 = 128;

#[allow(non_camel_case_types)]
pub type socklen_t = u32;

pub type ZmqSocklen = socklen_t;

pub type ZmqSaFamily = u32;

// struct sockaddr_storage {
//            sa_family_t     ss_family;      /* Address family */
//        };
#[derive(Default, Debug, Clone)]
pub struct ZmqSockaddrStorage {
    pub ss_family: ZmqSaFamily,
    // sa_data: [u8; 14],
    // sin_port: u16,
    // sin_addr: u32,
    // sin6_flowinfo: u32,
    // sin6_addr: [u8; 16],
    // sin6_scope_id: u32,
    pub sa_data: [u8; std::mem::size_of::<ZmqSockAddrIn6>()],
}

pub const COMMAND_PIPE_GRANULARITY: usize = 16;
pub const MESSAGE_PIPE_GRANULARITY: usize = 256;
pub const INBOUND_POLL_RATE: u32 = 100;
pub const MAX_WM_DELTA: u32 = 1024;
pub const MAX_IO_EVENTS: u32 = 256;
pub const PROXY_BURST_SIZE: u32 = 1000;
pub const MAX_COMMAND_DELAY: u32 = 3000000;
pub const CLOCK_PRECISION: u32 = 1000000;
pub const SIGNALER_PORT: u32 = 0;

pub type ZmqConditionVariable = Mutex<()>;

#[cfg(target_os = "windows")]
pub type ZmqPid = i32;
#[cfg(not(target_os = "windows"))]
pub type ZmqPid = pid_t;

pub const MORE_FLAG: u8 = 1;
pub const LARGE_FLAG: u8 = 2;
pub const COMMAND_FLAG: u8 = 4;

pub type ZmqSubscriptions = HashSet<String>;

pub const SOCKET_TYPE_PAIR: &'static str = "PAIR";
pub const SOCKET_TYPE_PUB: &'static str = "PUB";
pub const SOCKET_TYPE_SUB: &'static str = "SUB";
pub const SOCKET_TYPE_REQ: &'static str = "REQ";
pub const SOCKET_TYPE_REP: &'static str = "REP";
pub const SOCKET_TYPE_DEALER: &'static str = "DEALER";
pub const SOCKET_TYPE_ROUTER: &'static str = "ROUTER";
pub const SOCKET_TYPE_PULL: &'static str = "PULL";
pub const SOCKET_TYPE_PUSH: &'static str = "PUSH";
pub const SOCKET_TYPE_XPUB: &'static str = "XPUB";
pub const SOCKET_TYPE_XSUB: &'static str = "XSUB";
pub const SOCKET_TYPE_STREAM: &'static str = "STREAM";
pub const SOCKET_TYPE_SERVER: &'static str = "SERVER";
pub const SOCKET_TYPE_CLIENT: &'static str = "CLIENT";
pub const SOCKET_TYPE_RADIO: &'static str = "RADIO";
pub const SOCKET_TYPE_DISH: &'static str = "DISH";
pub const SOCKET_TYPE_GATHER: &'static str = "GATHER";
pub const SOCKET_TYPE_SCATTER: &'static str = "SCATTER";
pub const SOCKET_TYPE_DGRAM: &'static str = "DGRAM";
pub const SOCKET_TYPE_PEER: &'static str = "PEER";
pub const SOCKET_TYPE_CHANNEL: &'static str = "CHANNEL";

#[derive(Default, Debug, Clone)]
pub struct ZmqSockAddrIn {
    pub sin_family: u16,
    pub sin_port: u16,
    pub sin_addr: u32,
    pub sin_zero: [u8; 8],
}

#[derive(Default, Debug, Clone)]
pub struct ZmqInAddr {
    pub s_addr: u32,
}

#[derive(Default, Debug, Clone)]
pub struct ZmqSockAddr {
    pub sa_family: u16,
    pub sa_data: [u8; 14],
}

#[derive(Default, Debug, Clone)]
pub struct ZmqSockAddrIn6 {
    pub sin6_family: u16,
    pub sin6_port: u16,
    pub sin6_flowinfo: u32,
    pub sin6_addr: [u8; 16],
    pub sin6_scope_id: u32,
}

#[derive(Default, Debug, Clone)]
pub struct ZmqIn6Addr {
    pub s6_addr: [u8; 16],
}

pub const ERROR_COMMAND_NAME: &'static str = "\x05ERROR";
pub const ERROR_COMMAND_NAME_LEN: usize = 6;
pub const ERROR_REASON_LEN_SIZE: usize = 1;

pub const READY_COMMAND_NAME: &'static str = "\x05READY";
pub const READY_COMMAND_NAME_LEN: usize = 6;

///////////////
// SOCKET Types
///////////////
pub const SOCK_STREAM: i32 = 1;
pub const SOCK_DGRAM: i32 = 2;
pub const SOCK_RAW: i32 = 3;
pub const SOCK_RDM: i32 = 4;
pub const SOCK_SEQPACKET: i32 = 5;
pub const SOCK_DCCP: i32 = 6;
pub const SOCK_PACKET: i32 = 10;

pub const SOCK_CLOEXEC: i32 = 02000000;

pub const SOCK_NONBLOCK: i32 = 04000;

/////////////////
// IP PROTO TYPES
/////////////////
// enum {
//   IPPROTO_IP = 0,		/* Dummy protocol for TCP		*/
// #define IPPROTO_IP		IPPROTO_IP
pub const IPPROTO_IP: i32 = 0;
//   IPPROTO_ICMP = 1,		/* Internet Control Message Protocol	*/
// #define IPPROTO_ICMP		IPPROTO_ICMP
pub const IPPROTO_ICMP: i32 = 1;
//   IPPROTO_IGMP = 2,		/* Internet Group Management Protocol	*/
// #define IPPROTO_IGMP		IPPROTO_IGMP
pub const IPPROTO_IGMP: i32 = 2;
//   IPPROTO_IPIP = 4,		/* IPIP tunnels (older KA9Q tunnels use 94) */
// #define IPPROTO_IPIP		IPPROTO_IPIP
pub const IPPROTO_IPIP: i32 = 4;
//   IPPROTO_TCP = 6,		/* Transmission Control Protocol	*/
// #define IPPROTO_TCP		IPPROTO_TCP
pub const IPPROTO_TCP: i32 = 6;
//   IPPROTO_EGP = 8,		/* Exterior Gateway Protocol		*/
// #define IPPROTO_EGP		IPPROTO_EGP
pub const IPPROTO_EGP: i32 = 8;
//   IPPROTO_PUP = 12,		/* PUP protocol				*/
// #define IPPROTO_PUP		IPPROTO_PUP
pub const IPPROTO_PUP: i32 = 12;
//   IPPROTO_UDP = 17,		/* User Datagram Protocol		*/
// #define IPPROTO_UDP		IPPROTO_UDP
pub const IPPROTO_UDP: i32 = 17;
//   IPPROTO_IDP = 22,		/* XNS IDP protocol			*/
// #define IPPROTO_IDP		IPPROTO_IDP
pub const IPPROTO_IDP: i32 = 22;
//   IPPROTO_TP = 29,		/* SO Transport Protocol Class 4	*/
// #define IPPROTO_TP		IPPROTO_TP
pub const IPPROTO_TP: i32 = 29;
//   IPPROTO_DCCP = 33,		/* Datagram Congestion Control Protocol */
// #define IPPROTO_DCCP		IPPROTO_DCCP
pub const IPPROTO_DCCP: i32 = 33;
//   IPPROTO_IPV6 = 41,		/* IPv6-in-IPv4 tunnelling		*/
// #define IPPROTO_IPV6		IPPROTO_IPV6
pub const IPPROTO_IPV6: i32 = 41;
//   IPPROTO_RSVP = 46,		/* RSVP Protocol			*/
// #define IPPROTO_RSVP		IPPROTO_RSVP
pub const IPPROTO_RSVP: i32 = 46;
//   IPPROTO_GRE = 47,		/* Cisco GRE tunnels (rfc 1701,1702)	*/
// #define IPPROTO_GRE		IPPROTO_GRE
pub const IPPROTO_GRE: i32 = 47;
//   IPPROTO_ESP = 50,		/* Encapsulation Security Payload protocol */
// #define IPPROTO_ESP		IPPROTO_ESP
pub const IPPROTO_ESP: i32 = 50;
//   IPPROTO_AH = 51,		/* Authentication Header protocol	*/
// #define IPPROTO_AH		IPPROTO_AH
pub const IPPROTO_AH: i32 = 51;
//   IPPROTO_MTP = 92,		/* Multicast Transport Protocol		*/
// #define IPPROTO_MTP		IPPROTO_MTP
pub const IPPROTO_MTP: i32 = 92;
//   IPPROTO_BEETPH = 94,		/* IP option pseudo header for BEET	*/
// #define IPPROTO_BEETPH		IPPROTO_BEETPH
pub const IPPROTO_BEETPH: i32 = 94;
//   IPPROTO_ENCAP = 98,		/* Encapsulation Header			*/
// #define IPPROTO_ENCAP		IPPROTO_ENCAP
pub const IPPROTO_ENCAP: i32 = 98;
//   IPPROTO_PIM = 103,		/* Protocol Independent Multicast	*/
// #define IPPROTO_PIM		IPPROTO_PIM
pub const IPPROTO_PIM: i32 = 103;
//   IPPROTO_COMP = 108,		/* Compression Header Protocol		*/
// #define IPPROTO_COMP		IPPROTO_COMP
pub const IPPROTO_COMP: i32 = 108;
//   IPPROTO_L2TP = 115,		/* Layer 2 Tunnelling Protocol		*/
// #define IPPROTO_L2TP		IPPROTO_L2TP
pub const IPPROTO_L2TP: i32 = 115;
//   IPPROTO_SCTP = 132,		/* Stream Control Transport Protocol	*/
// #define IPPROTO_SCTP		IPPROTO_SCTP
pub const IPPROTO_SCTP: i32 = 132;
//   IPPROTO_UDPLITE = 136,	/* UDP-Lite (RFC 3828)			*/
// #define IPPROTO_UDPLITE		IPPROTO_UDPLITE
pub const IPPROTO_UDPLITE: i32 = 136;
//   IPPROTO_MPLS = 137,		/* MPLS in IP (RFC 4023)		*/
// #define IPPROTO_MPLS		IPPROTO_MPLS
pub const IPPROTO_MPLS: i32 = 137;
//   IPPROTO_ETHERNET = 143,	/* Ethernet-within-IPv6 Encapsulation	*/
// #define IPPROTO_ETHERNET	IPPROTO_ETHERNET
pub const IPPROTO_ETHERNET: i32 = 143;
//   IPPROTO_RAW = 255,		/* Raw IP packets			*/
// #define IPPROTO_RAW		IPPROTO_RAW
pub const IPPROTO_RAW: i32 = 255;
//   IPPROTO_MPTCP = 262,		/* Multipath TCP connection		*/
// #define IPPROTO_MPTCP		IPPROTO_MPTCP
pub const IPPROTO_MPTCP: i32 = 262;
//   IPPROTO_MAX
// };

pub const INADDR_ANY: u32 = 0;

// #[derive(Default,Debug,Clone)]
// pub struct ZmqIn6Addr {
//     pub s6_addr:[u8;16]
// }

pub const IN6ADDR_ANY: ZmqIn6Addr = ZmqIn6Addr { s6_addr: [0; 16] };

//
// ADDRESS FAMILIES
//

///* Protocol families.  */
// #define PF_UNSPEC	0	/* Unspecified.  */
pub const PF_UNSPEC: i32 = 0;
// #define PF_LOCAL	1	/* Local to host (pipes and file-domain).  */
pub const PF_LOCAL: i32 = 1;
// #define PF_UNIX		PF_LOCAL /* POSIX name for PF_LOCAL.  */
pub const PF_UNIX: i32 = PF_LOCAL;
// #define PF_FILE		PF_LOCAL /* Another non-standard name for PF_LOCAL.  */
pub const PF_FILE: i32 = PF_LOCAL;
// #define PF_INET		2	/* IP protocol family.  */
pub const PF_INET: i32 = 2;
// #define PF_AX25		3	/* Amateur Radio AX.25.  */
pub const PF_AX25: i32 = 3;
// #define PF_IPX		4	/* Novell Internet Protocol.  */
pub const PF_IPX: i32 = 4;
// #define PF_APPLETALK	5	/* Appletalk DDP.  */
pub const PF_APPLETALK: i32 = 5;
// #define PF_NETROM	6	/* Amateur radio NetROM.  */
pub const PF_NETROM: i32 = 6;
// #define PF_BRIDGE	7	/* Multiprotocol bridge.  */
pub const PF_BRIDGE: i32 = 7;
// #define PF_ATMPVC	8	/* ATM PVCs.  */
pub const PF_ATMPVC: i32 = 8;
// #define PF_X25		9	/* Reserved for X.25 project.  */
pub const PF_X25: i32 = 9;
// #define PF_INET6	10	/* IP version 6.  */
pub const PF_INET6: i32 = 10;
// #define PF_ROSE		11	/* Amateur Radio X.25 PLP.  */
pub const PF_ROSE: i32 = 11;
#[allow(non_upper_case_globals)]// #define PF_DECnet	12	/* Reserved for DECnet project.  */
pub const PF_DECnet: i32 = 12;
// #define PF_NETBEUI	13	/* Reserved for 802.2LLC project.  */
pub const PF_NETBEUI: i32 = 13;
// #define PF_SECURITY	14	/* Security callback pseudo AF.  */
pub const PF_SECURITY: i32 = 14;
// #define PF_KEY		15	/* PF_KEY key management API.  */
pub const PF_KEY: i32 = 15;
// #define PF_NETLINK	16
pub const PF_NETLINK: i32 = 16;
// #define PF_ROUTE	PF_NETLINK /* Alias to emulate 4.4BSD.  */
pub const PF_ROUTE: i32 = PF_NETLINK;
// #define PF_PACKET	17	/* Packet family.  */
pub const PF_PACKET: i32 = 17;
// #define PF_ASH		18	/* Ash.  */
pub const PF_ASH: i32 = 18;
// #define PF_ECONET	19	/* Acorn Econet.  */
pub const PF_ECONET: i32 = 19;
// #define PF_ATMSVC	20	/* ATM SVCs.  */
pub const PF_ATMSVC: i32 = 20;
// #define PF_RDS		21	/* RDS sockets.  */
pub const PF_RDS: i32 = 21;
// #define PF_SNA		22	/* Linux SNA Project */
pub const PF_SNA: i32 = 22;
// #define PF_IRDA		23	/* IRDA sockets.  */
pub const PF_IRDA: i32 = 23;
// #define PF_PPPOX	24	/* PPPoX sockets.  */
pub const PF_PPPOX: i32 = 24;
// #define PF_WANPIPE	25	/* Wanpipe API sockets.  */
pub const PF_WANPIPE: i32 = 25;
// #define PF_LLC		26	/* Linux LLC.  */
pub const PF_LLC: i32 = 26;
// #define PF_IB		27	/* Native InfiniBand address.  */
pub const PF_IB: i32 = 27;
// #define PF_MPLS		28	/* MPLS.  */
pub const PF_MPLS: i32 = 28;
// #define PF_CAN		29	/* Controller Area Network.  */
pub const PF_CAN: i32 = 29;
// #define PF_TIPC		30	/* TIPC sockets.  */
pub const PF_TIPC: i32 = 30;
// #define PF_BLUETOOTH	31	/* Bluetooth sockets.  */
pub const PF_BLUETOOTH: i32 = 31;
// #define PF_IUCV		32	/* IUCV sockets.  */
pub const PF_IUCV: i32 = 32;
// #define PF_RXRPC	33	/* RxRPC sockets.  */
pub const PF_RXRPC: i32 = 33;
// #define PF_ISDN		34	/* mISDN sockets.  */
pub const PF_ISDN: i32 = 34;
// #define PF_PHONET	35	/* Phonet sockets.  */
pub const PF_PHONET: i32 = 35;
// #define PF_IEEE802154	36	/* IEEE 802.15.4 sockets.  */
pub const PF_IEEE802154: i32 = 36;
// #define PF_CAIF		37	/* CAIF sockets.  */
pub const PF_CAIF: i32 = 37;
// #define PF_ALG		38	/* Algorithm sockets.  */
pub const PF_ALG: i32 = 38;
// #define PF_NFC		39	/* NFC sockets.  */
pub const PF_NFC: i32 = 39;
// #define PF_VSOCK	40	/* vSockets.  */
pub const PF_VSOCK: i32 = 40;
// #define PF_KCM		41	/* Kernel Connection Multiplexor.  */
pub const PF_KCM: i32 = 41;
// #define PF_QIPCRTR	42	/* Qualcomm IPC Router.  */
pub const PF_QIPCRTR: i32 = 42;
// #define PF_SMC		43	/* SMC sockets.  */
pub const PF_SMC: i32 = 43;
// #define PF_XDP		44	/* XDP sockets.  */
pub const PF_XDP: i32 = 44;
// #define PF_MCTP		45	/* Management component transport protocol.  */
pub const PF_MCTP: i32 = 45;
// #define PF_MAX		46	/* For now..  */
pub const PF_MAX: i32 = 46;

// /* Address families.  */
// #define AF_UNSPEC	PF_UNSPEC
pub const AF_UNSPEC: i32 = PF_UNSPEC;
// #define AF_LOCAL	PF_LOCAL
pub const AF_LOCAL: i32 = PF_LOCAL;
// #define AF_UNIX		PF_UNIX
pub const AF_UNIX: i32 = PF_UNIX;
// #define AF_FILE		PF_FILE
pub const AF_FILE: i32 = PF_FILE;
// #define AF_INET		PF_INET
pub const AF_INET: i32 = PF_INET;
// #define AF_AX25		PF_AX25
pub const AF_AX25: i32 = PF_AX25;
// #define AF_IPX		PF_IPX
pub const AF_IPX: i32 = PF_IPX;
// #define AF_APPLETALK	PF_APPLETALK
pub const AF_APPLETALK: i32 = PF_APPLETALK;
// #define AF_NETROM	PF_NETROM
pub const AF_NETROM: i32 = PF_NETROM;
// #define AF_BRIDGE	PF_BRIDGE
pub const AF_BRIDGE: i32 = PF_BRIDGE;
// #define AF_ATMPVC	PF_ATMPVC
pub const AF_ATMPVC: i32 = PF_ATMPVC;
// #define AF_X25		PF_X25
pub const AF_X25: i32 = PF_X25;
// #define AF_INET6	PF_INET6
pub const AF_INET6: i32 = PF_INET6;
// #define AF_ROSE		PF_ROSE
pub const AF_ROSE: i32 = PF_ROSE;
// #define AF_DECnet	PF_DECnet
pub const AF_DECnet: i32 = PF_DECnet;
// #define AF_NETBEUI	PF_NETBEUI
pub const AF_NETBEUI: i32 = PF_NETBEUI;
// #define AF_SECURITY	PF_SECURITY
pub const AF_SECURITY: i32 = PF_SECURITY;
// #define AF_KEY		PF_KEY
pub const AF_KEY: i32 = PF_KEY;
// #define AF_NETLINK	PF_NETLINK
pub const AF_NETLINK: i32 = PF_NETLINK;
// #define AF_ROUTE	PF_ROUTE
pub const AF_ROUTE: i32 = PF_ROUTE;
// #define AF_PACKET	PF_PACKET
pub const AF_PACKET: i32 = PF_PACKET;
// #define AF_ASH		PF_ASH
pub const AF_ASH: i32 = PF_ASH;
// #define AF_ECONET	PF_ECONET
pub const AF_ECONET: i32 = PF_ECONET;
// #define AF_ATMSVC	PF_ATMSVC
pub const AF_ATMSVC: i32 = PF_ATMSVC;
// #define AF_RDS		PF_RDS
pub const AF_RDS: i32 = PF_RDS;
// #define AF_SNA		PF_SNA
pub const AF_SNA: i32 = PF_SNA;
// #define AF_IRDA		PF_IRDA
pub const AF_IRDA: i32 = PF_IRDA;
// #define AF_PPPOX	PF_PPPOX
pub const AF_PPPOX: i32 = PF_PPPOX;
// #define AF_WANPIPE	PF_WANPIPE
pub const AF_WANPIPE: i32 = PF_WANPIPE;
// #define AF_LLC		PF_LLC
pub const AF_LLC: i32 = PF_LLC;
// #define AF_IB		PF_IB
pub const AF_IB: i32 = PF_IB;
// #define AF_MPLS		PF_MPLS
pub const AF_MPLS: i32 = PF_MPLS;
// #define AF_CAN		PF_CAN
pub const AF_CAN: i32 = PF_CAN;
// #define AF_TIPC		PF_TIPC
pub const AF_TIPC: i32 = PF_TIPC;
// #define AF_BLUETOOTH	PF_BLUETOOTH
pub const AF_BLUETOOTH: i32 = PF_BLUETOOTH;
// #define AF_IUCV		PF_IUCV
pub const AF_IUCV: i32 = PF_IUCV;
// #define AF_RXRPC	PF_RXRPC
pub const AF_RXRPC: i32 = PF_RXRPC;
// #define AF_ISDN		PF_ISDN
pub const AF_ISDN: i32 = PF_ISDN;
// #define AF_PHONET	PF_PHONET
pub const AF_PHONET: i32 = PF_PHONET;
// #define AF_IEEE802154	PF_IEEE802154
pub const AF_IEEE802154: i32 = PF_IEEE802154;
// #define AF_CAIF		PF_CAIF
pub const AF_CAIF: i32 = PF_CAIF;
// #define AF_ALG		PF_ALG
pub const AF_ALG: i32 = PF_ALG;
// #define AF_NFC		PF_NFC
pub const AF_NFC: i32 = PF_NFC;
// #define AF_VSOCK	PF_VSOCK
pub const AF_VSOCK: i32 = PF_VSOCK;
// #define AF_KCM		PF_KCM
pub const AF_KCM: i32 = PF_KCM;
// #define AF_QIPCRTR	PF_QIPCRTR
pub const AF_QIPCRTR: i32 = PF_QIPCRTR;
// #define AF_SMC		PF_SMC
pub const AF_SMC: i32 = PF_SMC;
// #define AF_XDP		PF_XDP
pub const AF_XDP: i32 = PF_XDP;
// #define AF_MCTP		PF_MCTP
pub const AF_MCTP: i32 = PF_MCTP;
// #define AF_MAX		PF_MAX
pub const AF_MAX: i32 = PF_MAX;

//
// Struct AddrInfo
//
#[derive(Default, Debug, Clone)]
pub struct ZmqAddrInfo {
    pub ai_flags: i32,
    pub ai_family: i32,
    pub ai_socktype: i32,
    pub ai_protocol: i32,
    pub ai_addrlen: usize,
    pub ai_addr: *mut ZmqSockAddr,
    pub ai_canonname: *mut i8,
    pub ai_next: *mut ZmqAddrInfo,
}

//
// getaddrinfo flag values
//
pub const AI_PASSIVE: i32 = 1;
pub const AI_CANONNAME: i32 = 1;
pub const AI_NUMERICHOST: i32 = 4;
pub const AI_MASK: i32 = AI_PASSIVE | AI_CANONNAME | AI_NUMERICHOST;
pub const AI_ALL: i32 = 0x100;
pub const AI_V4MAPPED_CFG: i32 = 0x200;
pub const AI_ADDRCONFIG: i32 = 0x400;
pub const AI_V4MAPPED: i32 = 0x800;
pub const AI_DEFAULT: i32 = AI_V4MAPPED_CFG | AI_ADDRCONFIG;

//
// addrinfo error values
//
// #define EAI_ADDRFAMILY   1      /* address family for hostname not supported */
pub const EAI_ADDRFAMILY: i32 = 1;
// #define EAI_AGAIN        2      /* temporary failure in name resolution */
pub const EAI_AGAIN: i32 = 2;
// #define EAI_BADFLAGS     3      /* invalid value for ai_flags */
pub const EAI_BADFLAGS: i32 = 3;
// #define EAI_FAIL         4      /* non-recoverable failure in name resolution */
pub const EAI_FAIL: i32 = 4;
// #define EAI_FAMILY       5      /* ai_family not supported */
pub const EAI_FAMILY: i32 = 5;
// #define EAI_MEMORY       6      /* memory allocation failure */
pub const EAI_MEMORY: i32 = 6;
// #define EAI_NODATA       7      /* no address associated with hostname */
pub const EAI_NODATA: i32 = 7;
// #define EAI_NONAME       8      /* hostname nor servname provided, or not known */
pub const EAI_NONAME: i32 = 8;
// #define EAI_SERVICE      9      /* servname not supported for ai_socktype */
pub const EAI_SERVICE: i32 = 9;
// #define EAI_SOCKTYPE    10      /* ai_socktype not supported */
pub const EAI_SOCKTYPE: i32 = 10;
// #define EAI_SYSTEM      11      /* system error returned in errno */
pub const EAI_SYSTEM: i32 = 11;
// #define EAI_BADHINTS    12
pub const EAI_BADHINTS: i32 = 12;
// #define EAI_PROTOCOL    13
pub const EAI_PROTOCOL: i32 = 13;
// #define EAI_MAX         14
pub const EAI_MAX: i32 = 14;

//
// Nameinfo constants
//

//#define NI_NUMERICHOST 1
pub const NI_NUMERICHOST: i32 = 1;
// #define NI_NUMERICSERV 2
pub const NI_NUMERICSERV: i32 = 2;
// #define NI_NOFQDN 4
pub const NI_NOFQDN: i32 = 4;
// #define NI_NAMEREQD 8
pub const NI_NAMEREQD: i32 = 8;
// #define NI_DGRAM 16
pub const NI_DGRAM: i32 = 16;
//
// /* POSIX extensions */
//
// #ifndef NI_MAXHOST
// #define NI_MAXHOST 64
pub const NI_MAXHOST: i32 = 64;
// #endif

//
// ip mreq
//
#[derive(Default, Debug, Clone)]
pub struct ZmqIpMreq {
    pub imr_multiaddr: ZmqInAddr,
    pub imr_interface: ZmqInAddr,
}

#[derive(Default, Debug, Clone)]
pub struct ZmqIpv6Mreq {
    pub ipv6mr_multiaddr: ZmqIn6Addr,
    pub ipv6mr_interface: u32,
}

pub const INADDR_NONE: u32 = 0xffffffff;

//
//
//

// 96 #define IP_MULTICAST_IF         32
pub const IP_MULTICAST_IF: i32 = 32;
//    97 #define IP_MULTICAST_TTL        33
pub const IP_MULTICAST_TTL: i32 = 33;
//    98 #define IP_MULTICAST_LOOP       34
pub const IP_MULTICAST_LOOP: i32 = 34;
//    99 #define IP_ADD_MEMBERSHIP       35
pub const IP_ADD_MEMBERSHIP: i32 = 35;
//   100 #define IP_DROP_MEMBERSHIP      36
pub const IP_DROP_MEMBERSHIP: i32 = 36;
//   101 #define IP_UNBLOCK_SOURCE       37
pub const IP_UNBLOCK_SOURCE: i32 = 37;
//   102 #define IP_BLOCK_SOURCE         38
pub const IP_BLOCK_SOURCE: i32 = 38;
//   103 #define IP_ADD_SOURCE_MEMBERSHIP    39
pub const IP_ADD_SOURCE_MEMBERSHIP: i32 = 39;
//   104 #define IP_DROP_SOURCE_MEMBERSHIP   40
pub const IP_DROP_SOURCE_MEMBERSHIP: i32 = 40;

//
//
//

//   143 #define IPV6_ADDRFORM       1
pub const IPV6_ADDRFORM: i32 = 1;
//   144 #define IPV6_2292PKTINFO    2
pub const IPV6_2292PKTINFO: i32 = 2;
//   145 #define IPV6_2292HOPOPTS    3
pub const IPV6_2292HOPOPTS: i32 = 3;
//   146 #define IPV6_2292DSTOPTS    4
pub const IPV6_2292DSTOPTS: i32 = 4;
//   147 #define IPV6_2292RTHDR      5
pub const IPV6_2292RTHDR: i32 = 5;
//   148 #define IPV6_2292PKTOPTIONS 6
pub const IPV6_2292PKTOPTIONS: i32 = 6;
//   149 #define IPV6_CHECKSUM       7
pub const IPV6_CHECKSUM: i32 = 7;
//   150 #define IPV6_2292HOPLIMIT   8
pub const IPV6_2292HOPLIMIT: i32 = 8;
//   151 #define IPV6_NEXTHOP        9
pub const IPV6_NEXTHOP: i32 = 9;
//   152 #define IPV6_AUTHHDR        10  /* obsolete */
pub const IPV6_AUTHHDR: i32 = 10;
//   153 #define IPV6_FLOWINFO       11
pub const IPV6_FLOWINFO: i32 = 11;
//   154
//   155 #define IPV6_UNICAST_HOPS   16
pub const IPV6_UNICAST_HOPS: i32 = 16;
//   156 #define IPV6_MULTICAST_IF   17
pub const IPV6_MULTICAST_IF: i32 = 17;
//   157 #define IPV6_MULTICAST_HOPS 18
pub const IPV6_MULTICAST_HOPS: i32 = 18;
//   158 #define IPV6_MULTICAST_LOOP 19
pub const IPV6_MULTICAST_LOOP: i32 = 19;
//   159 #define IPV6_ADD_MEMBERSHIP 20
pub const IPV6_ADD_MEMBERSHIP: i32 = 20;
//   160 #define IPV6_DROP_MEMBERSHIP    21
pub const IPV6_DROP_MEMBERSHIP: i32 = 21;
//   161 #define IPV6_ROUTER_ALERT   22
pub const IPV6_ROUTER_ALERT: i32 = 22;
//   162 #define IPV6_MTU_DISCOVER   23
pub const IPV6_MTU_DISCOVER: i32 = 23;
//   163 #define IPV6_MTU        24
pub const IPV6_MTU: i32 = 24;
//   164 #define IPV6_RECVERR        25
pub const IPV6_RECVERR: i32 = 25;
//   165 #define IPV6_V6ONLY     26
pub const IPV6_V6ONLY: i32 = 26;
//   166 #define IPV6_JOIN_ANYCAST   27
pub const IPV6_JOIN_ANYCAST: i32 = 27;
//   167 #define IPV6_LEAVE_ANYCAST  28
pub const IPV6_LEAVE_ANYCAST: i32 = 28;

//    15 #define SO_DEBUG    0x0001
pub const SO_DEBUG: i32 = 0x0001;
//    16 #define SO_REUSEADDR    0x0004
pub const SO_REUSEADDR: i32 = 0x0004;
//    17 #define SO_KEEPALIVE    0x0008
pub const SO_KEEPALIVE: i32 = 0x0008;
//    18 #define SO_DONTROUTE    0x0010
pub const SO_DONTROUTE: i32 = 0x0010;
//    19 #define SO_BROADCAST    0x0020
pub const SO_BROADCAST: i32 = 0x0020;
//    20 #define SO_LINGER   0x0080
pub const SO_LINGER: i32 = 0x0080;
//    21 #define SO_OOBINLINE    0x0100
pub const SO_OOBINLINE: i32 = 0x0100;
//    22 /* To add :#define SO_REUSEPORT 0x0200 */
//    23
//    24 #define SO_TYPE     0x1008
pub const SO_TYPE: i32 = 0x1008;
//    25 #define SO_ERROR    0x1007
pub const SO_ERROR: i32 = 0x1007;
//    26 #define SO_SNDBUF   0x1001
pub const SO_SNDBUF: i32 = 0x1001;
//    27 #define SO_RCVBUF   0x1002
pub const SO_RCVBUF: i32 = 0x1002;
//    28 #define SO_SNDBUFFORCE  0x100a
pub const SO_SNDBUFFORCE: i32 = 0x100a;
//    29 #define SO_RCVBUFFORCE  0x100b
pub const SO_RCVBUFFORCE: i32 = 0x100b;
//    30 #define SO_RCVLOWAT 0x1010
pub const SO_RCVLOWAT: i32 = 0x1010;
//    31 #define SO_SNDLOWAT 0x1011
pub const SO_SNDLOWAT: i32 = 0x1011;
//    32 #define SO_RCVTIMEO 0x1012
pub const SO_RCVTIMEO: i32 = 0x1012;
//    33 #define SO_SNDTIMEO 0x1013
pub const SO_SNDTIMEO: i32 = 0x1013;
//    34 #define SO_ACCEPTCONN   0x1014
pub const SO_ACCEPTCONN: i32 = 0x1014;
//    35 #define SO_PROTOCOL 0x1028
pub const SO_PROTOCOL: i32 = 0x1028;
//    36 #define SO_DOMAIN   0x1029
pub const SO_DOMAIN: i32 = 0x1029;
//    37
//    38 /* linux-specific, might as well be the same as on i386 */
//    39 #define SO_NO_CHECK 11
pub const SO_NO_CHECK: i32 = 11;
//    40 #define SO_PRIORITY 12
pub const SO_PRIORITY: i32 = 12;
//    41 #define SO_BSDCOMPAT    14
pub const SO_BSDCOMPAT: i32 = 14;
//    42
//    43 #define SO_PASSCRED 17
pub const SO_PASSCRED: i32 = 17;
//    44 #define SO_PEERCRED 18
pub const SO_PEERCRED: i32 = 18;
//    45 #define SO_BINDTODEVICE 25
pub const SO_BINDTODEVICE: i32 = 25;
//    46
//    47 /* Socket filtering */
//    48 #define SO_ATTACH_FILTER        26
pub const SO_ATTACH_FILTER: i32 = 26;
//    49 #define SO_DETACH_FILTER        27
pub const SO_DETACH_FILTER: i32 = 27;
//    50
//    51 #define SO_PEERNAME     28
pub const SO_PEERNAME: i32 = 28;
//    52 #define SO_TIMESTAMP        29
pub const SO_TIMESTAMP: i32 = 29;
//    53 #define SCM_TIMESTAMP       SO_TIMESTAMP
pub const SCM_TIMESTAMP: i32 = SO_TIMESTAMP;
//    54
//    55 #define SO_PEERSEC      30
pub const SO_PEERSEC: i32 = 30;
//    56 #define SO_PASSSEC      34
pub const SO_PASSSEC: i32 = 34;
//    57 #define SO_TIMESTAMPNS      35
pub const SO_TIMESTAMPNS: i32 = 35;

pub const SCM_TIMESTAMPNS: i32 = SO_TIMESTAMPNS;

pub const SO_MARK: i32 = 36;

pub const SO_TIMESTAMPING: i32 = 37;

pub const SCM_TIMESTAMPING: i32 = SO_TIMESTAMPING;

// pub const SO_PROTOCOL: i32 = 38;

// pub const SO_DOMAIN: i32 = 39;

pub const SO_RXQ_OVFL: i32 = 40;

pub const SO_WIFI_STATUS: i32 = 41;

pub const SCM_WIFI_STATUS: i32 = SO_WIFI_STATUS;

pub const SO_PEEK_OFF: i32 = 42;

pub const SO_NOFCS: i32 = 43;

pub const SO_LOCK_FILTER: i32 = 44;

pub const SO_SELECT_ERR_QUEUE: i32 = 45;

pub const SO_BUSY_POLL: i32 = 46;

pub const SOL_SOCKET: u16 = 0xffff;

// #define MAX_UDP_MSG 8192
pub const MAX_UDP_MSG: usize = 8192;

// #define MSG_OOB		1
pub const MSG_OOB: i32 = 1;
// #define MSG_PEEK	2
pub const MSG_PEEK: i32 = 2;
// #define MSG_DONTROUTE	4
pub const MSG_DONTROUTE: i32 = 4;
// #define MSG_TRYHARD     4       /* Synonym for MSG_DONTROUTE for DECnet */
pub const MSG_TRYHARD: i32 = 4;
// #define MSG_CTRUNC	8
pub const MSG_CTRUNC: i32 = 8;
// #define MSG_PROBE	0x10	/* Do not send. Only probe path f.e. for MTU */
pub const MSG_PROBE: i32 = 0x10;
// #define MSG_TRUNC	0x20
pub const MSG_TRUNC: i32 = 0x20;
// #define MSG_DONTWAIT	0x40	/* Nonblocking io		 */
pub const MSG_DONTWAIT: i32 = 0x40;
// #define MSG_EOR         0x80	/* End of record */
pub const MSG_EOR: i32 = 0x80;
// #define MSG_WAITALL	0x100	/* Wait for a full request */
pub const MSG_WAITALL: i32 = 0x100;
// #define MSG_FIN         0x200
pub const MSG_FIN: i32 = 0x200;
// #define MSG_SYN		0x400
pub const MSG_SYN: i32 = 0x400;
// #define MSG_CONFIRM	0x800	/* Confirm path validity */
pub const MSG_CONFIRM: i32 = 0x800;
// #define MSG_RST		0x1000
pub const MSG_RST: i32 = 0x1000;
// #define MSG_ERRQUEUE	0x2000	/* Fetch message from error queue */
pub const MSG_ERRQUEUE: i32 = 0x2000;
// #define MSG_NOSIGNAL	0x4000	/* Do not generate SIGPIPE */
pub const MSG_NOSIGNAL: i32 = 0x4000;
// #define MSG_MORE	0x8000	/* Sender will send more */
pub const MSG_MORE: i32 = 0x8000;

// #define MSG_WAITFORONE	0x10000	/* recvmmsg(): block until 1+ packets avail */
pub const MSG_WAITFORONE: i32 = 0x10000;
// #define MSG_SENDPAGE_NOPOLICY 0x10000 /* sendpage() internal : do no apply policy */
pub const MSG_SENDPAGE_NOPOLICY: i32 = 0x10000;
// #define MSG_BATCH	0x40000 /* sendmmsg(): more messages coming */
pub const MSG_BATCH: i32 = 0x40000;
// #define MSG_EOF         MSG_FIN
pub const MSG_EOF: i32 = MSG_FIN;
// #define MSG_NO_SHARED_FRAGS 0x80000 /* sendpage() internal : page frags are not shared */
pub const MSG_NO_SHARED_FRAGS: i32 = 0x80000;
// #define MSG_SENDPAGE_DECRYPTED	0x100000 /* sendpage() internal : page may carry
// 					  * plain text and require encryption
// 					  */
//
pub const MSG_SENDPAGE_DECRYPTED: i32 = 0x100000;
// #define MSG_ZEROCOPY	0x4000000	/* Use user data in kernel path */
pub const MSG_ZEROCOPY: i32 = 0x4000000;
// #define MSG_SPLICE_PAGES 0x8000000	/* Splice the pages from the iterator in sendmsg() */
pub const MSG_SPLICE_PAGES: i32 = 0x8000000;
// #define MSG_FASTOPEN	0x20000000	/* Send data in TCP SYN */
pub const MSG_FASTOPEN: i32 = 0x20000000;
// #define MSG_CMSG_CLOEXEC 0x40000000	/* Set close_on_exec for file
// 					   descriptor received through
// 					   SCM_RIGHTS */
pub const MSG_CMSG_CLOEXEC: i32 = 0x40000000;
// #if defined(CONFIG_COMPAT)
// #define MSG_CMSG_COMPAT	0x80000000	/* This message needs 32 bit fixups */
pub const MSG_CMSG_COMPAT: i32 = 0x80000000;
// #else
// #define MSG_CMSG_COMPAT	0		/* We never have 32 bit fixups */
// #endif
//
// /* Flags to be cleared on entry by sendmsg and sendmmsg syscalls */
// #define MSG_INTERNAL_SENDMSG_FLAGS \
// 	(MSG_SPLICE_PAGES | MSG_SENDPAGE_NOPOLICY | MSG_SENDPAGE_DECRYPTED)
pub const MSG_INTERNAL_SENDMSG_FLAGS: i32 = MSG_SPLICE_PAGES | MSG_SENDPAGE_NOPOLICY | MSG_SENDPAGE_DECRYPTED;

//
// IP_XXX
//

// #define IP_TOS		1
pub const IP_TOS: i32 = 1;
// #define IP_TTL		2
pub const IP_TTL: i32 = 2;
// #define IP_HDRINCL	3
pub const IP_HDRINCL: i32 = 3;
// #define IP_OPTIONS	4
pub const IP_OPTIONS: i32 = 4;
// #define IP_ROUTER_ALERT	5
pub const IP_ROUTER_ALERT: i32 = 5;
// #define IP_RECVOPTS	6
pub const IP_RECVOPTS: i32 = 6;
// #define IP_RETOPTS	7
pub const IP_RETOPTS: i32 = 7;
// #define IP_PKTINFO	8
pub const IP_PKTINFO: i32 = 8;
// #define IP_PKTOPTIONS	9
pub const IP_PKTOPTIONS: i32 = 9;
// #define IP_MTU_DISCOVER	10
pub const IP_MTU_DISCOVER: i32 = 10;
// #define IP_RECVERR	11
pub const IP_RECVERR: i32 = 11;
// #define IP_RECVTTL	12
pub const IP_RECVTTL: i32 = 12;
// #define	IP_RECVTOS	13
pub const IP_RECVTOS: i32 = 13;
// #define IP_MTU		14
pub const IP_MTU: i32 = 14;
// #define IP_FREEBIND	15
pub const IP_FREEBIND: i32 = 15;
// #define IP_IPSEC_POLICY	16
pub const IP_IPSEC_POLICY: i32 = 16;
// #define IP_XFRM_POLICY	17
pub const IP_XFRM_POLICY: i32 = 17;
// #define IP_PASSSEC	18
pub const IP_PASSSEC: i32 = 18;
// #define IP_TRANSPARENT	19
pub const IP_TRANSPARENT: i32 = 19;
//
// /* BSD compatibility */
// #define IP_RECVRETOPTS	IP_RETOPTS
pub const IP_RECVRETOPTS: i32 = IP_RETOPTS;
//
// /* TProxy original addresses */
// #define IP_ORIGDSTADDR       20
pub const IP_ORIGDSTADDR: i32 = 20;
// #define IP_RECVORIGDSTADDR   IP_ORIGDSTADDR
pub const IP_RECVORIGDSTADDR: i32 = IP_ORIGDSTADDR;
//
// #define IP_MINTTL       21
pub const IP_MINTTL: i32 = 21;
// #define IP_NODEFRAG     22
pub const IP_NODEFRAG: i32 = 22;
// #define IP_CHECKSUM	23
pub const IP_CHECKSUM: i32 = 23;
// #define IP_BIND_ADDRESS_NO_PORT	24
pub const IP_BIND_ADDRESS_NO_PORT: i32 = 24;
// #define IP_RECVFRAGSIZE	25
pub const IP_RECVFRAGSIZE: i32 = 25;
// #define IP_RECVERR_RFC4884	26
pub const IP_RECVERR_RFC4884: i32 = 26;
//
// /* IP_MTU_DISCOVER values */
// #define IP_PMTUDISC_DONT		0	/* Never send DF frames */
pub const IP_PMTUDISC_CONT: i32 = 0;
// #define IP_PMTUDISC_WANT		1	/* Use per route hints	*/
pub const IP_PMTUDISC_WANT: i32 = 1;
// #define IP_PMTUDISC_DO			2	/* Always DF		*/
pub const IP_PMTUDISC_DO: i32 = 2;
// #define IP_PMTUDISC_PROBE		3       /* Ignore dst pmtu      */
pub const IP_PMTUDISC_PROBE: i32 = 3;
// /* Always use interface mtu (ignores dst pmtu) but don't set DF flag.
//  * Also incoming ICMP frag_needed notifications will be ignored on
//  * this socket to prevent accepting spoofed ones.
//  */
// #define IP_PMTUDISC_INTERFACE		4
pub const IP_PMTUDISC_INTERFACE: i32 = 4;
// /* weaker version of IP_PMTUDISC_INTERFACE, which allows packets to get
//  * fragmented if they exeed the interface mtu
//  */
// #define IP_PMTUDISC_OMIT		5
pub const IP_PMTUDISC_OMIT: i32 = 5;
//
// #define IP_MULTICAST_IF			32
// pub const IP_MULTICAST_IF: i32 = 32;
// #define IP_MULTICAST_TTL 		33
// pub const IP_MULTICAST_TTL: i32 = 33;
// #define IP_MULTICAST_LOOP 		34
// pub const IP_MULTICAST_LOOP : i32 = 34;
// #define IP_ADD_MEMBERSHIP		35
// pub const IP_ADD_MEMBERSHIP: i32 = 35;
// #define IP_DROP_MEMBERSHIP		36
// #define IP_UNBLOCK_SOURCE		37
// #define IP_BLOCK_SOURCE			38
// #define IP_ADD_SOURCE_MEMBERSHIP	39
// #define IP_DROP_SOURCE_MEMBERSHIP	40
// #define IP_MSFILTER			41
pub const IP_MSFILTER: i32 = 41;
// #define MCAST_JOIN_GROUP		42
pub const MCAST_JOIN_GROUP: i32 = 42;
// #define MCAST_BLOCK_SOURCE		43
pub const MCAST_BLOCK_SOURCE: i32 = 43;
// #define MCAST_UNBLOCK_SOURCE		44
pub const MCAST_UNBLOCK_SOURCE: i32 = 44;
// #define MCAST_LEAVE_GROUP		45
pub const MCAST_LEAVE_GROUP: i32 = 45;
// #define MCAST_JOIN_SOURCE_GROUP		46
pub const MCAST_JOIN_SOURCE_GROUP: i32 = 46;
// #define MCAST_LEAVE_SOURCE_GROUP	47
pub const MCAST_LEAVE_SOURCE_GROUP: i32 = 47;
// #define MCAST_MSFILTER			48
pub const MCAST_MSFILTER: i32 = 48;
// #define IP_MULTICAST_ALL		49
pub const IP_MULTICAST_ALL: i32 = 49;
// #define IP_UNICAST_IF			50
pub const IP_UNICAST_IF: i32 = 50;
// #define IP_LOCAL_PORT_RANGE		51
pub const IP_LOCAL_PORT_RANGE: i32 = 51;
// #define IP_PROTOCOL			52
pub const IP_PROTOCOL: i32 = 52;
