use crate::defines::*;
use libc::{c_void, size_t};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::ptr;
use crate::address::tcp_address::TcpAddressMask;
use crate::defines::err::ZmqError;
use crate::defines::err::ZmqError::OptionsError;
use crate::utils::{copy_bytes, zmq_z85_decode};

pub const CURVE_KEYSIZE: usize = 32;
pub const CURVE_KEYSIZE_Z85: usize = 40;
pub const DEFAULT_HWM: i32 = 1000;
pub const CURVE_KEYSIZE_Z85_1: usize = CURVE_KEYSIZE_Z85 + 1;
pub const DECISECONDS_PER_MILLISECOND: u32 = 100;

#[derive(Debug, Clone)]
pub struct ZmqOptions {
    pub affinity: u64,
    pub sndhwm: i32,
    pub rcvhwm: i32,
    pub routing_id_size: u8,
    pub routing_id: [u8; 256],
    pub rate: i32,
    pub recovery_ivl: i32,
    pub multicast_hops: i32,
    pub multicast_maxtpdu: i32,
    pub sndbuf: i32,
    pub rcvbuf: i32,
    pub tos: i32,
    pub priority: i32,
    pub socket_type: u32,
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
    pub tcp_accept_filters: Vec<TcpAddressMask>,
    pub mechanism: i32,
    pub as_server: i32,
    pub zap_domain: String,
    pub plain_username: String,
    pub plain_password: String,
    pub curve_public_key: [u8; CURVE_KEYSIZE],
    pub curve_secret_key: [u8; CURVE_KEYSIZE],
    pub curve_server_key: [u8; CURVE_KEYSIZE],
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
    pub heartbeat_interval: i32,
    pub heartbeat_timeout: i32,
    pub use_fd: i32,
    pub bound_device: String,
    pub zap_enforce_domain: bool,
    pub loopback_fastpath: bool,
    pub multicast_loop: bool,
    pub in_batch_size: i32,
    pub out_batch_size: usize,
    pub zero_copy: bool,
    pub router_notify: i32,
    pub app_metadata: HashMap<String, String>,
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
    pub busy_poll: i32,
}

impl ZmqOptions {
    pub fn new() -> Self {
        let mut out = Self {
            sndhwm: DEFAULT_HWM,
            rcvhwm: DEFAULT_HWM,
            rate: 100,
            recovery_ivl: 10000,
            multicast_hops: 1,
            multicast_maxtpdu: 1500,
            sndbuf: -1,
            rcvbuf: -1,
            socket_type: 0,
            linger: 0,
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
            mechanism: ZMQ_NULL as i32,
            gss_principal_nt: ZMQ_GSSAPI_NT_HOSTBASED as i32,
            gss_service_principal_nt: ZMQ_GSSAPI_NT_HOSTBASED as i32,
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
            // norm_mode: ZMQ_NORM,
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

    pub fn set_curve_key(
        &mut self,
        destination: &mut [u8],
        optval_: &[u8],
        optvallen_: size_t,
    ) -> Result<(),ZmqError> {
        match optvallen_ {
            CURVE_KEYSIZE => {
                // libc::memcpy(destination.as_mut_ptr() as *mut c_void, optval_, optvallen_);
                destination.copy_from_slice(optval_);
                self.mechanism = ZMQ_CURVE as i32;
                return Ok(());
            }
            CURVE_KEYSIZE_Z85_1 => {
                // let s = std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                //     optval_ as *const u8,
                //     optvallen_ as usize,
                // ));
                let s = unsafe{std::str::from_utf8_unchecked(optval_)};
                if zmq_z85_decode(destination.as_mut_ptr(), s.as_ptr() as *const i8) {
                    self.mechanism = ZMQ_CURVE as i32;
                    return Ok(());
                }
            }
            CURVE_KEYSIZE_Z85 => {
                let mut z85_key: [u8; CURVE_KEYSIZE_Z85_1] = [0; CURVE_KEYSIZE_Z85_1];
                // libc::memcpy(z85_key.as_mut_ptr() as *mut c_void, optval_, optvallen_);
                z85_key.copy_from_slice(optval_);
                z85_key[CURVE_KEYSIZE_Z85] = 0;
                if zmq_z85_decode(destination.as_mut_ptr(), z85_key.as_ptr() as *const i8) {
                    self.mechanism = ZMQ_CURVE as i32;
                    return Ok(());
                }
            }
            _ => {}
        }

        return Err(OptionsError("failed to set curve key"));
    }

    pub fn setsockopt(&mut self, option_: i32, optval_: &[u8], optvallen_: size_t) -> Result<(),ZmqError> {
        let is_int = optvallen_ == std::mem::size_of::<i32>();
        let mut value = 0i32;
        if is_int {
            value = u32::from_le_bytes(optval_.try_into().unwrap()) as i32;
        }

        match option_ as u32 {
            ZMQ_SNDHWM => {
                if is_int {
                    self.sndhwm = value;
                    return Ok(());
                }
            }
            ZMQ_RCVHWM => {
                if is_int {
                    self.rcvhwm = value;
                    return Ok(());
                }
            }
            ZMQ_AFFINITY => {
                return do_setsockopt(optval_, optvallen_, &self.affinity);
            }
            ZMQ_ROUTING_ID => {
                if optvallen_ > 0 && optvallen_ <= u8::MAX as usize {
                    self.routing_id_size = optvallen_ as u8;
                    let routing_id_ref = &mut self.routing_id;
                    let routing_id_ptr = routing_id_ref as *mut u8;
                    copy_bytes(optval_, 0, optvallen_, routing_id_ref, 0);
                    // libc::memcpy(
                    //     routing_id_ptr as *mut c_void,
                    //     optval_,
                    //     self.routing_id_size as size_t,
                    // );
                    return Ok(());
                }
            }
            ZMQ_RATE => {
                if is_int {
                    self.rate = value;
                    return Ok(());
                }
            }
            ZMQ_RECOVERY_IVL => {
                if is_int {
                    self.recovery_ivl = value;
                    return Ok(());
                }
            }
            ZMQ_SNDBUF => {
                if is_int && value >= 0 {
                    self.sndbuf = value;
                    return Ok(());
                }
            }
            ZMQ_RCVBUF => {
                if is_int && value >= 0 {
                    self.rcvbuf = value;
                    return Ok(());
                }
            }
            ZMQ_TOS => {
                if is_int && value >= 0 {
                    self.tos = value;
                    return Ok(());
                }
            }
            ZMQ_LINGER => {
                if is_int && value >= 0 {
                    self.linger = value as u64;
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
                if is_int && value >= 0 {
                    self.reconnect_stop = value;
                    return Ok(());
                }
            }
            ZMQ_RECONNECT_IVL => {
                if is_int && value >= 0 {
                    self.reconnect_ivl = value;
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
                return do_setsockopt(optval_, optvallen_, &self.maxmsgsize);
            }
            ZMQ_MULTICAST_HOPS => {
                if is_int && value >= 0 {
                    self.multicast_hops = value;
                    return Ok(());
                }
            }
            ZMQ_MULTICAST_MAXTPDU => {
                if is_int && value >= 0 {
                    self.multicast_maxtpdu = value;
                    return Ok(());
                }
            }
            ZMQ_RCVTIMEO => {
                if is_int && value >= 0 {
                    self.rcvtimeo = value;
                    return Ok(());
                }
            }
            ZMQ_SNDTIMEO => {
                if is_int && value >= 0 {
                    self.sndtimeo = value;
                    return Ok(());
                }
            }
            ZMQ_IPV4ONLY => {
                let mut value = false;
                let rc = do_setsockopt_int_as_bool_relaxed(optval_, optvallen_, &mut value);
                if rc == 0 {
                    // self.ipv4only = value;
                    self.ipv6 = !value;
                }
                // return rc;
                return if rc == 0 {
                    Ok(())
                } else {
                    Err(OptionsError("failed to set ipv4only"))
                };
            }
            ZMQ_IPV6 => {
                return do_setsockopt_int_as_bool_relaxed(optval_, optvallen_, &mut self.ipv6);
            }
            ZMQ_SOCKS_PROXY => {
                return do_setsockopt_string_allow_empty_relaxed(
                    optval_,
                    optvallen_,
                    &mut self.socks_proxy_address,
                    usize::MAX,
                );
            }
            ZMQ_SOCKS_USERNAME => {
                return if optval_ == ptr::null() || optvallen_ == 0 {
                    self.socks_proxy_username.clear();
                    0
                } else {
                    do_setsockopt_string_allow_empty_relaxed(
                        optval_,
                        optvallen_,
                        &mut self.socks_proxy_username,
                        255,
                    )
                }
            }
            ZMQ_SOCKS_PASSWORD => {
                return if optval_ == ptr::null() || optvallen_ == 0 {
                    self.socks_proxy_password.clear();
                    0
                } else {
                    do_setsockopt_string_allow_empty_relaxed(
                        optval_,
                        optvallen_,
                        &mut self.socks_proxy_password,
                        255,
                    )
                }
            }
            ZMQ_TCP_KEEPALIVE => {
                if is_int && value >= 0 {
                    self.tcp_keepalive = value;
                    return Ok(());
                }
            }
            ZMQ_TCP_KEEPALIVE_CNT => {
                if is_int && value >= 0 {
                    self.tcp_keepalive_cnt = value;
                    return Ok(());
                }
            }
            ZMQ_TCP_KEEPALIVE_IDLE => {
                if is_int && value >= 0 {
                    self.tcp_keepalive_idle = value;
                    return Ok(());
                }
            }
            ZMQ_TCP_KEEPALIVE_INTVL => {
                if is_int && value >= 0 {
                    self.tcp_keepalive_intvl = value;
                    return Ok(());
                }
            }
            ZMQ_IMMEDIATE => {
                if is_int && value == 0 || value == 1 {
                    self.immediate = value;
                    return Ok(());
                }
            }
            ZMQ_TCP_ACCEPT_FILTER => {
                let mut filter_str = String::new();
                let mut rc = do_setsockopt_string_allow_empty_relaxed(
                    optval_,
                    optvallen_,
                    &mut filter_str,
                    u8::MAX as size_t,
                );
                if rc == 0 {
                    if filter_str.is_empty() {
                        self.tcp_accept_filters.clear();
                    } else {
                        let mut mask: tcp_address_mask_t = tcp_address_mask_t::default();
                        if mask.resolve(filter_str.as_str(), self.ipv6).is_ok() {
                            self.tcp_accept_filters.push(mask);
                            rc = 0;
                        }
                    }
                }
                return if rc == 0 {
                    Ok(())
                } else {
                    Err(OptionsError("failed to set tcp accept filter"))
                };
            }
            ZMQ_PLAIN_SERVER => {
                if is_int && (value == 0 || value == 1) {
                    self.as_server = value;
                    self.mechanism = if value != 0 { ZMQ_PLAIN } else { ZMQ_NULL } as i32;
                    return Ok(());
                }
            }
            ZMQ_PLAIN_USERNAME => {
                if optvallen_ == 0 && optval_ == ptr::null() {
                    self.mechanism = ZMQ_NULL as i32;
                    return 0;
                } else if optvallen_ > 0
                    && optvallen_ <= u8::MAX as size_t
                    && optval_ != ptr::null()
                {
                    self.plain_username = String::from_raw_parts(
                        optval_ as *mut u8,
                        optvallen_ as usize,
                        optvallen_ as usize,
                    );
                    self.as_server = 0;
                    self.mechanism = ZMQ_PLAIN as i32;
                    return Ok(());
                }
            }
            ZMQ_PLAIN_PASSWORD => {
                if optvallen_ == 0 && optval_ == ptr::null() {
                    self.mechanism = ZMQ_NULL as i32;
                    return Ok(());
                } else if optvallen_ > 0
                    && optvallen_ <= u8::MAX as size_t
                    && optval_ != ptr::null()
                {
                    self.plain_password = String::from_raw_parts(
                        optval_ as *mut u8,
                        optvallen_ as usize,
                        optvallen_ as usize,
                    );
                    self.as_server = 0;
                    self.mechanism = ZMQ_PLAIN as i32;
                    return Ok(());
                }
            }
            ZMQ_ZAP_DOMAIN => {
                return do_setsockopt_string_allow_empty_relaxed(
                    optval_,
                    optvallen_,
                    &mut self.zap_domain,
                    255,
                );
            }
            ZMQ_CONFLATE => {
                return do_setsockopt_int_as_bool_relaxed(optval_, optvallen_, &mut self.conflate);
            }
            ZMQ_HANDSHAKE_IVL => {
                if is_int && value >= 0 {
                    self.handshake_ivl = value;
                    return Ok(());
                }
            }
            ZMQ_INVERT_MATCHING => {
                return do_setsockopt_int_as_bool_relaxed(
                    optval_,
                    optvallen_,
                    &mut self.invert_matching,
                );
            }
            ZMQ_HEARTBEAT_IVL => {
                if is_int && value >= 0 {
                    self.heartbeat_interval = value;
                    return Ok(());
                }
            }
            ZMQ_HEARTBEAT_TTL => {
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
            ZMQ_USE_FD => {
                if is_int && value >= 0 {
                    self.use_fd = value;
                    return Ok(());
                }
            }
            ZMQ_BINDTODEVICE => {
                return do_setsockopt_string_allow_empty_relaxed(
                    optval_,
                    optvallen_,
                    &mut self.bound_device,
                    255,
                );
            }
            ZMQ_ZAP_ENFORCE_DOMAIN => {
                return do_setsockopt_int_as_bool_relaxed(
                    optval_,
                    optvallen_,
                    &mut self.zap_enforce_domain,
                );
            }
            ZMQ_LOOPBACK_FASTPATH => {
                return do_setsockopt_int_as_bool_relaxed(
                    optval_,
                    optvallen_,
                    &mut self.loopback_fastpath,
                );
            }
            ZMQ_METADATA => {
                if optvallen_ > 0 && !is_int {
                    let mut s = String::new();
                    s = String::from_raw_parts(
                        optval_ as *mut u8,
                        optvallen_ as usize,
                        optvallen_ as usize,
                    );
                    let pos = s.find(':');
                    if pos.is_some() {
                        let (key, value) = s.split_at(pos.unwrap());
                        self.app_metadata.insert(key.to_string(), value.to_string());
                        return Ok(());
                    }
                }
                // errno = EINVAL
                return Err(OptionsError("failed to set metadata"));
            }
            ZMQ_MULTICAST_LOOP => {
                do_setsockopt_int_as_bool_relaxed(optval_, optvallen_, &mut self.multicast_loop);
            }
            ZMQ_IN_BATCH_SIZE => {
                if is_int && value >= 0 {
                    self.in_batch_size = value;
                    return Ok(());
                }
            }
            ZMQ_OUT_BATCH_SIZE => {
                if is_int && value >= 0 {
                    self.out_batch_size = value;
                    return Ok(());
                }
            }
            ZMQ_BUSY_POLL => {
                if is_int && value >= 0 {
                    self.busy_poll = value;
                    return Ok(());
                }
            }
            ZMQ_HELLO_MSG => {
                if optvallen_ > 0 {
                    let bytes = optval_ as *mut u8;
                    self.hello_msg =
                        Vec::from_raw_parts(bytes, optvallen_ as usize, optvallen_ as usize);
                } else {
                    self.hello_msg = Vec::new();
                }
            }
            ZMQ_DISCONNECT_MSG => {
                if optvallen_ > 0 {
                    let bytes = optval_ as *mut u8;
                    self.disconnect_msg =
                        Vec::from_raw_parts(bytes, optvallen_ as usize, optvallen_ as usize);
                } else {
                    self.disconnect_msg = Vec::new();
                }
            }
            ZMQ_PRIORITY => {
                if is_int && value >= 0 && value <= u8::MAX as i32 {
                    self.priority = value as i32;
                    return Ok(());
                }
            }
            ZMQ_HICCUP_MSG => {
                if optvallen_ > 0 {
                    let bytes = optval_ as *mut u8;
                    self.hiccup_msg =
                        Vec::from_raw_parts(bytes, optvallen_ as usize, optvallen_ as usize);
                } else {
                    self.hiccup_msg = Vec::new();
                }
            }
            _ => {
                return Err(OptionsError("failed to set socket option"));
            }
        }

        return Err(OptionsError("failed to set socket option"));
    }

    pub fn getsockopt(
        &mut self,
        option_: u32,
    ) -> Result<Vec<u8>, ZmqError> {
        let is_int = *optvallen_ == std::mem::size_of::<i32>();
        let value: *mut i32 = optval_ as *mut i32;
        match option_ {
            ZMQ_SNDHWM => {
                if is_int {
                    return Ok(self.sndhwm.to_le_bytes().into_vec());
                }
            }
            ZMQ_RCVHWM => {
                if is_int {
                    // *value = self.rcvhwm;
                    // return 0;
                    return Ok(self.rcvhwm.to_le_bytes().into_vec());
                }
            }
            ZMQ_AFFINITY => {
                if *optvallen_ == std::mem::size_of::<u64>() {
                    // let value: *mut u64 = optval_ as *mut u64;
                    // *value = self.affinity;
                    // return 0;
                    return Ok(self.affinity.to_le_bytes().into_vec());
                }
            }
            ZMQ_ROUTING_ID => {
                return do_getsockopt(optval_, optvallen_, &self.routing_id);
            }
            ZMQ_RATE => {
                if is_int {
                    // *value = self.rate;
                    // return 0;
                    return Ok(self.rate.to_le_bytes().into_vec());
                }
            }
            ZMQ_RECOVERY_IVL => {
                if is_int {
                    // *value = self.recovery_ivl;
                    // return 0;
                    return Ok(self.recovery_ivl.to_le_bytes().into_vec());
                }
            }
            ZMQ_SNDBUF => {
                if is_int {
                    // *value = self.sndbuf;
                    // return 0;
                    return Ok(self.sndbuf.to_le_bytes().into_vec());
                }
            }
            ZMQ_RCVBUF => {
                if is_int {
                    // *value = self.rcvbuf;
                    // return 0;
                    return Ok(self.rcvbuf.to_le_bytes().into_vec());
                }
            }
            ZMQ_TOS => {
                if is_int {
                    // *value = self.tos;
                    // return 0;
                    return Ok(self.tos.to_le_bytes().into_vec());
                }
            }
            ZMQ_TYPE => {
                if is_int {
                    // *value = self.socket_type as i32;
                    // return 0;
                    return Ok(self.socket_type.to_le_bytes().into_vec());
                }
            }
            ZMQ_LINGER => {
                if is_int {
                    // *value = self.linger as i32;
                    // return 0;
                    return Ok(self.linger.to_le_bytes().into_vec());
                }
            }
            ZMQ_CONNECT_TIMEOUT => {
                if is_int {
                    // *value = self.connect_timeout;
                    // return 0;
                    return Ok(self.connect_timeout.to_le_bytes().into_vec());
                }
            }
            ZMQ_TCP_MAXRT => {
                if is_int {
                    // *value = self.tcp_maxrt;
                    // return 0;
                    return Ok(self.tcp_maxrt.to_le_bytes().into_vec());
                }
            }
            ZMQ_RECONNECT_STOP => {
                if is_int {
                    // *value = self.reconnect_stop;
                    // return 0;
                    return Ok(self.reconnect_stop.to_le_bytes().into_vec());
                }
            }
            ZMQ_RECONNECT_IVL => {
                if is_int {
                    // *value = self.reconnect_ivl;
                    // return 0;
                    return Ok(self.reconnect_ivl.to_le_bytes().into_vec());
                }
            }
            ZMQ_RECONNECT_IVL_MAX => {
                if is_int {
                    // *value = self.reconnect_ivl_max;
                    // return 0;
                    return Ok(self.reconnect_ivl_max.to_le_bytes().into_vec());
                }
            }
            ZMQ_BACKLOG => {
                if is_int {
                    // *value = self.backlog;
                    // return 0;
                    return Ok(self.backlog.to_le_bytes().into_vec());
                }
            }
            ZMQ_MAXMSGSIZE => {
                if is_int {
                    // *value = self.maxmsgsize as i32;
                    // return 0;
                    return Ok(self.maxmsgsize.to_le_bytes().into_vec());
                }
            }
            ZMQ_MULTICAST_HOPS => {
                if is_int {
                    // *value = self.multicast_hops;
                    // return 0;
                    return Ok(self.multicast_hops.to_le_bytes().into_vec());
                }
            }
            ZMQ_MULTICAST_MAXTPDU => {
                if is_int {
                    // *value = self.multicast_maxtpdu;
                    // return 0;
                    return Ok(self.multicast_maxtpdu.to_le_bytes().into_vec());
                }
            }
            ZMQ_RCVTIMEO => {
                if is_int {
                    // *value = self.rcvtimeo;
                    // return 0;
                    return Ok(self.rcvtimeo.to_le_bytes().into_vec());
                }
            }
            ZMQ_SNDTIMEO => {
                if is_int {
                    // *value = self.sndtimeo;
                    // return 0;
                    return Ok(self.sndtimeo.to_le_bytes().into_vec());
                }
            }
            ZMQ_IPV4ONLY => {
                if is_int {
                    // *value = self.ipv4only;
                    // return 0;
                    return Ok(self.ipv4only.to_le_bytes().into_vec());
                }
            }
            ZMQ_IPV6 => {
                if is_int {
                    // *value = i32::from(self.ipv6);
                    // return 0;
                    return Ok(self.ipv6.to_le_bytes().into_vec());
                }
            }
            ZMQ_IMMEDIATE => {
                if is_int {
                    // *value = self.immediate;
                    // return 0;
                    return Ok(self.immediate.to_le_bytes().into_vec());
                }
            }
            ZMQ_SOCKS_PROXY => {
                return do_getsockopt(optval_, optvallen_, &self.socks_proxy_address);
            }
            ZMQ_SOCKS_USERNAME => {
                return do_getsockopt(optval_, optvallen_, &self.socks_username);
            }
            ZMQ_SOCKS_PASSWORD => {
                return do_getsockopt(optval_, optvallen_, &self.socks_password);
            }
            ZMQ_TCP_KEEPALIVE => {
                if is_int {
                    // *value = self.tcp_keepalive;
                    // return 0;
                    return Ok(self.tcp_keepalive.to_le_bytes().into_vec());
                }
            }
            ZMQ_TCP_KEEPALIVE_CNT => {
                if is_int {
                    // *value = self.tcp_keepalive_cnt;
                    // return 0;
                    return Ok(self.tcp_keepalive_cnt.to_le_bytes().into_vec());
                }
            }
            ZMQ_TCP_KEEPALIVE_IDLE => {
                if is_int {
                    // *value = self.tcp_keepalive_idle;
                    // return 0;
                    return Ok(self.tcp_keepalive_idle.to_le_bytes().into_vec());
                }
            }
            ZMQ_TCP_KEEPALIVE_INTVL => {
                if is_int {
                    // *value = self.tcp_keepalive_intvl;
                    // return 0;
                    return Ok(self.tcp_keepalive_intvl.to_le_bytes().into_vec());
                }
            }
            ZMQ_MECHANISM => {
                if is_int {
                    // *value = self.mechanism as i32;
                    // return 0;
                    return Ok(self.mechanism.to_le_bytes().into_vec());
                }
            }
            ZMQ_PLAIN_SERVER => {
                if is_int {
                    // *value = self.plain_server;
                    // return 0;
                    return Ok(self.plain_server.to_le_bytes().into_vec());
                }
            }
            ZMQ_PLAIN_USERNAME => {
                return do_getsockopt(optval_, optvallen_, &self.plain_username);
            }
            ZMQ_PLAIN_PASSWORD => {
                return do_getsockopt(optval_, optvallen_, &self.plain_password);
            }
            ZMQ_ZAP_DOMAIN => {
                return do_getsockopt(optval_, optvallen_, &self.zap_domain);
            }
            ZMQ_CONFLATE => {
                if is_int {
                    // *value = i32::from(self.conflate);
                    // return 0;
                    return Ok(self.conflate.to_le_bytes().into_vec());
                }
            }
            ZMQ_HANDSHAKE_IVL => {
                if is_int {
                    // *value = self.handshake_ivl;
                    // return 0;
                    return Ok(self.handshake_ivl.to_le_bytes().into_vec());
                }
            }
            ZMQ_INVERT_MATCHING => {
                if is_int {
                    // *value = i32::from(self.invert_matching);
                    // return 0;
                    return Ok(self.invert_matching.to_le_bytes().into_vec());
                }
            }
            ZMQ_HEARTBEAT_IVL => {
                if is_int {
                    // *value = self.heartbeat_ivl;
                    // return 0;
                    return Ok(self.heartbeat_interval.to_le_bytes().into_vec());
                }
            }
            ZMQ_HEARTBEAT_TTL => {
                if is_int {
                    // *value = self.heartbeat_ttl as i32;
                    // return 0;
                    return Ok(self.heartbeat_ttl.to_le_bytes().into_vec());
                }
            }
            ZMQ_HEARTBEAT_TIMEOUT => {
                if is_int {
                    // *value = self.heartbeat_timeout;
                    // return 0;
                    return Ok(self.heartbeat_timeout.to_le_bytes().into_vec());
                }
            }
            ZMQ_USE_FD => {
                if is_int {
                    // *value = self.use_fd;
                    // return 0;
                    return Ok(self.use_fd.to_le_bytes().into_vec());
                }
            }
            ZMQ_BINDTODEVICE => {
                return do_getsockopt(optval_, optvallen_, &self.bindtodevice);
            }
            ZMQ_ZAP_ENFORCE_DOMAIN => {
                if is_int {
                    // *value = i32::from(self.zap_enforce_domain);
                    // return 0;
                    return Ok(self.zap_enforce_domain.to_le_bytes().into_vec());
                }
            }
            ZMQ_LOOPBACK_FASTPATH => {
                if is_int {
                    // *value = i32::from(self.loopback_fastpath);
                    // return 0;
                    return Ok(self.loopback_fastpath.to_le_bytes().into_vec());
                }
            }
            ZMQ_MULTICAST_LOOP => {
                if is_int {
                    // *value = self.lmulticast_loop;
                    // return 0;
                    return Ok(self.lmulticast_loop.to_le_bytes().into_vec());
                }
            }
            ZMQ_ROUTER_NOTIFY => {
                if is_int {
                    // *value = self.router_notify;
                    // return 0;
                    return Ok(self.router_notify.to_le_bytes().into_vec());
                }
            }
            ZMQ_IN_BATCH_SIZE => {
                if is_int {
                    // *value = self.in_batch_size;
                    // return 0;
                    return Ok(self.in_batch_size.to_le_bytes().into_vec());
                }
            }
            ZMQ_OUT_BATCH_SIZE => {
                if is_int {
                    // *value = self.out_batch_size;
                    // return 0;
                    return Ok(self.out_batch_size.to_le_bytes().into_vec())
                }
            }
            ZMQ_PRIORITY => {
                if is_int {
                    // *value = self.priority;
                    // return 0;
                    return Ok(self.priority.to_le_bytes().into_vec())
                }
            }
            ZMQ_BUSY_POLL => {
                if is_int {
                    // *value = self.busy_poll;
                    // return 0;
                    return Ok(self.busy_poll.to_le_bytes().into_vec())
                }
            }
            _ => {
                return Err(OptionsError("failed to get socket option"));
            }
        }
        // errno = EINVAL
        return Err(OptionsError("failed to get socket option"));
    }
}

pub fn get_effective_conflate_option(options: &ZmqOptions) -> bool {
    return options.conflate
        && (options.socket_type == ZMQ_DEALER
            || options.socket_type == ZMQ_PULL
            || options.socket_type == ZMQ_PUSH
            || options.socket_type == ZMQ_PUB
            || options.socket_type == ZMQ_SUB);
}

pub fn do_getsockopt<T>(value_: T) -> Result<[u8], ZmqError> {
    todo!()
    // do_getsockopt4(optval_,*optvallen_, &value_, std::mem::size_of::<T>())
}

pub fn do_getsockopt2(
    optval_: *mut c_void,
    optvallen_: *const size_t,
    value_: &String,
) -> Result<(),ZmqError> {
    do_getsockopt3(
        optval_,
        optvallen_,
        (value_.as_ptr()) as *const c_void,
        value_.len() + 1,
    )
}

pub fn do_getsockopt3(
    optval_: *mut c_void,
    optvallen_: *const size_t,
    value: *const c_void,
    value_len: size_t,
) -> Result<(),ZmqError> {
    if *optvallen_ < value_len {
        return Err(OptionsError("failed to get socket option"));
    }

    copy_void(value, 0, value_len, optval_, 0, *optvallen_);
    return Ok(());
}

// TODO
pub fn do_setsockopt<T>(optval_: &[u8], optvallen_: size_t, out_value_: *const T) -> Result<(),ZmqError> {
    if optvallen_ != std::mem::size_of::<T>() {
        return Err(OptionsError("failed to set socket option"));
    }
    todo!()

    // copy_bytes(optval_, 0, std::mem::size_of::<T>(), out_value_ as *mut c_void, 0);
    // return 0;
}

pub fn sockopt_invalid() -> Result<(),ZmqError> {
    // set errno = EINVAL
    return Err(OptionsError("failed to set socket option"));
}

pub fn do_setsockopt_int_as_bool_strict(
    optval_: &[u8],
    optvallen_: size_t,
    out_value_: &mut bool,
) -> Result<(),ZmqError> {
    let value = -1;
    do_setsockopt(optval_, optvallen_, &value)?;
    if value == 0 || value == 1 {
        *out_value_ = value != 0;
        return Ok(());
    }
    return sockopt_invalid();
}

pub fn do_setsockopt_int_as_bool_relaxed(
    optval_: *const c_void,
    optvallen_: size_t,
    out_value_: &mut bool,
) -> Result<(),ZmqError> {
    let mut value = -1;
    // if do_setsockopt(optval_, optvallen_, &value) == -1 {
    //     return -1;
    // }
    do_setsockopt(optval_, optvallen_, &value)?;
    *out_value_ = value != 0;
    return Ok(());
}

pub fn do_setsockopt_string_allow_empty_strict(
    optval_: *const c_void,
    optvallen_: size_t,
    out_value: &mut String,
    max_len: size_t,
) -> Result<(),ZmqError> {
    if optval_ == std::ptr::null() {
        out_value.clear();
        return Ok(());
    }

    if optval_ != std::ptr::null() && optvallen_ > 0 && optvallen_ <= max_len {
        *out_value = std::str::from_utf8_unchecked(std::slice::from_raw_parts(
            optval_ as *const u8,
            optvallen_,
        ))
        .to_string();
        return Ok(());
    }

    return sockopt_invalid();
}

pub fn do_setsockopt_string_allow_empty_relaxed(
    optval_: *const c_void,
    optvallen: size_t,
    out_value_: &mut String,
    max_len: size_t,
) -> Result<(),ZmqError> {
    if optvallen > 0 && optvallen <= max_len {
        *out_value_ = std::str::from_utf8_unchecked(std::slice::from_raw_parts(
            optval_ as *const u8,
            optvallen,
        ))
        .to_string();
        return Ok(());
    }

    return sockopt_invalid();
}

pub fn do_setsockopt_set<T: Eq + Hash>(
    optval_: *const c_void,
    optvallen_: size_t,
    set_: &mut HashSet<T>,
) -> Result<(),ZmqError>{
    if optvallen_ == 0 && optval_ == std::ptr::null() {
        set_.clear();
        return Err(OptionsError("failed to set socket option"));
    }

    // if optvallen_ == std::mem::size_of::<T>() && optval_ != std::ptr::null() {
    //     let value = (optval_ as *const T).clone();
    //     set_.insert(*(value.clone()));
    //     return;
    // }
    todo!()
}
