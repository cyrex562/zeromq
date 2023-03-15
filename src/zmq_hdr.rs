use libc::c_void;

//   Version macros for compile-time API version detection
// #define ZMQ_VERSION_MAJOR 4
pub const ZMQ_VERSION_MAJOR: u32 = 4;
// #define ZMQ_VERSION_MINOR 3
pub const ZMQ_VERSION_MINOR: u32 = 4;
// #define ZMQ_VERSION_PATCH 5
pub const ZMQ_VERSION_PATCH: u32 = 5;

// #define ZMQ_MAKE_VERSION(major, minor, patch)                                  \
//     ((major) *10000 + (minor) *100 + (patch))
// #define ZMQ_VERSION                                                            \
//     ZMQ_MAKE_VERSION (ZMQ_VERSION_MAJOR, ZMQ_VERSION_MINOR, ZMQ_VERSION_PATCH)


// ****************************************************************************
//   0MQ errors.
// ****************************************************************************

//   A number random enough not to collide with different errno ranges on
//   different OSes. The assumption is that error_t is at least 32-bit type.
// #define ZMQ_HAUSNUMERO 156384712

//   On Windows platform some of the standard POSIX errnos are not defined.
// #ifndef ENOTSUP
// #define ENOTSUP (ZMQ_HAUSNUMERO + 1)
// #endif
// #ifndef EPROTONOSUPPORT
// #define EPROTONOSUPPORT (ZMQ_HAUSNUMERO + 2)
// #endif
// #ifndef ENOBUFS
// #define ENOBUFS (ZMQ_HAUSNUMERO + 3)
// #endif
// #ifndef ENETDOWN
// #define ENETDOWN (ZMQ_HAUSNUMERO + 4)
// #endif
// #ifndef EADDRINUSE
// #define EADDRINUSE (ZMQ_HAUSNUMERO + 5)
// #endif
// #ifndef EADDRNOTAVAIL
// #define EADDRNOTAVAIL (ZMQ_HAUSNUMERO + 6)
// #endif
// #ifndef ECONNREFUSED
// #define ECONNREFUSED (ZMQ_HAUSNUMERO + 7)
// #endif
// #ifndef EINPROGRESS
// #define EINPROGRESS (ZMQ_HAUSNUMERO + 8)
// #endif
// #ifndef ENOTSOCK
// #define ENOTSOCK (ZMQ_HAUSNUMERO + 9)
// #endif
// #ifndef EMSGSIZE
// #define EMSGSIZE (ZMQ_HAUSNUMERO + 10)
// #endif
// #ifndef EAFNOSUPPORT
// #define EAFNOSUPPORT (ZMQ_HAUSNUMERO + 11)
// #endif
// #ifndef ENETUNREACH
// #define ENETUNREACH (ZMQ_HAUSNUMERO + 12)
// #endif
// #ifndef ECONNABORTED
// #define ECONNABORTED (ZMQ_HAUSNUMERO + 13)
// #endif
// #ifndef ECONNRESET
// #define ECONNRESET (ZMQ_HAUSNUMERO + 14)
// #endif
// #ifndef ENOTCONN
// #define ENOTCONN (ZMQ_HAUSNUMERO + 15)
// #endif
// #ifndef ETIMEDOUT
// #define ETIMEDOUT (ZMQ_HAUSNUMERO + 16)
// #endif
// #ifndef EHOSTUNREACH
// #define EHOSTUNREACH (ZMQ_HAUSNUMERO + 17)
// #endif
// #ifndef ENETRESET
// #define ENETRESET (ZMQ_HAUSNUMERO + 18)
// #endif

//   Native 0MQ error codes.
// #define EFSM (ZMQ_HAUSNUMERO + 51)
// #define ENOCOMPATPROTO (ZMQ_HAUSNUMERO + 52)
// #define ETERM (ZMQ_HAUSNUMERO + 53)
// #define EMTHREAD (ZMQ_HAUSNUMERO + 54)

// ****************************************************************************
//   0MQ infrastructure (a.k.a. context) initialisation & termination.
// ****************************************************************************

//   Context options
pub const ZMQ_IO_THREADS: i32 = 1;
pub const ZMQ_MAX_SOCKETS: u8 = 2;
pub const ZMQ_SOCKET_LIMIT: u8 = 3;
pub const ZMQ_THREAD_PRIORITY: u8 = 3;
pub const ZMQ_THREAD_SCHED_POLICY: u8 = 4;
pub const ZMQ_MAX_MSGSZ: u8 = 5;
pub const ZMQ_MESSAGE_SIZE: u8 = 6;
pub const ZMQ_THREAD_AFFINITY_CPU_ADD: u8 = 7;
pub const ZMQ_THREAD_AFFINITY_CPU_REMOVE: u8 = 8;
pub const ZMQ_THREAD_NAME_PREFIX: u8 = 9;

//   Default for new contexts
pub const ZMQ_IO_THREADS_DFLT: i32 =  1;
pub const ZMQ_MAX_SOCKETS_DFLT: i32 = 1023;
pub const ZMQ_THREAD_PRIORITY_DFLT: i32 = -1;
pub const ZMQ_THREAD_SCHED_POLICY_DFLT: i32 = -1;



// ****************************************************************************
//   0MQ message definition.
// ****************************************************************************

// /* Some architectures, like sparc64 and some variants of aarch64, enforce pointer
//  * alignment and raise sigbus on violations. Make sure applications allocate
//  * zmq_ZmqMessage on addresses aligned on a pointer-size boundary to avoid this issue.
//  */
// typedef struct zmq_ZmqMessage
// #[derive(Default,Debug,Clone)]
// pub struct ZmqMessage {
//     // #if defined(_MSC_VER) && (defined(_M_X64) || defined(_M_ARM64))
// //     __declspec(align (8)) unsigned char _[64];
// // #elif defined(_MSC_VER)                                                        \
// //   && (defined(_M_IX86) || defined(_M_ARM_ARMV7VE) || defined(_M_ARM))
// //     __declspec(align (4)) unsigned char _[64];
// // #elif defined(__GNUC__) || defined(__INTEL_COMPILER)                           \
// //   || (defined(__SUNPRO_C) && __SUNPRO_C >= 0x590)                              \
// //   || (defined(__SUNPRO_CC) && __SUNPRO_CC >= 0x590)
// //     unsigned char _[64] __attribute__ ((aligned (sizeof (void *))));
// // #else
// //     unsigned char _[64];
// // #endif
//     pub _x: [u8; 64],
// }
// zmq_ZmqMessage;

// ****************************************************************************
//   0MQ socket definition.
// ****************************************************************************

//   Socket types.
pub const ZMQ_PAIR: i32 = 0;
pub const ZMQ_PUB: u8 = 1;
pub const ZMQ_SUB: u8 = 2;
pub const ZMQ_REQ: u8 = 3;
pub const ZMQ_REP: u8 = 4;
pub const ZMQ_DEALER: u8 = 5;
pub const ZMQ_ROUTER: u8 = 6;
pub const ZMQ_PULL: u8 = 7;
pub const ZMQ_PUSH: u8 = 8;
pub const ZMQ_XPUB: u8 = 9;
pub const ZMQ_XSUB: u8 = 10;
pub const ZMQ_STREAM: u8 = 11;

//   Deprecated aliases
pub const ZMQ_XREQ: u8 = ZMQ_DEALER;
pub const ZMQ_XREP: u8 = ZMQ_ROUTER;

//   Socket options.
pub const ZMQ_AFFINITY: u8 = 4;
pub const ZMQ_ROUTING_ID: u8 = 5;
pub const ZQM_SUBSCRIBE: u8 = 6;
pub const ZMQ_UNSUBSCRIBE: u8 = 7;
pub const ZMQ_RATE: u8 = 8;
pub const ZMQ_RECOVERY_IVL: u8 = 9;
pub const ZMQ_SNDBUF: u8 = 11;
pub const ZMQ_RCVBUF: u8 = 12;
pub const ZMQ_RCVMORE: u8 = 13;
pub const ZMQ_FD: u8 = 14;
pub const ZMQ_EVENTS: u8 = 15;
pub const ZMQ_TYPE: u8 = 16;
pub const ZMQ_LINGER: i32 = 17;
pub const ZMQ_RECONNECT_IVL: u8 = 18;
pub const ZMQ_BACKLOG: u8 = 19;
pub const ZMQ_RECONNECT_IVL_MAX: u8 = 21;
pub const ZMQ_MAXMSGSIZE: u8 = 22;
pub const ZMQ_SNDHWM: i32 = 23;
pub const ZMQ_RCVHWM: u8 = 24;
pub const ZMQ_MULTICAST_HOPS: u8 = 25;
pub const ZMQ_RCVTIMEO: u8 = 27;
pub const ZMQ_SNDTIMEO: u8 = 28;
pub const ZMQ_LAST_ENDPOINT: u8 = 32;
pub const ZMQ_ROUTER_MANDATORY: u8 = 33;
pub const ZMQ_TCP_KEEPALIVE: u8 = 34;
pub const ZMQ_TCP_KEEPALIVE_CNT: u8 = 35;
pub const ZMQ_TCP_KEEPALIVE_IDLE: u8 = 36;
pub const ZMQ_TCP_KEEPALIVE_INTVL: u8 = 37;
pub const ZMQ_IMMEDIATE: u8 = 39;
pub const ZMQ_XPUB_VERBOSE: u8 = 40;
pub const ZMQ_ROUTER_RAW: u8 = 41;
pub const ZMQ_IPV6: u8 = 42;
pub const ZMQ_MECHANISM: u8 = 43;
pub const ZMQ_PLAIN_SERVER: u8 = 44;
pub const ZMQ_PLAIN_USERNAME: u8 = 45;
pub const ZMQ_PLAIN_PASSWORD: u8 = 46;
pub const ZMQ_CURVE_SERVER: u8 = 47;
pub const ZMQ_CURVE_PUBLICKEY: u8 = 48;
pub const ZMQ_CURVE_SECRETKEY: u8 = 49;
pub const ZMQ_CURVE_SERVERKEY: u8 = 50;
pub const ZMQ_PROBE_ROUTER: u8 = 51;
pub const ZMQ_REQ_CORRELATE: u8 = 52;
pub const ZMQ_REQ_RELAXED: u8 = 53;
pub const ZMQ_CONFLATE: u8 = 54;
pub const ZMQ_ZAP_DOMAIN: u8 = 55;
pub const ZMQ_ROUTER_HANDOVER: u8 = 56;
pub const ZMQ_TOS: u8 = 57;
pub const ZMQ_CONNECT_ROUTING_ID: i32 = 61;
pub const ZMQ_GSSAPI_SERVER: u8 = 62;
pub const ZMQ_GSSAPI_PRINCIPAL: u8 = 63;
pub const ZMQ_GSSAPI_SERVICE_PRINCIPAL: u8 = 64;
pub const ZMQ_GSSAPI_PLAINTEXT: u8 = 65;
pub const ZMQ_HANDSHAKE_IVL: u8 = 66;
pub const ZMQ_SOCKS_USERNAME: i32 = 67;
pub const ZMQ_SOCKS_PROXY: u8 = 68;
pub const ZMQ_XPUB_NODROP: u8 = 69;
pub const ZMQ_BLOCKY: u8 = 70;
pub const ZMQ_XPUB_MANUAL: u8 = 71;
pub const ZMQ_XPUB_WELCOME_MSG: u8 = 72;
pub const ZMQ_STREAM_NOTIFY: u8 = 73;
pub const ZMQ_INVERT_MATCHING: u8 = 74;
pub const ZMQ_HEARTBEAT_IVL: u8 = 75;
pub const ZMQ_HEARTBEAT_TTL: u8 = 76;
pub const ZMQ_HEARTBEAT_TIMEOUT: u8 = 77;
pub const ZMQ_XPUB_VERBOSER: u8 = 78;
pub const ZMQ_CONNECT_TIMEOUT: u8 = 79;
pub const ZMQ_TCP_MAXRT: u8 = 80;
pub const ZMQ_THREAD_SAFE: u8 = 81;
pub const ZMQ_MULTICAST_MAXTPDU: u8 = 84;
pub const ZMQ_VMCI_BUFFER_SIZE: u8 = 85;
pub const ZMQ_VMCI_BUFFER_MIN_SIZE: u8 = 86;
pub const ZMQ_VMCI_BUFFER_MAX_SIZE: u8 = 87;
pub const ZMQ_VMCI_CONNECT_TIMEOUT: u8 = 88;
pub const ZMQ_USE_FD: u8 = 89;
pub const ZMQ_GSSAPI_PRINCIPAL_NAMETYPE: u8 = 90;
pub const ZMQ_GSSAPI_SERVICE_PRINCIPAL_NAMETYPE: u8 = 91;
pub const ZMQ_BINDTODEVICE: u8 = 92;

//   Message options
pub const ZMQ_MORE: u8 = 1;
pub const ZMQ_SHARED: u8 = 3;

//   Send/recv options.
pub const ZMQ_DONTWAIT: u8 = 1;
pub const ZMQ_SNDMORE: i32 = 2;

//   Security mechanisms
pub const ZMQ_NULL: u8 = 0;
pub const ZMQ_PLAIN: u8 = 1;
pub const ZMQ_CURVE: u8 = 2;
pub const ZMQ_GSSAPI: u8 = 3;

//   RADIO-DISH protocol
pub const ZMQ_GROUP_MAX_LENGTH: usize = 255;

//   Deprecated options and aliases
pub const ZMQ_IDENTITY: u8 = ZMQ_ROUTING_ID;
pub const ZMQ_CONNECT_RID: u8 = ZMQ_CONNECT_ROUTING_ID;
pub const ZMQ_TCP_ACCEPT_FILTER: u8 = 38;
pub const ZMQ_IPC_FILTER_PID: u8 = 58;
pub const ZMQ_IPC_FILTER_UID: u8 = 59;
pub const ZMQ_IPC_FILTER_GID: u8 = 60;
pub const ZMQ_IPV4ONLY: u8 = 31;
pub const ZMQ_DELAY_ATTACH_ON_CONNECT: u8 = ZMQ_IMMEDIATE;
pub const ZMQ_NONBLOCK: u8 = ZMQ_DONTWAIT;
pub const ZMQ_FAIL_UNROUTABLE: u8 = ZMQ_ROUTER_MANDATORY;
pub const ZMQ_ROUTER_BEHAVIOR: u8 = ZMQ_ROUTER_MANDATORY;

//   Deprecated Message options
pub const ZMQ_SRCFD: u8 = 2;

// ****************************************************************************
//   GSSAPI definitions
// ****************************************************************************

//   GSSAPI principal name types
pub const ZMQ_GSSAPI_NT_HOSTBASED: u8 = 0;
pub const ZMQ_GSSAPI_NT_USER_NAME: u8 = 1;
pub const ZMQ_GSSAPI_NT_KRB5_PRINCIPAL: u8 = 2;

// ****************************************************************************
//   0MQ socket events and monitoring
// ****************************************************************************

//  Socket transport events (TCP, IPC and TIPC only)
pub const ZMQ_EVENT_CONNECTED: u16 = 0x0001;
pub const ZMQ_EVENT_CONNECT_DELAYED: u16 = 0x0002;
pub const ZMQ_EVENT_CONNECT_RETRIED: u16 = 0x0004;
pub const ZMQ_EVENT_LISTENING: u16 = 0x0008;
pub const ZMQ_EVENT_BIND_FAILED: u16 = 0x0010;
pub const ZMQ_EVENT_ACCEPTED: u16 = 0x0020;
pub const ZMQ_EVENT_ACCEPT_FAILED: u16 = 0x0040;
pub const ZMQ_EVENT_CLOSED: u16 = 0x0080;
pub const ZMQ_EVENT_CLOSE_FAILED: u16 = 0x0100;
pub const ZMQ_EVENT_DISCONNECTED: u16 = 0x0200;
pub const ZMQ_EVENT_MONITOR_STOPPED: u16 = 0x0400;
pub const ZMQ_EVENT_ALL: u16 = 0xffff;
//   Unspecified system errors during handshake. Event value is an errno.
pub const ZMQ_EVENT_HANDSHAKE_FAILED_NO_DETAIL: u16 = 0x0800;
// /*  Handshake complete successfully with successful authentication (if        *
//  *  enabled). Event value is unused.                                          */
pub const ZMQ_EVENT_HANDSHAKE_SUCCEEDED: u16 = 0x1000;
// /*  Protocol errors between ZMTP peers or between server and ZAP handler.     *
//  *  Event value is one of ZMQ_PROTOCOL_ERROR_*                                */
pub const ZMQ_EVENT_HANDSHAKE_FAILED_PROTOCOL: u16 = 0x2000;
// /*  Failed authentication requests. Event value is the numeric ZAP status     *
//  *  code, i.e. 300, 400 or 500.                                               */
pub const ZMQ_EVENT_HANDSHAKE_FAILED_AUTH: u16 = 0x4000;
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

// #if defined _WIN32
// Windows uses a pointer-sized unsigned integer to store the socket fd.
// #if defined _WIN64
// typedef unsigned __int64 zmq_fd_t;
// #else
// #[cfg(all(windows, target_pointer_width = "64"))]
// type zmq_fd_t = u64;

// // typedef unsigned int zmq_fd_t;
// #[cfg(all(windows, target_pointer_width = "32"))]
// type zmq_fd_t = u32;
// // #endif
// // #else
// // typedef int zmq_fd_t;
// #[cfg(not(windows))]
// type zmq_fd_t = i32;
// // #endif

pub const ZMQ_POLLIN: u8 = 1;
pub const ZMQ_POLLOUT: u8 = 2;
pub const ZMQ_POLLERR: u8 = 4;
pub const ZMQ_POLLPRI: u8 = 8;

pub const ZMQ_POLLITEMS_DFLT: usize = 16;

pub const ZMQ_HAS_CAPABILITIES: i32 = 1;
pub const ZMQ_STREAMER: u8 = 1;
pub const ZMQ_FORWARDER: u8 = 2;
pub const ZMQ_QUEUE: u8 = 3;


pub type zmq_timer_fn = fn(timer_id: i32, arg: &mut [u8]);

pub type zmq_thread_fn = fn(&mut[u8]);

//  DRAFT Socket types.
pub const ZMQ_SERVER: u8 = 12;
pub const ZMQ_CLIENT: u8 = 13;
pub const ZMQ_RADIO: u8 = 14;
pub const ZMQ_DISH: u8 = 15;
pub const ZMQ_GATHER: u8 = 16;
pub const ZMQ_SCATTER: u8 = 17;
pub const ZMQ_DGRAM: u8 = 18;
pub const ZMQ_PEER: i32 = 19;
pub const ZMQ_CHANNEL: u8 = 20;

//  DRAFT Socket options.
pub const ZMQ_ZAP_ENFORCE_DOMAIN: u8 = 93;
pub const ZMQ_LOOPBACK_FASTPATH: u8 = 94;
pub const ZMQ_METADATA: u8 = 95;
pub const ZMQ_MULTICAST_LOOP: u8 = 96;
pub const ZMQ_ROUTER_NOTIFY: u8 = 97;
pub const ZMQ_XPUB_MANUAL_LAST_VALUE: u8 = 98;
pub const ZMQ_SOCKS_USSERNAME: u8 = 99;
pub const ZMQ_SOCKS_PASSWORD: u8 = 100;
pub const ZMQ_IN_BATCH_SIZE: u8 = 101;
pub const ZMQ_OUT_BATCH_SIZE: u8 = 102;
pub const ZMQ_WSS_KEY_PEM: u8 = 103;
pub const ZMQ_WSS_CERT_PEM: u8 = 104;
pub const ZMQ_WSS_TRUST_PEM: u8 = 105;
pub const ZMQ_WSS_HOSTNAME: u8 = 106;
pub const ZMQ_WSS_TRUST_SYSTEM: u8 = 107;
pub const ZMQ_ONLY_FIRST_SUBSCRIBE: u8 = 108;
pub const ZMQ_RECONNECT_STOP: u8 = 109;
pub const ZMQ_HELLO_MSG: u8 = 110;
pub const ZMQ_DISCONNECT_MSG: u8 = 111;
pub const ZMQ_PRIORITY: u8 = 112;
pub const ZMQ_BUSY_POLL: u8 = 113;
pub const ZMQ_HICCUP_MSG: u8 = 114;
pub const ZMQ_XSUB_VERBOSE_UNSUBSCRIBE: u8 = 115;
pub const ZMQ_TOPICS_COUNT: u8 = 116;

//  DRAFT ZMQ_RECONNECT_STOP options
pub const ZMQ_RECONNECT_STOP_CONN_REFUSED: u8 = 0x1;
pub const ZMQ_RECONNECT_STOP_HANDSHAKE_FAILED: u8 = 0x2;
pub const ZMQ_RECONNECT_STOP_AFTER_DISCONNECT: u8 = 0x4;

//  DRAFT Context options
pub const ZMQ_ZERO_COPY_RECV: i32 = 10;

//  DRAFT Msg property names.
pub const ZMQ_MSG_PROPERTY_ROUTING_ID: &'static str = "Routing-Id";
pub const ZMQ_MSG_PROPERTY_SOCKET_TYPE: &'static str = "Socket-Type";
pub const ZMQ_MSG_PROPERTY_USER_ID: &'static str = "User-Id";
pub const ZMQ_MSG_PROPERTY_PEER_ADDRESS: &'static str = "Peer-Address";

//  Router notify options
pub const ZMQ_NOTIFY_CONNECT: i32 = 1;
pub const ZMQ_NOTIFY_DISCONNECT: i32 = 2;

//  DRAFT Socket monitoring events
pub const ZMQ_EVENT_PIPES_STATS: u32 = 0x10000;

pub const ZMQ_CURRENT_EVENT_VERSION: u32 = 1;
pub const ZMQ_CURRENT_EVENT_VERSION_DRAFT: u32 = 2;

pub const ZMQ_EVENT_ALL_V1: u32 = ZMQ_EVENT_ALL as u32;
pub const ZMQ_EVENT_ALL_V2: u32 = ZMQ_EVENT_ALL_V1 | ZMQ_EVENT_PIPES_STATS;
