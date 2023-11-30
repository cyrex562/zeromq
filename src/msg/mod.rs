use std::sync::atomic::AtomicU32;

use libc::size_t;
use content::ZmqContent;
use defines::{CANCEL_CMD_NAME_SIZE, CONTENT_T_PTR_SIZE, GROUP_T_SIZE, MAX_VSM_SIZE, METADATA_T_PTR_SIZE, MSG_T_SIZE, PING_CMD_NAME_SIZE, SUB_CMD_NAME_SIZE, TYPE_CMSG, TYPE_DELIMITER, TYPE_JOIN, TYPE_LEAVE, TYPE_LMSG, TYPE_MAX, TYPE_MIN, TYPE_VSM, TYPE_ZCLMSG, VOID_PTR_SIZE};

use crate::defines::{
    CMD_TYPE_MASK, ZMQ_GROUP_MAX_LENGTH, ZMQ_MSG_CANCEL, ZMQ_MSG_CLOSE_CMD, ZMQ_MSG_COMMAND,
    ZMQ_MSG_CREDENTIAL, ZMQ_MSG_PING, ZMQ_MSG_PONG, ZMQ_MSG_ROUTING_ID, ZMQ_MSG_SHARED,
    ZMQ_MSG_SUBSCRIBE,
};

use crate::defines::err::ZmqError;
use crate::defines::err::ZmqError::MessageError;
use crate::metadata::ZmqMetadata;
use crate::msg::ZmqGroupType::{GroupTypeLong, GroupTypeShort};

mod raw_msg;

pub mod defines;
pub mod content;

pub type MsgFreeFn = fn(&mut [u8], &mut [u8]);

#[repr(u8)]
pub enum ZmqGroupType {
    GroupTypeShort = 0,
    GroupTypeLong = 1,
}

impl From<u8> for ZmqGroupType {
    fn from(value: u8) -> Self {
        match value {
            0 => GroupTypeShort,
            1 => GroupTypeLong,
            _ => panic!("Invalid group_type_t value: {}", value),
        }
    }
}

pub struct ZmqLongGroup {
    pub group: [u8; ZMQ_GROUP_MAX_LENGTH + 1],
    pub refcnt: AtomicU32,
}

#[derive(Copy, Clone)]
pub struct ZmqSGroup {
    pub type_: u8,
    pub group: [u8; 15],
}

#[derive(Copy, Clone)]
pub struct ZmqLGroup {
    pub type_: u8,
    pub content: *mut ZmqLongGroup,
}

#[derive(Copy, Clone)]
pub union ZmqGroup {
    pub type_: u8,
    pub sgroup: ZmqSGroup,
    pub lgroup: ZmqLGroup,
}

#[derive(Clone, Copy)]
pub struct ZmqBase {
    pub metadata: *mut ZmqMetadata,
    pub unused: [u8; METADATA_T_PTR_SIZE + 2 + 4 + GROUP_T_SIZE],
    pub type_: u8,
    pub flags: u8,
    pub routing_id: u32,
    pub group: ZmqGroup,
}

#[derive(Clone, Copy)]
pub struct ZmqVsm {
    pub metadata: *mut ZmqMetadata,
    pub data: [u8; MAX_VSM_SIZE],
    pub size: u8,
    pub type_: u8,
    pub flags: u8,
    pub routing_id: u32,
    pub group: ZmqGroup,
}

#[derive(Clone, Copy)]
pub struct ZmqLMsg {
    pub metadata: *mut ZmqMetadata,
    pub content: *mut ZmqContent,
    pub unused: [u8; MSG_T_SIZE - METADATA_T_PTR_SIZE + CONTENT_T_PTR_SIZE + 2 + 4 + GROUP_T_SIZE],
    pub type_: u8,
    pub flags: u8,
    pub routing_id: u32,
    pub group: ZmqGroup,
}

#[derive(Clone, Copy)]
pub struct ZmqZclMsg {
    pub metadata: *mut ZmqMetadata,
    pub content: *mut ZmqContent,
    pub unused: [u8; MSG_T_SIZE - METADATA_T_PTR_SIZE + CONTENT_T_PTR_SIZE + 2 + 4 + GROUP_T_SIZE],
    pub type_: u8,
    pub flags: u8,
    pub routing_id: u32,
    pub group: ZmqGroup,
}

#[derive(Clone, Copy)]
pub struct ZmqCMsg {
    pub metadata: *mut ZmqMetadata,
    pub data: *mut u8,
    pub size: size_t,
    pub unused: [u8; MSG_T_SIZE - METADATA_T_PTR_SIZE + VOID_PTR_SIZE + 2 + 4 + GROUP_T_SIZE],
    pub type_: u8,
    pub flags: u8,
    pub routing_id: u32,
    pub group: ZmqGroup,
}

#[derive(Clone, Copy)]
pub struct ZmqDelimiter {
    pub metadata: *mut ZmqMetadata,
    pub unused: [u8; MSG_T_SIZE - METADATA_T_PTR_SIZE + 2 + 4 + GROUP_T_SIZE],
    pub type_: u8,
    pub flags: u8,
    pub routing_id: u32,
    pub group: ZmqGroup,
}

#[derive(Clone, Copy)]
pub union ZmqMsgU {
    pub base: ZmqBase,
    pub vsm: ZmqVsm,
    pub lmsg: ZmqLMsg,
    pub zclmsg: ZmqZclMsg,
    pub cmsg: ZmqCMsg,
    pub delimiter: ZmqDelimiter,
}

#[derive(Default, Clone)]
pub struct ZmqMsg {
    pub refcnt: AtomicU32,
    // pub _u: ZmqMsgU,
    pub metadata: ZmqMetadata,
    pub content: Option<ZmqContent>,
    // pub unused: [u8; METADATA_T_PTR_SIZE + 2 + 4 + GROUP_T_SIZE],
    pub type_: u8,
    pub flags: u8,
    pub routing_id: u32,
    // pub group: ZmqGroup,
    pub group_type: u8,
    // pub sgroup: ZmqSGroup,
    pub sgroup_type: u8,
    pub group: [u8; 15],
    // pub lgroup: ZmqLGroup,
    pub lgroup_type: u8,
    // pub content: &'a mut ZmqLongGroup,
    pub data: [u8; MAX_VSM_SIZE],
    pub size: u8,
}

impl Default for ZmqMsg {
    fn default() -> Self {
        Self {
            refcnt: AtomicU32::new(0),
            metadata: ZmqMetadata::default(),
            content: None,
            type_: 0,
            flags: 0,
            routing_id: 0,
            group_type: 0,
            sgroup_type: 0,
            group: [0u8; 15],
            lgroup_type: 0,
            data: [0u8; MAX_VSM_SIZE],
            size: 0,
        }
    }
}

impl ZmqMsg {
    pub fn is_subscribe(&self) -> bool {
        self.flags & CMD_TYPE_MASK == ZMQ_MSG_SUBSCRIBE
    }

    pub fn is_cancel(&self) -> bool {
        self.flags & CMD_TYPE_MASK == ZMQ_MSG_CANCEL
    }

    pub fn check(&self) -> bool {
        self.type_ >= TYPE_MIN && self.type_ <= TYPE_MAX
    }

    pub fn init(
        &mut self,
        data: &mut [u8],
        size: size_t,
        free_fn: Option<MsgFreeFn>,
        hint: &mut [u8],
        content: &mut ZmqContent,
    ) -> Result<(), ZmqError> {
        if size < MAX_VSM_SIZE {
            self.init_size(size)?;
            // if rc != -1 {
            //     libc::memcpy(self.data(), data_, size_);
            //     return 0;
            // }
            self.data_mut().copy_from_slice(data);

            // return Err();
        }
        if content.is_some() {
            return self.init_external_storage(content, data, size, free_fn, hint);
        }
        return self.init_data(data, size, free_fn, hint);
    }

    pub fn init3(
        &mut self,
        data: &mut [u8],
        hint: &mut [u8],
        content: &mut ZmqContent,
    ) -> Result<(), ZmqError> {
        if data.len() < MAX_VSM_SIZE {
            self.init_size(data.len())?;
            self.data.copy_from_slice(data);
        }
        if content != ZmqContent::default() {
            return self.init_external_storage2(content, data, hint);
        }
        return self.init_data2(data, hint);
    }

    pub fn init2(&mut self) -> Result<(), ZmqError> {
        self.metadata = ZmqMetadata::default();
        self.type_ = TYPE_VSM;
        self.flags = 0;
        self.size = 0;
        self.routing_id = 0;
        self.group_type = GroupTypeShort as u8;
        self.group[0] = 0;
        return Ok(());
    }

    pub fn init_size(&mut self, size_: size_t) -> Result<(), ZmqError> {
        if size_ <= MAX_VSM_SIZE {
            self.metadata = ZmqMetadata::default();
            self.type_ = TYPE_VSM;
            self.flags = 0;
            self.size = size_ as u8;
            self.routing_id = 0;
            self.group_type = GroupTypeShort as u8;
        } else {
            self.metadata = ZmqMetadata::default();
            self.type_ = TYPE_LMSG;
            self.flags = 0;
            self.routing_id = 0;
            self.group_type = GroupTypeShort as u8;
            self.group[0] = 0;
            self.content = ZmqContent::default();
            if std::mem::size_of::<ZmqContent>() + size_ > size_ {
                self.content = ZmqContent::default();
            }

            (self.content).data = self.content.add(1);
            (self.content).size = size_;
            (self.content).ffn = None;
            (self.content).hint = vec![];
            (self.content).refcnt = AtomicU32::new(0);
        }
        Ok(())
    }

    pub fn init_buffer(&mut self, buf_: &[u8], size_: size_t) -> Result<(), ZmqError> {
        self.init_size(size_)?;
        // if rc < 0 {
        //     return -1;
        // }
        if size_ > 0 {
            // libc::memcpy(self.data(), buf_, size_);
            self.data_mut().copy_from_slice(buf_);
        }
        return Ok(());
    }

    pub fn init_external_storage(
        &mut self,
        content_: &mut ZmqContent,
        data: &mut [u8],
        size_: size_t,
        ffn_: Option<MsgFreeFn>,
        hint: &mut [u8],
    ) -> Result<(), ZmqError> {
        self.metadata = ZmqMetadata::default();
        self.type_ = TYPE_ZCLMSG;
        self.flags = 0;
        self.group[0] = 0;
        self.group_type = GroupTypeShort as u8;
        self.routing_id = 0;

        self.content = content_.clone();
        (*self.content).data.clone_from_slice(data);
        (*self.content).size = size_;
        (*self.content).ffn = ffn_;
        (*self.content).hint.clone_from_slice(hint);
        // new (&_u.zclmsg.content->refcnt) zmq::atomic_counter_t ();
        (*self.content).refcnt = AtomicU32::new(0);

        Ok(())
    }

    pub fn init_external_storage2(
        &mut self,
        content_: &mut ZmqContent,
        data: &mut [u8],
        hint: &mut [u8],
    ) -> Result<(), ZmqError> {
        self.metadata = ZmqMetadata::default();
        self.type_ = TYPE_ZCLMSG;
        self.flags = 0;
        self.group[0] = 0;
        self.group_type = GroupTypeShort as u8;
        self.routing_id = 0;

        self.content = content_.clone();
        (*self.content).data.clone_from_slice(data);
        (*self.content).size = data.len();
        // (*self.content).ffn = Some(ZmqSharedMessageMemoryAllocator::call_dec_ref);
        (*self.content).hint.clone_from_slice(hint);
        // new (&_u.zclmsg.content->refcnt) zmq::atomic_counter_t ();
        (*self.content).refcnt = AtomicU32::new(0);

        Ok(())
    }

    pub fn init_data(
        &mut self,
        data_: &[u8],
        size_: size_t,
        ffn_: Option<MsgFreeFn>,
        hint_: &[u8],
    ) -> Result<(), ZmqError> {
        if ffn_.is_none() {
            self.metadata = ZmqMetadata::default();
            self.type_ = TYPE_CMSG;
            self.flags = 0;
            self.data.clone_from_slice(data_);
            self.size = size_ as u8;
            self.group[0] = 0;
            self.group_type = GroupTypeShort as u8;
            self.routing_id = 0;
        } else {
            self.metadata = ZmqMetadata::default();
            self.type_ = TYPE_LMSG;
            self.flags = 0;
            self.group[0] = 0;
            self.group_type = GroupTypeShort as u8;
            self.routing_id = 0;
            // self.content =
            //     libc::malloc(std::mem::size_of::<ZmqContent>() + size_) as *mut ZmqContent;
            // if (self.content != null_mut()) {
            //     // errno = ENOMEM;
            //     return -1;
            // }
            self.content = ZmqContent::default();

            (*self.content).data.clone_from_slice(data_);
            (*self.content).size = size_;
            (*self.content).ffn = ffn_;
            (*self.content).hint.clone_from_slice(hint_);
            // new (&_u.lmsg.content.refcnt) zmq::atomic_counter_t ();
            (*self.content).refcnt = AtomicU32::new(0);
        }
        Ok(())
    }

    pub fn init_data2(&mut self, data: &[u8], hint_: &[u8]) -> Result<(), ZmqError> {
        self.metadata = ZmqMetadata::default();
        self.type_ = TYPE_CMSG;
        self.flags = 0;
        self.data.clone_from_slice(data);
        self.size = data.len() as u8;
        self.group[0] = 0;
        self.group_type = GroupTypeShort as u8;
        self.routing_id = 0;
        Ok(())
    }

    pub fn init_delimiter(&mut self) -> Result<(),ZmqError> {
        self.metadata = ZmqMetadata::default();
        self.type_ = TYPE_DELIMITER;
        self.flags = 0;
        self.group[0] = 0;
        self.group_type = GroupTypeShort as u8;
        self.routing_id = 0;
        return Ok(())
    }

    pub fn init_join(&mut self) -> Result<(),ZmqError> {
        self.metadata = ZmqMetadata::default();
        self.type_ = TYPE_JOIN;
        self.flags = 0;
        self.group[0] = 0;
        self.group_type = GroupTypeShort as u8;
        self.routing_id = 0;
        return Ok(());
    }

    pub fn init_leave(&mut self) -> Result<(),ZmqError> {
        self.metadata = ZmqMetadata::default();
        self.type_ = TYPE_LEAVE;
        self.flags = 0;
        self.group[0] = 0;
        self.group_type = GroupTypeShort as u8;
        self.routing_id = 0;
        return Ok(());
    }

    pub fn init_subscribe(
        &mut self,
        size_: size_t,
        topic: &mut [u8],
    ) -> Result<(), ZmqError> {
        let res = self.init_size(size_);
        if res.is_ok() {
            self.set_flags(ZMQ_MSG_SUBSCRIBE);

            //  We explicitly allow a NULL subscription with size zero
            if size_ != 0 {
                // assert (topic_);
                // libc::memcpy(self.data_mut(), topic_ as *const c_void, size_);
                self.data_mut().clone_from_slice(topic);
            }
        }
        return res;
    }

    pub fn init_cancel(&mut self, size_: size_t, topic_: &mut [u8]) -> Result<(), ZmqError> {
        let rc = self.init_size(size_);
        if rc.is_ok() {
            self.set_flags(ZMQ_MSG_CANCEL);

            //  We explicitly allow a NULL subscription with size zero
            if size_ != 0 {
                // assert (topic_);
                // memcpy(self.data_mut(), topic_ as *const c_void, size_);
                self.data_mut().clone_from_slice(topic_);
            }
        }
        return rc;
    }

    pub fn close(&mut self) -> Result<(), ZmqError> {
        if !self.check() {
            return Err(MessageError("msg check failed"));
        }

        if self.type_ == TYPE_LMSG && (!self.flags & ZMQ_MSG_SHARED == 0) || ((*self.content).refcnt.sub(1) != 0) {
            // _u.lmsg.content->refcnt.~atomic_counter_t ();
            if (*self.content).ffn.is_some() {
                (*self.content).ffn.unwrap()(
                    (*self.content).data.as_mut_slice(),
                    (*self.content).hint.as_mut_slice(),
                );
            }
            // libc::free(self.content as *mut c_void);
        }

        if self.is_zcmsg() && (!(self.flags & ZMQ_MSG_SHARED != 0) || !(*self.content).refcnt.sub(1) != 0) {
            //  We used "placement new" operator to initialize the reference
            //  counter so we call the destructor explicitly now.
            // _u.zclmsg.content->refcnt.~atomic_counter_t ();

            (*self.content).ffn.unwrap()(
                (*self.content).data.as_mut_slice(),
                (*self.content).hint.as_mut_slice(),
            );
        }

        if self.metadata != ZmqMetadata::default() {
            if (*self.metadata).drop_ref() {
                // TODO
                // LIBZMQ_DELETE (_u.base.metadata);
            }
            self.metadata = ZmqMetadata::default()
        }

        if self.group_type == GroupTypeLong as u8 && (!(*self.content).refcnt.sub(1) != 0) {
            //  We used "placement new" operator to initialize the reference
            //  counter so we call the destructor explicitly now.
            // self.group.lgroup.content.refcnt.~atomic_counter_t ();
            // libc::free(self.content as *mut c_void);
        }

        self.type_ = 0;
        Ok(())
    }

    pub fn move_(&mut self, src_: &mut ZmqMsg) -> Result<(), ZmqError> {
        if !src_.check() {
            return Err(MessageError("msg check failed"));
        }

        self.close()?;

        // TODO
        // self. = src_.clone();
        src_.init2()?;
        Ok(())
    }

    pub fn copy(&mut self, src_msg: &mut ZmqMsg) -> Result<(), ZmqError> {
        if !src_msg.check() {
            return Err(MessageError("msg check failed"));
        }

        self.close()?;
        // if rc < 0 {
        //     return rc;
        // }

        let mut initial_shared_refcnt = AtomicU32::new(2);

        if src_msg.is_lmsg() || src_msg.is_zcmsg() {
            if src_msg.flags() & ZMQ_MSG_SHARED != 0 {
                src_msg.refcnt.add(1);
            } else {
                src_msg.set_flags(ZMQ_MSG_SHARED);
                (*src_msg.refcnt).set(initial_shared_refcnt.get())
            }
        }

        if src_msg.metadata != ZmqMetadata::default() {
            (*src_msg.metadata).add_ref();
        }

        if src_msg.group_type == GroupTypeLong as u8 {
            (*src_msg.content).refcnt.add(1);
        }

        // TODO
        // self._u = src_._u.clone();
        self = *src_msg.clone();

        Ok(())
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        return match self.type_ {
            TYPE_VSM => &mut self.data,
            TYPE_LMSG => (*self.content).data.as_mut_slice(),
            TYPE_CMSG => &mut self.data,
            TYPE_ZCLMSG => (*self.content).data.as_mut_slice(),
            _ => {
                // return self.;
                &mut [0u8]
            }
        };
    }

    pub fn size(&mut self) -> size_t {
        return match self.type_ {
            TYPE_VSM => self.size as size_t,
            TYPE_LMSG => (*self.content).size,
            TYPE_CMSG => self.size as size_t,
            TYPE_ZCLMSG => (*self.content).size,
            _ => {
                // return self.size;
                0
            }
        };
    }

    pub fn shrink(&mut self, new_size_: size_t) {
        match self.type_ {
            TYPE_VSM => {
                self.size = new_size_ as u8;
            }
            TYPE_LMSG => {
                (*self.content).size = new_size_;
            }
            TYPE_CMSG => {
                self.size = new_size_ as u8;
            }
            TYPE_ZCLMSG => {
                (*self.content).size = new_size_;
            }
            _ => {
                // self.size = new_size_;
            }
        }
    }

    pub fn flags(&self) -> u8 {
        return self.flags;
    }

    pub fn flag_set(&self, flag_: u8) -> bool {
        return self.flags & flag_ != 0;
    }

    pub fn flag_clear(&self, flag_: u8) -> bool {
        return self.flags & flag_ == 0;
    }

    pub fn set_flags(&mut self, flags_: u8) {
        self.flags |= flags_;
    }

    pub fn reset_flags(&mut self, flags_: u8) {
        self.flags &= !flags_;
    }

    // pub unsafe fn set_flags(&mut self, flags_: u8) {
    //     self.flags |= flags_;
    // }
    //
    // pub unsafe fn reset_flags(&mut self, flags_: u8) {
    //     self.flags &= !flags_;
    // }

    pub fn metadata(&mut self) -> &mut ZmqMetadata {
        return &mut self.metadata;
    }

    pub fn set_metadata(&mut self, metadata_: &mut ZmqMetadata) {
        metadata_.add_ref();
        self.metadata = metadata_.clone();
    }

    pub fn reset_metadata(&mut self) {
        if self.metadata != ZmqMetadata::default() {
            self.metadata.drop_ref();
            self.metadata = ZmqMetadata::default();
        }
    }

    pub fn is_routing_id(&self) -> bool {
        return self.flags & ZMQ_MSG_ROUTING_ID == ZMQ_MSG_ROUTING_ID;
    }

    pub fn is_credential(&self) -> bool {
        return self.flags & ZMQ_MSG_CREDENTIAL == ZMQ_MSG_CREDENTIAL;
    }

    pub fn is_delimiter(&self) -> bool {
        return self.flags & TYPE_DELIMITER == TYPE_DELIMITER;
    }

    pub fn is_vsm(&self) -> bool {
        return self.type_ == TYPE_VSM;
    }

    pub fn is_cmsg(&self) -> bool {
        return self.type_ == TYPE_CMSG;
    }

    pub fn is_lmsg(&self) -> bool {
        return self.type_ == TYPE_LMSG;
    }

    pub fn is_zcmsg(&self) -> bool {
        return self.type_ == TYPE_ZCLMSG;
    }

    pub fn is_join(&self) -> bool {
        return self.type_ == TYPE_JOIN;
    }

    pub fn is_leave(&self) -> bool {
        return self.type_ == TYPE_LEAVE;
    }

    pub fn is_ping(&self) -> bool {
        return self.flags & CMD_TYPE_MASK == ZMQ_MSG_PING;
    }

    pub fn is_pong(&self) -> bool {
        return self.flags & CMD_TYPE_MASK == ZMQ_MSG_PONG;
    }

    pub fn is_close_cmd(&self) -> bool {
        return self.flags & CMD_TYPE_MASK == ZMQ_MSG_CLOSE_CMD;
    }

    pub fn command_body_size(&mut self) -> size_t {
        if self.is_ping() || self.is_poing() {
            return self.size() - PING_CMD_NAME_SIZE as usize;
        } else if !((self.flags() & ZMQ_MSG_COMMAND) != 0) && (self.is_subscribe() || self.is_cancel()) {
            return self.size();
        } else if self.is_subscribe() {
            return self.size() - SUB_CMD_NAME_SIZE as usize;
        } else if self.is_cancel() {
            return self.size() - CANCEL_CMD_NAME_SIZE as usize;
        }
        return 0;
    }

    pub fn command_body(&mut self) -> &mut [u8] {
        let mut data: &mut [u8];
        if self.is_ping() || self.is_poing() {
            data = self.data_mut().add(PING_CMD_NAME_SIZE as usize);
        } else if !(self.flags() & ZMQ_MSG_COMMAND != 0) && (self.is_subscribe() || self.is_cancel()) {
            data = self.data_mut();
        } else if self.is_subscribe() {
            data = self.data_mut().add(SUB_CMD_NAME_SIZE as usize);
        } else if self.is_cancel() {
            data = self.data_mut().add(CANCEL_CMD_NAME_SIZE as usize);
        }

        return data;
    }

    pub fn add_refs(&mut self, refs_: i32) {
        if refs_ == 0 {
            return;
        }

        if self.type_ == TYPE_LMSG || self.is_zcmsg() {
            if self.flags & ZMQ_MSG_SHARED != 0 {
                (self.refcnt()).add(refs_);
            } else {
                (self.refcnt()).set(refs_ + 1);
                self.flags |= ZMQ_MSG_SHARED
            }
        }
    }

    pub fn rm_refs(&mut self, refs_: i32) -> Result<(), ZmqError> {
        if refs_ == 0 {
            return Ok(());
        }

        if self.type_ != TYPE_ZCLMSG && self.type_ != TYPE_LMSG || !(self.flags & ZMQ_MSG_SHARED != 0) {
            self.close()?;
            return Err(MessageError("invalid message type"));
        }
        if self.type_ == TYPE_LMSG && !((*self.content).refcnt.sub(refs_) == 0) {
            // u.lmsg.content->refcnt.~atomic_counter_t ();
            if (self.content).ffn.is_some() {
                (self.content).ffn.unwrap()(
                    (self.content).hint.as_mut_slice(),
                    (self.content).data.as_mut_slice(),
                );
            }

            return Err(MessageError("unknown error"));
        }

        if self.is_zcmsg() && !((*self.content).refcnt.sub(refs_) == 0) {
            if (*self.content).ffn.is_some() {
                (*self.content).ffn.unwrap()(
                    (*self.content).hint.as_mut_slice(),
                    (*self.content).data.as_mut_slice(),
                );
            }

            return Err(MessageError("unknown error"));
        }

        Ok(())
    }

    pub fn get_routing_id(&self) -> i32 {
        return self.routing_id as i32;
    }

    pub fn set_routing_id(&mut self, routing_id_: i32) {
        self.routing_id = routing_id_ as u32;
    }

    pub fn reset_routing_id(&mut self) -> Result<(),ZmqError> {
        self.routing_id = 0;
        Ok(())
    }

    pub fn group(&self) -> String {
        if self.group_type == GroupTypeLong as u8 {
            // TODO
            // return String::from_utf8_lossy(
            //     &(*self.content).group[0..ZMQ_GROUP_MAX_LENGTH],
            // )
            // .to_string();
            todo!()
        } else {
            return String::from_utf8_lossy(&self.group[0..ZMQ_GROUP_MAX_LENGTH]).to_string();
        }
    }

    pub fn set_group(&mut self, group_: &str) -> Result<(),ZmqError> {
        if group_.len() > ZMQ_GROUP_MAX_LENGTH {
            return Err(MessageError("invalid group length"));
        }
        self.set_group2(group_, group_.len())
    }

    pub fn set_group2(&mut self, group_: &str, length_: size_t) -> Result<(),ZmqError> {
        if length_ > ZMQ_GROUP_MAX_LENGTH {
            return Err(MessageError("invalid group length"));
        }

        if length_ > 14 {
            self.group_type = GroupTypeLong as u8;
            self.content = ZmqContent::default();
            // libc::malloc(std::mem::size_of::<ZmqLongGroup>()) as *mut ZmqLongGroup;
            (*self.content).refcnt.set(1);
            // TODO
            // libc::strncpy(
            //     (&mut (*self.content).group as *mut u8) as *mut c_char,
            //     group_.as_ptr() as *const c_char,
            //     length_,
            // );
            (*self.group).group[length_] = 0;
        } else {
            // libc::strncpy(
            //     &mut self.group as *mut u8 as *mut c_char,
            //     group_.as_ptr() as *const c_char,
            //     length_,
            // );
            self.group.clone_from_slice(group_.as_bytes());
            self.group[length_] = 0;
        }

        return Ok(());
    }

    pub fn refcnt(&mut self) -> &mut AtomicU32 {
        return &mut (self.metadata).ref_cnt;
    }
}

pub fn close_and_return(mut msg: &mut ZmqMsg, echo: i32) -> Result<i32, ZmqError> {
    // let err: i32 = errno();
    (msg).close()?;
    // errno = err;
    return Ok(echo);
}

pub fn close_and_return2(
    msg_: &mut [ZmqMsg],
    count_: i32,
    echo_: i32
) -> Result<i32, ZmqError> {
    for i in 0..count_ {
        close_and_return(&mut msg_[i as usize], 0)?;
    }
    return Ok(echo_);
}
