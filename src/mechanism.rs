use std::ffi::{c_char, c_void};
use crate::defines::{MSG_ROUTING_ID, SOCKET_TYPE_CHANNEL, SOCKET_TYPE_CLIENT, SOCKET_TYPE_DEALER, SOCKET_TYPE_DGRAM, SOCKET_TYPE_DISH, SOCKET_TYPE_GATHER, SOCKET_TYPE_PAIR, SOCKET_TYPE_PEER, SOCKET_TYPE_PUB, SOCKET_TYPE_PULL, SOCKET_TYPE_PUSH, SOCKET_TYPE_RADIO, SOCKET_TYPE_REP, SOCKET_TYPE_REQ, SOCKET_TYPE_ROUTER, SOCKET_TYPE_SCATTER, SOCKET_TYPE_SERVER, SOCKET_TYPE_STREAM, SOCKET_TYPE_SUB, SOCKET_TYPE_XPUB, SOCKET_TYPE_XSUB, ZMQ_DEALER, ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_UNSPECIFIED, ZMQ_REQ, ZMQ_ROUTER};
use crate::metadata::ZmqDict;
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::session_base::ZmqSession;
use crate::utils::{get_u32, put_u32};
use crate::zap_client::ZapClient;

pub enum MechanismStatus {
    Handshaking,
    Ready,
    Error,
}

pub struct ZmqMechanism<'a> {
    // pub options: ZmqOptions,
    pub _zmtp_properties: ZmqDict,
    pub _zap_properties: ZmqDict,
    pub _routing_id: Vec<u8>,
    pub _user_id: Vec<u8>,
    pub session: &'a mut ZmqSession<'a>,
    pub zap_client: ZapClient,
    pub _ready_command_sent: bool,
    pub _error_command_sent: bool,
    pub _ready_command_received: bool,
    pub _error_command_received: bool,
    pub _zap_request_sent: bool,
    pub _zap_reply_received: bool,
}

pub const NAME_LEN_SIZE: u32 = 1;
pub const VALUE_LEN_SIZE: u32 = 4;

pub const ZMTP_PROPERTY_SOCKET_TYPE: &'static str = "Socket-Type";
pub const ZMTP_PROPERTY_IDENTITY: &'static str = "Identity";

impl ZmqMechanism {
    pub fn new(session: &mut ZmqSession) -> Self {
        Self {
            _zmtp_properties: ZmqDict::new(),
            _zap_properties: ZmqDict::new(),
            _user_id: vec![],
            session,
            zap_client: Default::default(),
            _ready_command_sent: false,
            _error_command_sent: false,
            _ready_command_received: false,
            _error_command_received: false,
            _zap_request_sent: false,
            _routing_id: vec![],

            _zap_reply_received: false,
        }
    }

    pub fn set_peer_routing_id(&mut self, id_ptr: *mut c_void, id_size_: usize) {
        self._routing_id.set(id_ptr as *mut u8, id_size_);
    }

    pub unsafe fn peer_routing_id(&mut self, msg_: *mut ZmqMsg) {
        let rc = (*msg_).init_size(self._routing_id.size());
        libc::memcpy((*msg_).data_mut(), self._routing_id.data() as *const c_void, self._routing_id.size());
        (*msg_).set_flags(MSG_ROUTING_ID);
    }

    pub fn set_user_id(&mut self, user_id_: *mut c_void, size_: usize) {
        self._user_id.set(user_id_ as *mut u8, size_);
        self._zap_properties.insert("user_id".to_string(), self._user_id._data());
    }

    pub fn get_user_id(&mut self) -> Vec<u8> {
        self._user_id.clone()
    }

    pub fn get_zmtp_properties(&mut self) -> &mut ZmqDict {
        return &mut self._zmtp_properties;
    }

    pub fn get_zap_properties(&mut self) -> &mut ZmqDict {
        return &mut self._zap_properties;
    }

    pub fn socket_type_string(&mut self, socket_type_: i32) -> &'static str {
        let names = [SOCKET_TYPE_PAIR, SOCKET_TYPE_PUB, SOCKET_TYPE_SUB, SOCKET_TYPE_REQ, SOCKET_TYPE_REP, SOCKET_TYPE_DEALER, SOCKET_TYPE_ROUTER, SOCKET_TYPE_PULL, SOCKET_TYPE_PUSH, SOCKET_TYPE_XPUB, SOCKET_TYPE_XSUB, SOCKET_TYPE_STREAM, SOCKET_TYPE_SERVER, SOCKET_TYPE_CLIENT, SOCKET_TYPE_RADIO, SOCKET_TYPE_DISH, SOCKET_TYPE_GATHER, SOCKET_TYPE_SCATTER, SOCKET_TYPE_DGRAM, SOCKET_TYPE_PEER, SOCKET_TYPE_CHANNEL];
        names[socket_type_ as usize]
    }

    pub unsafe fn add_property(&mut self, mut ptr_: *mut u8, ptr_capacity_: usize, name_: *mut u8, value: *mut c_void, value_len: usize) -> usize {
        let name_len = name_len(name_ as *mut c_char);
        let total_len = property_len(name_len, value_len);
        *ptr_ = name_len as u8;
        ptr_ = ptr_.add(NAME_LEN_SIZE as usize);
        libc::memcpy(ptr_ as *mut c_void, name_ as *mut c_void, name_len);
        put_u32(ptr_, value_len as u32);
        ptr_ = ptr_.add(VALUE_LEN_SIZE as usize);
        libc::memcpy(ptr_ as *mut c_void, value, value_len);
        total_len
    }

    pub unsafe fn property_len(&mut self, name: *mut c_char, value_len_: usize) -> usize {
        property_len(name_len(name), value_len_)
    }

    pub unsafe fn add_basic_properties(&mut self, ptr_: *mut u8, ptr_capacity_: usize) -> usize {
        let mut ptr = ptr_;
        let socket_type = self.socket_type_string(self.options.type_ as i32);
        ptr = ptr.add(
            self.add_property(ptr, ptr_capacity_, ZMTP_PROPERTY_SOCKET_TYPE.as_ptr() as *mut u8, socket_type as *mut c_void, libc::strlen(socket_type.as_ptr() as *mut c_char)) as usize
        );

        if self.options.type_ == ZMQ_REQ as i8 || self.options.type_ == ZMQ_DEALER as i8 || self.options.type_ == ZMQ_ROUTER as i8 {
            ptr = ptr.add(
                self.add_property(ptr, ptr_capacity_, ZMTP_PROPERTY_IDENTITY.as_ptr() as *mut u8, self._routing_id.data() as *mut c_void, self._routing_id.size()) as usize
            );
        }

        for it in self.options.app_metadata.iter() {
            ptr = ptr.add(
                self.add_property(ptr, ptr_capacity_, it.0.as_ptr() as *mut u8, it.1 as *mut c_void, it.1.len()) as usize
            );
        }

        ptr.sub(ptr_ as usize) as usize
    }

    pub unsafe fn basic_properties_len(&mut self) -> usize {
        let socket_type = self.socket_type_string(self.options.type_ as i32);
        let mut meta_len = 0usize;
        for it in self.options.app_metadata.iter() {
            meta_len += property_len(it.0.len(), it.1.len());
        }
        return self.property_len(ZMTP_PROPERTY_SOCKET_TYPE.as_ptr() as *mut c_char, socket_type.len()) + meta_len + if self.options.type_ == ZMQ_REQ as i8 || self.options.type_ == ZMQ_DEALER as i8 || self.options.type_ == ZMQ_ROUTER as i8 { self.property_len(ZMTP_PROPERTY_IDENTITY.as_ptr() as *mut c_char, self._routing_id.size()) } else { 0 };
    }

    pub unsafe fn make_command_with_basic_properties(&mut self, msg_: *mut ZmqMsg, prefix_: *mut c_char, prefix_len_: usize) {
        let command_size = prefix_len_ + self.basic_properties_len();
        let rc = (*msg_).init_size(command_size);
        // errno_assert (rc == 0);

        let mut ptr = ((*msg_).data_mut());

        //  Add prefix
        libc::memcpy(ptr, prefix_ as *const c_void, prefix_len_);
        ptr = ptr.add(prefix_len_);

        self.add_basic_properties(
            ptr as *mut u8, command_size - (ptr.offset_from((*msg_).data_mut())));
    }

    pub unsafe fn parse_metadata(&mut self, mut ptr_: *mut u8, length_: usize, zap_flag_: bool) -> i32 {
        let mut bytes_left = length_;

        while (bytes_left > 1) {
            let name_length = (*ptr_);
            ptr_ = ptr_.add(NAME_LEN_SIZE as usize);
            bytes_left -= NAME_LEN_SIZE;
            if (bytes_left < name_length as usize) {
                break;
            }

            let name = String::from(ptr_);
            // std::string (reinterpret_cast<const char *> (ptr_), name_length);
            ptr_ = ptr_.add(name_length as usize);
            bytes_left -= name_length;
            if (bytes_left < VALUE_LEN_SIZE as usize) {
                break;
            }

            let value_length = get_u32(ptr_);
            ptr_ = ptr_.add(VALUE_LEN_SIZE as usize);
            bytes_left -= VALUE_LEN_SIZE;
            if (bytes_left < value_length as usize) {
                break;
            }

            let value = ptr_;
            ptr_ = ptr_.add(value_length as usize);
            bytes_left -= value_length;

            if name == ZMTP_PROPERTY_IDENTITY && self.options.recv_routing_id {
                self.set_peer_routing_id(value as *mut c_void, value_length as usize);
            } else if name == ZMTP_PROPERTY_SOCKET_TYPE {
                if !self.check_socket_type(&String::from(value as *mut c_char),
                                           value_length as usize) {
                    // errno = EINVAL;
                    return -1;
                }
            } else {
                let rc = self.property(&name, value as *const c_void, value_length as usize);
                if rc == -1 {
                    return -1;
                }
            }
            // if (zap_flag_  _zap_properties : _zmtp_properties)
            // .ZMQ_MAP_INSERT_OR_EMPLACE (
            //     name,
            //     std::string (reinterpret_cast<const char *> (value), value_length));
            if zap_flag_ {
                self._zap_properties.insert(name.clone(), String::from(value));
            } else {
                self._zmtp_properties.insert(name.clone(), String::from(value));
            }
        }
        if bytes_left > 0 {
            // errno = EPROTO;
            return -1;
        }
        return 0;
    }

    pub fn property(&mut self, name: &str, value_: *const c_void, length_: usize) -> i32 {
        0
    }

    pub fn check_socket_type(&mut self, type_: &str, len_: usize) -> bool {
        match (self.options.type_) {
            ZMQ_REQ => {
                return strequals(type_, len_, SOCKET_TYPE_REP) || strequals(type_, len_, SOCKET_TYPE_ROUTER);
            }
            ZMQ_REP => {
                return strequals(type_, len_, SOCKET_TYPE_REQ) || strequals(type_, len_, SOCKET_TYPE_DEALER);
            }
            ZMQ_DEALER => {
                return strequals(type_, len_, SOCKET_TYPE_REP) || strequals(type_, len_, SOCKET_TYPE_DEALER) || strequals(type_, len_, SOCKET_TYPE_ROUTER);
            }
            ZMQ_ROUTER => {
                return strequals(type_, len_, SOCKET_TYPE_REQ) || strequals(type_, len_, SOCKET_TYPE_DEALER) || strequals(type_, len_, SOCKET_TYPE_ROUTER);
            }
            ZMQ_PUSH => {
                return strequals(type_, len_, SOCKET_TYPE_PULL);
            }
            ZMQ_PULL => {
                return strequals(type_, len_, SOCKET_TYPE_PUSH);
            }
            ZMQ_PUB => {
                return strequals(type_, len_, SOCKET_TYPE_SUB) || strequals(type_, len_, SOCKET_TYPE_XSUB);
            }
            ZMQ_SUB => {
                return strequals(type_, len_, SOCKET_TYPE_PUB) || strequals(type_, len_, SOCKET_TYPE_XPUB);
            }
            ZMQ_XPUB => {
                return strequals(type_, len_, SOCKET_TYPE_SUB) || strequals(type_, len_, SOCKET_TYPE_XSUB);
            }
            ZMQ_XSUB => {
                return strequals(type_, len_, SOCKET_TYPE_PUB) || strequals(type_, len_, SOCKET_TYPE_XPUB);
            }
            ZMQ_PAIR => {
                return strequals(type_, len_, SOCKET_TYPE_PAIR);
            }
            // #ifdef ZMQ_BUILD_DRAFT_API
            ZMQ_SERVER => {
                return strequals(type_, len_, SOCKET_TYPE_CLIENT);
            }
            ZMQ_CLIENT => {
                return strequals(type_, len_, SOCKET_TYPE_SERVER);
            }
            ZMQ_RADIO => {
                return strequals(type_, len_, SOCKET_TYPE_DISH);
            }
            ZMQ_DISH => {
                return strequals(type_, len_, SOCKET_TYPE_RADIO);
            }
            ZMQ_GATHER => {
                return strequals(type_, len_, SOCKET_TYPE_SCATTER);
            }
            ZMQ_SCATTER => {
                return strequals(type_, len_, SOCKET_TYPE_GATHER);
            }
            ZMQ_DGRAM => {
                return strequals(type_, len_, SOCKET_TYPE_DGRAM);
            }
            ZMQ_PEER => {
                return strequals(type_, len_, SOCKET_TYPE_PEER);
            }
            ZMQ_CHANNEL => {
                return strequals(type_, len_, SOCKET_TYPE_CHANNEL);
            }
            // #endif
            // default:
            // break;
        }
        return false;
    }

    pub fn check_basic_command_structure(&mut self, options: &ZmqOptions, msg: &mut ZmqMsg) -> i32 {
        if (msg).size() <= 1 || (msg).size() <= ((msg).data_mut())[0] as usize {
            self.session.get_socket().event_handshake_failed_protocol(
                options,
                self.session.get_endpoint(),
                ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_UNSPECIFIED as i32);
            // errno = EPROTO;
            return -1;
        }
        return 0;
    }

    pub fn handle_error_reason(&mut self, options: &ZmqOptions, error_reason_: &str, error_reason_len_: usize) {
        let status_code_len = 3;
        let zero_digit = '0';
        let significant_digit_index = 0;
        let first_zero_digit_index = 1;

        let second_zero_digit_index = 2;
        let factor = 100;
        if error_reason_len_ == status_code_len && error_reason_.chars().nth(first_zero_digit_index).unwrap() == zero_digit && error_reason_.chars().nth(second_zero_digit_index).unwrap() == zero_digit && error_reason_.chars().nth(significant_digit_index).unwrap() >= '3' && error_reason_.chars().nth(significant_digit_index).unwrap() <= '5' {
            // it is a ZAP Error status code (300, 400 or 500), so emit an authentication failure event
            self.session.get_socket().event_handshake_failed_auth(
                options,
                self.session.get_endpoint(),
                (error_reason_.chars().nth(significant_digit_index).unwrap() as u8 - zero_digit as u8) * factor);
        } else {
            // this is a violation of the ZAP protocol
            // TODO zmq_assert in this case?
        }
    }

    pub fn zap_required(&mut self, options: &ZmqOptions ) -> bool {
        return !options.zap_domain.empty();
    }
}

pub fn strequals(a: &str, c: usize, b: &str) -> bool {
    a == b
}

pub fn property_len(name_len_: usize, value_len_: usize) -> usize {
    (NAME_LEN_SIZE + name_len_ + VALUE_LEN_SIZE + value_len_) as usize
}

pub unsafe fn name_len(name_: *mut c_char) -> usize {
    let name_len_ = libc::strlen(name_);
    name_len_ as usize
}
