/*  DRAFT Socket types.                                                       */
use libc::SOCKET;

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
pub const ZMQ_RECONNECT_STOP_CONN_REFUSED: u32 = 1;
pub const ZMQ_RECONNECT_STOP_HANDSHAKE_FAILED: u32 = 2;
pub const ZMQ_RECONNECT_STOP_AFTER_DISCONNECT: u32 = 4;

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

// #if defined _WIN32
// typedef SOCKET zmq_fd_t;
// #else
// typedef int zmq_fd_t;
// #endif
#[cfg(target_os="windows")]
pub type zmq_fd_t = SOCKET;
#[cfg(not(target_os="windows"))]
pub type zmq_fd_t = i32;


pub struct zmq_poller_event_tj<'a>
{
    // void *socket;
    pub socket: &'a [u8],
    // zmq_fd_t fd;
    pub fd: zmq_fd_t,
    // void *user_data;
    pub user_data: &'a [u8],
    // short events;
    pub events: i16,
}
