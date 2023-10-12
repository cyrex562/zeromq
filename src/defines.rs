#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use std::ffi::c_void;
use std::os::windows::raw::HANDLE;
use windows::Win32::Networking::WinSock::sa_family_t;

#[cfg(target_os = "windows")]
#[cfg(target_arch = "x86_64")]
pub type zmq_fd_t = u64;
#[cfg(target_os = "windows")]
#[cfg(target_arch = "x86")]
pub type zmq_fd_t = u32;
#[cfg(target_os = "linux")]
pub type zmq_fd_t = i32;


/*  Socket options.                                                           */
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


/*  Message options                                                           */
pub const ZMQ_MORE: u32 = 1;
pub const ZMQ_SHARED: u32 = 3;

/*  Send/recv options.                                                        */
pub const ZMQ_DONTWAIT: u32 = 1;
pub const ZMQ_SNDMORE: u32 = 2;

/*  Security mechanisms                                                       */
pub const ZMQ_NULL: i32 = 0;
pub const ZMQ_PLAIN: u32 = 1;
pub const ZMQ_CURVE: u32 = 2;
pub const ZMQ_GSSAPI: u32 = 3;

/*  RADIO-DISH protocol                                                       */
pub const ZMQ_GROUP_MAX_LENGTH: usize = 255;

/*  Deprecated options and aliases                                            */
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

/*  Deprecated Message options                                                */
pub const ZMQ_SRCFD: u32 = 2;

pub const ZMQ_GSSAPI_NT_HOSTBASED: i32 = 0;
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
/*  Unspecified system errors during handshake. Event value is an errno.      */
pub const ZMQ_EVENT_HANDSHAKE_FAILED_NO_DETAIL: u32 = 0x0800;
/*  Handshake complete successfully with successful authentication (if        *
 *  enabled). Event value is unused.                                          */
pub const ZMQ_EVENT_HANDSHAKE_SUCCEEDED: u32 = 0x1000;
/*  Protocol errors between ZMTP peers or between server and ZAP handler.     *
 *  Event value is one of ZMQ_PROTOCOL_ERROR_*                                */
pub const ZMQ_EVENT_HANDSHAKE_FAILED_PROTOCOL: u32 = 0x2000;
/*  Failed authentication requests. Event value is the numeric ZAP status     *
 *  code, i.e. 300, 400 or 500.                                               */
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

pub const ZMQ_HAS_CAPABILITIES: u32 = 1;

pub const ZMQ_STREAMER: u32 = 1;
pub const ZMQ_FORWARDER: u32 = 2;
pub const ZMQ_QUEUE: u32 = 3;


/*  DRAFT Socket types.                                                       */
pub const ZMQ_SERVER: u32 = 12;
pub const ZMQ_CLIENT: u32 = 13;
pub const ZMQ_RADIO: u32 = 14;
pub const ZMQ_DISH: u32 = 15;
pub const ZMQ_GATHER: u32 = 16;
pub const ZMQ_SCATTER: u32 = 17;
pub const ZMQ_DGRAM: u32 = 18;
pub const ZMQ_PEER: u32 = 19;
pub const ZMQ_CHANNEL: i8 = 20;

/*  DRAFT Socket options.                                                     */
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

/*  DRAFT ZMQ_NORM_MODE options                                               */
pub const ZMQ_NORM_FIXED: u32 = 0;
pub const ZMQ_NORM_CC: u32 = 1;
pub const ZMQ_NORM_CCL: u32 = 2;
pub const ZMQ_NORM_CCE: u32 = 3;
pub const ZMQ_NORM_CCE_ECNONLY: u32 = 4;

/*  DRAFT ZMQ_RECONNECT_STOP options                                          */
pub const ZMQ_RECONNECT_STOP_CONN_REFUSED: u32 = 0x1;
pub const ZMQ_RECONNECT_STOP_HANDSHAKE_FAILED: u32 = 0x2;
pub const ZMQ_RECONNECT_STOP_AFTER_DISCONNECT: u32 = 0x4;

/*  DRAFT Context options                                                     */
pub const ZMQ_ZERO_COPY_RECV: u32 = 10;


//  Number of new messages in message pipe needed to trigger new memory
//  allocation. Setting this parameter to 256 decreases the impact of
//  memory allocation by approximately 99.6%
pub const message_pipe_granularity: usize = 256;

//  Commands in pipe per allocation event.
pub const command_pipe_granularity: u32 = 16;

//  Determines how often does socket poll for new commands when it
//  still has unprocessed messages to handle. Thus, if it is set to 100,
//  socket will process 100 inbound messages before doing the poll.
//  If there are no unprocessed messages available, poll is done
//  immediately. Decreasing the value trades overall latency for more
//  real-time behaviour (less latency peaks).
pub const inbound_poll_rate: u32 = 100;

//  Maximal delta between high and low watermark.
pub const max_wm_delta: u32 = 1024;

//  Maximum number of events the I/O thread can process in one go.
pub const max_io_events: u32 = 256;

//  Maximal batch size of packets forwarded by a ZMQ proxy.
//  Increasing this value improves throughput at the expense of
//  latency and fairness.
pub const proxy_burst_size: u32 = 1000;

//  Maximal delay to process command in API thread (in CPU ticks).
//  3,000,000 ticks equals to 1 - 2 milliseconds on current CPUs.
//  Note that delay is only applied when there is continuous stream of
//  messages to process. If not so, commands are processed immediately.
pub const max_command_delay: u32 = 3000000;

//  Low-precision clock precision in CPU ticks. 1ms. Value of 1000000
//  should be OK for CPU frequencies above 1GHz. If should work
//  reasonably well for CPU frequencies above 500MHz. For lower CPU
//  frequencies you may consider lowering this value to get best
//  possible latencies.
pub const clock_precision: u32 = 1000000;

//  On some OSes the signaler has to be emulated using a TCP
//  connection. In such cases following port is used.
//  If 0, it lets the OS choose a free port without requiring use of a
//  global mutex. The original implementation of a Windows signaler
//  socket used port 5905 instead of letting the OS choose a free port.
//  https://github.com/zeromq/libzmq/issues/1542
pub const signaler_port: u32 = 0;

pub  type handle_t = *mut c_void;

pub type fd_t = handle_t;

pub type WSAEVENT = HANDLE;


//     9 #define _NSIG       64
pub const _NSIG: u32 = 64;

//    10 #define _NSIG_BPW   32v
pub const _NSIG_BPW: u32 = 32;
//    11 #define _NSIG_WORDS (_NSIG / _NSIG_BPW)
pub const _NSIG_WORDS: u32 = _NSIG / _NSIG_BPW;
//    12
//    13 typedef unsigned long old_sigset_t;     /* at least 32 bits */
pub type old_sigset_t = u32;
//    14
//    15 typedef struct {
//    16     unsigned long sig[_NSIG_WORDS];
//    17 } sigset_t;
pub struct sigset_t {
    pub sig: [u32; _NSIG_WORDS as usize],
}
//    18
//    19 struct old_sigaction {
//    20     __sighandler_t sa_handler;
//    21     old_sigset_t sa_mask;
//    22     unsigned long sa_flags;
//    23     __sigrestore_t sa_restorer;
//    24 };


// struct sockaddr_storage {
//            sa_family_t     ss_family;      /* Address family */
//        };
#[derive(Default,Debug,Clone)]
pub struct sockaddr_storage {
    pub ss_family: sa_family_t,
}
