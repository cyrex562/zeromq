//  Check whether the sizes of public representation of the message (zmq_ZmqMessage)
//  and private representation of the message (ZmqMessage) match.

use std::mem;
use std::mem::size_of;
use libc::c_long;
use crate::atomic_counter::AtomicCounter;
use crate::metadata::metadata_t;
use crate::zmq_hdr::ZMQ_GROUP_MAX_LENGTH;

#[derive(Default, Debug, Clone)]
pub struct ZmqContent {
    data: Vec<u8>,
    size: usize,
    // msg_free_fn: *ffn;
    hint: Vec<u8>,
    refcnt: AtomicCounter,
}

// enum
//     {
//         ZmqMessage_size = 64
//     }
pub const ZmqMessage_size: usize = 64;

// enum
//     {
//         max_vsm_size =
//           ZmqMessage_size - (sizeof (metadata_t *) + 3 + 16 + mem::size_of::<uint32_t>())
//     }
pub const max_vsm_size: usize = ZmqMessage_size - size_of::<*mut metadata_t> + 3 + 16 + size_of::<u32>();

// enum
//     {
//         ping_cmd_name_size = 5,   // 4PING
//         cancel_cmd_name_size = 7, // 6CANCEL
//         sub_cmd_name_size = 10    // 9SUBSCRIBE
//     }
pub const ping_cmd_name_size: usize = 5;   // 4PING

pub const cancel_cmd_name_size: usize = 7; // 6CANCEL

pub const sub_cmd_name_size: usize = 10;    // 9SUBSCRIBE

// enum {
    pub const more: u8 = 1;
    //  Followed by more parts
    pub const command: u8 = 2;
    //  Command frame (see ZMTP spec)
    //  Command types, use only bits 2-5 and compare with ==, not bitwise,
    //  a command can never be of more that one type at the same time
    pub const ping: u8 = 4;
    pub const pong: u8 = 8;
    pub const subscribe: u8 = 12;
    pub const cancel: u8 = 16;
    pub const close_cmd: u8 = 20;
    pub const credential: u8 = 32;
    pub const routing_id: u8 = 64;
    pub const shared: u8 = 128;
// }

// enum ZmqMessageType {
    pub const type_min: u8 = 101;
    //  VSM messages store the content in the message itself
    pub const type_vsm: u8 = 101;
    //  LMSG messages store the content in malloc-ed memory
    pub const type_lmsg: u8 = 102;
    //  Delimiter messages are used in envelopes
    pub const type_delimiter: u8 = 103;
    //  CMSG messages point to constant data
    pub const type_cmsg: u8 = 104;

    // zero-copy LMSG message for v2_decoder
    pub const type_zclmsg: u8 = 105;

    //  Join message for radio_dish
    pub const type_join: u8 = 106;

    //  Leave message for radio_dish
    pub const type_leave: u8 = 107;

    pub const type_max: u8 = 107;
// }

// enum GroupType {
   pub const  group_type_short: u8 = 0;
   pub const group_type_long: u8 = 1;
// }

#[derive(Default, Debug, Clone)]
pub struct LongGroup {
    pub group: [u8; ZMQ_GROUP_MAX_LENGTH + 1],
    pub refcnt: AtomicCounter,
}

#[derive(Default,Debug,Clone)]
pub struct GroupSgroup {
    pub type_: u8,
    pub group: [u8; 15],
}

#[derive(Default,Debug,Clone)]
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
    pub metadata: Option<metadata_t>,
    pub unused: [u8; ZmqMessage_size - size_of::<*mut metadata_t>() + 2 + size_of::<u32>() + size_of::<ZmqMsgGrp>()],
    pub type_: u8,
    pub flags: u8,
    pub routing_id: u32,
    pub group: ZmqMsgGrp,
}

#[derive(Default,Debug,Clone)]
pub struct MsgUnionVsm {
    pub metadata: Option<metadata_t>,
    pub data: [u8; max_vsm_size],
    pub size: u8,
    pub type_: u8,
    pub flags: u8,
    pub routing_id: u32,
    pub group: ZmqMsgGrp,
}

#[derive(Default,Debug,Clone)]
pub struct MsgUnionLmsg
{
    pub metadata: Option<metadata_t>,
    pub content: ZmqContent,
    pub unused: [u8;size_of::<*mut metadata_t>() + size_of::<*mut ZmqContent>() + 2 + size_of::<u32>() + size_of::<ZmqMsgGrp>()],
    pub type_: u8,
    pub flags: u8,
    pub routing_id: u32,
    pub group: ZmqMsgGrp
}

#[derive(Default,Debug,Clone)]
pub struct MsgUnionZclmsg
{
    pub metadata: Option<metadata_t>,
    pub content: ZmqContent,
    pub unused: [u8;size_of::<*mut metadata_t>() + size_of::<*mut ZmqContent>() + 2 + size_of::<u32>() + size_of::<ZmqMsgGrp>()],
    pub type_: u8,
    pub flags: u8,
    pub routing_id: u32,
    pub group: ZmqMsgGrp,
}

#[derive(Default,Debug,Clone)]
pub struct MsgUnionCmsg
{
    pub metadata: Option<metadata_t>,
    pub content: ZmqContent,
    pub data: Vec<u8>,
    pub size: usize,
    pub unused: [u8;size_of::<*mut metadata_t>() + size_of::<*mut ZmqContent>() + 2 + size_of::<u32>() + size_of::<ZmqMsgGrp>()],
    pub type_: u8,
    pub flags: u8,
    pub routing_id: u32,
    pub group: ZmqMsgGrp,
}

#[derive(Default,Debug,Clone)]
pub struct MsgUnionDelimiter
{
    pub metadata: Option<metadata_t>,
    pub unused: [u8;size_of::<*mut metadata_t>() + size_of::<*mut ZmqContent>() + 2 + size_of::<u32>() + size_of::<ZmqMsgGrp>()],
    pub type_: u8,
    pub flags: u8,
    pub routing_id: u32,
    pub group: ZmqMsgGrp
}

//  Note that fields shared between different message types are not
//  moved to the parent class (ZmqMessage). This way we get tighter packing
//  of the data. Shared fields can be accessed via 'base' member of
//  the union.
#[derive(Default,Debug,Clone)]
pub union MsgUnion
{
    pub base: MsgUnionBase,
    pub vsm: MsgUnionVsm,
    pub lmsg: MsgUnionLmsg,
    pub zclmsg: MsgUnionZclmsg,
    pub cmsg: MsgUnionCmsg,
    pub delimiter: MsgUnionDelimiter
}


pub const cancel_cmd_name: String = String::from("\0x6CANCEL");
pub const sub_cmd_name: String = String::from("\0x9SUBSCRIBE");

#[derive(Default,Debug,Clone)]
pub struct ZmqMessage
{
// public:
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
  // private:
    refcnt: AtomicCounter,
    //  Different message types.
    _u: MsgUnion,
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
    // void set_flags (unsigned char flags_);
    // void reset_flags (unsigned char flags_);
    // metadata_t *metadata () const;
    // void set_metadata (metadata_t *metadata_);
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
    //     return (_u.base.flags & CMD_TYPE_MASK) == subscribe;
    // }
    //
    // bool is_cancel () const
    // {
    //     return (_u.base.flags & CMD_TYPE_MASK) == cancel;
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
    // void shrink (new_size_: usize);
    
    pub fn check (&mut self)->bool
    {
        return self._u.base.type_ >= type_min && self._u.base.type_ <= type_max;
    }
    
    pub fn init (&mut self, data: &mut [u8], size: usize, hint: &mut [u8], content: Option<&mut ZmqContent>) -> i32
    {
        if size < max_vsm_size {
            let rc: i32 = self.init_size(size);
    
            if (rc != -1) {
                // TODO:
                // memcpy (data (), data, size);
                return 0;
            }
            return -1;
        }
        if content.is_some() {
            return self.init_external_storage (content.unwrap(), data, size, hint);
        }
        return self.init_data(data, size, hint);
    }
    
    pub fn init2(&mut self) ->i32
    {
        self._u.vsm.metadata = None;
        self._u.vsm.type_ = type_vsm;
        self._u.vsm.flags = 0;
        self._u.vsm.size = 0;
        self._u.vsm.group.sgroup.group[0] = 0;
        self._u.vsm.group.type_ = group_type_short;
        self._u.vsm.routing_id = 0;
        return 0;
    }
    
    pub fn init_size (&mut self, size: usize) -> i32
    {
        if size <= max_vsm_size {
            self._u.vsm.metadata = None;
            self._u.vsm.type_ = type_vsm;
            self._u.vsm.flags = 0;
            self._u.vsm.size = size as u8;
            self._u.vsm.group.sgroup.group[0] = 0;
            self._u.vsm.group.type_ = group_type_short;
            self._u.vsm.routing_id = 0;
        } else {
            self._u.lmsg.metadata = None;
            self._u.lmsg.type_ = type_lmsg;
            self._u.lmsg.flags = 0;
            self._u.lmsg.group.sgroup.group[0] = 0;
            self._u.lmsg.group.type_ = group_type_short;
            self._u.lmsg.routing_id = 0;
            self._u.lmsg.content = NULL;
            // if (mem::size_of::<ZmqContent>() + size > size)
            // if mem::size_of::<ZmqContent>() + size > size
            // {
            //     self._u.lmsg.content = static_cast < ZmqContent * > (malloc(mem::size_of::<ZmqContent>() + size));
            // }
            // if (unlikely (!_u.lmsg.content)) {
            //     errno = ENOMEM;
            //     return -1;
            // }
            self._u.lmsg.content = ZmqContent::default();
            // self._u.lmsg.content.data = self._u.lmsg.content + 1;
            // self._u.lmsg.content.size = size;
            // self._u.lmsg.content.ffn = NULL;
            // self._u.lmsg.content.hint = NULL;
            // new (&self._u.lmsg.content->refcnt) AtomicCounter ();
        }
        return 0;
    }
    
    pub fn init_buffer (&mut self, buf_: &mut [u8], size: usize) -> i32
    {
        let rc: i32 = self.init_size (size);
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
    
    pub fn init_external_storage (&mut self, content: &mut ZmqContent,
                                           data: &mut [u8],
                                           size: usize,
                                           hint: &mut [u8]) -> i32
    {
        // zmq_assert (NULL != data);
        // zmq_assert (NULL != content);
    
        self._u.zclmsg.metadata = None;
        self._u.zclmsg.type_ = type_zclmsg;
        self._u.zclmsg.flags = 0;
        self._u.zclmsg.group.sgroup.group[0] = 0;
        self._u.zclmsg.group.type_ = group_type_short;
        self._u.zclmsg.routing_id = 0;
        self._u.zclmsg.content = content.clone();
        self._u.zclmsg.content.data = data.clone();
        self._u.zclmsg.content.size = size;
        // self._u.zclmsg.content->ffn = ffn_;
        self._u.zclmsg.content.hint = hint.clone();
        // new (&_u.zclmsg.content->refcnt) AtomicCounter ();
        self._u.zclmsg.content.refcnt = AtomicCounter::new();
    
        return 0;
    }
    
    pub fn init_data (&mut self, data: &mut [u8], size: usize, hint: &mut [u8]) -> i32
    {
        //  If data is NULL and size is not 0, a segfault
        //  would occur once the data is accessed
        // zmq_assert (data != NULL || size == 0);
    
        //  Initialize constant message if there's no need to deallocate
        // if (ffn_ == NULL)
        // {
            self._u.cmsg.metadata = None;
            self._u.cmsg.type_ = type_cmsg;
            self._u.cmsg.flags = 0;
            self._u.cmsg.data.clone_from_slice(data);
            self._u.cmsg.size = size;
            self._u.cmsg.group.sgroup.group[0] = 0;
            self._u.cmsg.group.type_ = group_type_short;
            self._u.cmsg.routing_id = 0;
        // }
        // else {
        //     _u.lmsg.metadata = NULL;
        //     _u.lmsg.type = type_lmsg;
        //     _u.lmsg.flags = 0;
        //     _u.lmsg.group.sgroup.group[0] = 0;
        //     _u.lmsg.group.type = group_type_short;
        //     _u.lmsg.routing_id = 0;
        //     _u.lmsg.content =
        //       static_cast<ZmqContent *> (malloc (mem::size_of::<ZmqContent>()));
        //     if (!_u.lmsg.content) {
        //         errno = ENOMEM;
        //         return -1;
        //     }
        //
        //     _u.lmsg.content->data = data;
        //     _u.lmsg.content->size = size;
        //     _u.lmsg.content->ffn = ffn_;
        //     _u.lmsg.content->hint = hint;
        //     new (&_u.lmsg.content->refcnt) AtomicCounter ();
        // }
        return 0;
    }
    
    pub fn init_delimiter (&mut self) -> io32
    {
        self._u.delimiter.metadata = None;
        self._u.delimiter.type_ = type_delimiter;
        self._u.delimiter.flags = 0;
        self._u.delimiter.group.sgroup.group[0] = 0;
        self._u.delimiter.group.type_ = group_type_short;
        self._u.delimiter.routing_id = 0;
        return 0;
    }
    
    pub fn init_join (&mut self) -> i32
    {
        self._u.base.metadata = None;
        self._u.base.type_ = type_join;
        self._u.base.flags = 0;
        self._u.base.group.sgroup.group[0] = 0;
        self._u.base.group.type_ = group_type_short;
        self._u.base.routing_id = 0;
        return 0;
    }
    
    pub fn init_leave (&mut self) -> i32
    {
        self._u.base.metadata = None;
        self._u.base.type_ = type_leave;
        self._u.base.flags = 0;
        self._u.base.group.sgroup.group[0] = 0;
        self._u.base.group.type_ = group_type_short;
        self._u.base.routing_id = 0;
        return 0;
    }
    
    pub fn init_subscribe (&mut self, size: usize, topic: &mut[u8]) -> i32
    {
        let rc = self.init_size (size);
        if (rc == 0) {
            self.set_flags (subscribe);
    
            //  We explicitly allow a NULL subscription with size zero
            if (size) {
                // assert (topic);
                // TODO:
                // memcpy (data (), topic, size);
            }
        }
        return rc;
    }
    
    pub fn init_cancel (&mut self, size: usize, topic: &mut[u8]) -> i32
    {
        let rc = self.init_size (size);
        if rc == 0 {
            self.set_flags (cancel);
    
            //  We explicitly allow a NULL subscription with size zero
            if size {
                // assert (topic);
                // TODO
                // memcpy (data (), topic, size);
            }
        }
        return rc;
    }
    
    pub fn close (&mut self) -> i32
    {
        //  Check the validity of the message.
        // if (unlikely (!check ())) {
        //     errno = EFAULT;
        //     return -1;
        // }
    
        if self._u.base.type_ == type_lmsg {
            //  If the content is not shared, or if it is shared and the reference
            //  count has dropped to zero, deallocate it.
            if !(self._u.lmsg.flags & shared) != 0
                || !self._u.lmsg.content.refcnt.sub (1) {
                //  We used "placement new" operator to initialize the reference
                //  counter so we call the destructor explicitly now.
                // self._u.lmsg.content->refcnt.~AtomicCounter ();
    
                // if (_u.lmsg.content->ffn)
                //     _u.lmsg.content->ffn (_u.lmsg.content->data,
                //                           _u.lmsg.content->hint);
                // free (_u.lmsg.content);
            }
        }
    
        if (self.is_zcmsg ()) {
            // zmq_assert (_u.zclmsg.content->ffn);
    
            //  If the content is not shared, or if it is shared and the reference
            //  count has dropped to zero, deallocate it.
            if (!(self._u.zclmsg.flags & shared) != 0
                || !self._u.zclmsg.content.refcnt.sub (1)) {
                //  We used "placement new" operator to initialize the reference
                //  counter so we call the destructor explicitly now.
                // self._u.zclmsg.content.refcnt.~AtomicCounter ();
    
                // _u.zclmsg.content->ffn (_u.zclmsg.content->data,
                //                         _u.zclmsg.content->hint);
            }
        }
    
        if (self._u.base.metadata.is_some()) {
            if (_u.base.metadata.drop_ref ()) {
                // LIBZMQ_DELETE (_u.base.metadata);
            }
            self._u.base.metadata = None;
        }
    
        if (self._u.base.group.type_ == group_type_long) {
            if (!self._u.base.group.lgroup.content.refcnt.sub (1)) {
                //  We used "placement new" operator to initialize the reference
                //  counter so we call the destructor explicitly now.
                // self._u.base.group.lgroup.content.refcnt.~AtomicCounter ();
    
                // free (_u.base.group.lgroup.content);
            }
        }
    
        //  Make the message invalid.
        self._u.base.type_ = 0;
    
        return 0;
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
    //     if (src_._u.base.metadata != NULL)
    //         src_._u.base.metadata->add_ref ();
    //
    //     if (src_._u.base.group.type == group_type_long)
    //         src_._u.base.group.lgroup.content->refcnt.add (1);
    //
    //     *this = src_;
    //
    //     return 0;
    // }
    
    pub fn data (&mut self) -> Option<Vec<u8>>
    {
        //  Check the validity of the message.
        // zmq_assert (check ());
    
        match self._u.base.type_ {
             type_vsm=> Some(Vec::from(self._u.vsm.data)),
             type_lmsg=> Some(self._u.lmsg.content.data.clone()),
             type_cmsg=> Some(self._u.cmsg.data.clone()),
            type_zclmsg => Some(self._u.zclmsg.content.data.clone()),
            _=> None
        }
    }
    
    size_t size () const
    {
        //  Check the validity of the message.
        zmq_assert (check ());
    
        switch (_u.base.type) {
            case type_vsm:
                return _u.vsm.size;
            case type_lmsg:
                return _u.lmsg.content->size;
            case type_zclmsg:
                return _u.zclmsg.content->size;
            case type_cmsg:
                return _u.cmsg.size;
            default:
                zmq_assert (false);
                return 0;
        }
    }
    
    void shrink (new_size_: usize)
    {
        //  Check the validity of the message.
        zmq_assert (check ());
        zmq_assert (new_size_ <= size ());
    
        switch (_u.base.type) {
            case type_vsm:
                _u.vsm.size = static_cast<unsigned char> (new_size_);
                break;
            case type_lmsg:
                _u.lmsg.content->size = new_size_;
                break;
            case type_zclmsg:
                _u.zclmsg.content->size = new_size_;
                break;
            case type_cmsg:
                _u.cmsg.size = new_size_;
                break;
            default:
                zmq_assert (false);
        }
    }
    
    unsigned char flags () const
    {
        return _u.base.flags;
    }
    
    void set_flags (unsigned char flags_)
    {
        _u.base.flags |= flags_;
    }
    
    void reset_flags (unsigned char flags_)
    {
        _u.base.flags &= ~flags_;
    }
    
    metadata_t *metadata () const
    {
        return _u.base.metadata;
    }
    
    void set_metadata (metadata_t *metadata_)
    {
        assert (metadata_ != NULL);
        assert (_u.base.metadata == NULL);
        metadata_->add_ref ();
        _u.base.metadata = metadata_;
    }
    
    void reset_metadata ()
    {
        if (_u.base.metadata) {
            if (_u.base.metadata->drop_ref ()) {
                LIBZMQ_DELETE (_u.base.metadata);
            }
            _u.base.metadata = NULL;
        }
    }
    
    bool is_routing_id () const
    {
        return (_u.base.flags & routing_id) == routing_id;
    }
    
    bool is_credential () const
    {
        return (_u.base.flags & credential) == credential;
    }
    
    bool is_delimiter () const
    {
        return _u.base.type == type_delimiter;
    }
    
    bool is_vsm () const
    {
        return _u.base.type == type_vsm;
    }
    
    bool is_cmsg () const
    {
        return _u.base.type == type_cmsg;
    }
    
    bool is_lmsg () const
    {
        return _u.base.type == type_lmsg;
    }
    
    bool is_zcmsg () const
    {
        return _u.base.type == type_zclmsg;
    }
    
    bool is_join () const
    {
        return _u.base.type == type_join;
    }
    
    bool is_leave () const
    {
        return _u.base.type == type_leave;
    }
    
    bool is_ping () const
    {
        return (_u.base.flags & CMD_TYPE_MASK) == ping;
    }
    
    bool is_pong () const
    {
        return (_u.base.flags & CMD_TYPE_MASK) == pong;
    }
    
    bool is_close_cmd () const
    {
        return (_u.base.flags & CMD_TYPE_MASK) == close_cmd;
    }
    
    size_t command_body_size () const
    {
        if (this->is_ping () || this->is_pong ())
            return this->size () - ping_cmd_name_size;
        else if (!(this->flags () & command)
                 && (this->is_subscribe () || this->is_cancel ()))
            return this->size ();
        else if (this->is_subscribe ())
            return this->size () - sub_cmd_name_size;
        else if (this->is_cancel ())
            return this->size () - cancel_cmd_name_size;
    
        return 0;
    }
    
    void *command_body ()
    {
        unsigned char *data = NULL;
    
        if (this->is_ping () || this->is_pong ())
            data =
              static_cast<unsigned char *> (this->data ()) + ping_cmd_name_size;
        //  With inproc, command flag is not set for sub/cancel
        else if (!(this->flags () & command)
                 && (this->is_subscribe () || this->is_cancel ()))
            data = static_cast<unsigned char *> (this->data ());
        else if (this->is_subscribe ())
            data = static_cast<unsigned char *> (this->data ()) + sub_cmd_name_size;
        else if (this->is_cancel ())
            data =
              static_cast<unsigned char *> (this->data ()) + cancel_cmd_name_size;
    
        return data;
    }
    
    void add_refs (refs_: i32)
    {
        zmq_assert (refs_ >= 0);
    
        //  Operation not supported for messages with metadata.
        zmq_assert (_u.base.metadata == NULL);
    
        //  No copies required.
        if (!refs_)
            return;
    
        //  VSMs, CMSGS and delimiters can be copied straight away. The only
        //  message type that needs special care are long messages.
        if (_u.base.type == type_lmsg || is_zcmsg ()) {
            if (_u.base.flags & shared)
                refcnt ()->add (refs_);
            else {
                refcnt ()->set (refs_ + 1);
                _u.base.flags |= shared;
            }
        }
    }
    
    bool rm_refs (refs_: i32)
    {
        zmq_assert (refs_ >= 0);
    
        //  Operation not supported for messages with metadata.
        zmq_assert (_u.base.metadata == NULL);
    
        //  No copies required.
        if (!refs_)
            return true;
    
        //  If there's only one reference close the message.
        if ((_u.base.type != type_zclmsg && _u.base.type != type_lmsg)
            || !(_u.base.flags & shared)) {
            close ();
            return false;
        }
    
        //  The only message type that needs special care are long and zcopy messages.
        if (_u.base.type == type_lmsg && !_u.lmsg.content->refcnt.sub (refs_)) {
            //  We used "placement new" operator to initialize the reference
            //  counter so we call the destructor explicitly now.
            _u.lmsg.content->refcnt.~AtomicCounter ();
    
            if (_u.lmsg.content->ffn)
                _u.lmsg.content->ffn (_u.lmsg.content->data, _u.lmsg.content->hint);
            free (_u.lmsg.content);
    
            return false;
        }
    
        if (is_zcmsg () && !_u.zclmsg.content->refcnt.sub (refs_)) {
            // storage for rfcnt is provided externally
            if (_u.zclmsg.content->ffn) {
                _u.zclmsg.content->ffn (_u.zclmsg.content->data,
                                        _u.zclmsg.content->hint);
            }
    
            return false;
        }
    
        return true;
    }
    
    uint32_t get_routing_id () const
    {
        return _u.base.routing_id;
    }
    
    int set_routing_id (uint32_t routing_id_)
    {
        if (routing_id_) {
            _u.base.routing_id = routing_id_;
            return 0;
        }
        errno = EINVAL;
        return -1;
    }
    
    int reset_routing_id ()
    {
        _u.base.routing_id = 0;
        return 0;
    }
    
    const char *group () const
    {
        if (_u.base.group.type == group_type_long)
            return _u.base.group.lgroup.content->group;
        return _u.base.group.sgroup.group;
    }
    
    int set_group (group_: *const c_char)
    {
        size_t length = strnlen (group_, ZMQ_GROUP_MAX_LENGTH);
    
        return set_group (group_, length);
    }
    
    int set_group (group_: *const c_char, length_: usize)
    {
        if (length_ > ZMQ_GROUP_MAX_LENGTH) {
            errno = EINVAL;
            return -1;
        }
    
        if (length_ > 14) {
            _u.base.group.lgroup.type = group_type_long;
            _u.base.group.lgroup.content =
              (long_group_t *) malloc (mem::size_of::<long_group_t>());
            assert (_u.base.group.lgroup.content);
            new (&_u.base.group.lgroup.content->refcnt) AtomicCounter ();
            _u.base.group.lgroup.content->refcnt.set (1);
            strncpy (_u.base.group.lgroup.content->group, group_, length_);
            _u.base.group.lgroup.content->group[length_] = 0;
        } else {
            strncpy (_u.base.group.sgroup.group, group_, length_);
            _u.base.group.sgroup.group[length_] = 0;
        }
    
        return 0;
    }
    
    AtomicCounter *refcnt ()
    {
        switch (_u.base.type) {
            case type_lmsg:
                return &_u.lmsg.content->refcnt;
            case type_zclmsg:
                return &_u.zclmsg.content->refcnt;
            default:
                zmq_assert (false);
                return NULL;
        }
    }

}

pub fn close_and_return (msg: &mut ZmqMessage, echo: i32) -> i32
{
    // Since we abort on close failure we preserve errno for success case.
    let err: i32 = errno;
    let rc: i32 = msg.close ();
    errno_assert (rc == 0);
    errno = err;
    return echo;
}

pub fn close_and_return2 (msg: &mut [ZmqMessage], count: i32, echo: i32) -> i32
{
    // for (int i = 0; i < count; i++)
    for i in 0 .. count
    {
        close_and_return(&mut msg[i], 0);
    }
    return echo;
}

// typedef char
//   zmq_msg_size_check[2 * ((sizeof (ZmqMessage) == mem::size_of::<zmq_ZmqMessage>()) != 0)
//                      - 1];
