use std::os::raw::{c_short, c_void};
use libc::EINTR;
use crate::ctx::ctx_t;
use crate::options::options_t;
use crate::socket_base::socket_base_t;
use crate::utils::get_errno;
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
pub fn zmq_strerror(errnum_: i32) -> &'static str
{
    // return zmq::errno_to_string (errnum_);
    todo!()
}

// int zmq_errno (void)
pub fn zmq_errno() -> i32
{
    // return errno;
    todo!()
}

// void *zmq_ctx_new (void)
pub fn zmq_ctx_new() -> Option<ctx_t>
{
    //  We do this before the ctx constructor since its embedded mailbox_t
    //  object needs the network to be up and running (at least on Windows).
    if (!initialize_network ()) {
        return None;
    }

    //  Create 0MQ context.
    // zmq::ctx_t *ctx = new (std::nothrow) zmq::ctx_t;
    let mut ctx = ctx_t::new();
    if (ctx) {
        if (!ctx.valid ()) {
            // delete ctx;
            return None;
        }
    }
    return Some(ctx);
}

// pub zmq_ctx_term (void *ctx_)
pub fn zmq_ctx_term(ctx_: &mut ctx_t) -> i32
{
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
        shutdown_network ();
    }

    // errno = en;
    return rc;
}

// int zmq_ctx_shutdown (void *ctx_)
pub fn zmq_ctx_shutdown(ctx_: &mut ctx_t) -> i32
{
    // if (!ctx_ || !(static_cast<zmq::ctx_t *> (ctx_))->check_tag ())
    if ctx_.check_tag() == false
    {
        // errno = EFAULT;
        return -1;
    }
    // return (static_cast<zmq::ctx_t *> (ctx_))->shutdown ();
    ctx_.shutdown()
}

// int zmq_ctx_set (void *ctx_, int option_, int optval_)
pub fn zmq_ctx_set(ctx_: &mut ctx_t, option_: i32, optval_: i32) -> i32
{
    zmq_ctx_set_ext (ctx_, option_, &optval_, 4)
}

pub fn zmq_ctx_set_ext(ctx_: &mut ctx_t, option_: i32, optval_: &[u8], optvallen_: usize) -> i32 {
    // if (!ctx_ || !(static_cast<zmq::ctx_t *> (ctx_))->check_tag ()) {
    //         errno = EFAULT;
    //         return -1;
    //     }
    //     return (static_cast<zmq::ctx_t *> (ctx_))
    //       ->set (option_, optval_, optvallen_);
    if ctx_.check_tag() == false
    {
        // errno = EFAULT;
        return -1;
    }
    ctx_.set(option_,optval_, optvallen_)
}

// int zmq_ctx_get (void *ctx_, int option_)
pub fn zmq_ctx_get(ctx_: &mut ctx_t, option_: i32) -> i32
{
    // if (!ctx_ || !(static_cast<zmq::ctx_t *> (ctx_))->check_tag ()) {
    //     errno = EFAULT;
    //     return -1;
    // }
    // return (static_cast<zmq::ctx_t *> (ctx_))->get (option_);
    if ctx_.check_tag() == false
    {
        // errno = EFAULT;
        return -1;
    }
    ctx_.get(option_)
}

// int zmq_ctx_get_ext (void *ctx_, int option_, void *optval_, size_t *optvallen_)
pub fn zmq_ctx_get_ext(ctx_: &mut ctx_t, option_: i32, optval_: &[u8], optvallen_: usize) -> i32
{
    // if (!ctx_ || !(static_cast<zmq::ctx_t *> (ctx_))->check_tag ()) {
    //     errno = EFAULT;
    //     return -1;
    // }
    // return (static_cast<zmq::ctx_t *> (ctx_))
    //   ->get (option_, optval_, optvallen_);
    if ctx_.check_tag() == false
    {
        // errno = EFAULT;
        return -1;
    }
    ctx_.get(option_, optval_, optvallen_)
}

// void *zmq_init (int io_threads_)
pub fn zmq_init(io_threads_: i32) -> Option<ctx_t>
{
    if (io_threads_ >= 0) {
        // void *ctx = zmq_ctx_new ();
        let mut ctx = zmq_ctx_new().unwrap();
        zmq_ctx_set (&mut ctx, ZMQ_IO_THREADS as i32, io_threads_);
        return Some(ctx);
    }
    // errno = EINVAL;
    return None;
}

// int zmq_term (void *ctx_)
pub fn zmq_term(ctx_: &mut ctx_t) -> i32
{
    return zmq_ctx_term (ctx_);
}

// int zmq_ctx_destroy (void *ctx_)
pub fn zmq_ctx_destroy(ctx_: &mut ctx_t) -> i32
{
    return zmq_ctx_term (ctx_);
}

// static zmq::socket_base_t *as_socket_base_t (void *s_)
pub fn as_socket_base_t<'a>(s_: &mut socket_base_t) -> Option<&'a mut socket_base_t>
{
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

pub unsafe fn zmq_socket<'a>(ctx_: &mut ctx_t, type_: i32) -> Option<&'a mut socket_base_t>
{
    // if (!ctx_ || !(static_cast<zmq::ctx_t *> (ctx_))->check_tag ())
    if ctx_.check_tag() == false
    {
        // errno = EFAULT;
        return None;
    }
    // zmq::ctx_t *ctx = static_cast<zmq::ctx_t *> (ctx_);
    // zmq::socket_base_t *s = ctx->create_socket (type_);
    // return static_cast<void *> (s);
    return ctx_.create_socket(type_);

}

// int zmq_close (void *s_)
pub unsafe fn zmq_close(s_: &mut socket_base_t) -> i32
{
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    s_.close ();
    return 0;
}

pub unsafe fn zmq_setsockopt(options: &mut options_t, s_: &mut socket_base_t, option_: i32, optval_: &[u8], optvallen_: usize) -> i32
{
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    // return s->setsockopt (option_, optval_, optvallen_);
    s_.setsockopt(options, option_, optval_, optvallen_)
}

// int zmq_getsockopt (void *s_, int option_, void *optval_, size_t *optvallen_)
pub unsafe fn zmq_getosckopt(s_: &mut socket_base_t, option_: i32, optval_: &[u8], optvallen_: usize) -> i32
{
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    // return s->getsockopt (option_, optval_, optvallen_);
    s_.getsockopt(option_, optval_, optvallen_)
}

// int zmq_socket_monitor_versioned (
//   void *s_, const char *addr_, uint64_t events_, int event_version_, int type_)
pub unsafe fn zmq_socket_monitor_versioned(
    s_: &mut socket_base_t, addr_: &str, events_: u64, event_version_: i32, type_: i32) -> i32
{
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    // return s->monitor (addr_, events_, event_version_, type_);
    s_.monitor(addr_, events_, event_version_, type_)
}

// int zmq_socket_monitor (void *s_, const char *addr_, int events_)
pub unsafe fn zmq_socket_monitor(s_: &mut socket_base_t, addr_: &str, events_: i32) -> i32
{
    return zmq_socket_monitor_versioned (s_, addr_, events_ as u64, 1, ZMQ_PAIR as i32);
}

// int zmq_join (void *s_, const char *group_)
pub unsafe fn zmq_join(s_: &mut socket_base_t, group_: &str) -> i32
{
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    // return s->join (group_);
    s_.join(group_)
}

// int zmq_leave (void *s_, const char *group_)
pub unsafe fn zmq_leave(s_: &mut socket_base_t, group_: &str) -> i32
{
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    // return s->leave (group_);
    s_.leave(group_)
}

// int zmq_bind (void *s_, const char *addr_)
pub unsafe fn zmq_bind(s_: &mut socket_base_t, addr_: &str) -> i32
{
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    // return s->bind (addr_);
    s_.bind(addr_)
}

// int zmq_connect (void *s_, const char *addr_)
pub unsafe fn zmq_connect(s_: &mut socket_base_t, addr_: &str) -> i32
{
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    // return s->connect (addr_);
    s_.connect(addr_)
}
