#![allow(non_camel_case_types)]

use crate::atomic_counter::atomic_counter_t;
use crate::defines::ZMQ_GROUP_MAX_LENGTH;
use crate::metadata::metadata_t;
use crate::msg::group_type_t::{group_type_long, group_type_short};
use libc::{free, malloc, memcpy, size_t, ENOMEM};
use std::ffi::{c_char, c_void};
use std::ptr::{null, null_mut};
use windows::s;

pub const cancel_cmd_name: &'static str = "\x06CANCEL";
pub const sub_cmd_name: &'static str = "\x09SUBSCRIBE";

pub const CMD_TYPE_MASK: u8 = 0x1c;

pub type msg_free_fn = fn(*mut c_void, *mut c_void);

pub struct content_t {
    pub data: *mut u8,
    pub size: size_t,
    pub hint: *mut u8,
    pub refcnt: atomic_counter_t,
    pub ffn: Option<msg_free_fn>,
}

pub const more: u8 = 1;
pub const command: u8 = 2;
pub const ping: u8 = 4;
pub const pong: u8 = 8;
pub const subscribe: u8 = 12;
pub const cancel: u8 = 16;
pub const close_cmd: u8 = 20;
pub const credential: u8 = 32;
pub const routing_id: u8 = 64;
pub const shared: u8 = 128;

pub const msg_t_size: usize = 64;

pub const ping_cmd_name_size: i32 = 5;
pub const cancel_cmd_name_size: i32 = 7;
pub const sub_cmd_name_size: i32 = 10;

pub const max_vsm_size: usize = msg_t_size - 3 + 16 + 4 + std::mem::size_of::<*mut metadata_t>();

pub const type_min: u8 = 101;
pub const type_vsm: u8 = 101;
pub const type_lmsg: u8 = 102;
pub const type_delimiter: u8 = 103;
pub const type_cmsg: u8 = 104;
pub const type_zclmsg: u8 = 105;
pub const type_join: u8 = 106;
pub const type_leave: u8 = 107;
pub const type_max: u8 = 107;

pub const metadata_t_ptr_size: usize = std::mem::size_of::<*mut metadata_t>();
pub const content_t_ptr_size: usize = std::mem::size_of::<*mut content_t>();
pub const group_t_size: usize = std::mem::size_of::<group_t>();
pub const void_ptr_size: usize = std::mem::size_of::<*mut c_void>();
pub const size_t_size: usize = std::mem::size_of::<size_t>();

#[repr(u8)]
pub enum group_type_t {
    group_type_short = 0,
    group_type_long = 1,
}

impl From<u8> for group_type_t {
    fn from(value: u8) -> Self {
        match value {
            0 => group_type_short,
            1 => group_type_long,
            _ => panic!("Invalid group_type_t value: {}", value),
        }
    }
}

pub struct long_group_t {
    pub group: [u8; ZMQ_GROUP_MAX_LENGTH + 1],
    pub refcnt: atomic_counter_t,
}

#[derive(Copy, Clone)]
pub struct sgroup {
    pub type_: u8,
    pub group: [u8; 15],
}

#[derive(Copy, Clone)]
pub struct lgroup {
    pub type_: u8,
    pub content: *mut long_group_t,
}

#[derive(Copy, Clone)]
pub union group_t {
    pub type_: u8,
    pub sgroup: sgroup,
    pub lgroup: lgroup,
}

#[derive(Clone, Copy)]
pub struct base {
    pub metadata: *mut metadata_t,
    pub unused: [u8; metadata_t_ptr_size + 2 + 4 + group_t_size],
    pub type_: u8,
    pub flags: u8,
    pub routing_id: u32,
    pub group: group_t,
}

#[derive(Clone, Copy)]
pub struct vsm {
    pub metadata: *mut metadata_t,
    pub data: [u8; max_vsm_size],
    pub size: u8,
    pub type_: u8,
    pub flags: u8,
    pub routing_id: u32,
    pub group: group_t,
}

#[derive(Clone, Copy)]
pub struct lmsg {
    pub metadata: *mut metadata_t,
    pub content: *mut content_t,
    pub unused: [u8; msg_t_size - metadata_t_ptr_size + content_t_ptr_size + 2 + 4 + group_t_size],
    pub type_: u8,
    pub flags: u8,
    pub routing_id: u32,
    pub group: group_t,
}

#[derive(Clone, Copy)]
pub struct zclmsg {
    pub metadata: *mut metadata_t,
    pub content: *mut content_t,
    pub unused: [u8; msg_t_size - metadata_t_ptr_size + content_t_ptr_size + 2 + 4 + group_t_size],
    pub type_: u8,
    pub flags: u8,
    pub routing_id: u32,
    pub group: group_t,
}

#[derive(Clone, Copy)]
pub struct cmsg {
    pub metadata: *mut metadata_t,
    pub data: *mut u8,
    pub size: size_t,
    pub unused: [u8; msg_t_size - metadata_t_ptr_size + void_ptr_size + 2 + 4 + group_t_size],
    pub type_: u8,
    pub flags: u8,
    pub routing_id: u32,
    pub group: group_t,
}

#[derive(Clone, Copy)]
pub struct delimiter {
    pub metadata: *mut metadata_t,
    pub unused: [u8; msg_t_size - metadata_t_ptr_size + 2 + 4 + group_t_size],
    pub type_: u8,
    pub flags: u8,
    pub routing_id: u32,
    pub group: group_t,
}

#[derive(Clone, Copy)]
pub union msg_u {
    pub base: base,
    pub vsm: vsm,
    pub lmsg: lmsg,
    pub zclmsg: zclmsg,
    pub cmsg: cmsg,
    pub delimiter: delimiter,
}

#[derive(Clone, Copy)]
pub struct msg_t {
    pub refcnt: *mut atomic_counter_t,
    pub _u: msg_u,
}

impl msg_t {
    pub unsafe fn is_subscribe(&self) -> bool {
        self._u.base.flags & CMD_TYPE_MASK == subscribe
    }

    pub unsafe fn is_cancel(&self) -> bool {
        self._u.base.flags & CMD_TYPE_MASK == cancel
    }

    pub unsafe fn check(&self) -> bool {
        self._u.base.type_ >= type_min && self._u.base.type_ <= type_max
    }

    pub unsafe fn init(
        &mut self,
        data_: *mut c_void,
        size_: size_t,
        ffn_: msg_free_fn,
        hint_: *mut c_void,
        content_: *mut content_t,
    ) -> i32 {
        if size_ < max_vsm_size {
            let rc = self.init_size(size_);
            if rc != -1 {
                libc::memcpy(self.data(), data_, size_);
                return 0;
            }
            return -1;
        }
        if content_ != null_mut() {
            return self.init_external_storage(content_, data_, size_, ffn_, hint_);
        }
        return self.init_data(data_, size_, Some(ffn_), hint_);
    }

    pub unsafe fn init2(&mut self) -> i32 {
        self._u.vsm.metadata = null_mut();
        self._u.vsm.type_ = type_vsm;
        self._u.vsm.flags = 0;
        self._u.vsm.size = 0;
        self._u.vsm.routing_id = 0;
        self._u.vsm.group.type_ = group_type_short as u8;
        self._u.vsm.group.sgroup.group[0] = 0;
        return 0;
    }

    pub unsafe fn init_size(&mut self, size_: size_t) -> i32 {
        if size_ <= max_vsm_size {
            self._u.vsm.metadata = null_mut();
            self._u.vsm.type_ = type_vsm;
            self._u.vsm.flags = 0;
            self._u.vsm.size = size_ as u8;
            self._u.vsm.routing_id = 0;
            self._u.vsm.group.type_ = group_type_short as u8;
        } else {
            self._u.lmsg.metadata = null_mut();
            self._u.lmsg.type_ = type_lmsg;
            self._u.lmsg.flags = 0;
            self._u.lmsg.routing_id = 0;
            self._u.lmsg.group.type_ = group_type_short as u8;
            self._u.lmsg.group.sgroup.group[0] = 0;
            self._u.lmsg.content = null_mut();
            if std::mem::size_of::<content_t>() + size_ > size_ {
                self._u.lmsg.content =
                    libc::malloc(std::mem::size_of::<content_t>() + size_) as *mut content_t;
            }
            if self._u.lmsg.content == null_mut() {
                return -1;
            }

            (*self._u.lmsg.content).data = self._u.lmsg.content.add(1) as *mut c_void;
            (*self._u.lmsg.content).size = size_;
            (*self._u.lmsg.content).ffn = None;
            (*self._u.lmsg.content).hint = null_mut();
            (*self._u.lmsg.content).refcnt = atomic_counter_t::new(0);
        }
        return 0;
    }

    pub unsafe fn init_buffer(&mut self, buf_: *const c_void, size_: size_t) -> i32 {
        let rc = self.init_size(size_);
        if rc < 0 {
            return -1;
        }
        if size_ > 0 {
            libc::memcpy(self.data(), buf_, size_);
        }
        return 0;
    }

    pub unsafe fn init_external_storage(
        &mut self,
        content_: *mut content_t,
        data_: *mut c_void,
        size_: size_t,
        ffn_: msg_free_fn,
        hint_: *mut c_void,
    ) -> i32 {
        self._u.zclmsg.metadata = null_mut();
        self._u.zclmsg.type_ = type_zclmsg;
        self._u.zclmsg.flags = 0;
        self._u.zclmsg.group.sgroup.group[0] = 0;
        self._u.zclmsg.group.type_ = group_type_short as u8;
        self._u.zclmsg.routing_id = 0;

        self._u.zclmsg.content = content_;
        (*self._u.zclmsg.content).data = data_;
        (*self._u.zclmsg.content).size = size_;
        (*self._u.zclmsg.content).ffn = Some(ffn_);
        (*self._u.zclmsg.content).hint = hint_;
        // new (&_u.zclmsg.content->refcnt) zmq::atomic_counter_t ();
        (*self._u.zclmsg.content).refcnt = atomic_counter_t::new(0);

        return 0;
    }

    pub unsafe fn init_data(
        &mut self,
        data_: *mut c_void,
        size_: size_t,
        ffn_: Option<msg_free_fn>,
        hint_: *mut c_void,
    ) -> i32 {
        if ffn_.is_none() {
            self._u.cmsg.metadata = null_mut();
            self._u.cmsg.type_ = type_cmsg;
            self._u.cmsg.flags = 0;
            self._u.cmsg.data = data_;
            self._u.cmsg.size = size_;
            self._u.cmsg.group.sgroup.group[0] = 0;
            self._u.cmsg.group.type_ = group_type_short as u8;
            self._u.cmsg.routing_id = 0;
        } else {
            self._u.lmsg.metadata = null_mut();
            self._u.lmsg.type_ = type_lmsg;
            self._u.lmsg.flags = 0;
            self._u.lmsg.group.sgroup.group[0] = 0;
            self._u.lmsg.group.type_ = group_type_short as u8;
            self._u.lmsg.routing_id = 0;
            self._u.lmsg.content =
                libc::malloc(std::mem::size_of::<content_t>() + size_) as *mut content_t;
            if (self._u.lmsg.content != null_mut()) {
                // errno = ENOMEM;
                return -1;
            }

            (*self._u.lmsg.content).data = data_;
            (*self._u.lmsg.content).size = size_;
            (*self._u.lmsg.content).ffn = ffn_;
            (*self._u.lmsg.content).hint = hint_;
            // new (&_u.lmsg.content.refcnt) zmq::atomic_counter_t ();
            (*self._u.lmsg.content).refcnt = atomic_counter_t::new(0);
        }
        return 0;
    }

    pub unsafe fn init_delimiter(&mut self) -> i32 {
        self._u.delimiter.metadata = null_mut();
        self._u.delimiter.type_ = type_delimiter;
        self._u.delimiter.flags = 0;
        self._u.delimiter.group.sgroup.group[0] = 0;
        self._u.delimiter.group.type_ = group_type_short as u8;
        self._u.delimiter.routing_id = 0;
        return 0;
    }

    pub unsafe fn init_join(&mut self) -> i32 {
        self._u.base.metadata = null_mut();
        self._u.base.type_ = type_join;
        self._u.base.flags = 0;
        self._u.base.group.sgroup.group[0] = 0;
        self._u.base.group.type_ = group_type_short as u8;
        self._u.base.routing_id = 0;
        return 0;
    }

    pub unsafe fn init_leave(&mut self) -> i32 {
        self._u.base.metadata = null_mut();
        self._u.base.type_ = type_leave;
        self._u.base.flags = 0;
        self._u.base.group.sgroup.group[0] = 0;
        self._u.base.group.type_ = group_type_short as u8;
        self._u.base.routing_id = 0;
        return 0;
    }

    pub unsafe fn init_subscribe(&mut self, size_: size_t, topic_: *mut u8) -> i32 {
        let rc = self.init_size(size_);
        if (rc == 0) {
            self.set_flags(subscribe);

            //  We explicitly allow a NULL subscription with size zero
            if size_ != 0 {
                // assert (topic_);
                libc::memcpy(self.data(), topic_ as *const c_void, size_);
            }
        }
        return rc;
    }

    pub unsafe fn init_cancel(&mut self, size_: size_t, topic_: *mut u8) -> i32 {
        let rc = self.init_size(size_);
        if (rc == 0) {
            self.set_flags(cancel);

            //  We explicitly allow a NULL subscription with size zero
            if (size_ != 0) {
                // assert (topic_);
                memcpy(self.data(), topic_ as *const c_void, size_);
            }
        }
        return rc;
    }

    pub unsafe fn close(&mut self) -> i32 {
        if !self.check() {
            return -1;
        }

        if self._u.base.type_ == type_lmsg && (!self._u.lmsg.flags & shared == 0)
            || ((*self._u.lmsg.content).refcnt.sub(1) != 0)
        {
            // _u.lmsg.content->refcnt.~atomic_counter_t ();
            if (*self._u.lmsg.content).ffn.is_some() {
                (*self._u.lmsg.content).ffn.unwrap()(
                    (*self._u.lmsg.content).data,
                    (*self._u.lmsg.content).hint,
                );
            }
            libc::free(self._u.lmsg.content as *mut c_void);
        }

        if self.is_zcmsg()
            && (!(self._u.zclmsg.flags & shared != 0)
                || !(*self._u.zclmsg.content).refcnt.sub(1) != 0)
        {
            //  We used "placement new" operator to initialize the reference
            //  counter so we call the destructor explicitly now.
            // _u.zclmsg.content->refcnt.~atomic_counter_t ();

            (*self._u.zclmsg.content).ffn.unwrap()(
                (*self._u.zclmsg.content).data,
                (*self._u.zclmsg.content).hint,
            );
        }

        if self._u.base.metadata != null_mut() {
            if (*self._u.base.metadata).drop_ref() {
                // TODO
                // LIBZMQ_DELETE (_u.base.metadata);
            }
            self._u.base.metadata = null_mut();
        }

        if self._u.base.group.type_ == group_type_long as u8
            && (!(*self._u.base.group.lgroup.content).refcnt.sub(1) != 0)
        {
            //  We used "placement new" operator to initialize the reference
            //  counter so we call the destructor explicitly now.
            // self._u.base.group.lgroup.content.refcnt.~atomic_counter_t ();
            libc::free(self._u.base.group.lgroup.content as *mut c_void);
        }

        self._u.base.type_ = 0;
        return 0;
    }

    pub unsafe fn move_(&mut self, src_: &mut msg_t) -> i32 {
        if !src_.check() {
            return -1;
        }

        let mut rc = self.close();
        if rc < 0 {
            return rc;
        }

        self._u = src_._u.clone();
        rc = src_.init2();
        if rc < 0 {
            return rc;
        }
        return 0;
    }

    pub unsafe fn copy(&mut self, src_: &mut msg_t) -> i32 {
        if !src_.check() {
            return -1;
        }

        let mut rc = self.close();
        if rc < 0 {
            return rc;
        }

        let mut initial_shared_refcnt = atomic_counter_t::new(2);

        if src_.is_lmsg() || src_.is_zcmsg() {
            if src_.flags() & shared != 0 {
                src_.refcnt.add(1);
            } else {
                src_.set_flags(shared as u8);
                (*src_.refcnt).set(initial_shared_refcnt.get())
            }
        }

        if src_._u.base.metadata != null_mut() {
            (*src_._u.base.metadata).add_ref();
        }

        if src_._u.base.group.type_ == group_type_long as u8 {
            (*src_._u.base.group.lgroup.content).refcnt.add(1);
        }

        self._u = src_._u.clone();

        return 0;
    }

    pub unsafe fn data(&mut self) -> *mut u8 {
        match self._u.base.type_ {
            type_vsm => {
                return &mut self._u.vsm.data as *mut u8;
            }
            type_lmsg => {
                return (*self._u.lmsg.content).data;
            }
            type_cmsg => {
                return self._u.cmsg.data;
            }
            type_zclmsg => {
                return (*self._u.zclmsg.content).data;
            }
            _ => {
                // return self._u.base.;
                return null_mut();
            }
        }
    }

    pub unsafe fn size(&mut self) -> size_t {
        match self._u.base.type_ {
            type_vsm => {
                return self._u.vsm.size as size_t;
            }
            type_lmsg => {
                return (*self._u.lmsg.content).size;
            }
            type_cmsg => {
                return self._u.cmsg.size;
            }
            type_zclmsg => {
                return (*self._u.zclmsg.content).size;
            }
            _ => {
                // return self._u.base.size;
                return 0;
            }
        }
    }

    pub unsafe fn shrink(&mut self, new_size_: size_t) {
        match self._u.base.type_ {
            type_vsm => {
                self._u.vsm.size = new_size_ as u8;
            }
            type_lmsg => {
                (*self._u.lmsg.content).size = new_size_;
            }
            type_cmsg => {
                self._u.cmsg.size = new_size_;
            }
            type_zclmsg => {
                (*self._u.zclmsg.content).size = new_size_;
            }
            _ => {
                // self._u.base.size = new_size_;
            }
        }
    }

    pub unsafe fn flags(&self) -> u8 {
        return self._u.base.flags;
    }

    pub unsafe fn flag_set(&self, flag_: u8) -> bool {
        return self._u.base.flags & flag_ != 0;
    }

    pub unsafe fn flag_clear(&self, flag_: u8) -> bool {
        return self._u.base.flags & flag_ == 0;
    }

    pub unsafe fn set_flags(&mut self, flags_: u8) {
        self._u.base.flags |= flags_;
    }

    pub unsafe fn reset_flags(&mut self, flags_: u8) {
        self._u.base.flags &= !flags_;
    }

    // pub unsafe fn set_flags(&mut self, flags_: u8) {
    //     self._u.base.flags |= flags_;
    // }
    //
    // pub unsafe fn reset_flags(&mut self, flags_: u8) {
    //     self._u.base.flags &= !flags_;
    // }

    pub unsafe fn metadata(&mut self) -> *mut metadata_t {
        return self._u.base.metadata;
    }

    pub unsafe fn set_metadata(&mut self, metadata_: *mut metadata_t) {
        (*metadata_).add_ref();
        self._u.base.metadata = metadata_;
    }

    pub unsafe fn reset_metadata(&mut self) {
        if self._u.base.metadata != null_mut() {
            (*self._u.base.metadata).drop_ref();
            self._u.base.metadata = null_mut();
        }
    }

    pub unsafe fn is_routing_id(&self) -> bool {
        return self._u.base.flags & routing_id == routing_id as u8;
    }

    pub unsafe fn is_credential(&self) -> bool {
        return self._u.base.flags & credential == credential as u8;
    }

    pub unsafe fn is_delimiter(&self) -> bool {
        return self._u.base.flags & type_delimiter == type_delimiter as u8;
    }

    pub unsafe fn is_vsm(&self) -> bool {
        return self._u.base.type_ == type_vsm;
    }

    pub unsafe fn is_cmsg(&self) -> bool {
        return self._u.base.type_ == type_cmsg;
    }

    pub unsafe fn is_lmsg(&self) -> bool {
        return self._u.base.type_ == type_lmsg;
    }

    pub unsafe fn is_zcmsg(&self) -> bool {
        return self._u.base.type_ == type_zclmsg;
    }

    pub unsafe fn is_join(&self) -> bool {
        return self._u.base.type_ == type_join;
    }

    pub unsafe fn is_leave(&self) -> bool {
        return self._u.base.type_ == type_leave;
    }

    pub unsafe fn is_ping(&self) -> bool {
        return self._u.base.flags & CMD_TYPE_MASK == ping as u8;
    }

    pub unsafe fn is_poing(&self) -> bool {
        return self._u.base.flags & CMD_TYPE_MASK == pong as u8;
    }

    pub unsafe fn is_close_cmd(&self) -> bool {
        return self._u.base.flags & CMD_TYPE_MASK == close_cmd as u8;
    }

    pub unsafe fn command_body_size(&mut self) -> size_t {
        if self.is_ping() || self.is_poing() {
            return self.size() - ping_cmd_name_size as usize;
        } else if !((self.flags() & command) != 0) && (self.is_subscribe() || self.is_cancel()) {
            return self.size();
        } else if self.is_subscribe() {
            return self.size() - sub_cmd_name_size as usize;
        } else if self.is_cancel() {
            return self.size() - cancel_cmd_name_size as usize;
        }
        return 0;
    }

    pub unsafe fn command_body(&mut self) -> *mut c_void {
        let mut data: *mut u8 = null_mut();
        if self.is_ping() || self.is_poing() {
            data = self.data().add(ping_cmd_name_size as usize) as *mut u8;
        } else if !(self.flags() & command != 0) && (self.is_subscribe() || self.is_cancel()) {
            data = self.data() as *mut u8;
        } else if self.is_subscribe() {
            data = self.data().add(sub_cmd_name_size as usize) as *mut u8;
        } else if self.is_cancel() {
            data = self.data().add(cancel_cmd_name_size as usize) as *mut u8;
        }

        return data as *mut c_void;
    }

    pub unsafe fn add_refs(&mut self, refs_: i32) {
        if refs_ == 0 {
            return;
        }

        if self._u.base.type_ == type_lmsg || self.is_zcmsg() {
            if self._u.base.flags & shared != 0 {
                self.refcnt().add(refs_ as usize);
            } else {
                (*self.refcnt()).set(refs_ + 1);
                self._u.base.flags |= shared
            }
        }
    }

    pub unsafe fn rm_refs(&mut self, refs_: i32) -> bool {
        if refs_ == 0 {
            return true;
        }

        if self._u.base.type_ != type_zclmsg && self._u.base.type_ != type_lmsg
            || !(self._u.base.flags & shared != 0)
        {
            self.close();
            return false;
        }
        if self._u.base.type_ == type_lmsg && !((*self._u.lmsg.content).refcnt.sub(refs_) == 0) {
            // u.lmsg.content->refcnt.~atomic_counter_t ();
            if (*self._u.lmsg.content).ffn.is_some() {
                (*self._u.lmsg.content).ffn.unwrap()(
                    (*self._u.lmsg.content).hint,
                    (*self._u.lmsg.content).data,
                );
            }

            return false;
        }

        if self.is_zcmsg() && !((*self._u.zclmsg.content).refcnt.sub(refs_) == 0) {
            if (*self._u.zclmsg.content).ffn.is_some() {
                (*self._u.zclmsg.content).ffn.unwrap()(
                    (*self._u.zclmsg.content).hint,
                    (*self._u.zclmsg.content).data,
                );
            }

            return false;
        }

        return true;
    }

    pub unsafe fn get_routing_id(&self) -> i32 {
        return self._u.base.routing_id as i32;
    }

    pub unsafe fn set_routing_id(&mut self, routing_id_: i32) {
        self._u.base.routing_id = routing_id_ as u32;
    }

    pub unsafe fn reset_routing_id(&mut self) -> i32 {
        self._u.base.routing_id = 0;
        return 0;
    }

    pub unsafe fn group(&self) -> String {
        if self._u.base.group.type_ == group_type_long as u8 {
            return String::from_utf8_lossy(
                &(*self._u.base.group.lgroup.content).group[0..ZMQ_GROUP_MAX_LENGTH],
            )
            .to_string();
        } else {
            return String::from_utf8_lossy(
                &self._u.base.group.sgroup.group[0..ZMQ_GROUP_MAX_LENGTH],
            )
            .to_string();
        }
    }

    pub unsafe fn set_group(&mut self, group_: &str) -> i32 {
        if group_.len() > ZMQ_GROUP_MAX_LENGTH {
            return -1;
        }
        return self.set_group2(group_, group_.len());
    }

    pub unsafe fn set_group2(&mut self, group_: &str, length_: size_t) -> i32 {
        if length_ > ZMQ_GROUP_MAX_LENGTH {
            return -1;
        }

        if length_ > 14 {
            self._u.base.group.lgroup.type_ = group_type_long as u8;
            self._u.base.group.lgroup.content =
                libc::malloc(std::mem::size_of::<long_group_t>()) as *mut long_group_t;
            (*self._u.base.group.lgroup.content).refcnt.set(1);
            libc::strncpy(
                (&mut (*self._u.base.group.lgroup.content).group as *mut u8) as *mut c_char,
                group_.as_ptr() as *const c_char,
                length_,
            );
            (*self._u.base.group.lgroup.content).group[length_] = 0;
        } else {
            libc::strncpy(
                &mut self._u.base.group.sgroup.group as *mut u8 as *mut c_char,
                group_.as_ptr() as *const c_char,
                length_,
            );
            self._u.base.group.sgroup.group[length_] = 0;
        }

        return 0;
    }

    pub unsafe fn refcnt(&mut self) -> *mut atomic_counter_t {
        return &mut (*self._u.base.metadata)._ref_cnt;
    }
}

pub unsafe fn close_and_return(mut msg_: *mut msg_t, echo_: i32) -> i32 {
    // let err: i32 = errno();
    let rc = (*msg_).close();
    // errno = err;
    return echo_;
}

pub unsafe fn close_and_return2(msg_: &mut [msg_t], count_: i32, echo_: i32) -> i32 {
    for i in 0..count_ {
        close_and_return(&mut msg_[i as usize], 0);
    }
    return echo_;
}
