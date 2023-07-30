use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use libc::{c_void, size_t};

use crate::utils::{copy_bytes, copy_void};

pub const CURVE_KEYSIZE: usize = 32;
pub const CURVE_KEYSIZE_Z85: usize = 40;

#[derive(Default,Debug,Clone)]
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

pub const default_hwm: i32 = 1000;

impl options_t
{
    pub fn new() -> Self {
        let mut out = Self{
            sndhwm: default_hwm,
          rcvhwm: default_hwm,
          rate: 100,
            recovery_ivl: 10000,
          multicast_hops: 1,
          multicast_maxtpdu: 1500,
          sndbuf: -1,
          rcvbuf: -1,
          type_: -1,
          linger: -1,
          reconnect_ivl: 100,
          backlog: 100,
            maxmsgsize: -1,
          rcvtimeo: -1,
          sndtimeo: -1,
          ipv6: false,
          filter: false,
          invert_matching: false,
          recv_routing_id: false,
            raw_socket: false,
            raw_notify: false,
            tcp_keepalive: -1,
          tcp_keepalive_cnt: -1,
          tcp_keepalive_idle: -1,
            tcp_keepalive_intvl: -1,
          mechanism: ZMQ_NULL,
          gss_principal_nt: ZMQ_GSSAPI_NT_HOSTBASED,
            gss_service_principal_nt: ZMQ_GSSAPI_NT_HOSTBASED,
            gss_plaintext: false,
          conflate: false,
          handshake_ivl: 30000,
          connected: false,
          heartbeat_timeout: -1,
          use_fd: -1,
          zap_enforce_domain: false,
          loopback_fastpath: true,
            multicast_loop: true,
            in_batch_size: 8192,
          out_batch_size: 8192,
          zero_copy: true,
          monitor_event_version: 1,
          wss_trust_system: false,
          can_send_hello_msg: false,
          can_recv_disconnect_msg: false,
          can_recv_hiccup_msg: false,
          norm_mode: ZMQ_NORM,
            norm_unicast_nacks: false,
          norm_buffer_size: 2048,
          norm_segment_size: 1400,
          norm_block_size: 16,
          norm_num_parity: 4,
          norm_push_enable: false,
          ..Default::default()
        };
        out
    }

    pub const CURVE_KEYSIZE_Z85_1: usize = CURVE_KEYSIZE_Z85 + 1;
    pub const deciseconds_per_millisecond: u32 = 100;

    pub unsafe fn set_curve_key(&mut self, destination: &mut [u8], optval_: *const c_void, optvallen_: size_t) -> i32
    {
        match optvallen_ {
            CURVE_KEYSIZE => {
                libc::memcpy(destination.as_mut_ptr() as *mut c_void, optval_, optvallen_);
                self.mechanism = ZMQ_CURVE;
                return 0;
            },
            CURVE_KEYSIZE_Z85_1 =>  {
                let s = std::str::from_utf8_unchecked(std::slice::from_raw_parts(optval_ as *const u8, optvallen_ as usize));
                if zmq_z85_decode(destination.as_mut_ptr() as *mut c_void, s) {
                    self.mechanism = ZMQ_CURVE;
                    return 0;
                }
            }
            CURVE_KEYSIZE_Z85 => {
               let mut z85_key: [i8;CURVE_KEYSIZE_Z85_1] = [0;CURVE_KEYSIZE_Z85_1];
                libc::memcpy(z85_key.as_mut_ptr() as *mut c_void, optval_, optvallen_);
                z85_key[CURVE_KEYSIZE_Z85] = 0;
                if zmq_z85_decode(destination.as_mut_ptr() as *mut c_void, z85_key.as_ptr() as *const i8) {
                    self.mechanism = ZMQ_CURVE;
                    return 0;
                }
            }
            _ => {}
        }

        return -1;
    }

    pub unsafe fn setsockopt(&mut self, option_: i32, optval_: *const c_void, optvallen_: size_t) -> i32 {
        let is_int = optvallen_ == std::mem::size_of::<i32>();
        let mut value = 0i32;
        if is_int {
            libc::memcpy(&mut value as *mut i32 as *mut c_void, optval_, optvallen_);
        }

        match option_ {
            ZMQ_SNDHWM => {
                if is_int {
                    self.sndhwm = value;
                    return 0;
                }
            },
            ZMQ_RCVHWM => {
                if is_int {
                    self.rcvhwm = value;
                    return 0;
                }
            },
            ZMQ_AFFINITY => {
               return do_setsockopt(optval_, optvallen_, &self.affinity);
            },
            ZMQ_ROUTING_ID => {
                if optvallen_ > 0 && optvallen_ <= UCHAR_MAX {
                    self.routing_id_size = optvallen_;
                    libc::memcpy(routing_id, optval_, self.routing_id_size);
                    reeturn 0;
                }
            },
            ZMQ_RATE => {
                if is_int {
                    self.rate = value;
                    return 0;
                }
            },
            ZMQ_RECOVERY_IVL => {
                if is_int {
                    self.recovery_ivl = value;
                    return 0;
                }
            },
            ZMQ_SNDBUF => {
                if is_int & value >=0 {
                    self.sndbuf = value;
                    return 0;
                }
            },
            ZMQ_RCVBUF => {
                if is_int & value >=0 {
                    self.rcvbuf = value;
                    return 0;
                }
            },
            ZMQ_TOS => {
                if is_int & value >=0 {
                    self.tos = value;
                    return 0;
                }
            },
            ZMQ_LINGER => {
                if is_int & value >=0 {
                    self.linger = value;
                    return 0;
                }
            },
            ZMQ_CONNECT_TIMEOUT => {
                if is_int & value >=0 {
                    self.connect_timeout = value;
                    return 0;
                }
            },
            ZMQ_TCP_MAXRT => {
                if is_int & value >=0 {
                    self.tcp_maxrt = value;
                    return 0;
                }
            },
            ZMQ_RECONNECT_STOP => {
                if is_int & value >=0 {
                    self.reconnect_stop = value;
                    return 0;
                }
            },
            ZMQ_RECONNECT_IVL => {
                if is_int & value >=0 {
                    self.reconnect_ivl = value;
                    return 0;
                }
            },
            ZMQ_BACKLOG => {
                if is_int & value >=0 {
                    self.backlog = value;
                    return 0;
                }
            },
            ZMQ_MAXMSGSIZE => {
                return do_setsockopt(optval_, optvallen_, &maxmsgsize)
            },
            ZMQ_MULTICAST_HOPS => {
                if is_int & value >=0 {
                    self.multicast_hops = value;
                    return 0;
                }
            },
            ZMQ_MULTICAST_MAXTPDU => {
                if is_int & value >=0 {
                    self.multicast_maxtpdu = value;
                    return 0;
                }
            },
            ZMQ_RCVTIMEO => {
                if is_int & value >=0 {
                    self.rcvtimeo = value;
                    return 0;
                }
            },
            ZMQ_SNDTIMEO => {
                if is_int & value >=0 {
                    self.sndtimeo = value;
                    return 0;
                }
            },
            ZMQ_IPV4ONLY => {
                let mut value = false;
                let rc = do_setsockopt_int_as_bool_relaxed(optval_, optvallen_, &mut value);
                if rc == 0 {
                    // self.ipv4only = value;
                    self.ipv6 = !value;
                }
                return rc;
            },
            ZMQ_IPV6=> {
                return do_setsockopt_int_as_bool_relaxed(optval_, optvallen_, &mut self.ipv6);
            }
            ZMQ_SOCKS_PROXY=> {
                return do_setsockopt_string_allow_empty_relaxed(optval_, optvallen_, &mut self.socks_proxy_address, SIZE_MAX);
            }
            _ => {
                return -1;
            }
        }
    }

    return -1;
}

pub fn get_effective_conflate_option(options: &options_t) -> bool
{
    return options.conflate && (options.type_ == ZMQ_DEALER || options.type_ == ZMQ_PULL || options.type_ == ZMQ_PUSH || options.type_ == ZMQ_PUB || options.type_ == ZMQ_SUB);
}

pub unsafe fn do_getsockopt<T>(optval_: *mut c_void, optvallen_: *const size_t, value_: T) -> i32

{
    todo!()
    // do_getsockopt4(optval_,*optvallen_, &value_, std::mem::size_of::<T>())
}

pub unsafe fn do_getsockopt2(optval_: *mut c_void, optvallen_: *const size_t, value_: &String) -> i32 {
    do_getsockopt3(optval_, optvallen_, (value_.as_ptr()) as *const c_void, value_.len() + 1)
}

pub unsafe fn do_getsockopt3(optval_: *mut c_void, optvallen_: *const size_t, value: *const c_void, value_len: size_t) -> i32 {
    if *optvallen_ < value_len {
        return -1;
    }

    copy_void(value, 0, value_len, optval_, 0, *optvallen_);
    return 0;
}

pub unsafe fn do_setsockopt<T>(optval_: *const c_void, optvallen_: size_t, out_value_: *const T) -> i32 {
    if optvallen_ != std::mem::size_of::<T>() {
        return -1;
    }
    todo!()

    // copy_bytes(optval_, 0, std::mem::size_of::<T>(), out_value_ as *mut c_void, 0);
    // return 0;
}

pub fn sockopt_invalid() -> i32 {
    // set errno = EINVAL
    return -1;
}

pub unsafe fn do_setsockopt_int_as_bool_strict(optval_: *const c_void, optvallen_: size_t, out_value_: *mut bool) -> i32 {
    let value = -1;
    if do_setsockopt(optval_, optvallen_, &value) == -1 {
        return -1;
    }
    if value == 0 || value == 1 {
        *out_value_ = (value != 0);
        return 0
    }
    return sockopt_invalid();
}

pub unsafe fn do_setsockopt_int_as_bool_relaxed(optval_: *const c_void, optvallen_: size_t, out_value_: &mut bool) -> i32 {
    let mut value = -1;
    if do_setsockopt(optval_, optvallen_, &value) == -1 {
        return -1;
    }
    *out_value_ = value != 0;
    return 0;
}

pub unsafe fn do_setsockopt_string_allow_empty_strict(optval_: *const c_void, optvallen_: size_t, out_value: &mut String, max_len: size_t) -> i32 {
    if optval_ == std::ptr::null() {
        out_value.clear();
        return 0;
    }

    if optval_ != std::ptr::null() && optvallen_ > 0 && optvallen_ <= max_len {
        *out_value = std::str::from_utf8_unchecked(std::slice::from_raw_parts(optval_ as *const u8, optvallen_)).to_string();
        return 0;
    }

    return sockopt_invalid();
}

pub unsafe fn do_setsockopt_string_allow_empty_relaxed(
    optval_: *const c_void,
    optvallen: size_t,
    out_value_: &mut String,
    max_len: size_t,
) -> i32 {
    if optvallen > 0 && optvallen <= max_len {
        *out_value_ = std::str::from_utf8_unchecked(std::slice::from_raw_parts(optval_ as *const u8, optvallen)).to_string();
        return 0;
    }

    return sockopt_invalid();
}


pub unsafe fn do_setsockopt_set<T: Eq + Hash>(optval_: *const c_void, optvallen_: size_t, set_: &mut HashSet<T>)
{
    if optvallen_ == 0 && optval_ == std::ptr::null() {
        set_.clear();
        return;
    }

    // if optvallen_ == std::mem::size_of::<T>() && optval_ != std::ptr::null() {
    //     let value = (optval_ as *const T).clone();
    //     set_.insert(*(value.clone()));
    //     return;
    // }
    todo!()
}