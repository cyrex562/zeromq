//  Check whether the sizes of public representation of the message (zmq_ZmqMessage)
//  and private representation of the message (ZmqMessage) match.

use crate::atomic_counter::AtomicCounter;
use crate::content::ZmqContent;
use crate::defines::ZMQ_GROUP_MAX_LENGTH;
use crate::metadata::ZmqMetadata;
use anyhow::anyhow;
use libc::{c_long, EINVAL};
use serde::{Deserialize, Serialize};
use std::mem;
use std::mem::size_of;

// enum
//     {
//         ZMQ_MSG_SIZE = 64
//     }
pub const ZMQ_MSG_SIZE: usize = 64;

// enum
//     {
//         MAX_VSM_SIZE =
//           ZMQ_MSG_SIZE - (sizeof (ZmqMetadata *) + 3 + 16 + mem::size_of::<uint32_t>())
//     }
pub const MAX_VSM_SIZE: usize =
    ZMQ_MSG_SIZE - size_of::<*mut ZmqMetadata> + 3 + 16 + size_of::<u32>();

pub const PING_CMD_NAME_SIZE: usize = 5; // 4PING
pub const CANCEL_CMD_NAME_SIZE: usize = 7; // 6CANCEL
pub const SUB_CMD_NAME_SIZE: usize = 10; // 9SUBSCRIBE

// enum {
pub const ZMQ_MSG_MORE: u8 = 1;
//  Followed by more parts
pub const ZMQ_MSG_COMMAND: u8 = 2;
//  Command frame (see ZMTP spec)
//  Command types, use only bits 2-5 and compare with ==, not bitwise,
//  a command can never be of more that one type at the same time
pub const ZMQ_MSG_PING: u8 = 4;
pub const ZMQ_MSG_PONG: u8 = 8;
pub const ZMQ_MSG_SUBSCRIBE: u8 = 12;
pub const ZMQ_MSG_CANCEL: u8 = 16;
pub const ZMQ_MSG_CLOSE_CMD: u8 = 20;
pub const ZMQ_MSG_CREDENTIAL: u8 = 32;
pub const ZMQ_MSG_ROUTING_ID: u8 = 64;
pub const ZMQ_MSG_SHARED: u8 = 128;
// }

// enum ZmqMessageType {
pub const TYPE_MIN: u8 = 101;
//  VSM messages store the content in the message itself
pub const TYPE_VSM: u8 = 101;
//  LMSG messages store the content in malloc-ed memory
pub const TYPE_LMSG: u8 = 102;
//  Delimiter messages are used in envelopes
pub const TYPE_DELIMITER: u8 = 103;
//  CMSG messages point to constant data
pub const TYPE_CMSG: u8 = 104;
// zero-copy LMSG message for v2_decoder
pub const TYPE_ZCLMSG: u8 = 105;
//  Join message for radio_dish
pub const TYPE_JOIN: u8 = 106;
//  Leave message for radio_dish
pub const TYPE_LEAVE: u8 = 107;

pub const TYPE_MAX: u8 = 107;
// }

// enum GroupType {
pub const GROUP_TYPE_SHORT: u8 = 0;
pub const GROUP_TYPE_LONG: u8 = 1;
// }

#[derive(Default, Debug, Clone)]
pub struct LongGroup {
    pub group: [u8; ZMQ_GROUP_MAX_LENGTH + 1],
    pub refcnt: AtomicCounter,
}

#[derive(Default, Debug, Clone)]
pub struct GroupSgroup {
    pub type_: u8,
    pub group: [u8; 15],
}

#[derive(Default, Debug, Clone)]
pub struct GroupLgroup {
    pub type_: u8,
    pub content: *mut c_long,
}

#[derive(Default, Debug, Clone)]
pub union ZmqMsgGrp {
    pub type_: u8,
    pub sgroup: GroupSgroup,
    pub lgroup: GroupLgroup,
}

#[derive(Default, Debug, Clone)]
pub struct MsgUnionBase {
    pub metadata: Option<ZmqMetadata>,
    pub unused: [u8; ZMQ_MSG_SIZE - size_of::<*mut ZmqMetadata>()
        + 2
        + size_of::<u32>()
        + size_of::<ZmqMsgGrp>()],
    pub type_: u8,
    pub flags: u8,
    pub routing_id: u32,
    pub group: ZmqMsgGrp,
}

#[derive(Default, Debug, Clone)]
pub struct MsgUnionVsm {
    pub metadata: Option<ZmqMetadata>,
    pub data: [u8; MAX_VSM_SIZE],
    pub size: usize,
    pub type_: u8,
    pub flags: u8,
    pub routing_id: u32,
    pub group: ZmqMsgGrp,
}

#[derive(Default, Debug, Clone)]
pub struct MsgUnionLmsg {
    pub metadata: Option<ZmqMetadata>,
    pub content: ZmqContent,
    pub unused: [u8; size_of::<*mut ZmqMetadata>()
        + size_of::<*mut ZmqContent>()
        + 2
        + size_of::<u32>()
        + size_of::<ZmqMsgGrp>()],
    pub type_: u8,
    pub flags: u8,
    pub routing_id: u32,
    pub group: ZmqMsgGrp,
}

#[derive(Default, Debug, Clone)]
pub struct MsgUnionZclmsg {
    pub metadata: Option<ZmqMetadata>,
    pub content: ZmqContent,
    pub unused: [u8; size_of::<*mut ZmqMetadata>()
        + size_of::<*mut ZmqContent>()
        + 2
        + size_of::<u32>()
        + size_of::<ZmqMsgGrp>()],
    pub type_: u8,
    pub flags: u8,
    pub routing_id: u32,
    pub group: ZmqMsgGrp,
}

#[derive(Default, Debug, Clone)]
pub struct Cmsg {
    pub metadata: Option<ZmqMetadata>,
    pub content: ZmqContent,
    pub data: Vec<u8>,
    pub size: usize,
    pub unused: [u8; size_of::<ZmqMetadata>()
        + size_of::<ZmqContent>()
        + 2
        + size_of::<u32>()
        + size_of::<ZmqMsgGrp>()],
    pub type_: u8,
    pub flags: u8,
    pub routing_id: u32,
    pub group: ZmqMsgGrp,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct DelimiterMsg {
    pub metadata: Option<ZmqMetadata>,
    pub unused: [u8; size_of::<ZmqMetadata>()
        + size_of::<ZmqContent>()
        + 2
        + size_of::<u32>()
        + size_of::<ZmqMsgGrp>()],
    pub type_: u8,
    pub flags: u8,
    pub routing_id: u32,
    pub group: ZmqMsgGrp,
}

//  Note that fields shared between different message types are not
//  moved to the parent class (ZmqMessage). This way we get tighter packing
//  of the data. Shared fields can be accessed via 'base' member of
//  the union.
#[derive(Default, Debug, Clone)]
pub union MsgUnion {
    pub base: MsgUnionBase,
    pub vsm: MsgUnionVsm,
    pub lmsg: MsgUnionLmsg,
    pub zclmsg: MsgUnionZclmsg,
    pub cmsg: Cmsg,
    pub delimiter: DelimiterMsg,
    pub raw: [u8; 64],
}

pub enum MessageType {
    Base,
    Vsm,
    Lmsg,
    Zclmsg,
    Cmsg,
    Delimiter,
}

pub const CANCEL_CMD_NAME: &[u8] = b"\x06CANCEL";
pub const SUB_CMD_NAME: &[u8] = b"\x09SUBSCRIBE";

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ZmqMessage {
    //
    //  Shared message buffer. Message data are either allocated in one
    //  continuous block along with this structure - thus avoiding one
    //  malloc/free pair or they are stored in user-supplied memory.
    //  In the latter case, ffn member stores pointer to the function to be
    //  used to deallocate the data. If the buffer is actually shared (there
    //  are at least 2 references to it) refcount member contains number of
    //  references.
    //  Message flags.
    //  Size in bytes of the largest message that is still copied around
    //  rather than being reference-counted.
    //
    // refcnt: AtomicCounter,
    //  Different message types.
    // pub u: MsgUnion,
    pub raw: Vec<u8>,
    pub top_msg_type: MessageType,
    pub metadata: Option<ZmqMetadata>,
    pub content: ZmqContent,
    pub data: [u8; MAX_VSM_SIZE],
    pub size: usize,
    pub msg_type: u8,
    pub flags: u8,
    pub routing_id: u32,
    // pub group: ZmqMsgGrp
    pub group_type: u8,
    pub group: [u8; 15],
}

impl ZmqMessage {
    // bool check () const;
    // int init ();
    //
    // int init (data: *mut c_void,
    //           size: usize,
    //           msg_free_fn *ffn_,
    //           hint: *mut c_void,
    //           ZmqContent *content = NULL);
    //
    // int init_size (size: usize);
    // int init_buffer (const buf: *mut c_void, size: usize);
    // int init_data (data: *mut c_void, size: usize, msg_free_fn *ffn_, hint: *mut c_void);
    // int init_external_storage (content: &mut ZmqContent,
    //                            data: *mut c_void,
    //                            size: usize,
    //                            msg_free_fn *ffn_,
    //                            hint: *mut c_void);
    // int init_delimiter ();
    // int init_join ();
    // int init_leave ();
    // int init_subscribe (const size: usize, const unsigned char *topic);
    // int init_cancel (const size: usize, const unsigned char *topic);
    // int close ();
    // int move (ZmqMessage &src_);
    // int copy (ZmqMessage &src_);
    // void *data ();
    // size_t size () const;
    // unsigned char flags () const;
    // void set_flags (unsigned char flags);
    // void reset_flags (unsigned char flags);
    // ZmqMetadata *metadata () const;
    // void set_metadata (ZmqMetadata *metadata);
    // void reset_metadata ();
    // bool is_routing_id () const;
    // bool is_credential () const;
    // bool is_delimiter () const;
    // bool is_join () const;
    // bool is_leave () const;
    // bool is_ping () const;
    // bool is_pong () const;
    // bool is_close_cmd () const;
    //
    // //  These are called on each message received by the session_base class,
    // //  so get them inlined to avoid the overhead of 2 function calls per msg
    // bool is_subscribe () const
    // {
    //     return (_u.flags & CMD_TYPE_MASK) == subscribe;
    // }
    //
    // bool is_cancel () const
    // {
    //     return (_u.flags & CMD_TYPE_MASK) == cancel;
    // }
    //
    // size_t command_body_size () const;
    // void *command_body ();
    // bool is_vsm () const;
    // bool is_cmsg () const;
    // bool is_lmsg () const;
    // bool is_zcmsg () const;
    // uint32_t get_routing_id () const;
    // int set_routing_id (uint32_t routing_id_);
    // int reset_routing_id ();
    // const char *group () const;
    // int set_group (group_: *const c_char);
    // int set_group (const char *, length_: usize);
    //
    // //  After calling this function you can copy the message in POD-style
    // //  refs_ times. No need to call copy.
    // void add_refs (refs_: i32);
    //
    // //  Removes references previously added by add_refs. If the number of
    // //  references drops to 0, the message is closed and false is returned.
    // bool rm_refs (refs_: i32);
    //
    // void shrink (new_size: usize);

    pub fn check(&mut self) -> bool {
        return self.msg_type >= TYPE_MIN && self.msg_type <= TYPE_MAX;
    }

    pub fn init(
        &mut self,
        data: &mut [u8],
        size: usize,
        hint: &mut [u8],
        content: Option<&mut ZmqContent>,
    ) -> i32 {
        if size < MAX_VSM_SIZE {
            let rc: i32 = self.init_size(size);

            if (rc != -1) {
                // TODO:
                // memcpy (data (), data, size);
                return 0;
            }
            return -1;
        }
        if content.is_some() {
            return self.init_external_storage(content.unwrap(), data, size, hint);
        }
        return self.init_data(data, size, hint);
    }

    pub fn init2(&mut self) -> anyhow::Result<()> {
        self.metadata = None;
        self.msg_type = TYPE_VSM;
        self.flags = 0;
        self.size = 0;
        self.group.sgroup.group[0] = 0;
        self.group_type = GROUP_TYPE_SHORT;
        self.routing_id = 0;
        Ok(())
    }

    pub fn init_size(&mut self, size: usize) -> i32 {
        if size <= MAX_VSM_SIZE {
            self.metadata = None;
            self.msg_type = TYPE_VSM;
            self.flags = 0;
            self.size = size;
            self.group.sgroup.group[0] = 0;
            self.group_type = GROUP_TYPE_SHORT;
            self.routing_id = 0;
        } else {
            self.metadata = None;
            self.msg_type = TYPE_LMSG;
            self.flags = 0;
            self.group[0] = 0;
            self.group_type = GROUP_TYPE_SHORT;
            self.routing_id = 0;
            // self._u.content = null_mut();
            // if (mem::size_of::<ZmqContent>() + size > size)
            // if mem::size_of::<ZmqContent>() + size > size
            // {
            //     self._u.content = static_cast < ZmqContent * > (malloc(mem::size_of::<ZmqContent>() + size));
            // }
            // if (unlikely (!_u.content)) {
            //     errno = ENOMEM;
            //     return -1;
            // }
            self.content = ZmqContent::default();
            // self._u.content.data = self._u.content + 1;
            // self._u.content.size = size;
            // self._u.content.ffn = NULL;
            // self._u.content.hint = NULL;
            // new (&self._u.content->refcnt) AtomicCounter ();
        }
        return 0;
    }

    pub fn init_buffer(&mut self, buf_: &mut [u8], size: usize) -> i32 {
        let rc: i32 = self.init_size(size);
        // if (unlikely (rc < 0)) {
        //     return -1;
        // }
        if size {
            // NULL and zero size is allowed
            // assert (NULL != buf_);
            // TODO
            // memcpy (data (), buf_, size);
        }
        return 0;
    }

    pub fn init_external_storage(
        &mut self,
        content: &mut ZmqContent,
        data: &mut [u8],
        size: usize,
        hint: &mut [u8],
    ) -> i32 {
        // zmq_assert (NULL != data);
        // zmq_assert (NULL != content);

        self.metadata = None;
        self.msg_type = TYPE_ZCLMSG;
        self.flags = 0;
        self.group[0] = 0;
        self.group_type = GROUP_TYPE_SHORT;
        self.routing_id = 0;
        self.content = content.clone();
        self.content.data = data.clone();
        self.content.size = size;
        // self._u.content->ffn = ffn_;
        self.content.hint = hint.clone();
        // new (&_u.content->refcnt) AtomicCounter ();
        self.content.refcnt = AtomicCounter::new();

        return 0;
    }

    pub fn init_data(&mut self, data: &mut [u8], size: usize, hint: &mut [u8]) -> i32 {
        //  If data is NULL and size is not 0, a segfault
        //  would occur once the data is accessed
        // zmq_assert (data != NULL || size == 0);

        //  Initialize constant message if there's no need to deallocate
        // if (ffn_ == NULL)
        // {
        self.metadata = None;
        self.msg_type = TYPE_CMSG;
        self.flags = 0;
        self.data.clone_from_slice(data);
        self.size = size;
        self.group[0] = 0;
        self.group_type = GROUP_TYPE_SHORT;
        self.routing_id = 0;
        // }
        // else {
        //     _u.metadata = NULL;
        //     _u.type = type_lmsg;
        //     _u.flags = 0;
        //     _u.group[0] = 0;
        //     _u.group.type = group_type_short;
        //     _u.routing_id = 0;
        //     _u.content =
        //       static_cast<ZmqContent *> (malloc (mem::size_of::<ZmqContent>()));
        //     if (!_u.content) {
        //         errno = ENOMEM;
        //         return -1;
        //     }
        //
        //     _u.content->data = data;
        //     _u.content->size = size;
        //     _u.content->ffn = ffn_;
        //     _u.content->hint = hint;
        //     new (&_u.content->refcnt) AtomicCounter ();
        // }
        return 0;
    }

    pub fn init_delimiter(&mut self) -> io32 {
        self.metadata = None;
        self.msg_type = TYPE_DELIMITER;
        self.flags = 0;
        self.group[0] = 0;
        self.group_type = GROUP_TYPE_SHORT;
        self.routing_id = 0;
        return 0;
    }

    pub fn init_join(&mut self) -> i32 {
        self.metadata = None;
        self.msg_type = TYPE_JOIN;
        self.flags = 0;
        self.group[0] = 0;
        self.group_type = GROUP_TYPE_SHORT;
        self.routing_id = 0;
        return 0;
    }

    pub fn init_leave(&mut self) -> i32 {
        self.metadata = None;
        self.msg_type = TYPE_LEAVE;
        self.flags = 0;
        self.group[0] = 0;
        self.group_type = GROUP_TYPE_SHORT;
        self.routing_id = 0;
        return 0;
    }

    pub fn init_subscribe(&mut self, size: usize, topic: &mut [u8]) -> i32 {
        let rc = self.init_size(size);
        if (rc == 0) {
            self.set_flags(subscribe);

            //  We explicitly allow a NULL subscription with size zero
            if (size) {
                // assert (topic);
                // TODO:
                // memcpy (data (), topic, size);
            }
        }
        return rc;
    }

    pub fn init_cancel(&mut self, size: usize, topic: &mut [u8]) -> i32 {
        let rc = self.init_size(size);
        if rc == 0 {
            self.set_flags(cancel);

            //  We explicitly allow a NULL subscription with size zero
            if size {
                // assert (topic);
                // TODO
                // memcpy (data (), topic, size);
            }
        }
        return rc;
    }

    pub fn close(&mut self) -> anyhow::Result<()> {
        //  Check the validity of the message.
        // if (unlikely (!check ())) {
        //     errno = EFAULT;
        //     return -1;
        // }

        if self.msg_type == TYPE_LMSG {
            //  If the content is not shared, or if it is shared and the reference
            //  count has dropped to zero, deallocate it.
            if !(self.flags & shared) != 0 || !self.content.refcnt.sub(1) {
                //  We used "placement new" operator to initialize the reference
                //  counter so we call the destructor explicitly now.
                // self._u.content->refcnt.~AtomicCounter ();

                // if (_u.content->ffn)
                //     _u.content->ffn (_u.content->data,
                //                           _u.content->hint);
                // free (_u.content);
            }
        }

        if self.is_zcmsg() {
            // zmq_assert (_u.content->ffn);

            //  If the content is not shared, or if it is shared and the reference
            //  count has dropped to zero, deallocate it.
            if (!(self.flags & shared) != 0 || !self.content.refcnt.sub(1)) {
                //  We used "placement new" operator to initialize the reference
                //  counter so we call the destructor explicitly now.
                // self._u.content.refcnt.~AtomicCounter ();

                // _u.content->ffn (_u.content->data,
                //                         _u.content->hint);
            }
        }

        if self.metadata.is_some() {
            if self.metadata.drop_ref() {
                // LIBZMQ_DELETE (_u.metadata);
            }
            self.metadata = None;
        }

        if (self.group_type == GROUP_TYPE_LONG) {
            if !self.lgroup.content.refcnt.sub(1) {
                //  We used "placement new" operator to initialize the reference
                //  counter so we call the destructor explicitly now.
                // self._u.lgroup.content.refcnt.~AtomicCounter ();

                // free (_u.lgroup.content);
            }
        }

        //  Make the message invalid.
        self.msg_type = 0;

        Ok(())
    }

    // pub fn move_(&mut self, src: &mut ZmqMessage) -> i32
    // {
    //     //  Check the validity of the source.
    //     if (unlikely (!src_.check ())) {
    //         errno = EFAULT;
    //         return -1;
    //     }
    //
    //     int rc = close ();
    //     if (unlikely (rc < 0))
    //         return rc;
    //
    //     *this = src_;
    //
    //     rc = src_.init ();
    //     if (unlikely (rc < 0))
    //         return rc;
    //
    //     return 0;
    // }

    // int copy (ZmqMessage &src_)
    // {
    //     //  Check the validity of the source.
    //     if (unlikely (!src_.check ())) {
    //         errno = EFAULT;
    //         return -1;
    //     }
    //
    //     let rc: i32 = close ();
    //     if (unlikely (rc < 0))
    //         return rc;
    //
    //     // The initial reference count, when a non-shared message is initially
    //     // shared (between the original and the copy we create here).
    //     const AtomicCounter::integer_t initial_shared_refcnt = 2;
    //
    //     if (src_.is_lmsg () || src_.is_zcmsg ()) {
    //         //  One reference is added to shared messages. Non-shared messages
    //         //  are turned into shared messages.
    //         if (src_.flags () & shared)
    //             src_.refcnt ()->add (1);
    //         else {
    //             src_.set_flags (shared);
    //             src_.refcnt ()->set (initial_shared_refcnt);
    //         }
    //     }
    //
    //     if (src_._u.metadata != NULL)
    //         src_._u.metadata->add_ref ();
    //
    //     if (src_._u.group.type == group_type_long)
    //         src_._u.lgroup.content->refcnt.add (1);
    //
    //     *this = src_;
    //
    //     return 0;
    // }

    pub fn data(&mut self) -> &[u8] {
        //  Check the validity of the message.
        // zmq_assert (check ());

        match self.msg_type {
            TYPE_VSM => self.data.as_slice(),
            TYPE_LMSG => self.content.data.as_slice(),
            TYPE_CMSG => self.content.data.as_slice(),
            TYPE_DELIMITER => self.unused.as_slice(),
            _ => self.raw.as_slice(),
        }
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        match self.msg_type {
            TYPE_VSM => self.data.as_mut_slice(),
            TYPE_LMSG => self.content.data.as_mut_slice(),
            TYPE_CMSG => self.content.data.as_mut_slice(),
            TYPE_DELIMITER => self.unused.as_mut_slice(),
            _ => self.raw.as_mut_slice(),
        }
    }

    pub fn size(&self) -> usize {
        //  Check the validity of the message.
        // zmq_assert (check ());

        match self.msg_type {
            TYPE_VSM => self.size,
            TYPE_LMSG => self.content.size,
            TYPE_ZCLMSG => self.content.size,
            TYPE_CMSG => self.size,
            _ => 0, // zmq_assert (false);
                    // return 0;
        }
    }

    pub fn shrink(&mut self, new_size: usize) {
        //  Check the validity of the message.
        // zmq_assert (check ());
        // zmq_assert (new_size <= size ());

        match self.msg_type {
            TYPE_VSM => self.size = new_size,
            TYPE_LMSG => self.content.size = new_size,
            TYPE_ZCLMSG => self.content.size = new_size,
            TYPE_CMSG => self.size = new_size,
            _ => {} // zmq_assert (false);
        }
    }

    pub fn flags(&self) -> u8 {
        return self.flags;
    }

    pub fn set_flags(&mut self, flags: u8) {
        self.flags |= flags;
    }

    pub fn reset_flags(&mut self, flags: u8) {
        self.flags &= !flags;
    }

    // ZmqMetadata *metadata () const
    pub fn metadata(&mut self) -> Option<ZmqMetadata> {
        return self.metadata.clone();
    }

    pub fn set_metadata(&mut self, metadata: &mut ZmqMetadata) {
        // assert (metadata != NULL);
        // assert (_u.metadata == NULL);
        metadata.add_ref();
        self.metadata = Some(metadata.clone());
    }

    pub fn reset_metadata(&mut self) {
        if (self.metadata) {
            if (self.metadata.drop_ref()) {
                // LIBZMQ_DELETE (_u.metadata);
            }
            self.metadata = None;
        }
    }

    pub fn is_routing_id(&self) -> bool {
        return (self.flags & routing_id) == routing_id;
    }

    pub fn is_credential(&self) -> bool {
        return (self.flags & credential) == credential;
    }

    pub fn is_delimiter(&self) -> bool {
        return self.msg_type == TYPE_DELIMITER;
    }

    pub fn is_vsm(&self) -> bool {
        return self.msg_type == TYPE_VSM;
    }

    pub fn is_cmsg(&self) -> bool {
        return self.msg_type == TYPE_CMSG;
    }

    pub fn is_lmsg(&self) -> bool {
        return self.msg_type == TYPE_LMSG;
    }

    pub fn is_zcmsg(&self) -> bool {
        return self.msg_type == TYPE_ZCLMSG;
    }

    pub fn is_join(&self) -> bool {
        return self.msg_type == TYPE_JOIN;
    }

    pub fn is_leave(&self) -> bool {
        return self.msg_type == TYPE_LEAVE;
    }

    pub fn is_ping(&self) -> bool {
        return (self.flags & CMD_TYPE_MASK) == ping;
    }

    pub fn is_pong(&self) -> bool {
        return (self.flags & CMD_TYPE_MASK) == pong;
    }

    pub fn is_close_cmd(&self) -> bool {
        return (self.flags & CMD_TYPE_MASK) == close_cmd;
    }

    pub fn command_body_size(&self) -> usize {
        if self.is_ping() || self.is_pong() {
            return self.size() - PING_CMD_NAME_SIZE;
        } else if (!(self.flags() & command != 0) && (self.is_subscribe() || self.is_cancel())) {
            return self.size();
        } else if (self.is_subscribe()) {
            return self.size() - SUB_CMD_NAME_SIZE;
        } else if (self.is_cancel()) {
            return self.size() - CANCEL_CMD_NAME_SIZE;
        }

        return 0;
    }

    pub fn command_body(&mut self) -> Vec<u8> {
        // unsigned char *data = NULL;
        let mut data: Vec<u8> = Vec::new();

        if self.is_ping() || self.is_pong() {
            data = (self.data().unwrap()) + PING_CMD_NAME_SIZE;
        }
        //  With inproc, command flag is not set for sub/cancel
        else if !(self.flags() & command != 0) && (self.is_subscribe() || self.is_cancel()) {
            data = (self.data().unwrap());
        } else if self.is_subscribe() {
            data = (self.data().unwrap()) + SUB_CMD_NAME_SIZE;
        } else if self.is_cancel() {
            data = (self.data().unwrawp()) + CANCEL_CMD_NAME_SIZE;
        }

        return data;
    }

    // pub fn add_refs(&mut self, refs_: i32) {
    //     // zmq_assert (refs_ >= 0);
    //
    //     //  Operation not supported for messages with metadata.
    //     // zmq_assert (_u.metadata == NULL);
    //
    //     //  No copies required.
    //     if !refs_ {
    //         return;
    //     }
    //
    //     //  VSMs, CMSGS and delimiters can be copied straight away. The only
    //     //  message type that needs special care are long messages.
    //     if self.type_ == TYPE_LMSG || self.is_zcmsg() {
    //         if self.flags & shared {
    //             self.refcnt().add(refs_);
    //         } else {
    //             self.refcnt().set(refs_ + 1);
    //             self.flags |= shared;
    //         }
    //     }
    // }

    // pub fn rm_refs(&mut self, refs_: i32) -> bool {
    //     // zmq_assert (refs_ >= 0);
    //
    //     //  Operation not supported for messages with metadata.
    //     // zmq_assert (_u.metadata == NULL);
    //
    //     //  No copies required.
    //     if (!refs_) {
    //         return true;
    //     }
    //
    //     //  If there's only one reference close the message.
    //     if (self.type_ != TYPE_ZCLMSG && self.type_ != TYPE_LMSG) || !(self.flags & shared != 0) {
    //         self.close();
    //         return false;
    //     }
    //
    //     //  The only message type that needs special care are long and zcopy messages.
    //     if self.type_ == TYPE_LMSG && !self.content.refcnt.sub(refs_ as u32) {
    //         //  We used "placement new" operator to initialize the reference
    //         //  counter so we call the destructor explicitly now.
    //         // self._u.content.refcnt.~AtomicCounter ();
    //
    //         // if (_u.content->ffn)
    //         //     _u.content->ffn (_u.content->data, _u.content->hint);
    //         // free (_u.content);
    //
    //         return false;
    //     }
    //
    //     if self.is_zcmsg() && !self.content.refcnt.sub(refs_ as u32) {
    //         // storage for rfcnt is provided externally
    //         // if (self._u.content->ffn) {
    //         //     self._u.content->ffn (_u.content->data,
    //         //                             _u.content->hint);
    //         // }
    //
    //         return false;
    //     }
    //
    //     return true;
    // }

    pub fn get_routing_id(&self) -> u32 {
        return self.routing_id;
    }

    pub fn set_routing_id(&mut self, routing_id_: u32) -> i32 {
        if routing_id_ {
            self.routing_id = routing_id_;
            return 0;
        }
        errno = EINVAL;
        return -1;
    }

    pub fn reset_routing_id(&mut self) -> i32 {
        self.routing_id = 0;
        return 0;
    }

    pub fn group(&mut self) -> String {
        if self.group_type == GROUP_TYPE_LONG {
            return self.lgroup.content.group;
        }
        return String::from_utf8_lossy(&self.group).into_string();
    }

    pub fn set_group(&mut self, group_: &str) -> i32 {
        let length = usize::max(group_.len(), ZMQ_GROUP_MAX_LENGTH);

        return self.set_group2(group_, length);
    }

    pub fn set_group2(&mut self, group_: &str, length_: usize) -> i32 {
        if length_ > ZMQ_GROUP_MAX_LENGTH {
            errno = EINVAL;
            return -1;
        }

        if length_ > 14 {
            self.lgroup.type_ = GROUP_TYPE_LONG;
            self.lgroup.content = long_group_t::new();
            //   (long_group_t *) malloc (mem::size_of::<long_group_t>());
            // assert (_u.lgroup.content);
            // new (&_u.lgroup.content->refcnt) AtomicCounter ();
            self.lgroup.content.refcnt.set(1);
            // strncpy (_u.lgroup.content->group, group_, length_);
            self.lgroup.content.group = group_;
            self.lgroup.content.group[length_] = 0;
        } else {
            // strncpy (_u.group, group_, length_);
            self.u
                .group
                .sgroup
                .group
                .clone_from_slice(group_.as_bytes());
            self.group[length_] = 0;
        }

        return 0;
    }

    pub fn refcnt(&mut self) -> Option<AtomicCounter> {
        match (self.msg_type) {
            TYPE_LMSG => Some(self.content.refcnt.clone()),
            TYPE_ZCLMSG => Some(self.content.refcnt.clone()),
            _ => None, // zmq_assert (false);
                       // return NULL;
        }
    }
}

pub fn close_and_return(msg: &mut ZmqMessage, echo: i32) -> anyhow::Resuylt<i32> {
    // Since we abort on close failure we preserve errno for success case.
    // let err: i32 = errno;
    match msg.close() {
        Ok(_) => Ok(echo),
        Err(e) => Err(anyhow!("error: {}", e)),
    }
    // errno_assert(rc == 0);
    // errno = err;
    // return echo;
}

pub fn close_and_return2(msg: &mut [ZmqMessage], count: i32, echo: i32) -> i32 {
    // for (int i = 0; i < count; i+= 1)
    for i in 0..count {
        close_and_return(&mut msg[i], 0);
    }
    return echo;
}

// typedef char
//   zmq_msg_size_check[2 * ((sizeof (ZmqMessage) == mem::size_of::<zmq_ZmqMessage>()) != 0)
//                      - 1];
