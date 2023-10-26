use std::intrinsics::size_of;
use std::os::raw::{c_short, c_void};
use libc::{EINTR, timespec, timeval};
use windows::Win32::Networking::WinSock::{FD_SET, POLLIN, POLLOUT, POLLPRI};
use windows::Win32::System::Threading::Sleep;
use crate::clock::ZmqClock;
use crate::ctx::ZmqContext;
use crate::fd::fd_t;
use crate::msg::MSG_SHARED;
use crate::options::ZmqOptions;
use crate::polling_util::{OptimizedFdSet, valid_pollset_bytes};
use crate::select::{FD_SET, FD_ZERO};
use crate::socket_base::ZmqSocketBase;
use crate::socket_poller::ZmqSocketPoller;
use crate::timers::Timers;
use crate::utils::{FD_ISSET, get_errno};
use crate::zmq_draft::zmq_fd_t;

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

// /* Some architectures, like sparc64 and some variants of aarch64, enforce pointer
//  * alignment and raise sigbus on violations. Make sure applications allocate
//  * zmq_msg_t on addresses aligned on a pointer-size boundary to avoid this issue.
//  */
pub struct zmq_msg_t {
    // #if defined(_MSC_VER) && (defined(_M_X64) || defined(_M_ARM64))
//     __declspec(align (8)) unsigned char _[64];
// #elif defined(_MSC_VER)                                                        \
//   && (defined(_M_IX86) || defined(_M_ARM_ARMV7VE) || defined(_M_ARM))
//     __declspec(align (4)) unsigned char _[64];
// #elif defined(__GNUC__) || defined(__INTEL_COMPILER)                           \
//   || (defined(__SUNPRO_C) && __SUNPRO_C >= 0x590)                              \
//   || (defined(__SUNPRO_CC) && __SUNPRO_CC >= 0x590)
//     unsigned char _[64] __attribute__ ((aligned (sizeof (void *))));
// #else
//     unsigned char _[64];
// #endif
    pub _x: [u8; 64],
}

/*  Socket types.                                                             */
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

/*  Deprecated aliases                                                        */
pub const ZMQ_XREQ: u32 = ZMQ_DEALER;
pub const ZMQ_XREP: u32 = ZMQ_ROUTER;

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

/*  Message options                                                           */
pub const ZMQ_MORE: u32 = 1;
pub const ZMQ_SHARED: u32 = 3;

/*  Send/recv options.                                                        */
pub const ZMQ_DONTWAIT: u32 = 1;
pub const ZMQ_SNDMORE: u32 = 2;

/*  Security mechanisms                                                       */
pub const ZMQ_NULL: u32 = 0;
pub const ZMQ_PLAIN: u32 = 1;
pub const ZMQ_CURVE: u32 = 2;
pub const ZMQ_GSSAPI: u32 = 3;

/*  RADIO-DISH protocol                                                       */
pub const ZMQ_GROUP_MAX_LENGTH: u32 = 255;

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

/******************************************************************************/
/*  GSSAPI definitions                                                        */
/******************************************************************************/

/*  GSSAPI principal name types                                               */
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

// #if defined _WIN32
// // Windows uses a pointer-sized unsigned integer to store the socket fd.
// #if defined _WIN64
// typedef unsigned __int64 zmq_fd_t;
// #else
// typedef unsigned int zmq_fd_t;
// #endif
// #else
// typedef int zmq_fd_t;
// #endif

pub const ZMQ_POLLIN: u32 = 1;
pub const ZMQ_POLLOUT: u32 = 2;
pub const ZMQ_POLLERR: u32 = 4;
pub const ZMQ_POLLPRI: u32 = 8;

pub struct zmq_pollitem_t {
    pub socket: *mut c_void,
    pub fd: zmq_fd_t,
    pub events: c_short,
    pub revents: c_short,
}

pub const ZMQ_POLLITEMS_DFLT: u32 = 16;

pub const ZMQ_HAS_CAPABILITIES: u32 = 1;

/*  Deprecated aliases */
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
pub const ZMQ_CHANNEL: u32 = 20;

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

/*  DRAFT Msg property names.                                                 */
pub const ZMQ_MSG_PROPERTY_ROUTING_ID: &'static str = "Routing-Id";
pub const ZMQ_MSG_PROPERTY_SOCKET_TYPE: &'static str = "Socket-Type";
pub const ZMQ_MSG_PROPERTY_USER_ID: &'static str = "User-Id";
pub const ZMQ_MSG_PROPERTY_PEER_ADDRESS: &'static str = "Peer-Address";

/*  Router notify options                                                     */
pub const ZMQ_NOTIFY_CONNECT: u32 = 1;
pub const ZMQ_NOTIFY_DISCONNECT: u32 = 2;

pub struct zmq_poller_event_t {
    pub socket: *mut c_void,
    pub fd: zmq_fd_t,
    pub user_data: *mut c_void,
    pub events: i16,
}

/*  DRAFT Socket monitoring events                                            */
pub const ZMQ_EVENT_PIPES_STATS: u32 = 0x10000;

pub const ZMQ_CURRENT_EVENT_VERSION: u32 = 1;
pub const ZMQ_CURRENT_EVENT_VERSION_DRAFT: u32 = 2;

pub const ZMQ_EVENT_ALL_V1: u32 = ZMQ_EVENT_ALL;
pub const ZMQ_EVENT_ALL_V2: u32 = ZMQ_EVENT_ALL_V1 | ZMQ_EVENT_PIPES_STATS;

//  Compile time check whether msg_t fits into zmq_msg_t.
// typedef char
//   check_msg_t_size[sizeof (zmq::msg_t) == sizeof (zmq_msg_t) ? 1 : -1];

pub fn zmq_version() -> (u32, u32, u32) {
    (ZMQ_VERSION_MAJOR, ZMQ_VERSION_MINOR, ZMQ_VERSION_PATCH)
}

// const char *zmq_strerror (int errnum_)
pub fn zmq_strerror(errnum_: i32) -> &'static str {
    // return zmq::errno_to_string (errnum_);
    todo!()
}

// int zmq_errno (void)
pub fn zmq_errno() -> i32 {
    // return errno;
    todo!()
}

// void *zmq_ctx_new (void)
pub fn zmq_ctx_new() -> Option<ZmqContext> {
    //  We do this before the ctx constructor since its embedded mailbox_t
    //  object needs the network to be up and running (at least on Windows).
    if (!initialize_network()) {
        return None;
    }

    //  Create 0MQ context.
    // zmq::ctx_t *ctx = new (std::nothrow) zmq::ctx_t;
    let mut ctx = ZmqContext::new();
    if (ctx) {
        if (!ctx.valid()) {
            // delete ctx;
            return None;
        }
    }
    return Some(ctx);
}

// pub zmq_ctx_term (void *ctx_)
pub fn zmq_ctx_term(ctx_: &mut ZmqContext) -> i32 {
    // if (!ctx_ || !(static_cast<zmq::ctx_t *> (ctx_))->check_tag ()) {
    if ctx_.check_tag() == false {
        // errno = EFAULT;
        return -1;
    }

    // const int rc = (static_cast<zmq::ctx_t *> (ctx_))->terminate ();
    let rc = ctx_.terminate();
    // const int en = errno;
    let en = get_errno();

    //  Shut down only if termination was not interrupted by a signal.
    if (!rc || en != EINTR) {
        shutdown_network();
    }

    // errno = en;
    return rc;
}

// int zmq_ctx_shutdown (void *ctx_)
pub fn zmq_ctx_shutdown(ctx_: &mut ZmqContext) -> i32 {
    // if (!ctx_ || !(static_cast<zmq::ctx_t *> (ctx_))->check_tag ())
    if ctx_.check_tag() == false {
        // errno = EFAULT;
        return -1;
    }
    // return (static_cast<zmq::ctx_t *> (ctx_))->shutdown ();
    ctx_.shutdown()
}

// int zmq_ctx_set (void *ctx_, int option_, int optval_)
pub fn zmq_ctx_set(ctx_: &mut ZmqContext, option_: i32, optval_: i32) -> i32 {
    zmq_ctx_set_ext(ctx_, option_, &optval_, 4)
}

pub fn zmq_ctx_set_ext(ctx_: &mut ZmqContext, option_: i32, optval_: &[u8], optvallen_: usize) -> i32 {
    // if (!ctx_ || !(static_cast<zmq::ctx_t *> (ctx_))->check_tag ()) {
    //         errno = EFAULT;
    //         return -1;
    //     }
    //     return (static_cast<zmq::ctx_t *> (ctx_))
    //       ->set (option_, optval_, optvallen_);
    if ctx_.check_tag() == false {
        // errno = EFAULT;
        return -1;
    }
    ctx_.set(option_, optval_, optvallen_)
}

// int zmq_ctx_get (void *ctx_, int option_)
pub fn zmq_ctx_get(ctx_: &mut ZmqContext, option_: i32) -> i32 {
    // if (!ctx_ || !(static_cast<zmq::ctx_t *> (ctx_))->check_tag ()) {
    //     errno = EFAULT;
    //     return -1;
    // }
    // return (static_cast<zmq::ctx_t *> (ctx_))->get (option_);
    if ctx_.check_tag() == false {
        // errno = EFAULT;
        return -1;
    }
    ctx_.get(option_)
}

// int zmq_ctx_get_ext (void *ctx_, int option_, void *optval_, size_t *optvallen_)
pub fn zmq_ctx_get_ext(ctx_: &mut ZmqContext, option_: i32, optval_: &[u8], optvallen_: usize) -> i32 {
    // if (!ctx_ || !(static_cast<zmq::ctx_t *> (ctx_))->check_tag ()) {
    //     errno = EFAULT;
    //     return -1;
    // }
    // return (static_cast<zmq::ctx_t *> (ctx_))
    //   ->get (option_, optval_, optvallen_);
    if ctx_.check_tag() == false {
        // errno = EFAULT;
        return -1;
    }
    ctx_.get(option_, optval_, optvallen_)
}

// void *zmq_init (int io_threads_)
pub fn zmq_init(io_threads_: i32) -> Option<ZmqContext> {
    if (io_threads_ >= 0) {
        // void *ctx = zmq_ctx_new ();
        let mut ctx = zmq_ctx_new().unwrap();
        zmq_ctx_set(&mut ctx, ZMQ_IO_THREADS as i32, io_threads_);
        return Some(ctx);
    }
    // errno = EINVAL;
    return None;
}

// int zmq_term (void *ctx_)
pub fn zmq_term(ctx_: &mut ZmqContext) -> i32 {
    return zmq_ctx_term(ctx_);
}

// int zmq_ctx_destroy (void *ctx_)
pub fn zmq_ctx_destroy(ctx_: &mut ZmqContext) -> i32 {
    return zmq_ctx_term(ctx_);
}

// static zmq::socket_base_t *as_socket_base_t (void *s_)
pub fn as_socket_base_t<'a>(s_: &mut ZmqSocketBase) -> Option<&'a mut ZmqSocketBase> {
    // zmq::socket_base_t *s = static_cast<zmq::socket_base_t *> (s_);
    // if (!s_ || !s->check_tag ()) {
    //     errno = ENOTSOCK;
    //     return NULL;
    // }
    // return s;
    if s_.check_tag() == false {
        // errno = ENOTSOCK;
        return None;
    }
    return Some(s_);
}

pub unsafe fn zmq_socket<'a>(ctx_: &mut ZmqContext, type_: i32) -> Option<&'a mut ZmqSocketBase> {
    // if (!ctx_ || !(static_cast<zmq::ctx_t *> (ctx_))->check_tag ())
    if ctx_.check_tag() == false {
        // errno = EFAULT;
        return None;
    }
    // zmq::ctx_t *ctx = static_cast<zmq::ctx_t *> (ctx_);
    // zmq::socket_base_t *s = ctx->create_socket (type_);
    // return static_cast<void *> (s);
    return ctx_.create_socket(type_);
}

// int zmq_close (void *s_)
pub unsafe fn zmq_close(s_: &mut ZmqSocketBase) -> i32 {
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    s_.close();
    return 0;
}

pub unsafe fn zmq_setsockopt(options: &mut ZmqOptions, s_: &mut ZmqSocketBase, option_: i32, optval_: &[u8], optvallen_: usize) -> i32 {
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    // return s->setsockopt (option_, optval_, optvallen_);
    s_.setsockopt(options, option_, optval_, optvallen_)
}

// int zmq_getsockopt (void *s_, int option_, void *optval_, size_t *optvallen_)
pub unsafe fn zmq_getosckopt(s_: &mut ZmqSocketBase, option_: i32, optval_: &[u8], optvallen_: usize) -> i32 {
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    // return s->getsockopt (option_, optval_, optvallen_);
    s_.getsockopt(option_, optval_, optvallen_)
}

// int zmq_socket_monitor_versioned (
//   void *s_, const char *addr_, uint64_t events_, int event_version_, int type_)
pub unsafe fn zmq_socket_monitor_versioned(
    s_: &mut ZmqSocketBase, addr_: &str, events_: u64, event_version_: i32, type_: i32) -> i32 {
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    // return s->monitor (addr_, events_, event_version_, type_);
    s_.monitor(addr_, events_, event_version_, type_)
}

// int zmq_socket_monitor (void *s_, const char *addr_, int events_)
pub unsafe fn zmq_socket_monitor(s_: &mut ZmqSocketBase, addr_: &str, events_: i32) -> i32 {
    return zmq_socket_monitor_versioned(s_, addr_, events_ as u64, 1, ZMQ_PAIR as i32);
}

// int zmq_join (void *s_, const char *group_)
pub unsafe fn zmq_join(s_: &mut ZmqSocketBase, group_: &str) -> i32 {
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    // return s->join (group_);
    s_.join(group_)
}

// int zmq_leave (void *s_, const char *group_)
pub unsafe fn zmq_leave(s_: &mut ZmqSocketBase, group_: &str) -> i32 {
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    // return s->leave (group_);
    s_.leave(group_)
}

// int zmq_bind (void *s_, const char *addr_)
pub unsafe fn zmq_bind(s_: &mut ZmqSocketBase, addr_: &str) -> i32 {
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    // return s->bind (addr_);
    s_.bind(addr_)
}

// int zmq_connect (void *s_, const char *addr_)
pub unsafe fn zmq_connect(s_: &mut ZmqSocketBase, addr_: &str) -> i32 {
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    // return s->connect (addr_);
    s_.connect(addr_)
}

// uint32_t zmq_connect_peer (void *s_, const char *addr_)
pub unsafe fn zmq_connect_peer(s_: &mut ZmqSocketBase, addr_: &str) -> u32 {
    // zmq::peer_t *s = static_cast<zmq::peer_t *> (s_);
    // if (!s_ || !s->check_tag ()) {
    //     errno = ENOTSOCK;
    //     return 0;
    // }
    if s_.check_tag() == false {
        return 0;
    }

    // int socket_type;
    let mut socket_type = 0i32;
    // size_t socket_type_size = sizeof (socket_type);
    let mut socket_type_size = 4;
    if (s_.getsockopt(ZMQ_TYPE, &socket_type, &socket_type_size) != 0) {
        return 0;
    }

    if (socket_type != ZMQ_PEER) {
        // errno = ENOTSUP;
        return 0;
    }

    return s_.connect_peer(addr_);
}

// int zmq_unbind (void *s_, const char *addr_)
pub unsafe fn unbind(s_: &mut ZmqSocketBase, addr_: &str) -> i32 {
    // zmq::socket_base_t *s = as_socket_base_t (s_);

    // if (!s)
    //     return -1;
    return s_.term_endpoint(addr_);
}

// int zmq_disconnect (void *s_, const char *addr_)
pub unsafe fn zmq_disconnect(s_: &mut ZmqSocketBase, addr_: &str) -> i32 {
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    return s_.term_endpoint(addr_);
}

// static inline int
// s_sendmsg (zmq::socket_base_t *s_, zmq_msg_t *msg_, int flags_)
pub unsafe fn s_sendmsg(s_: &mut ZmqSocketBase, msg_: &mut zmq_msg_t, flags_: i32) -> i32 {
    let sz = zmq_msg_size(msg_);
    let rc = s_.send((msg_), flags_);
    if ((rc < 0)) {
        return -1;
    }

    //  This is what I'd like to do, my C++ fu is too weak -- PH 2016/02/09
    //  int max_msgsz = s_->parent->get (ZMQ_MAX_MSGSZ);
    let max_msgsz = i32::MAX;

    //  Truncate returned size to INT_MAX to avoid overflow to negative values
    return if sz < max_msgsz { sz } else { max_msgsz };
}

// int zmq_sendmsg (void *s_, zmq_msg_t *msg_, int flags_)
pub unsafe fn zmq_sendmsq(s_: &mut ZmqSocketBase, msg_: &mut zmq_msg_t, flags_: i32) -> i32 {
    return zmq_msg_send(msg_, s_, flags_);
}

// int zmq_send (void *s_, const void *buf_, size_t len_, int flags_)
pub unsafe fn zmq_send(s_: &mut ZmqSocketBase, buf_: &[u8], len_: usize, flags_: i32) -> i32 {
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    // zmq_msg_t msg;
    let msg = zmq_msg_t::default();
    let rc = zmq_msg_init_buffer(&msg, buf_, len_);
    if ((rc < 0)) {
        return -1;
    }

    rc = s_sendmsg(s, &msg, flags_);
    if ((rc < 0)) {
        // const int err = errno;
        let rc2 = zmq_msg_close(&msg);
        // errno_assert (rc2 == 0);
        // errno = err;
        return -1;
    }
    //  Note the optimisation here. We don't close the msg object as it is
    //  empty anyway. This may change when implementation of zmq_msg_t changes.
    return rc;
}

// int zmq_send_const (void *s_, const void *buf_, size_t len_, int flags_)
pub unsafe fn zmq_send_const(s_: &mut ZmqSocketBase, buf_: &[u8], len_: usize, flags_: i32) -> i32 {
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    // zmq_msg_t msg;
    let msg = zmq_msg_t::default();
    let rc = zmq_msg_init_data(&msg, (buf_), len_, None, None);
    if (rc != 0) {
        return -1;
    }

    rc = s_sendmsg(s_, &msg, flags_);
    if ((rc < 0)) {
        // const int err = errno;
        let rc2 = zmq_msg_close(&msg);
        // errno_assert (rc2 == 0);
        // errno = err;
        return -1;
    }
    //  Note the optimisation here. We don't close the msg object as it is
    //  empty anyway. This may change when implementation of zmq_msg_t changes.
    return rc;
}

// int zmq_sendiov (void *s_, iovec *a_, size_t count_, int flags_)
pub unsafe fn zmq_sendiov(s_: &mut ZmqSocketBase, a_: &[iovec], count_: usize, flags_: i32) -> i32 {
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    if ((count_ <= 0 || !a_)) {
        // errno = EINVAL;
        return -1;
    }

    let mut rc = 0;
    // zmq_msg_t msg;
    let msg = zmq_msg_t::default();

    // for (size_t i = 0; i < count_; ++i)
    for i in 0..count_ {
        rc = zmq_msg_init_size(&msg, a_[i].iov_len);
        if (rc != 0) {
            rc = -1;
            break;
        }
        libc::memcpy(zmq_msg_data(&msg), a_[i].iov_base, a_[i].iov_len);
        if (i == count_ - 1) {
            flags_ = flags_ & ~ZMQ_SNDMORE;
        }
        rc = s_sendmsg(s, &msg, flags_);
        if ((rc < 0)) {
            // const int err = errno;
            let rc2 = zmq_msg_close(&msg);
            // errno_assert (rc2 == 0);
            // errno = err;
            rc = -1;
            break;
        }
    }
    return rc;
}

// static int s_recvmsg (zmq::socket_base_t *s_, zmq_msg_t *msg_, int flags_)
pub unsafe fn s_recvmsg(s_: &mut ZmqSocketBase, msg_: &mut zmq_msg_t, flags_: i32) -> i32 {
    let rc = s_.recv((msg_), flags_);
    if ((rc < 0)) {
        return -1;
    }

    //  Truncate returned size to INT_MAX to avoid overflow to negative values
    let sz = zmq_msg_size(msg_);
    return (if sz < i32::MAX { sz } else { INT_MAX });
}

// int zmq_recvmsg (void *s_, zmq_msg_t *msg_, int flags_)
pub unsafe fn zmq_recvmsg(s_: &mut ZmqSocketBase, msg_: &mut zmq_msg_t, flags_: i32) -> i32 {
    return zmq_msg_recv(msg_, s_, flags_);
}

// int zmq_recv (void *s_, void *buf_, size_t len_, int flags_)
pub unsafe fn zmq_recv(s_: &mut ZmqSocketBase, buf_: &[u8], len_: usize, flags_: i32) -> i32 {
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    // zmq_msg_t msg;
    let msg = zmq_msg_t::default();
    let rc = zmq_msg_init(&msg);
    // errno_assert (rc == 0);

    let nbytes = s_recvmsg(s_, &msg, flags_);
    if ((nbytes < 0)) {
        // let err = errno;
        rc = zmq_msg_close(&msg);
        // errno_assert (rc == 0);
        // errno = err;
        return -1;
    }

    //  An oversized message is silently truncated.
    let to_copy = if (nbytes) < len_ { nbytes } else { len_ };

    //  We explicitly allow a null buffer argument if len is zero
    if (to_copy) {
        // assert (buf_);
        libc::memcpy(buf_, zmq_msg_data(&msg), to_copy);
    }
    rc = zmq_msg_close(&msg);
    // errno_assert (rc == 0);

    return nbytes;
}

// int zmq_recviov (void *s_, iovec *a_, size_t *count_, int flags_)
pub unsafe fn zmq_recviov(s_: &mut ZmqSocketBase, a_: &[iovec], count_: &mut usize, flags_: i32) -> i32 {
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //    return -1;
    if ((!count_ || *count_ <= 0 || !a_)) {
        // errno = EINVAL;
        return -1;
    }

    let count = *count_;
    let mut nread = 0;
    let recvmore = true;

    *count_ = 0;

    // for (size_t i = 0; recvmore && i < count; ++i)
    for i in 0..count {
        // zmq_msg_t msg;
        let msg = zmq_msg_t::default();
        let mut rc = zmq_msg_init(&msg);
        // errno_assert (rc == 0);

        let nbytes = s_recvmsg(s_, &msg, flags_);
        if ((nbytes < 0)) {
            // const int err = errno;
            rc = zmq_msg_close(&msg);
            // errno_assert (rc == 0);
            // errno = err;
            nread = -1;
            break;
        }

        a_[i].iov_len = zmq_msg_size(&msg);
        a_[i].iov_base = (libc::malloc(a_[i].iov_len));
        if ((!a_[i].iov_base)) {
            // errno = ENOMEM;
            return -1;
        }
        libc::memcpy(a_[i].iov_base, (zmq_msg_data(&msg)),
                     a_[i].iov_len);
        // Assume zmq_socket ZMQ_RVCMORE is properly set.
        let p_msg = (&msg);
        recvmore = p_msg.flags() & MSG_MORE;
        rc = zmq_msg_close(&msg);
        // errno_assert (rc == 0);
        // ++*count_;
        *count_ += 1;
        // ++nread;
        nread += 1;
    }
    return nread;
}

// int zmq_msg_init (zmq_msg_t *msg_)
pub unsafe fn zmq_msg_int(msg_: &mut zmq_msg_t) -> i32 {
    return ((msg_)).init();
}

// int zmq_msg_init_size (zmq_msg_t *msg_, size_t size_)
pub unsafe fn zmq_msg_int_size(msg_: &mut zmq_msg_t, size_: usize) -> i32 {
    return msg_.init_size(size_);
}

// int zmq_msg_init_buffer (zmq_msg_t *msg_, const void *buf_, size_t size_)
pub unsafe fn zmq_msg_init_buffer(msg_: &mut zmq_msg_t, buf_: &[u8], size_: usize) -> i32 {
    return msg_.init_buffer(buf_, size_);
}

// int zmq_msg_init_data (
//   zmq_msg_t *msg_, void *data_, size_t size_, zmq_free_fn *ffn_, void *hint_)
pub unsafe fn zmq_msg_init_data(
    msg_: &mut zmq_msg_t, data_: &[u8], size_: usize, ffn_: fn(), hint_: &[u8],
) -> i32 {
    return msg_.init_data(data_, size_, ffn_, hint_);
}

// int zmq_msg_send (zmq_msg_t *msg_, void *s_, int flags_)
pub unsafe fn zmq_msg_send(msg_: &mut zmq_msg_t, s_: &mut ZmqSocketBase, flags_: i32) -> i32 {
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    return s_sendmsg(s_, msg_, flags_);
}

// int zmq_msg_recv (zmq_msg_t *msg_, void *s_, int flags_)
pub unsafe fn zmq_msg_recv(msg_: &mut zmq_msg_t, s_: &mut ZmqSocketBase, flags_: i32) -> i32 {
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    return s_recvmsg(s_, msg_, flags_);
}

// int zmq_msg_close (zmq_msg_t *msg_)
pub unsafe fn close(msg_: &mut zmq_msg_t) -> i32 {
    return (msg_).close();
}

// int zmq_msg_move (zmq_msg_t *dest_, zmq_msg_t *src_)
pub unsafe fn zmq_msg_move(dest_: &mut zmq_msg_t, src_: &mut zmq_msg_t) -> i32 {
    return (dest_.; move (src_));
}

// int zmq_msg_copy (zmq_msg_t *dest_, zmq_msg_t *src_)
pub unsafe fn zmq_msg_copy(dest_: &mut zmq_msg_t, src_: &mut zmq_msg_t) -> i32 {
    return (dest_).copy(src_);
}

pub unsafe fn zmq_msg_data(msg_: &mut zmq_msg_t) -> *mut c_void {
    return (msg_).data();
}

pub unsafe fn zmq_msg_size(msg_: &mut zmq_msg_t) -> usize {
    return (msg_).size();
}

pub unsafe fn zmq_msg_more(msg_: &mut zmq_msg_t) -> i32 {
    return zmq_msg_get(msg_, ZMQ_MORE);
}

// int zmq_msg_get (const zmq_msg_t *msg_, int property_)
pub unsafe fn zmq_msg_get(msg_: &zmq_msg_t, property_: i32) -> i32 {
    let mut fd_string: String = String::new();

    match (property_) {
        ZMQ_MORE => {
            // return (((zmq::msg_t *)
            // msg_) -> flags() & zmq::msg_t::more) ? 1: 0;
            if msg_.flags() & MSG_MORE != 0 { return 1; } else { return 0; }
        }
        ZMQ_SRCFD => {
            let fd_string = zmq_msg_gets(msg_, "__fd");
            // if (fd_string == NULL)
            // return -1;

            return atoi(fd_string);
        }
        ZMQ_SHARED => {
            // return (((zmq::msg_t *)
            // msg_) -> is_cmsg())
            // || (((zmq::msg_t *)
            // msg_) -> flags() & zmq::msg_t::shared) ? 1: 0;
            return msg_.is_cmsg() || msg_.flags() & MSG_SHARED != 0;
            { 1 } else { 0 }
        }
        _ => {
            // errno = EINVAL;
            return -1;
        }
    }
}

// int zmq_msg_set (zmq_msg_t *, int, int)
pub unsafe fn zmq_msg_set(msg: &mut zmq_msg_t, a: i32, b: i32) -> i32 {
    //  No properties supported at present
    // errno = EINVAL;
    return -1;
}

pub unsafe fn zmq_msg_set_routing_id(msg: &mut zmq_msg_t, routing_id_: u32) -> i32 {
    msg_.set_routing_id(routing_id_)
}

pub unsafe fn zmq_msg_routing_id(msg_: &mut zmq_msg_t) -> u32 {
    msg_.get_routing_id()
}

pub unsafe fn zmq_msg_set_group(msg_: &mut zmq_msg_t, group_: &str) -> i32 {
    msg_.set_group(group_)
}

pub unsafe fn zmq_msg_group(msg_: &mut zmq_msg_t) -> &str {
    msg_.group()
}

pub unsafe fn zmq_msg_gets(msg_: &zmq_msg_t, property_: &str) -> &'static str {
    let metadata = msg_.metadata();
    let value = metadata.get(property_);
    return value;
}

pub unsafe fn zmq_poller_poll(items_: &[zmq_pollitem_t], nitems_: i32, timeout_: i32) -> i32 {
    // implement zmq_poll on top of zmq_poller
    // int rc;
    let mut rc = 0i32;
    // zmq_poller_event_t *events;
    let mut events: [zmq_poller_event_t; 1024] = [zmq_poller_event_t::default(); 1024];
    // zmq::socket_poller_t poller;
    let mut poller = ZmqSocketPoller::default();
    // events = new (std::nothrow) zmq_poller_event_t[nitems_];

    // alloc_assert (events);

    let repeat_items = false;
    //  Register sockets with poller
    // for (int i = 0; i < nitems_; i++)
    for i in 0..nitems_ {
        items_[i].revents = 0;

        let modify = false;
        let e = items_[i].events;
        if (items_[i].socket) {
            //  Poll item is a 0MQ socket.
            // for (int j = 0; j < i; ++j)
            for j in 0..i {
                // Check for repeat entries
                if (items_[j].socket == items_[i].socket) {
                    repeat_items = true;
                    modify = true;
                    e |= items_[j].events;
                }
            }
            if (modify) {
                rc = zmq_poller_modify(&poller, items_[i].socket, e);
            } else {
                rc = zmq_poller_add(&poller, items_[i].socket, NULL, e);
            }
            if (rc < 0) {
                // delete[] events;
                return rc;
            }
        } else {
            //  Poll item is a raw file descriptor.
            // for (int j = 0; j < i; ++j)
            for j in 0..i {
                // Check for repeat entries
                if (!items_[j].socket && items_[j].fd == items_[i].fd) {
                    repeat_items = true;
                    modify = true;
                    e |= items_[j].events;
                }
            }
            if (modify) {
                rc = zmq_poller_modify_fd(&poller, items_[i].fd, e);
            } else {
                rc = zmq_poller_add_fd(&poller, items_[i].fd, NULL, e);
            }
            if (rc < 0) {
                // delete[] events;
                return rc;
            }
        }
    }


    //  Wait for events
    rc = zmq_poller_wait_all(&poller, events, nitems_, timeout_);
    if (rc < 0) {
        delete
        []
        events;
        if (zmq_errno() == EAGAIN) {
            return 0;
        }
        return rc;
    }

    //  Transform poller events into zmq_pollitem events.
    //  items_ contains all items, while events only contains fired events.
    //  If no sockets are repeated (likely), the two are still co-ordered, so step through the items
    //  checking for matches only on the first event.
    //  If there are repeat items, they cannot be assumed to be co-ordered,
    //  so each pollitem must check fired events from the beginning.
    // int j_start = 0, found_events = rc;
    let mut j_start = 0;
    let mut found_events = rc;

    // for (int i = 0; i < nitems_; i++)
    for i in 0..nitems_ {
        // for (int j = j_start; j < found_events; ++j)
        for j in j_start..found_events {
            if ((items_[i].socket && items_[i].socket == events[j].socket) || (!(items_[i].socket || events[j].socket) && items_[i].fd == events[j].fd)) {
                items_[i].revents = events[j].events & items_[i].events;
                if (!repeat_items) {
                    // no repeats, we can ignore events we've already seen
                    j_start += 1;
                }
                break;
            }
            if (!repeat_items) {
                // no repeats, never have to look at j > j_start
                break;
            }
        }
    }

    //  Cleanup
    // delete[] events;
    return rc;
}


// int zmq_poll (zmq_pollitem_t *items_, int nitems_, long timeout_)
pub unsafe fn zmq_poll(items_: &[zmq_pollitem_t], nitems_: i32, timeout_: i32) -> i32 {
// #if defined ZMQ_HAVE_POLLER
    // if poller is present, use that if there is at least 1 thread-safe socket,
    // otherwise fall back to the previous implementation as it's faster.
    // for (int i = 0; i != nitems_; i++)
    for i in 0..nitems_ {
        if (items_[i].socket) {
            let s = as_socket_base_t(items_[i].socket);
            if (s) {
                if (s.is_thread_safe())
                return zmq_poller_poll(items_, nitems_, timeout_);
            } else {
                //as_socket_base_t returned NULL : socket is invalid
                return -1;
            }
        }
    }
// #endif // ZMQ_HAVE_POLLER
// #if defined ZMQ_POLL_BASED_ON_POLL || defined ZMQ_POLL_BASED_ON_SELECT
    if ((nitems_ < 0)) {
        // errno = EINVAL;
        return -1;
    }
    if ((nitems_ == 0)) {
        if (timeout_ == 0) {
            return 0;
        }
// #if defined ZMQ_HAVE_WINDOWS
        #[cfg(target_os = "windows")]
        {
            Sleep(timeout_ > 0? timeout_: INFINITE);
            return 0;
        }
// #elif defined ZMQ_HAVE_VXWORKS
//         struct timespec ns_;
//         ns_.tv_sec = timeout_ / 1000;
//         ns_.tv_nsec = timeout_ % 1000 * 1000000;
//         return nanosleep (&ns_, 0);
// #else
        #[cfg(not(target_os = "windows"))]
        {
            return usleep(timeout_ * 1000);
        }
// #endif
    }
    if (!items_) {
        // errno = EFAULT;
        return -1;
    }

    // zmq::clock_t clock;
    let mut clock = ZmqClock::default();
    let mut now = 0u64;
    let mut end = 0u64;
// #if defined ZMQ_POLL_BASED_ON_POLL
//     zmq::fast_vector_t<pollfd, ZMQ_POLLITEMS_DFLT> pollfds (nitems_);
    let pollfds = Vec < pollfd > ::new();

    //  Build pollset for poll () system call.
    // for (int i = 0; i != nitems_; i++)
    for i in 0..nitems_ {
        //  If the poll item is a 0MQ socket, we poll on the file descriptor
        //  retrieved by the ZMQ_FD socket option.
        if (items_[i].socket) {
            let zmq_fd_size = size_of::<fd_t>();
            if (zmq_getsockopt(items_[i].socket, ZMQ_FD, &pollfds[i].fd,
                               &zmq_fd_size) == -1) {
                return -1;
            }
            pollfds[i].events = if items_[i].events { POLLIN } else { 0 };
        }
        //  Else, the poll item is a raw file descriptor. Just convert the
        //  events to normal POLLIN/POLLOUT for poll ().
        else {
            pollfds[i].fd = items_[i].fd;
            pollfds[i].events = (if items_[i].events & ZMQ_POLLIN { POLLIN } else { 0 }) | (if items_[i].events & ZMQ_POLLOUT { POLLOUT } else { 0 }) | (if items_[i].events & ZMQ_POLLPRI { POLLPRI } else { 0 });
        }
    }
// #else
    //  Ensure we do not attempt to select () on more than FD_SETSIZE
    //  file descriptors.
    //  TODO since this function is called by a client, we could return errno EINVAL/ENOMEM/... here
    // zmq_assert (nitems_ <= FD_SETSIZE);

    // zmq::optimized_fd_set_t pollset_in (nitems_);
    let pollset_in = OptimizedFdSet::new(nitems_);
    // FD_ZERO (pollset_in.get ());
    FD_ZERO(pollset_in.get())
    let pollset_out = OptimizedFdSet::new(nitems_);
    FD_ZERO(pollset_out.get());
    let pollset_err = OptimizedFdSet::new(nitems_);
    FD_ZERO(pollset_err.get());

    let maxfd: fd_t = 0;

    //  Build the fd_sets for passing to select ().
    // for (int i = 0; i != nitems_; i++)
    for i in 0..nitems_ {
        //  If the poll item is a 0MQ socket we are interested in input on the
        //  notification file descriptor retrieved by the ZMQ_FD socket option.
        if (items_[i].socket) {
            let zmq_fd_size = size_of::<fd_t>();
            let mut notify_fd: fd_t = 0;
            if (zmq_getsockopt(items_[i].socket, ZMQ_FD, &notify_fd,
                               &zmq_fd_size) == -1) {
                return -1;
            }
            if (items_[i].events) {
                FD_SET(notify_fd, pollset_in.get());
                if (maxfd < notify_fd) {
                    maxfd = notify_fd;
                }
            }
        }
        //  Else, the poll item is a raw file descriptor. Convert the poll item
        //  events to the appropriate fd_sets.
        else {
            if (items_[i].events & ZMQ_POLLIN) {
                FD_SET(items_[i].fd, pollset_in.get());
            }
            if (items_[i].events & ZMQ_POLLOUT) {
                FD_SET(items_[i].fd, pollset_out.get());
            }
            if (items_[i].events & ZMQ_POLLERR) {
                FD_SET(items_[i].fd, pollset_err.get());
            }
            if (maxfd < items_[i].fd) {
                maxfd = items_[i].fd;
            }
        }
    }

    let mut inset = OptimizedFdSet::new(nitems_);
    let mut outset = OptimizedFdSet::new(nitems_);
    let mut errset = optimized_fd_set_t::new(nitems_);
// #endif

    let mut first_pass = true;
    let mut nevents = 0i32;

    loop {
// #if defined ZMQ_POLL_BASED_ON_POLL

        //  Compute the timeout for the subsequent poll.
        let timeout = compute_timeout(first_pass, timeout_, now, end);

        //  Wait for events.
        {
            let rc = poll(&pollfds[0], nitems_, timeout);
            if (rc == -1 && errno == EINTR) {
                return -1;
            }
            // errno_assert (rc >= 0);
        }
        //  Check for the events.
        // for (int i = 0; i != nitems_; i++)
        for i in 0..nitems_ {
            items_[i].revents = 0;

            //  The poll item is a 0MQ socket. Retrieve pending events
            //  using the ZMQ_EVENTS socket option.
            if (items_[i].socket) {
                let mut zmq_events_size = 4;
                let mut zmq_events: u32 = 0;
                if (zmq_getsockopt(items_[i].socket, ZMQ_EVENTS, &zmq_events,
                                   &zmq_events_size) == -1) {
                    return -1;
                }
                if ((items_[i].events & ZMQ_POLLOUT) && (zmq_events & ZMQ_POLLOUT)) {
                    items_[i].revents |= ZMQ_POLLOUT;
                }
                if ((items_[i].events & ZMQ_POLLIN) && (zmq_events & ZMQ_POLLIN)) {
                    items_[i].revents |= ZMQ_POLLIN;
                }
            }
            //  Else, the poll item is a raw file descriptor, simply convert
            //  the events to zmq_pollitem_t-style format.
            else {
                if (pollfds[i].revents & POLLIN) {
                    items_[i].revents |= ZMQ_POLLIN;
                }
                if (pollfds[i].revents & POLLOUT) {
                    items_[i].revents |= ZMQ_POLLOUT;
                }
                if (pollfds[i].revents & POLLPRI) {
                    items_[i].revents |= ZMQ_POLLPRI;
                }
                if (pollfds[i].revents & ~(POLLIN | POLLOUT | POLLPRI)){
                    items_[i].revents |= ZMQ_POLLERR;
                }
            }

            if (items_[i].revents) {
                nevents += 1;
            }
        }

// #else

        //  Compute the timeout for the subsequent poll.
        let mut timeout = timeval { tv_sec: 0, tv_usec: 0 };
        let mut ptimeout: &timeval = &timeout;
        if (first_pass) {
            timeout.tv_sec = 0;
            timeout.tv_usec = 0;
            ptimeout = &timeout;
        } else if (timeout_ < 0) {
            ptimeout = null_mut();
        } else {
            timeout.tv_sec = ((end - now) / 1000);
            timeout.tv_usec = ((end - now) % 1000 * 1000);
            ptimeout = &timeout;
        }

        //  Wait for events. Ignore interrupts if there's infinite timeout.
        loop {
            libc::memcpy(inset.get(), pollset_in.get(),
                         valid_pollset_bytes(*pollset_in.get()));
            libc::memcpy(outset.get(), pollset_out.get(),
                         valid_pollset_bytes(*pollset_out.get()));
            libc::memcpy(errset.get(), pollset_err.get(),
                         valid_pollset_bytes(*pollset_err.get()));
// #if defined ZMQ_HAVE_WINDOWS
            #[cfg(target_os = "windows")]
            {
                int
                rc = select(0, inset.get(), outset.get(), errset.get(), ptimeout);
                if (unlikely(rc == SOCKET_ERROR)) {
                    errno = zmq::wsa_error_to_errno(WSAGetLastError());
                    wsa_assert(errno == ENOTSOCK);
                    return -1;
                }
            }
// #else
            #[cfg(not(target_os = "windows"))]{
                let rc = select(maxfd + 1, inset.get(), outset.get(),
                                errset.get(), ptimeout);
                if (unlikely(rc == -1)) {
                    errno_assert(errno == EINTR || errno == EBADF);
                    return -1;
                }
            }
// #endif
            break;
        }

        //  Check for the events.
        // for (int i = 0; i != nitems_; i++)
        for i in 0..nitems_ {
            items_[i].revents = 0;

            //  The poll item is a 0MQ socket. Retrieve pending events
            //  using the ZMQ_EVENTS socket option.
            if (items_[i].socket) {
                let zmq_events_size = 4;
                let mut zmq_events = 0u32;
                if (zmq_getsockopt(items_[i].socket, ZMQ_EVENTS, &zmq_events,
                                   &zmq_events_size) == -1) {
                    return -1;
                }
                if ((items_[i].events & ZMQ_POLLOUT) && (zmq_events & ZMQ_POLLOUT)) {
                    items_[i].revents |= ZMQ_POLLOUT;
                }
                if ((items_[i].events & ZMQ_POLLIN) && (zmq_events & ZMQ_POLLIN)) {
                    items_[i].revents |= ZMQ_POLLIN;
                }
            }
            //  Else, the poll item is a raw file descriptor, simply convert
            //  the events to zmq_pollitem_t-style format.
            else {
                if (FD_ISSET(items_[i].fd, inset.get())) {
                    items_[i].revents |= ZMQ_POLLIN;
                }
                if (FD_ISSET(items_[i].fd, outset.get())) {
                    items_[i].revents |= ZMQ_POLLOUT;
                }
                if (FD_ISSET(items_[i].fd, errset.get())) {
                    items_[i].revents |= ZMQ_POLLERR;
                }
            }

            if (items_[i].revents) {
                nevents + +;
            }
        }
// #endif

        //  If timeout is zero, exit immediately whether there are events or not.
        if (timeout_ == 0) {
            break;
        }

        //  If there are events to return, we can exit immediately.
        if (nevents) {
            break;
        }

        //  At this point we are meant to wait for events but there are none.
        //  If timeout is infinite we can just loop until we get some events.
        if (timeout_ < 0) {
            if (first_pass) {
                first_pass = false;
            }
            continue;
        }

        //  The timeout is finite and there are no events. In the first pass
        //  we get a timestamp of when the polling have begun. (We assume that
        //  first pass have taken negligible time). We also compute the time
        //  when the polling should time out.
        if (first_pass) {
            now = clock.now_ms();
            end = now + timeout_;
            if (now == end) {
                break;
            }
            first_pass = false;
            continue;
        }

        //  Find out whether timeout have expired.
        now = clock.now_ms();
        if (now >= end) {
            break;
        }
    }

    return nevents;
// #else
    //  Exotic platforms that support neither poll() nor select().
    // errno = ENOTSUP;
    return -1;
// #endif
}

pub unsafe fn zmq_poll_check_items_(items_: &mut zmq_pollitem_t, nitems_: i32, timeout_: i32) -> i32 {
    if ((nitems_ < 0)) {
        // errno = EINVAL;
        return -1;
    }
    if (unlikely(nitems_ == 0)) {
        if (timeout_ == 0) {
            return 0;
        }
// #if defined ZMQ_HAVE_WINDOWS
        #[cfg(target_os = "windows")]
        {
            Sleep(if timeout_ > 0 { timeout_ } else { INFINITE });
            return 0;
        }
// #elif defined ZMQ_HAVE_VXWORKS
//         struct timespec ns_;
//         ns_.tv_sec = timeout_ / 1000;
//         ns_.tv_nsec = timeout_ % 1000 * 1000000;
//         return nanosleep (&ns_, 0);
// #else
        #[cfg(not(target_os = "windows"))]
        {
            return usleep(timeout_ * 1000);
        }
// #endif
    }
    if (!items_) {
        // errno = EFAULT;
        return -1;
    }
    return 1;
}

pub struct zmq_poll_select_fds_t_ {
    // explicit zmq_poll_select_fds_t_ (int nitems_) :
    //     pollset_in (nitems_),
    //     pollset_out (nitems_),
    //     pollset_err (nitems_),
    //     inset (nitems_),
    //     outset (nitems_),
    //     errset (nitems_),
    //     maxfd (0)
    // {
    //     FD_ZERO (pollset_in.get ());
    //     FD_ZERO (pollset_out.get ());
    //     FD_ZERO (pollset_err.get ());
    // }

    // zmq::optimized_fd_set_t pollset_in;
    pub pollset_in: OptimizedFdSet,
    // zmq::optimized_fd_set_t pollset_out;
    pub pollset_out: OptimizedFdSet,
    // zmq::optimized_fd_set_t pollset_err;
    pub pollset_err: OptimizedFdSet,
    // zmq::optimized_fd_set_t inset;
    pub inset: OptimizedFdSet,
    // zmq::optimized_fd_set_t outset;
    pub outset: OptimizedFdSet,
    // zmq::optimized_fd_set_t errset;
    pub errset: OptimizedFdSet,
    // zmq::fd_t maxfd;
    pub maxfd: fd_t,
}

// zmq_poll_select_fds_t_
// zmq_poll_build_select_fds_ (zmq_pollitem_t *items_, int nitems_, int &rc)
pub unsafe fn zmq_poll_build_select_fds_(items_: &[zmq_pollitem_t], nitems_: i32, rc: &mut i32) -> zmq_poll_select_fds_t_ {
    //  Ensure we do not attempt to select () on more than FD_SETSIZE
    //  file descriptors.
    //  TODO since this function is called by a client, we could return errno EINVAL/ENOMEM/... here
    // zmq_assert (nitems_ <= FD_SETSIZE);

    let fds = zmq_poll_select_fds_t_::new(nitems_);

    //  Build the fd_sets for passing to select ().
    // for (int i = 0; i != nitems_; i++)
    for i in 0..nitems_ {
        //  If the poll item is a 0MQ socket we are interested in input on the
        //  notification file descriptor retrieved by the ZMQ_FD socket option.
        if (items_[i].socket) {
            let zmq_fd_size = size_of::<fd_t>();
            // zmq::fd_t notify_fd;
            let mut notify_fd: fd_t;
            if (zmq_getsockopt(items_[i].socket, ZMQ_FD, &notify_fd,
                               &zmq_fd_size) == -1) {
                rc = -1;
                return fds;
            }
            if (items_[i].events) {
                FD_SET(notify_fd, fds.pollset_in.get());
                if (fds.maxfd < notify_fd) {
                    fds.maxfd = notify_fd;
                }
            }
        }
        //  Else, the poll item is a raw file descriptor. Convert the poll item
        //  events to the appropriate fd_sets.
        else {
            if (items_[i].events & ZMQ_POLLIN) {
                FD_SET(items_[i].fd, fds.pollset_in.get());
            }
            if (items_[i].events & ZMQ_POLLOUT) {
                1
            }
            if (items_[i].events & ZMQ_POLLERR) {
                FD_SET(items_[i].fd, fds.pollset_err.get());
            }
            if (fds.maxfd < items_[i].fd) {
                fds.maxfd = items_[i].fd;
            }
        }
    }

    rc = 0;
    return fds;
}

// timeval *zmq_poll_select_set_timeout_ (
//   long timeout_, bool first_pass, uint64_t now, uint64_t end, timeval &timeout)
pub unsafe fn zmq_poll_select_set_timeout_(timeout_: i32, first_pass: bool, now: u64, end: u64, timeout: &timeval) -> &mut timeval {
    // timeval *ptimeout;
    let ptimeout: &mut timeval;
    if (first_pass) {
        timeout.tv_sec = 0;
        timeout.tv_usec = 0;
        ptimeout = &timeout;
    } else if (timeout_ < 0)
    ptimeout = None;
    else {
        timeout.tv_sec = ((end - now) / 1000);
        timeout.tv_usec = ((end - now) % 1000 * 1000);
        ptimeout = &timeout;
    }
    return ptimeout;
}

// timespec *zmq_poll_select_set_timeout_2 (
//   long timeout_, bool first_pass, uint64_t now, uint64_t end, timespec &timeout)
pub unsafe fn zmq_poll_select_set_timeout_2(
    timeout_: i32, first_pass: bool, now: u64, end: u64, timeout: &timespec) -> &mut timespec {
    timespec * ptimeout;
    if (first_pass) {
        timeout.tv_sec = 0;
        timeout.tv_nsec = 0;
        ptimeout = &timeout;
    } else if (timeout_ < 0)
    ptimeout = NULL;
    else {
        timeout.tv_sec = ((end - now) / 1000);
        timeout.tv_nsec = ((end - now) % 1000 * 1000000);
        ptimeout = &timeout;
    }
    return ptimeout;
}


// int zmq_poll_select_check_events_ (zmq_pollitem_t *items_,
//                                    int nitems_,
//                                    zmq_poll_select_fds_t_ &fds,
//                                    int &nevents)
pub unsafe fn zmq_poll_select_check_events_(
    items_: &[zmq_pollitem_t, nitems_: i32, fds: &mut zmq_poll_select_fds_t_, nevents: &mut i32) -> i32 {
//  Check for the events.
// for (int i = 0; i != nitems_; i++)
    for i in 0..nitems_ {
        items_[i].revents = 0;

//  The poll item is a 0MQ socket. Retrieve pending events
//  using the ZMQ_EVENTS socket option.
        if (items_[i].socket) {
            let zmq_events_size = 4;
            let zmq_events: u32;
            if (zmq_getsockopt(items_[i].socket, ZMQ_EVENTS, &zmq_events,
                               &zmq_events_size) == -1) {
                return -1;
            }
            if ((items_[i].events & ZMQ_POLLOUT) & &(zmq_events & ZMQ_POLLOUT)) {
                items_[i].revents |= ZMQ_POLLOUT;
            }
            if ((items_[i].events & ZMQ_POLLIN) & &(zmq_events & ZMQ_POLLIN)) {
                items_[i].revents |= ZMQ_POLLIN;
            }
        }
//  Else, the poll item is a raw file descriptor, simply convert
//  the events to zmq_pollitem_t-style format.
        else {
            if (FD_ISSET(items_[i].fd, fds.inset.get())) {
                items_[i].revents |= ZMQ_POLLIN;
            }
            if (FD_ISSET(items_[i].fd, fds.outset.get())) {
                items_[i].revents |= ZMQ_POLLOUT;
            }
            if (FD_ISSET(items_[i].fd, fds.errset.get())) {
                items_[i].revents |= ZMQ_POLLERR;
            }
        }

        if (items_[i].revents) {
            nevents += 1;
        }
    }

    return 0;
}

pub unsafe fn zmq_poll_must_break_loop_(timeout_: i32, nevents: i32, first_pass: &bool, clock:* mut ZmqClock, now: &mut u64, end: &mut u64) -> bool

{
     //  If timeout is zero, exit immediately whether there are events or not.
    if (timeout_ == 0) {
        return true;
    }

    //  If there are events to return, we can exit immediately.
    if (nevents) {
        return true;
    }

    //  At this point we are meant to wait for events but there are none.
    //  If timeout is infinite we can just loop until we get some events.
    if (timeout_ < 0) {
        if (first_pass) {
            first_pass = false;
        }
        return false;
    }

    //  The timeout is finite and there are no events. In the first pass
    //  we get a timestamp of when the polling have begun. (We assume that
    //  first pass have taken negligible time). We also compute the time
    //  when the polling should time out.
    if (first_pass) {
        now = clock.now_ms ();
        end = now + timeout_;
        if (now == end) {
            return true;
        }
        first_pass = false;
        return false;
    }

    //  Find out whether timeout have expired.
    now = clock.now_ms ();
    if (now >= end) {
        return true;
    }

    // finally, in all other cases, we just continue
    return false;
}

// #if !defined _WIN32
// int zmq_ppoll (zmq_pollitem_t *items_,
//                int nitems_,
//                long timeout_,
//                const sigset_t *sigmask_)
// #else
// // Windows has no sigset_t
// int zmq_ppoll (zmq_pollitem_t *items_,
//                int nitems_,
//                long timeout_,
//                const void *sigmask_)
// #endif
pub unsafe fn zmq_ppoll(items_: &mut zmq_pollitem_t, nitems_: i32, timeout_: i32, sigmask_: &sigset_t) -> i32
{
// #ifdef ZMQ_HAVE_PPOLL
    let mut rc = zmq_poll_check_items_ (items_, nitems_, timeout_);
    if (rc <= 0) {
        return rc;
    }

    let mut clock: ZmqClock = ;
    let mut  now = 0;
    let mut end = 0;
    let fds =
      zmq_poll_build_select_fds_ (items_, nitems_, rc);
    if (rc == -1) {
        return -1;
    }

    let mut first_pass = true;
    let mut nevents = 0;

    loop {
        //  Compute the timeout for the subsequent poll.
        // timespec timeout;
        let timeout = timespec { tv_sec: 0, tv_nsec: 0 };
        let ptimeout = zmq_poll_select_set_timeout_ (timeout_, first_pass,
                                                           now, end, timeout);

        //  Wait for events. Ignore interrupts if there's infinite timeout.
        loop {
            libc::memcpy (fds.inset.get (), fds.pollset_in.get (),
                    valid_pollset_bytes (*fds.pollset_in.get ()));
            libc::memcpy (fds.outset.get (), fds.pollset_out.get (),
                    valid_pollset_bytes (*fds.pollset_out.get ()));
            libc::memcpy (fds.errset.get (), fds.pollset_err.get (),
                    valid_pollset_bytes (*fds.pollset_err.get ()));
            let mut rc =
              pselect (fds.maxfd + 1, fds.inset.get (), fds.outset.get (),
                       fds.errset.get (), ptimeout, sigmask_);
            if ((rc == -1)) {
                // errno_assert (errno == EINTR || errno == EBADF);
                return -1;
            }
            break;
        }

        rc = zmq_poll_select_check_events_ (items_, nitems_, fds, nevents);
        if (rc < 0) {
            return rc;
        }

        if (zmq_poll_must_break_loop_ (timeout_, nevents, first_pass, clock,
                                       now, end)) {
            break;
        }
    }

    return nevents;
// #else
//     errno = ENOTSUP;
//     return -1;
// #endif // ZMQ_HAVE_PPOLL
}

// void *zmq_poller_new (void)
pub unsafe fn zmq_poller_new() -> ZmqSocketPoller
{
    // zmq::socket_poller_t *poller = new (std::nothrow) zmq::socket_poller_t;
    // if (!poller) {
    //     errno = ENOMEM;
    // }
    // return poller;
    ZmqSocketPoller::new()
}

// int zmq_poller_destroy (void **poller_p_)
pub unsafe fn zmq_poller_destroy(poller_p_: &mut ZmqSocketPoller) -> i32
{
    // if (poller_p_) {
    //     const zmq::socket_poller_t *const poller =
    //       static_cast<const zmq::socket_poller_t *> (*poller_p_);
    //     if (poller && poller->check_tag ()) {
    //         delete poller;
    //         *poller_p_ = NULL;
    //         return 0;
    //     }
    // }
    // errno = EFAULT;
    // return -1;
    todo!()
}

// static int check_poller (void *const poller_)
pub unsafe fn check_poller(poller_: &ZmqSocketPoller) -> i32
{
    // if (!poller_
    //     || !(static_cast<zmq::socket_poller_t *> (poller_))->check_tag ()) {
    //     errno = EFAULT;
    //     return -1;
    // }
    //
    // return 0;
    if !poller_.check_tag() {
        return -1;
    } else {
        return 0;
    }
}

// static int check_events (const short events_)
pub unsafe fn check_events(events_: i16) -> i32
{
    if (events_ & !(ZMQ_POLLIN | ZMQ_POLLOUT | ZMQ_POLLERR | ZMQ_POLLPRI)) {
        // errno = EINVAL;
        return -1;
    }
    return 0;
}

// static int check_poller_registration_args (void *const poller_, void *const s_)
pub unsafe fn check_poller_registration_args(poller_: &ZmqSocketPoller, s_: &mut ZmqSocketBase) -> i32
{
    if (-1 == check_poller (poller_)) {
        return -1;
    }

    // if (!s_ || !(static_cast<zmq::socket_base_t *> (s_))->check_tag ()) {
    //     errno = ENOTSOCK;
    //     return -1;
    // }
    if !s_.check_tag() {
        return -1;
    }

    return 0;
}

// static int check_poller_fd_registration_args (void *const poller_,
//                                               const zmq::fd_t fd_)
pub unsafe fn check_poller_fd_registration_args(poller_: &ZmqSocketPoller, fd_: fd_t) -> i32
{
    if (-1 == check_poller (poller_)) {
        return -1;
    }

    if (fd_ == retired_fd) {
        // errno = EBADF;
        return -1;
    }

    return 0;
}

// int zmq_poller_size (void *poller_)
pub unsafe fn zmq_poller_size(poller_: &ZmqSocketPoller) -> i32
{
    if (-1 == check_poller (poller_)) {
        return -1;
    }

    // return (static_cast<zmq::socket_poller_t *> (poller_))->size ();
    poller_.size()
}

// int zmq_poller_add (void *poller_, void *s_, void *user_data_, short events_)
pub unsafe fn zmq_poller_add(poller_: &ZmqSocketPoller, s_: &mut ZmqSocketBase, user_data_: &mut c_void, events_: i16) -> i32
{
    if (-1 == check_poller_registration_args (poller_, s_)
        || -1 == check_events (events_)) {
        return -1;
    }

    // zmq::socket_base_t *socket = static_cast<zmq::socket_base_t *> (s_);

    // return (static_cast<zmq::socket_poller_t *> (poller_))
    //   ->add (socket, user_data_, events_);
    poller_.add(socket, user_data, events_)
}

// int zmq_poller_add_fd (void *poller_,
//                        zmq::fd_t fd_,
//                        void *user_data_,
//                        short events_)
pub unsafe fn zmq_poller_add_fd(poller_: &ZmqSocketPoller, fd_: fd_t, user_data_: &mut c_void, events_: i16) -> i32
{
    if (-1 == check_poller_fd_registration_args (poller_, fd_)
        || -1 == check_events (events_)) {
        return -1;
    }

    // return (static_cast<zmq::socket_poller_t *> (poller_))
    //   ->add_fd (fd_, user_data_, events_);
    poller_.add_fd(fd_, user_data_, events_)
}

// int zmq_poller_modify (void *poller_, void *s_, short events_)
pub unsafe fn zmq_poller_modify(poller_: &ZmqSocketPoller, s_: &mut ZmqSocketBase, events_: i16) -> i32
{
    if (-1 == check_poller_registration_args (poller_, s_)
        || -1 == check_events (events_)) {
        return -1;
    }

    // const zmq::socket_base_t *const socket =
    //   static_cast<const zmq::socket_base_t *> (s_);

    // return (static_cast<zmq::socket_poller_t *> (poller_))
    //   ->modify (socket, events_);
    poller_.modify(s_, events_)
}

// int zmq_poller_modify_fd (void *poller_, zmq::fd_t fd_, short events_)
pub unsafe fn zmq_poller_modify_fd(poller_: &mut ZmqSocketPoller, fd_: fd_t, events_: i16) -> i32
{
    if (-1 == check_poller_fd_registration_args (poller_, fd_)
        || -1 == check_events (events_)) {
        return -1;
    }

    // return (static_cast<zmq::socket_poller_t *> (poller_))
    //   ->modify_fd (fd_, events_);
    poller_.modify_fd(fd_, events_)
}

// int zmq_poller_remove (void *poller_, void *s_)
pub unsafe fn zmq_poller_remove(poller_: &mut ZmqSocketPoller, s_: &mut ZmqSocketBase) -> i32
{
    if (-1 == check_poller_registration_args (poller_, s_)) {
        return -1;
    }

    // zmq::socket_base_t *socket = static_cast<zmq::socket_base_t *> (s_);

    // return (static_cast<zmq::socket_poller_t *> (poller_))->remove (socket);
    poller_.remove(s_)
}

// int zmq_poller_remove_fd (void *poller_, zmq::fd_t fd_)
pub unsafe fn zmq_poller_remove_fd(poller_: &mut ZmqSocketPoller, fd_: fd_t) -> i32
{
    if (-1 == check_poller_fd_registration_args (poller_, fd_)) {
        return -1;
    }

    // return (static_cast<zmq::socket_poller_t *> (poller_))->remove_fd (fd_);
    poller_.remove_fd(fd_)
}

// int zmq_poller_wait (void *poller_, zmq_poller_event_t *event_, long timeout_)
puib unsafe fn zmq_poller_wait(poller_: &mut ZmqSocketPoller, event_: &mut zmq_poller_event_t, timeout_: i32) -> i32
{

    let rc = zmq_poller_wait_all (poller_, event_, 1, timeout_);

    if (rc < 0 && event_) {
        // event_->socket = NULL;
        // event_->fd = zmq::retired_fd;
        // event_->user_data = NULL;
        // event_->events = 0;
    }
    // wait_all returns number of events, but we return 0 for any success
    return if rc >= 0 { 0 }else { rc };
}

// int zmq_poller_wait_all (void *poller_,
//                          zmq_poller_event_t *events_,
//                          int n_events_,
//                          long timeout_)
pub unsafe fn zmq_poller_wait_all(poller_: &mut ZmqSocketPoller, events_: &mut zmq_poller_event_t, n_events_: i32, timeout_: i32) -> i32
{
    if (-1 == check_poller (poller_)) {
        return -1;
    }

    if (!events_) {
        // errno = EFAULT;
        return -1;
    }
    if (n_events_ < 0) {
        // errno = EINVAL;
        return -1;
    }

    let rc = poller_.wait (events_, n_events_, timeout_);

    return rc;
}

// int zmq_poller_fd (void *poller_, zmq_fd_t *fd_)
pub unsafe fn zmq_poller_fd(poller_: &mut ZmqSocketPoller, fd_: &mut zmq_fd_t) -> i32
{
    // if (!poller_
    //     || !(static_cast<zmq::socket_poller_t *> (poller_)->check_tag ())) {
    //     errno = EFAULT;
    //     return -1;
    // }
    if !poller_.check_tag() {
        return -1;
    }
    return (poller_).signaler_fd (fd_);
}

// int zmq_socket_get_peer_state (void *s_,
//                                const void *routing_id_,
//                                size_t routing_id_size_)
pub unsafe fn zmq_socket_get_peer_state(s_: &mut ZmqSocketBase, routing_id_: &c_void, routing_id_size_: usize) -> i32
{
    // const zmq::socket_base_t *const s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;

    // return s->get_peer_state (routing_id_, routing_id_size_);
    s_.get_peer_state(routing_id_, routing_id_size_)
}

// void *zmq_timers_new (void)
pub unsafe fn zmq_timers_new() -> Timers
{
    // zmq::timers_t *timers = new (std::nothrow) zmq::timers_t;
    // alloc_assert (timers);
    // return timers;
    Timers::new()
}

// int zmq_timers_destroy (void **timers_p_)
pub unsafe fn zmq_timers_destroy(timers_p_: &mut Timers) -> i32
{
    // void *timers = *timers_p_;
    // if (!timers || !(static_cast<zmq::timers_t *> (timers))->check_tag ()) {
    //     errno = EFAULT;
    //     return -1;
    // }
    // delete (static_cast<zmq::timers_t *> (timers));
    // *timers_p_ = NULL;
    // return 0;
    todo!()
}

// int zmq_timers_add (void *timers_,
//                     size_t interval_,
//                     zmq_timer_fn handler_,
//                     void *arg_)
pub unsafe fn zmq_timers_add(timers_: &mut Timers, interval_: usize, handler_: zmq_timer_fn, arg_: &mut c_void) -> i32
{
    // if (!timers_ || !(static_cast<zmq::timers_t *> (timers_))->check_tag ()) {
    //     errno = EFAULT;
    //     return -1;
    // }
    if !timers_.check_tag() {
        return -1;
    }

    // return (static_cast<zmq::timers_t *> (timers_))
    //   ->add (interval_, handler_, arg_);
    timers_.add(interval_, handler_, arg_)
}

// int zmq_timers_cancel (void *timers_, int timer_id_)
pub unsafe fn zmq_timers_cancel(timers_: &mut Timers, timer_id_: i32) -> i32
{
    // if (!timers_ || !(static_cast<zmq::timers_t *> (timers_))->check_tag ()) {
    //     errno = EFAULT;
    //     return -1;
    // }
    if !timers_.check_tag() {
        return -1;
    }

    // return (static_cast<zmq::timers_t *> (timers_))->cancel (timer_id_);
    timers_.cancel(timer_id_)
}

// int zmq_timers_set_interval (void *timers_, int timer_id_, size_t interval_)
pub unsafe fn zmq_timers_set_interval(timers_: &mut Timers, timer_id_: i32, interval_: usize) -> i32
{
    // if (!timers_ || !(static_cast<zmq::timers_t *> (timers_))->check_tag ()) {
    //     errno = EFAULT;
    //     return -1;
    // }
    if !timers_.check_tag() {
        return -1;
    }

    // return (static_cast<zmq::timers_t *> (timers_))
    //   ->set_interval (timer_id_, interval_);
    timers_.set_interval(timer_id_, interval_)
}

// int zmq_timers_reset (void *timers_, int timer_id_)
pub unsafe fn zmq_timers_reset(timers_: &mut Timers, timer_id_: i32) -> i32
{
    // if (!timers_ || !(static_cast<zmq::timers_t *> (timers_))->check_tag ()) {
    //     errno = EFAULT;
    //     return -1;
    // }
    if !timers_.check_tag() {
        return -1;
    }

    // return (static_cast<zmq::timers_t *> (timers_))->reset (timer_id_);
    timers_.reset(timer_id_)
}

// long zmq_timers_timeout (void *timers_)
pub unsafe fn zmq_timers_timeout(timers_: &mut Timers) -> i32
{
    // if (!timers_ || !(static_cast<zmq::timers_t *> (timers_))->check_tag ()) {
    //     errno = EFAULT;
    //     return -1;
    // }
    if !timers_.check_tag() {
        return -1;
    }

    // return (static_cast<zmq::timers_t *> (timers_))->timeout ();
    timers_.timeout()
}

// int zmq_timers_execute (void *timers_)
pub unsafe fn zmq_timers_execute(timers_: &mut Timers) -> i32
{
    // if (!timers_ || !(static_cast<zmq::timers_t *> (timers_))->check_tag ()) {
    //     errno = EFAULT;
    //     return -1;
    // }
    if !timers_.check_tag() {
        return -1;
    }

    // return (static_cast<zmq::timers_t *> (timers_))->execute ();
    timers_.execute()
}

// int zmq_proxy (void *frontend_, void *backend_, void *capture_)
pub unsafe fn zmq_proxy(frontend_: &mut ZmqSocketBase, backend_: &mut ZmqSocketBase, capture_: &mut ZmqSocketBase) -> i32
{
    if (!frontend_ || !backend_) {
        errno = EFAULT;
        return -1;
    }
    return proxy ( (frontend_),
                        (backend_),
                        (capture_));
}

// int zmq_proxy_steerable (void *frontend_,
//                          void *backend_,
//                          void *capture_,
//                          void *control_)
pub unsafe fn zmq_proxy_steerable(frontend_: &mut ZmqSocketBase, backend_: &mut ZmqSocketBase, capture_: &mut ZmqSocketBase, control_: &mut ZmqSocketBase) -> i32
{
    // if (!frontend_ || !backend_ || !control_) {
    //     errno = EFAULT;
    //     return -1;
    // }
    // return proxy_steerable ( (frontend_),
    //                                 (backend_),
    //                                 (capture_),
    //                                 (control_));
    return -1;
}
// {
//     if (!frontend_ || !backend_) {
//         errno = EFAULT;
//         return -1;
//     }
// #ifdef ZMQ_HAVE_WINDOWS
//     errno = WSAEOPNOTSUPP;
// #else
//     errno = EOPNOTSUPP;
// #endif
//   return -1;
// }

// int zmq_device (int /* type */, void *frontend_, void *backend_)
pub unsafe fn zmq_device(type_: i32, frontend_: &mut ZmqSocketBase, backend_: &mut ZmqSocketBase) -> i32
{
    return proxy ((frontend_),(backend_), None);
}


// int zmq_has (const char *capability_)
pub unsafe fn zmq_has(capability_: &str) -> i32
{
    todo!()
// #if defined(ZMQ_HAVE_IPC)
//     if (strcmp (capability_, zmq::protocol_name::ipc) == 0)
//         return true;
// #endif
// #if defined(ZMQ_HAVE_OPENPGM)
//     if (strcmp (capability_, zmq::protocol_name::pgm) == 0)
//         return true;
// #endif
// #if defined(ZMQ_HAVE_TIPC)
//     if (strcmp (capability_, zmq::protocol_name::tipc) == 0)
//         return true;
// #endif
// #if defined(ZMQ_HAVE_NORM)
//     if (strcmp (capability_, zmq::protocol_name::norm) == 0)
//         return true;
// #endif
// #if defined(ZMQ_HAVE_CURVE)
//     if (strcmp (capability_, "curve") == 0)
//         return true;
// #endif
// #if defined(HAVE_LIBGSSAPI_KRB5)
//     if (strcmp (capability_, "gssapi") == 0)
//         return true;
// #endif
// #if defined(ZMQ_HAVE_VMCI)
//     if (strcmp (capability_, zmq::protocol_name::vmci) == 0)
//         return true;
// #endif
// #if defined(ZMQ_BUILD_DRAFT_API)
//     if (strcmp (capability_, "draft") == 0)
//         return true;
// #endif
// #if defined(ZMQ_HAVE_WS)
//     if (strcmp (capability_, "WS") == 0)
//         return true;
// #endif
// #if defined(ZMQ_HAVE_WSS)
//     if (strcmp (capability_, "WSS") == 0)
//         return true;
// #endif
//     //  Whatever the application asked for, we don't have
//     return false;
}

// int zmq_socket_monitor_pipes_stats (void *s_)
pub unsafe fn zmq_socket_monitor_pipes_stats(s_: &mut ZmqSocketBase) -> i32
{
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    // return s->query_pipes_stats ();
    s_.query_pipes_stats()
}
