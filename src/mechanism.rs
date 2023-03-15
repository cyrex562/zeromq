/*
    Copyright (c) 2007-2016 Contributors as noted in the AUTHORS file

    This file is part of libzmq, the ZeroMQ core engine in C++.

    libzmq is free software; you can redistribute it and/or modify it under
    the terms of the GNU Lesser General Public License (LGPL) as published
    by the Free Software Foundation; either version 3 of the License, or
    (at your option) any later version.

    As a special exception, the Contributors give you permission to link
    this library with independent modules to produce an executable,
    regardless of the license terms of these independent modules, and to
    copy and distribute the resulting executable under terms of your choice,
    provided that you also meet, for each linked independent module, the
    terms and conditions of the license of that module. An independent
    module is a module which is not derived from or based on this library.
    If you modify this library, you must extend this exception to your
    version of the library.

    libzmq is distributed in the hope that it will be useful, but WITHOUT
    ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
    FITNESS FOR A PARTICULAR PURPOSE. See the GNU Lesser General Public
    License for more details.

    You should have received a copy of the GNU Lesser General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

// #include "precompiled.hpp"
// #include <string.h>
// #include <limits.h>

// #include "mechanism.hpp"
// #include "options.hpp"
// #include "msg.hpp"
// #include "err.hpp"
// #include "wire.hpp"
// #include "session_base.hpp"

use std::collections::HashMap;
use std::mem;
use anyhow::anyhow;
use crate::message::{ZMQ_MSG_ROUTING_ID, ZmqMessage};
use crate::options::ZmqOptions;
use crate::utils::{copy_bytes, get_u32, put_u32};
use crate::zmq_hdr::{ZMQ_DEALER, ZMQ_MSG_PROPERTY_USER_ID, ZMQ_REQ, ZMQ_ROUTER, ZMQ_REP, ZMQ_PUSH, ZMQ_PULL, ZMQ_PUB, ZMQ_SUB, ZMQ_XPUB, ZMQ_XSUB, ZMQ_PAIR, ZMQ_SERVER, ZMQ_CLIENT, ZMQ_RADIO, ZMQ_DISH, ZMQ_GATHER, ZMQ_SCATTER, ZMQ_DGRAM, ZMQ_PEER, ZMQ_CHANNEL};

pub enum ZmqMechanismStatus
{
    handshaking,
    ready,
    error,
}

pub const socket_type_pair: &str = "PAIR";
pub const socket_type_pub: &str = "PUB";
pub const socket_type_sub: &str = "SUB";
pub const socket_type_req: &str = "REQ";
pub const socket_type_rep: &str = "REP";
pub const socket_type_dealer: &str = "DEALER";
pub const socket_type_router: &str = "ROUTER";
pub const socket_type_pull: &str = "PULL";
pub const socket_type_push: &str = "PUSH";
pub const socket_type_xpub: &str = "XPUB";
pub const socket_type_xsub: &str = "XSUB";
pub const socket_type_stream: &str = "STREAM";
// #ifdef ZMQ_BUILD_DRAFT_API
pub const socket_type_server: &str = "SERVER";
pub const socket_type_client: &str = "CLIENT";
pub const socket_type_radio: &str = "RADIO";
pub const socket_type_dish: &str = "DISH";
pub const socket_type_gather: &str = "GATHER";
pub const socket_type_scatter: &str = "SCATTER";
pub const socket_type_dgram: &str = "DGRAM";
pub const socket_type_peer: &str = "PEER";
pub const socket_type_channel: &str = "CHANNEL";

pub const name_len_size: usize = mem::size_of::<u8>();

pub const value_len_size: usize = mem::size_of::<u32>();

// #define ZMTP_PROPERTY_SOCKET_TYPE "Socket-Type"
pub const ZMTP_PROPERTY_SOCKET_TYPE: &str = "Socket-Type";
// #define ZMTP_PROPERTY_IDENTITY "Identity"
pub const ZMTP_PROPERTY_IDENTITY: &str = "Identity";

pub fn property_len(name_len_: usize, value_len_: usize) -> usize
{
    name_len_size + name_len_ + value_len_size + value_len_
}

pub fn name_len(name_: &str) -> usize
{
// const size_t name_len = strlen (name_);
// zmq_assert (name_len <= UCHAR_MAX);
// return name_len;
    name_.len()
}

pub fn socket_type_string(socket_type_: i32) -> String
{
    // TODO the order must of the names must correspond to the values resp. order of ZMQ_* socket type definitions in zmq.h!
    let names: [&str; 21] = [socket_type_pair, socket_type_pub,
        socket_type_sub, socket_type_req,
        socket_type_rep, socket_type_dealer,
        socket_type_router, socket_type_pull,
        socket_type_push, socket_type_xpub,
        socket_type_xsub, socket_type_stream,
        // #ifdef ZMQ_BUILD_DRAFT_API
        socket_type_server, socket_type_client,
        socket_type_radio, socket_type_dish,
        socket_type_gather, socket_type_scatter,
        socket_type_dgram, socket_type_peer,
        socket_type_channel
        // #endif
    ];
    // static const size_t names_count = mem::size_of::<names>() / sizeof (names[0]);
    let names_count = names.len();
    // zmq_assert (socket_type_ >= 0
    // && socket_type_ < static_cast<int> (names_count));
    return String::from(names[socket_type_]);
}

pub struct ZmqMechanism
{
    // public:
    // const ZmqOptions options;
    pub options: ZmqOptions,
    // private:
    //  Properties received from ZMTP peer.
    // ZmqMetadata::dict_t _zmtp_properties;
    pub zmtp_properties: HashMap<String, String>,
    //  Properties received from ZAP server.
    // ZmqMetadata::dict_t _zap_properties;
    pub zap_properties: HashMap<String, String>,
    // Blob _routing_id;
    pub routing_id: Vec<u8>,
    // Blob _user_id;
    pub user_id: Vec<u8>,
}

impl ZmqMechanism {
    // ZmqMechanism (const ZmqOptions &options_);
    pub fn new(options: &ZmqOptions) -> Self {
        Self {
            options: options.clone(),
            zmtp_properties: HashMap::new(),
            zap_properties: HashMap::new(),
            routing_id: vec![],
            user_id: vec![],
        }
    }

    // void set_peer_routing_id (const id_ptr_: *mut c_void, id_size_: usize);
    pub fn set_peer_routing_id(&mut self, id_ptr: &[u8], id_size: usize) {
        self.routing_id.clone_from_slice(id_ptr);
    }

    // void peer_routing_id (msg: &mut ZmqMessage);
    pub fn peer_routing_id(&mut self, msg: &mut ZmqMessage) -> anyhow::Result<()>
    {
        msg.init_size(self.routing_id.size())?;
        // memcpy (msg.data (), _routing_id.data (), _routing_id.size ());
        copy_bytes(msg.data_mut(), 0, self.routing_id.as_slice(), 0, self.routing_id.len());
        msg.set_flags(ZMQ_MSG_ROUTING_ID);
        Ok(())
    }


    // void set_user_id (const user_id_: *mut c_void, size: usize);
    // void ZmqMechanism::set_user_id (const user_id_: *mut c_void, size: usize)
    // {
    //     _user_id.set (static_cast<const unsigned char *> (user_id_), size);
    //     _zap_properties.ZMQ_MAP_INSERT_OR_EMPLACE (
    //       std::string (ZMQ_MSG_PROPERTY_USER_ID),
    //       std::string (reinterpret_cast<const char *> (user_id_), size));
    // }
    pub fn set_user_id(&mut self, user_id: &[u8], size: usize) {
        self.user_id.clone_from_slice(user_id);
        self.zap_properties.insert(ZMQ_MSG_PROPERTY_USER_ID.parse().unwrap(), self.user_id.to_string());
    }

    // const Blob &get_user_id () const;
    pub fn get_user_id(&self) -> &[u8] {
        self.user_id.as_slice()
    }

    // const ZmqMetadata::dict_t &get_zmtp_properties () const
    // {
    // return _zmtp_properties;
    // }

    // const ZmqMetadata::dict_t &get_zap_properties () const
    // {
    // return _zap_properties;
    // }

    // protected:
    //  Only used to identify the socket for the Socket-Type
    //  property in the wire protocol.
    // static const char *socket_type_string (socket_type_: i32);


    // static size_t add_property (unsigned char *ptr_,
    // ptr_capacity_: usize,
    // name_: *const c_char,
    // const value_: *mut c_void,
    // value_len_: usize);
    // static size_t property_len (name_: *const c_char, value_len_: usize);
    pub fn add_property(&mut self, ptr_: &mut [u8],
                        ptr_capacity_: usize,
                        name_: &str,
                        value_: &[u8],
                        value_len_: usize) -> usize
    {
        let name_len = name_len(name_);
        lettotal_len = property_len(name_len, value_len_);
        // zmq_assert (total_len <= ptr_capacity_);

        // *ptr_ = static_cast<unsigned char> (name_len);
        // ptr_ += name_len_size;
        // memcpy (ptr_, name_, name_len);
        let mut dst_off = name_len_size;
        copy_bytes(ptr_, name_len_size, name_.as_ref(), 0, name_len);

        // ptr_ += name_len;
        dst_off += name_len;

        // zmq_assert (value_len_ <= 0x7FFFFFFF);
        put_u32(ptr_, dst_off, value_len_ as u32);
        // ptr_ += value_len_size;
        dst_off += value_len_size;
        // memcpy (ptr_, value_, value_len_);
        copy_bytes(ptr_, dst_off, value_, 0, value_len_);

        return total_len;
    }

    // size_t add_basic_properties (unsigned char *ptr_,
    // ptr_capacity_: usize) const;
    // size_t basic_properties_len () const;
    pub fn add_basic_properties(&mut self, ptr_: &mut [u8], ptr_capacity_: usize) -> usize
    {
        // unsigned char *ptr = ptr_;
        let mut ptr: &mut [u8] = ptr_;

        //  Add socket type property
        let socket_type = socket_type_string(self.options.type_);
        ptr += self.add_property(ptr, ptr_capacity_, ZMTP_PROPERTY_SOCKET_TYPE,
                                 socket_type.as_bytes(), socket_type.len());

        //  Add identity (aka routing id) property
        if (self.options.type_ == ZMQ_REQ || self.options.type_ == ZMQ_DEALER
            || self.options.type_ == ZMQ_ROUTER) {
            ptr += add_property(ptr, ptr_capacity_ - (ptr - ptr_),
                                ZMTP_PROPERTY_IDENTITY, self.options.routing_id,
                                self.options.routing_id_size);
        }


        // for (std::map<std::string, std::string>::const_iterator
        //     it = options.app_metadata.begin (),
        //     end = options.app_metadata.end ();
        //     it != end; ++it)
        for (first, second) in self.options.app_metadata.iter()
        {
            ptr +=
                add_property(ptr, ptr_capacity_ - (ptr - ptr_), first,
                             second, second.len());
        }

        return ptr - ptr_;
    }


    // void make_command_with_basic_properties (msg: &mut ZmqMessage
    // prefix_: *const c_char,
    // prefix_len_: usize) const;
    pub fn make_command_with_basic_properties(&mut self,
                                              msg: &mut ZmqMessage,
                                              prefix_: &str,
                                              prefix_len_: usize) -> anyhow::Result<()>
    {
        let command_size = prefix_len_ + self.basic_properties_len();
        msg.init_size(command_size)?;
        // errno_assert (rc == 0);

        // unsigned char *ptr = static_cast<unsigned char *> (msg.data ());
        let mut ptr = msg.data_mut();

        //  Add prefix
        // memcpy (ptr, prefix_, prefix_len_);
        copy_bytes(ptr, 0, prefix_.as_ref(), 0, prefix_len_);
        ptr += prefix_len_;

        self.add_basic_properties(
            ptr, command_size - ptr - msg.data().as_ref());
        Ok(())
    }


    //  Parses a metadata.
    //  Metadata consists of a list of properties consisting of
    //  name and value as size-specified strings.
    //  Returns 0 on success and -1 on error, in which case errno is set.
    // int parse_metadata (const unsigned char *ptr_,
    // length_: usize,
    // bool zap_flag_ = false);

    // //  Returns true iff socket associated with the mechanism
    //     //  is compatible with a given socket type 'type_'.
    //     bool check_socket_type (type_: *const c_char, len_: usize) const;

    fn property_len(&mut self, name_: &str, value_len_: usize) -> usize
    {
        property_len(name_len(name_), value_len_)
    }


    pub fn basic_properties_len(&self) -> usize
    {
        let socket_type = socket_type_string(self.options.type_);
        let mut meta_len = 0usize;

        // for (std::map<std::string, std::string>::const_iterator
        //        it = options.app_metadata.begin (),
        //        end = options.app_metadata.end ();
        //      it != end; ++it)
        for (first, second) in self.options.app_metadata.iter()
        {
            meta_len +=
                property_len(first.len(), second.len());
        }

        return property_len(ZMTP_PROPERTY_SOCKET_TYPE.len(), socket_type.len())
            + meta_len
            + if self.options.type_ == ZMQ_REQ || self.options.type_ == ZMQ_DEALER
            || self.options.type_ == ZMQ_ROUTER
        { property_len(ZMTP_PROPERTY_IDENTITY.len(), self.options.routing_id_size) } else { 0 };
    }

    pub fn parse_metadata(&mut self, ptr_: &[u8], length_: usize, zap_flag_: bool) -> anyhow::Result<()>
    {
        let mut bytes_left = length_;
        let mut ptr = ptr_;

        while (bytes_left > 1) {
            let name_length: usize = (ptr_[0]) as usize;
            ptr += name_len_size;
            bytes_left -= name_len_size;
            if (bytes_left < name_length) {
                break;
            }

            let name = String::from_utf8_lossy(&ptr_[..name_length]).to_string();
            // std::string (reinterpret_cast<const char *> (ptr_), name_length);
            ptr += name_length;
            bytes_left -= name_length;
            if (bytes_left < value_len_size) {
                break;
            }

            let value_length = get_u32(ptr_, 0) as usize;
            ptr += value_len_size;
            bytes_left -= value_len_size;
            if (bytes_left < value_length) {
                break;
            }

            let value = ptr;
            ptr += value_length;
            bytes_left -= value_length;

            if (name == ZMTP_PROPERTY_IDENTITY && self.options.recv_routing_id) {
                set_peer_routing_id(value, value_length);
            } else if (name == ZMTP_PROPERTY_SOCKET_TYPE) {
                if (!check_socket_type(value, value_length)) {
                    // errno = EINVAL;
                    // return -1;
                    return Err(anyhow!("EINVAL"));
                }
            } else {
                self.property(name, value, value_length)?;
            }
            if zap_flag_ {
                self.zap_properties.insert((&name).clone(), String::from_utf8_lossy(value).to_string());
            } else {
                self.zmtp_properties.insert((&name).clone(), String::from_utf8_lossy(value).to_string());
            }
        }
        if (bytes_left > 0) {
            // errno = EPROTO;
            // return -1;
            return Err(anyhow!("EPROTO"));
        }
        // return 0;
        Ok(())
    }

    pub fn check_socket_type(&self, type_: &str) -> bool
    {
        match (self.options.type_) {
            ZMQ_REQ => type_.eq(socket_type_rep) || type_.eq(socket_type_router),
            ZMQ_REP => type_.eq(socket_type_req) || type_.eq(socket_type_dealer),
            ZMQ_DEALER => type_.eq(socket_type_rep) || type_.eq(socket_type_dealer) || type_.eq(socket_type_router),
            ZMQ_ROUTER => type_.eq(socket_type_req) || type_.eq(socket_type_dealer) || type_.eq(socket_type_router),
            ZMQ_PUSH => type_.eq(socket_type_pull),
            ZMQ_PULL => type_.eq(socket_type_push),
            ZMQ_PUB => type_.eq(socket_type_sub) || type_.eq(socket_type_xsub),
            ZMQ_SUB => type_.eq(socket_type_pub) || type_.eq(socket_type_xpub),
            ZMQ_XPUB => type_.eq(socket_type_sub) || type_.eq(socket_type_xsub),
            ZMQ_XSUB => type_.eq(socket_type_pub) || type_.eq(socket_type_xpub),
            ZMQ_PAIR => type_.eq(socket_type_pair),
            ZMQ_SERVER => type_.eq(socket_type_client),
            ZMQ_CLIENT => type_.eq(socket_type_server),
            ZMQ_RADIO => type_.eq(socket_type_dish),
            ZMQ_DISH => type_.eq(socket_type_radio),
            ZMQ_GATHER => type_.eq(socket_type_scatter),
            ZMQ_SCATTER => type_.eq(socket_type_gather),
            ZMQ_DGRAM => type_.eq(socket_type_dgram),
            ZMQ_PEER => type_.eq(socket_type_peer),
            ZMQ_CHANNEL => type_.eq(socket_type_channel),
            _ => false
        }
    }
}

trait ZmqMechanismOps {
    // virtual ~ZmqMechanism ();

    //  Prepare next handshake command that is to be sent to the peer.
    // virtual int next_handshake_command (msg: &mut ZmqMessage) = 0;
    fn next_handshake_command(&mut self, msg: &mut ZmqMessage) -> anyhow::Result<()>;

    //  Process the handshake command received from the peer.
    // virtual int process_handshake_command (msg: &mut ZmqMessage) = 0;
    fn process_handshake_command(&mut self, msg: &mut ZmqMessage) -> anyhow::Result<()>;

    // virtual int encode (ZmqMessage *) { return 0; }
    fn encode(&mut self, msg: &mut ZmqMessage) -> anyhow::Result<()>;

    // virtual int decode (ZmqMessage *) { return 0; }
    fn decode(&mut self, msg: &mut ZmqMessage) -> anyhow::Result<()>;

    //  Notifies mechanism about availability of ZAP message.
    // virtual int zap_msg_available () { return 0; }
    fn zap_msg_available(&mut self, msg: &mut ZmqMessage) -> anyhow::Result<()>;

    //  Returns the status of this mechanism.
    // virtual status_t status () const = 0;
    fn status(&mut self) -> ZmqMechanismStatus;

    //  This is called by parse_property method whenever it
    //  parses a new property. The function should return 0
    //  on success and -1 on error, in which case it should
    //  set errno. Signaling error prevents parser from
    //  parsing remaining data.
    //  Derived classes are supposed to override this
    //  method to handle custom processing.
    // virtual int
    // property (const std::string &name_, const value_: *mut c_void, length_: usize);
    // int ZmqMechanism::property (const std::string & /* name_ */,
    // const void * /* value_ */,
    // size_t /* length_ */)
    fn property(&mut self, name_: &str, value: &[u8], length: usize) -> anyhow::Result<()>
    {
        //  Default implementation does not check
        //  property values and returns 0 to signal success.
        Ok(())
    }
}



