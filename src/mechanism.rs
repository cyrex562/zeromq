use std::ffi::{c_char, c_void};
use crate::blob::blob_t;
use crate::defines::{ZMQ_DEALER, ZMQ_REQ, ZMQ_ROUTER};
use crate::metadata::dict_t;
use crate::msg::{msg_t, routing_id};
use crate::options::options_t;
use crate::utils::{get_u32, put_u32};

pub enum status_t{
    handshaking,
    ready,
    error
}
pub struct mechanism_t {
    pub options: options_t,
    pub _zmtp_properties: dict_t,
    pub _zap_properties: dict_t,
    pub _routing_id: blob_t,
    pub _user_id: blob_t,
}

pub const socket_type_pair: &'static str = "PAIR";
pub const socket_type_pub: &'static str = "PUB";
pub const socket_type_sub: &'static str = "SUB";
pub const socket_type_req: &'static str = "REQ";
pub const socket_type_rep: &'static str = "REP";
pub const socket_type_dealer: &'static str = "DEALER";
pub const socket_type_router: &'static str = "ROUTER";
pub const socket_type_pull: &'static str = "PULL";
pub const socket_type_push: &'static str = "PUSH";
pub const socket_type_xpub: &'static str = "XPUB";
pub const socket_type_xsub: &'static str = "XSUB";
pub const socket_type_stream: &'static str = "STREAM";
pub const socket_type_server: &'static str = "SERVER";
pub const socket_type_client: &'static str = "CLIENT";
pub const socket_type_radio: &'static str = "RADIO";
pub const socket_type_dish: &'static str = "DISH";
pub const socket_type_gather: &'static str = "GATHER";
pub const socket_type_scatter: &'static str = "SCATTER";
pub const socket_type_dgram: &'static str = "DGRAM";
pub const socket_type_peer: &'static str = "PEER";
pub const socket_type_channel: &'static str = "CHANNEL";

pub const name_len_size: u32 = 1;
pub const value_len_size: u32 = 4;

pub const ZMTP_PROPERTY_SOCKET_TYPE: &'static str = "Socket-Type";
pub const ZMTP_PROPERTY_IDENTITY: &'static str = "Identity";

pub trait mechanism_ops {
    fn next_handshake_command(&mut self, msg_: *mut msg_t) -> i32;
    fn process_handshake_command(&mut self, msg_: *mut msg_t) -> i32;
    fn encode(&mut self, msg_: *mut msg_t) -> i32 { return 0; }
    fn decode(&mut self, msg_: *mut msg_t) -> i32 { return 0; }

    fn zap_msg_available(&mut self) -> i32 { return 0; }

    fn status(&mut self) -> status_t;

    fn property(&mut self, name_: &str, value_: *mut c_void, length: usize);
}

impl mechanism_t  {

    pub fn new(options: &options_t) -> Self {
        Self {
            options: options.clone(),
            _zmtp_properties: dict_t::new(),
            _zap_properties: dict_t::new(),
            _user_id: blob_t::new(),
            _routing_id: blob_t::new()
        }
    }

    pub fn set_peer_routing_id(&mut self, id_ptr: *mut c_void, id_size_: usize)
    {
        self._routing_id.set(id_ptr as *mut u8, id_size_);
    }

    pub unsafe fn peer_routing_id(&mut self, msg_: *mut msg_t) {
        let rc = (*msg_).init_size(self._routing_id.size());
        libc::memcpy((*msg_).data(), self._routing_id.data() as *const c_void, self._routing_id.size());
        (*msg_).set_flags(routing_id);
    }

    pub fn set_user_id(&mut self, user_id_: *mut c_void, size_: usize) {
        self._user_id.set(user_id_ as *mut u8, size_);
        self._zap_properties.insert("user_id".to_string(), self._user_id._data());
    }

    pub fn get_user_id(&mut self) -> blob_t {
        self._user_id.clone()
    }

    pub fn get_zmtp_properties(&mut self) -> &mut dict_t {
        return &mut self._zmtp_properties;
    }

    pub fn get_zap_properties(&mut self) -> &mut dict_t {
        return &mut self._zap_properties;
    }

    pub fn socket_type_string(&mut self, socket_type_: i32) -> &'static str
    {
        let names = [socket_type_pair, socket_type_pub, socket_type_sub, socket_type_req, socket_type_rep, socket_type_dealer, socket_type_router, socket_type_pull, socket_type_push, socket_type_xpub, socket_type_xsub, socket_type_stream, socket_type_server, socket_type_client, socket_type_radio, socket_type_dish, socket_type_gather, socket_type_scatter, socket_type_dgram, socket_type_peer, socket_type_channel];
        names[socket_type_ as usize]
    }

    pub unsafe fn add_property(&mut self, mut ptr_: *mut u8, ptr_capacity_: usize, name_: *mut u8, value: *mut c_void, value_len: usize) -> usize {
        let name_len = name_len(name_ as *mut c_char);
        let total_len = property_len(name_len, value_len);
        *ptr_ = name_len as u8;
        ptr_ = ptr_.add(name_len_size as usize);
        libc::memcpy(ptr_ as *mut c_void, name_ as *mut c_void, name_len);
        put_u32(ptr_, value_len as u32);
        ptr_ = ptr_.add(value_len_size as usize);
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

    pub unsafe fn make_command_with_basic_properties(&mut self, msg_: *mut msg_t, prefix_: *mut c_char, prefix_len_: usize){
        let command_size = prefix_len_ + self.basic_properties_len ();
        let rc = (*msg_).init_size (command_size);
        // errno_assert (rc == 0);

        let mut ptr = ((*msg_).data ());

        //  Add prefix
        libc::memcpy (ptr, prefix_ as *const c_void, prefix_len_);
        ptr = ptr.add(prefix_len_);

        self.add_basic_properties (
            ptr as *mut u8, command_size - (ptr.offset_from((*msg_).data())));
    }

    pub unsafe fn parse_metadata(&mut self, mut ptr_: *mut u8, length_: usize, zap_flag_: bool) -> i32 {
        let mut bytes_left = length_;

        while (bytes_left > 1) {
            let name_length = (*ptr_);
            ptr_ = ptr_.add(name_len_size as usize);
            bytes_left -= name_len_size;
            if (bytes_left < name_length as usize) {
                break;
            }

            let name = String::from(ptr_);
                // std::string (reinterpret_cast<const char *> (ptr_), name_length);
            ptr_ = ptr_.add(name_length as usize);
            bytes_left -= name_length;
            if (bytes_left < value_len_size as usize) {
                break;
            }

            let value_length = get_u32 (ptr_);
            ptr_ = ptr_.add(value_len_size as usize);
            bytes_left -= value_len_size;
            if (bytes_left < value_length as usize) {
                break;
            }

            let value = ptr_;
            ptr_ = ptr_.add(value_length as usize);
            bytes_left -= value_length;

            if name == ZMTP_PROPERTY_IDENTITY && self.options.recv_routing_id {
                self.set_peer_routing_id(value as *mut c_void, value_length as usize);
            }
            else if name == ZMTP_PROPERTY_SOCKET_TYPE {
                if !self.check_socket_type (&String::from(value as *mut c_char),
                                            value_length as usize) {
                    // errno = EINVAL;
                    return -1;
                }
            } else {
                let rc = self.property (&name, value as *const c_void, value_length as usize);
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
                return strequals(type_, len_, socket_type_rep)
                    || strequals(type_, len_, socket_type_router);
            },
            ZMQ_REP => {
                return strequals(type_, len_, socket_type_req)
                    || strequals(type_, len_, socket_type_dealer);
            },
            ZMQ_DEALER => {
                return strequals(type_, len_, socket_type_rep)
                    || strequals(type_, len_, socket_type_dealer)
                    || strequals(type_, len_, socket_type_router);
            },
            ZMQ_ROUTER => {
                return strequals(type_, len_, socket_type_req)
                    || strequals(type_, len_, socket_type_dealer)
                    || strequals(type_, len_, socket_type_router); },
            ZMQ_PUSH => {
                return strequals(type_, len_, socket_type_pull); },
            ZMQ_PULL => {
                return strequals(type_, len_, socket_type_push);
            },
            ZMQ_PUB => {
                return strequals(type_, len_, socket_type_sub)
                    || strequals(type_, len_, socket_type_xsub);
            },
            ZMQ_SUB => {
                return strequals(type_, len_, socket_type_pub)
                    || strequals(type_, len_, socket_type_xpub);
            },
            ZMQ_XPUB => {
                return strequals(type_, len_, socket_type_sub)
                    || strequals(type_, len_, socket_type_xsub);
            },
            ZMQ_XSUB => {
                return strequals(type_, len_, socket_type_pub)
                    || strequals(type_, len_, socket_type_xpub);
            },
            ZMQ_PAIR => {
                return strequals(type_, len_, socket_type_pair);
            },
            // #ifdef ZMQ_BUILD_DRAFT_API
            ZMQ_SERVER => {
                return strequals(type_, len_, socket_type_client);
            },
            ZMQ_CLIENT => {
                return strequals(type_, len_, socket_type_server);
            },
            ZMQ_RADIO => {
                return strequals(type_, len_, socket_type_dish);
            },
            ZMQ_DISH => {
                return strequals(type_, len_, socket_type_radio);
            },
            ZMQ_GATHER => {
                return strequals(type_, len_, socket_type_scatter);
            },
            ZMQ_SCATTER => {
                return strequals(type_, len_, socket_type_gather);
            },
            ZMQ_DGRAM => {
                return strequals(type_, len_, socket_type_dgram);
            },
            ZMQ_PEER => {
                return strequals(type_, len_, socket_type_peer);
            },
            ZMQ_CHANNEL => {
                return strequals(type_, len_, socket_type_channel);
            },
            // #endif
            // default:
            // break;
        }
        return false;
    }


}

pub fn strequals(a: &str, c: usize, b: &str,) -> bool {
    a == b
}

pub fn property_len(name_len_: usize, value_len_: usize) -> usize {
    (name_len_size + name_len_ + value_len_size + value_len_) as usize
}

pub unsafe fn name_len(name_: *mut c_char) -> usize {
        let name_len_ = libc::strlen(name_);
    name_len_ as usize
    }
