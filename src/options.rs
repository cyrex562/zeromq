use libc::{c_void, size_t};

use crate::utils::{copy_bytes, copy_bytes_void};

pub const CURVE_KEYSIZE: usize = 32;
pub const CURVE_KEYSIZE_Z85: usize = 40;

pub struct options_t
{
    pub affinity: u64,
    pub sndhwm: i32,
    pub rcvhwm: i32,
    pub routing_id_size: u8,
    pub routing_id: [u8;256],
    pub rate: i32,
    pub recovery_ivl: i32,
    pub multicast_hops: i32,
    pub multicast_maxtpdu: i32,
    pub sndbuf: i32,
    pub rcvbuf: i32,
    pub tos: i32,
    pub priority: i32,
    pub type_: i8,
    pub linger: u64,
    pub connect_timeout: i32,
    pub tcp_maxrt: i32,
    pub reconnect_stop: i32,
    pub reconnect_ivl: i32,
    pub reconnect_ivl_max: i32,
    pub backlog: i32,
    pub maxmsgsize: i64,
    pub rcvtimeo: i32,
    pub sndtimeo: i32,
    pub ipv6: bool,
    pub immediate: i32,
    pub filter: bool,
    pub invert_matching: bool,
    pub recv_routing_id: bool,
    pub raw_socket: bool,
    pub raw_notify: bool,
    pub socks_proxy_address: String,
    pub socks_proxy_username: String,
    pub socks_proxy_password: String,
    pub tcp_keepalive: i32,
    pub tcp_keepalive_idle: i32,
    pub tcp_keepalive_cnt: i32,
    pub tcp_keepalive_intvl: i32,

    pub tcp_accept_filters: Vec<tcp_address_mask_t>,
    pub mechanism: i32,
    pub as_server: i32,
    pub zap_domain: String,
    pub plain_username: String,
    pub plain_password: String,
    pub curve_public_key: [u8;CURVE_KEYSIZE],
    pub curve_secret_key: [u8;CURVE_KEYSIZE],
    pub curve_server_key: [u8;CURVE_KEYSIZE],
    pub gss_principal: String,
    pub gss_service_principal: String,
    pub gss_principal_nt: i32,
    pub gss_service_principal_nt: i32,
    pub gss_plaintext: bool,
    pub socket_id: i32,
    pub conflate: bool,
    pub handshake_ivl: i32,
    pub connected: bool,
    pub heartbeat_ttl: u16,
    pub heartbeat_timeout: i32,
    pub use_fd: i32,
    pub bound_device: String,
    pub zap_enforce_domain: bool,
    pub loopback_fastpath: bool,
    pub multicast_loop: bool,
    pub in_batch_size: i32,
    pub out_batch_size: i32,
    pub zero_copy: bool,
    pub router_notify: i32,
    pub app_metadata: HashMap< String,String>,
    pub monitor_event_version: i32,
    pub wss_key_pem: String,
    pub wss_cert_pem: String,
    pub wss_trust_pem: String,
    pub wss_hostname: String,
    pub wss_trust_system: bool,
    pub hello_msg: Vec<u8>,
    pub can_send_hello_msg: bool,
    pub can_recv_disconnect_msg: bool,
    pub disconnect_msg: Vec<u8>,
    pub hiccup_msg: Vec<u8>,
    pub can_recv_hiccup_msg: bool,
    pub norm_mode: i32,
    pub norm_unicast_nacks: bool,
    pub norm_buffer_size: i32,
    pub norm_segment_size: i32,
    pub norm_block_size: i32,
    pub norm_num_parity: i32,
    pub norm_num_autoparity: i32,
    pub norm_push_enable: bool,
    pub busy_poll: i32
}

pub fn get_effective_conflate_option(options: &options_t) -> bool
{
    return options.conflate && (options.type_ == ZMQ_DEALER || options.type_ == ZMQ_PULL || options.type_ == ZMQ_PUSH || options.type_ == ZMQ_PUB || options.type_ == ZMQ_SUB);
}

pub unsafe fn do_getsockopt<T>(optval_: *mut c_void, optvallen_: *const size_t, value_: T) -> i32
{
    do_getsockopt3(optval_,optvallen_, &value_ as *const c_void, std::mem::size_of::<T>())
}

pub unsafe fn do_getsockopt2(optval_: *mut c_void, optvallen_: *const size_t, value_: &String) -> i32 {
    do_getsockopt3(optval_, optvallen_, (value_.as_ptr()) as *const c_void, value_.len() + 1)
}

pub unsafe fn do_getsockopt3(optval_: *mut c_void, optvallen_: *const size_t, value: *const c_void, value_len: size_t) -> i32 {
    if *optvallen_ < value_len {
        return -1;
    }

    copy_bytes_void(value, 0, value_len, optval_, 0, *optvallen_);
    return 0;
}