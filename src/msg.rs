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
// #include "compat.hpp"
// #include "macros.hpp"
// #include "msg.hpp"

// #include <string.h>
// #include <stdlib.h>
// #include <new>

// #include "stdint.hpp"
// #include "likely.hpp"
// #include "metadata.hpp"
// #include "err.hpp"

//  Check whether the sizes of public representation of the message (zmq_msg_t)
//  and private representation of the message (zmq::msg_t) match.


pub const cancel_cmd_name: String = String::from("\0x6CANCEL");
pub const sub_cmd_name: String = String::from("\0x9SUBSCRIBE");
pub struct msg_t
{
// public:
    //  Shared message buffer. Message data are either allocated in one
    //  continuous block along with this structure - thus avoiding one
    //  malloc/free pair or they are stored in user-supplied memory.
    //  In the latter case, ffn member stores pointer to the function to be
    //  used to deallocate the data. If the buffer is actually shared (there
    //  are at least 2 references to it) refcount member contains number of
    //  references.
    struct content_t
    {
        data: *mut c_void;
        size: usize;
        msg_free_fn *ffn;
        hint: *mut c_void;
        zmq::atomic_counter_t refcnt;
    };

    //  Message flags.
    enum
    {
        more = 1,    //  Followed by more parts
        command = 2, //  Command frame (see ZMTP spec)
        //  Command types, use only bits 2-5 and compare with ==, not bitwise,
        //  a command can never be of more that one type at the same time
        ping = 4,
        pong = 8,
        subscribe = 12,
        cancel = 16,
        close_cmd = 20,
        credential = 32,
        routing_id = 64,
        shared = 128
    };

    bool check () const;
    int init ();

    int init (data_: *mut c_void,
              size_: usize,
              msg_free_fn *ffn_,
              hint_: *mut c_void,
              content_t *content_ = NULL);

    int init_size (size_: usize);
    int init_buffer (const buf_: *mut c_void, size_: usize);
    int init_data (data_: *mut c_void, size_: usize, msg_free_fn *ffn_, hint_: *mut c_void);
    int init_external_storage (content_t *content_,
                               data_: *mut c_void,
                               size_: usize,
                               msg_free_fn *ffn_,
                               hint_: *mut c_void);
    int init_delimiter ();
    int init_join ();
    int init_leave ();
    int init_subscribe (const size_: usize, const unsigned char *topic);
    int init_cancel (const size_: usize, const unsigned char *topic);
    int close ();
    int move (msg_t &src_);
    int copy (msg_t &src_);
    void *data ();
    size_t size () const;
    unsigned char flags () const;
    void set_flags (unsigned char flags_);
    void reset_flags (unsigned char flags_);
    metadata_t *metadata () const;
    void set_metadata (metadata_t *metadata_);
    void reset_metadata ();
    bool is_routing_id () const;
    bool is_credential () const;
    bool is_delimiter () const;
    bool is_join () const;
    bool is_leave () const;
    bool is_ping () const;
    bool is_pong () const;
    bool is_close_cmd () const;

    //  These are called on each message received by the session_base class,
    //  so get them inlined to avoid the overhead of 2 function calls per msg
    bool is_subscribe () const
    {
        return (_u.base.flags & CMD_TYPE_MASK) == subscribe;
    }

    bool is_cancel () const
    {
        return (_u.base.flags & CMD_TYPE_MASK) == cancel;
    }

    size_t command_body_size () const;
    void *command_body ();
    bool is_vsm () const;
    bool is_cmsg () const;
    bool is_lmsg () const;
    bool is_zcmsg () const;
    uint32_t get_routing_id () const;
    int set_routing_id (uint32_t routing_id_);
    int reset_routing_id ();
    const char *group () const;
    int set_group (group_: *const c_char);
    int set_group (const char *, length_: usize);

    //  After calling this function you can copy the message in POD-style
    //  refs_ times. No need to call copy.
    void add_refs (refs_: i32);

    //  Removes references previously added by add_refs. If the number of
    //  references drops to 0, the message is closed and false is returned.
    bool rm_refs (refs_: i32);

    void shrink (new_size_: usize);

    //  Size in bytes of the largest message that is still copied around
    //  rather than being reference-counted.
    enum
    {
        msg_t_size = 64
    };
    enum
    {
        max_vsm_size =
          msg_t_size - (sizeof (metadata_t *) + 3 + 16 + sizeof (uint32_t))
    };
    enum
    {
        ping_cmd_name_size = 5,   // 4PING
        cancel_cmd_name_size = 7, // 6CANCEL
        sub_cmd_name_size = 10    // 9SUBSCRIBE
    };

  // private:
    zmq::atomic_counter_t *refcnt ();

    //  Different message types.
    enum type_t
    {
        type_min = 101,
        //  VSM messages store the content in the message itself
        type_vsm = 101,
        //  LMSG messages store the content in malloc-ed memory
        type_lmsg = 102,
        //  Delimiter messages are used in envelopes
        type_delimiter = 103,
        //  CMSG messages point to constant data
        type_cmsg = 104,

        // zero-copy LMSG message for v2_decoder
        type_zclmsg = 105,

        //  Join message for radio_dish
        type_join = 106,

        //  Leave message for radio_dish
        type_leave = 107,

        type_max = 107
    };

    enum group_type_t
    {
        group_type_short,
        group_type_long
    };

    struct long_group_t
    {
        char group[ZMQ_GROUP_MAX_LENGTH + 1];
        atomic_counter_t refcnt;
    };

    union group_t
    {
        unsigned char type;
        struct
        {
            unsigned char type;
            char group[15];
        } sgroup;
        struct
        {
            unsigned char type;
            long_group_t *content;
        } lgroup;
    };

    //  Note that fields shared between different message types are not
    //  moved to the parent class (msg_t). This way we get tighter packing
    //  of the data. Shared fields can be accessed via 'base' member of
    //  the union.
    union
    {
        struct
        {
            metadata_t *metadata;
            unsigned char unused[msg_t_size
                                 - (sizeof (metadata_t *) + 2
                                    + sizeof (uint32_t) + sizeof (group_t))];
            unsigned char type;
            unsigned char flags;
            uint32_t routing_id;
            group_t group;
        } base;
        struct
        {
            metadata_t *metadata;
            unsigned char data[max_vsm_size];
            unsigned char size;
            unsigned char type;
            unsigned char flags;
            uint32_t routing_id;
            group_t group;
        } vsm;
        struct
        {
            metadata_t *metadata;
            content_t *content;
            unsigned char
              unused[msg_t_size
                     - (sizeof (metadata_t *) + sizeof (content_t *) + 2
                        + sizeof (uint32_t) + sizeof (group_t))];
            unsigned char type;
            unsigned char flags;
            uint32_t routing_id;
            group_t group;
        } lmsg;
        struct
        {
            metadata_t *metadata;
            content_t *content;
            unsigned char
              unused[msg_t_size
                     - (sizeof (metadata_t *) + sizeof (content_t *) + 2
                        + sizeof (uint32_t) + sizeof (group_t))];
            unsigned char type;
            unsigned char flags;
            uint32_t routing_id;
            group_t group;
        } zclmsg;
        struct
        {
            metadata_t *metadata;
            data: *mut c_void;
            size: usize;
            unsigned char unused[msg_t_size
                                 - (sizeof (metadata_t *) + sizeof (void *)
                                    + sizeof (size_t) + 2 + sizeof (uint32_t)
                                    + sizeof (group_t))];
            unsigned char type;
            unsigned char flags;
            uint32_t routing_id;
            group_t group;
        } cmsg;
        struct
        {
            metadata_t *metadata;
            unsigned char unused[msg_t_size
                                 - (sizeof (metadata_t *) + 2
                                    + sizeof (uint32_t) + sizeof (group_t))];
            unsigned char type;
            unsigned char flags;
            uint32_t routing_id;
            group_t group;
        } delimiter;
    } _u;
};

inline int close_and_return (zmq::msg_t *msg_, echo_: i32)
{
    // Since we abort on close failure we preserve errno for success case.
    const int err = errno;
    const int rc = msg_->close ();
    errno_assert (rc == 0);
    errno = err;
    return echo_;
}

inline int close_and_return (zmq::msg_t msg_[], count_: i32, echo_: i32)
{
    for (int i = 0; i < count_; i++)
        close_and_return (&msg_[i], 0);
    return echo_;
}

typedef char
  zmq_msg_size_check[2 * ((sizeof (zmq::msg_t) == sizeof (zmq_msg_t)) != 0)
                     - 1];

bool zmq::msg_t::check () const
{
    return _u.base.type >= type_min && _u.base.type <= type_max;
}

int zmq::msg_t::init (data_: *mut c_void,
                      size_: usize,
                      msg_free_fn *ffn_,
                      hint_: *mut c_void,
                      content_t *content_)
{
    if (size_ < max_vsm_size) {
        const int rc = init_size (size_);

        if (rc != -1) {
            memcpy (data (), data_, size_);
            return 0;
        }
        return -1;
    }
    if (content_) {
        return init_external_storage (content_, data_, size_, ffn_, hint_);
    }
    return init_data (data_, size_, ffn_, hint_);
}

int zmq::msg_t::init ()
{
    _u.vsm.metadata = NULL;
    _u.vsm.type = type_vsm;
    _u.vsm.flags = 0;
    _u.vsm.size = 0;
    _u.vsm.group.sgroup.group[0] = '\0';
    _u.vsm.group.type = group_type_short;
    _u.vsm.routing_id = 0;
    return 0;
}

int zmq::msg_t::init_size (size_: usize)
{
    if (size_ <= max_vsm_size) {
        _u.vsm.metadata = NULL;
        _u.vsm.type = type_vsm;
        _u.vsm.flags = 0;
        _u.vsm.size = static_cast<unsigned char> (size_);
        _u.vsm.group.sgroup.group[0] = '\0';
        _u.vsm.group.type = group_type_short;
        _u.vsm.routing_id = 0;
    } else {
        _u.lmsg.metadata = NULL;
        _u.lmsg.type = type_lmsg;
        _u.lmsg.flags = 0;
        _u.lmsg.group.sgroup.group[0] = '\0';
        _u.lmsg.group.type = group_type_short;
        _u.lmsg.routing_id = 0;
        _u.lmsg.content = NULL;
        if (sizeof (content_t) + size_ > size_)
            _u.lmsg.content =
              static_cast<content_t *> (malloc (sizeof (content_t) + size_));
        if (unlikely (!_u.lmsg.content)) {
            errno = ENOMEM;
            return -1;
        }

        _u.lmsg.content->data = _u.lmsg.content + 1;
        _u.lmsg.content->size = size_;
        _u.lmsg.content->ffn = NULL;
        _u.lmsg.content->hint = NULL;
        new (&_u.lmsg.content->refcnt) zmq::atomic_counter_t ();
    }
    return 0;
}

int zmq::msg_t::init_buffer (const buf_: *mut c_void, size_: usize)
{
    const int rc = init_size (size_);
    if (unlikely (rc < 0)) {
        return -1;
    }
    if (size_) {
        // NULL and zero size is allowed
        assert (NULL != buf_);
        memcpy (data (), buf_, size_);
    }
    return 0;
}

int zmq::msg_t::init_external_storage (content_t *content_,
                                       data_: *mut c_void,
                                       size_: usize,
                                       msg_free_fn *ffn_,
                                       hint_: *mut c_void)
{
    zmq_assert (NULL != data_);
    zmq_assert (NULL != content_);

    _u.zclmsg.metadata = NULL;
    _u.zclmsg.type = type_zclmsg;
    _u.zclmsg.flags = 0;
    _u.zclmsg.group.sgroup.group[0] = '\0';
    _u.zclmsg.group.type = group_type_short;
    _u.zclmsg.routing_id = 0;

    _u.zclmsg.content = content_;
    _u.zclmsg.content->data = data_;
    _u.zclmsg.content->size = size_;
    _u.zclmsg.content->ffn = ffn_;
    _u.zclmsg.content->hint = hint_;
    new (&_u.zclmsg.content->refcnt) zmq::atomic_counter_t ();

    return 0;
}

int zmq::msg_t::init_data (data_: *mut c_void,
                           size_: usize,
                           msg_free_fn *ffn_,
                           hint_: *mut c_void)
{
    //  If data is NULL and size is not 0, a segfault
    //  would occur once the data is accessed
    zmq_assert (data_ != NULL || size_ == 0);

    //  Initialize constant message if there's no need to deallocate
    if (ffn_ == NULL) {
        _u.cmsg.metadata = NULL;
        _u.cmsg.type = type_cmsg;
        _u.cmsg.flags = 0;
        _u.cmsg.data = data_;
        _u.cmsg.size = size_;
        _u.cmsg.group.sgroup.group[0] = '\0';
        _u.cmsg.group.type = group_type_short;
        _u.cmsg.routing_id = 0;
    } else {
        _u.lmsg.metadata = NULL;
        _u.lmsg.type = type_lmsg;
        _u.lmsg.flags = 0;
        _u.lmsg.group.sgroup.group[0] = '\0';
        _u.lmsg.group.type = group_type_short;
        _u.lmsg.routing_id = 0;
        _u.lmsg.content =
          static_cast<content_t *> (malloc (sizeof (content_t)));
        if (!_u.lmsg.content) {
            errno = ENOMEM;
            return -1;
        }

        _u.lmsg.content->data = data_;
        _u.lmsg.content->size = size_;
        _u.lmsg.content->ffn = ffn_;
        _u.lmsg.content->hint = hint_;
        new (&_u.lmsg.content->refcnt) zmq::atomic_counter_t ();
    }
    return 0;
}

int zmq::msg_t::init_delimiter ()
{
    _u.delimiter.metadata = NULL;
    _u.delimiter.type = type_delimiter;
    _u.delimiter.flags = 0;
    _u.delimiter.group.sgroup.group[0] = '\0';
    _u.delimiter.group.type = group_type_short;
    _u.delimiter.routing_id = 0;
    return 0;
}

int zmq::msg_t::init_join ()
{
    _u.base.metadata = NULL;
    _u.base.type = type_join;
    _u.base.flags = 0;
    _u.base.group.sgroup.group[0] = '\0';
    _u.base.group.type = group_type_short;
    _u.base.routing_id = 0;
    return 0;
}

int zmq::msg_t::init_leave ()
{
    _u.base.metadata = NULL;
    _u.base.type = type_leave;
    _u.base.flags = 0;
    _u.base.group.sgroup.group[0] = '\0';
    _u.base.group.type = group_type_short;
    _u.base.routing_id = 0;
    return 0;
}

int zmq::msg_t::init_subscribe (const size_: usize, const unsigned char *topic_)
{
    int rc = init_size (size_);
    if (rc == 0) {
        set_flags (zmq::msg_t::subscribe);

        //  We explicitly allow a NULL subscription with size zero
        if (size_) {
            assert (topic_);
            memcpy (data (), topic_, size_);
        }
    }
    return rc;
}

int zmq::msg_t::init_cancel (const size_: usize, const unsigned char *topic_)
{
    int rc = init_size (size_);
    if (rc == 0) {
        set_flags (zmq::msg_t::cancel);

        //  We explicitly allow a NULL subscription with size zero
        if (size_) {
            assert (topic_);
            memcpy (data (), topic_, size_);
        }
    }
    return rc;
}

int zmq::msg_t::close ()
{
    //  Check the validity of the message.
    if (unlikely (!check ())) {
        errno = EFAULT;
        return -1;
    }

    if (_u.base.type == type_lmsg) {
        //  If the content is not shared, or if it is shared and the reference
        //  count has dropped to zero, deallocate it.
        if (!(_u.lmsg.flags & msg_t::shared)
            || !_u.lmsg.content->refcnt.sub (1)) {
            //  We used "placement new" operator to initialize the reference
            //  counter so we call the destructor explicitly now.
            _u.lmsg.content->refcnt.~atomic_counter_t ();

            if (_u.lmsg.content->ffn)
                _u.lmsg.content->ffn (_u.lmsg.content->data,
                                      _u.lmsg.content->hint);
            free (_u.lmsg.content);
        }
    }

    if (is_zcmsg ()) {
        zmq_assert (_u.zclmsg.content->ffn);

        //  If the content is not shared, or if it is shared and the reference
        //  count has dropped to zero, deallocate it.
        if (!(_u.zclmsg.flags & msg_t::shared)
            || !_u.zclmsg.content->refcnt.sub (1)) {
            //  We used "placement new" operator to initialize the reference
            //  counter so we call the destructor explicitly now.
            _u.zclmsg.content->refcnt.~atomic_counter_t ();

            _u.zclmsg.content->ffn (_u.zclmsg.content->data,
                                    _u.zclmsg.content->hint);
        }
    }

    if (_u.base.metadata != NULL) {
        if (_u.base.metadata->drop_ref ()) {
            LIBZMQ_DELETE (_u.base.metadata);
        }
        _u.base.metadata = NULL;
    }

    if (_u.base.group.type == group_type_long) {
        if (!_u.base.group.lgroup.content->refcnt.sub (1)) {
            //  We used "placement new" operator to initialize the reference
            //  counter so we call the destructor explicitly now.
            _u.base.group.lgroup.content->refcnt.~atomic_counter_t ();

            free (_u.base.group.lgroup.content);
        }
    }

    //  Make the message invalid.
    _u.base.type = 0;

    return 0;
}

int zmq::msg_t::move (msg_t &src_)
{
    //  Check the validity of the source.
    if (unlikely (!src_.check ())) {
        errno = EFAULT;
        return -1;
    }

    int rc = close ();
    if (unlikely (rc < 0))
        return rc;

    *this = src_;

    rc = src_.init ();
    if (unlikely (rc < 0))
        return rc;

    return 0;
}

int zmq::msg_t::copy (msg_t &src_)
{
    //  Check the validity of the source.
    if (unlikely (!src_.check ())) {
        errno = EFAULT;
        return -1;
    }

    const int rc = close ();
    if (unlikely (rc < 0))
        return rc;

    // The initial reference count, when a non-shared message is initially
    // shared (between the original and the copy we create here).
    const atomic_counter_t::integer_t initial_shared_refcnt = 2;

    if (src_.is_lmsg () || src_.is_zcmsg ()) {
        //  One reference is added to shared messages. Non-shared messages
        //  are turned into shared messages.
        if (src_.flags () & msg_t::shared)
            src_.refcnt ()->add (1);
        else {
            src_.set_flags (msg_t::shared);
            src_.refcnt ()->set (initial_shared_refcnt);
        }
    }

    if (src_._u.base.metadata != NULL)
        src_._u.base.metadata->add_ref ();

    if (src_._u.base.group.type == group_type_long)
        src_._u.base.group.lgroup.content->refcnt.add (1);

    *this = src_;

    return 0;
}

void *zmq::msg_t::data ()
{
    //  Check the validity of the message.
    zmq_assert (check ());

    switch (_u.base.type) {
        case type_vsm:
            return _u.vsm.data;
        case type_lmsg:
            return _u.lmsg.content->data;
        case type_cmsg:
            return _u.cmsg.data;
        case type_zclmsg:
            return _u.zclmsg.content->data;
        default:
            zmq_assert (false);
            return NULL;
    }
}

size_t zmq::msg_t::size () const
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

void zmq::msg_t::shrink (new_size_: usize)
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

unsigned char zmq::msg_t::flags () const
{
    return _u.base.flags;
}

void zmq::msg_t::set_flags (unsigned char flags_)
{
    _u.base.flags |= flags_;
}

void zmq::msg_t::reset_flags (unsigned char flags_)
{
    _u.base.flags &= ~flags_;
}

zmq::metadata_t *zmq::msg_t::metadata () const
{
    return _u.base.metadata;
}

void zmq::msg_t::set_metadata (zmq::metadata_t *metadata_)
{
    assert (metadata_ != NULL);
    assert (_u.base.metadata == NULL);
    metadata_->add_ref ();
    _u.base.metadata = metadata_;
}

void zmq::msg_t::reset_metadata ()
{
    if (_u.base.metadata) {
        if (_u.base.metadata->drop_ref ()) {
            LIBZMQ_DELETE (_u.base.metadata);
        }
        _u.base.metadata = NULL;
    }
}

bool zmq::msg_t::is_routing_id () const
{
    return (_u.base.flags & routing_id) == routing_id;
}

bool zmq::msg_t::is_credential () const
{
    return (_u.base.flags & credential) == credential;
}

bool zmq::msg_t::is_delimiter () const
{
    return _u.base.type == type_delimiter;
}

bool zmq::msg_t::is_vsm () const
{
    return _u.base.type == type_vsm;
}

bool zmq::msg_t::is_cmsg () const
{
    return _u.base.type == type_cmsg;
}

bool zmq::msg_t::is_lmsg () const
{
    return _u.base.type == type_lmsg;
}

bool zmq::msg_t::is_zcmsg () const
{
    return _u.base.type == type_zclmsg;
}

bool zmq::msg_t::is_join () const
{
    return _u.base.type == type_join;
}

bool zmq::msg_t::is_leave () const
{
    return _u.base.type == type_leave;
}

bool zmq::msg_t::is_ping () const
{
    return (_u.base.flags & CMD_TYPE_MASK) == ping;
}

bool zmq::msg_t::is_pong () const
{
    return (_u.base.flags & CMD_TYPE_MASK) == pong;
}

bool zmq::msg_t::is_close_cmd () const
{
    return (_u.base.flags & CMD_TYPE_MASK) == close_cmd;
}

size_t zmq::msg_t::command_body_size () const
{
    if (this->is_ping () || this->is_pong ())
        return this->size () - ping_cmd_name_size;
    else if (!(this->flags () & msg_t::command)
             && (this->is_subscribe () || this->is_cancel ()))
        return this->size ();
    else if (this->is_subscribe ())
        return this->size () - sub_cmd_name_size;
    else if (this->is_cancel ())
        return this->size () - cancel_cmd_name_size;

    return 0;
}

void *zmq::msg_t::command_body ()
{
    unsigned char *data = NULL;

    if (this->is_ping () || this->is_pong ())
        data =
          static_cast<unsigned char *> (this->data ()) + ping_cmd_name_size;
    //  With inproc, command flag is not set for sub/cancel
    else if (!(this->flags () & msg_t::command)
             && (this->is_subscribe () || this->is_cancel ()))
        data = static_cast<unsigned char *> (this->data ());
    else if (this->is_subscribe ())
        data = static_cast<unsigned char *> (this->data ()) + sub_cmd_name_size;
    else if (this->is_cancel ())
        data =
          static_cast<unsigned char *> (this->data ()) + cancel_cmd_name_size;

    return data;
}

void zmq::msg_t::add_refs (refs_: i32)
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
        if (_u.base.flags & msg_t::shared)
            refcnt ()->add (refs_);
        else {
            refcnt ()->set (refs_ + 1);
            _u.base.flags |= msg_t::shared;
        }
    }
}

bool zmq::msg_t::rm_refs (refs_: i32)
{
    zmq_assert (refs_ >= 0);

    //  Operation not supported for messages with metadata.
    zmq_assert (_u.base.metadata == NULL);

    //  No copies required.
    if (!refs_)
        return true;

    //  If there's only one reference close the message.
    if ((_u.base.type != type_zclmsg && _u.base.type != type_lmsg)
        || !(_u.base.flags & msg_t::shared)) {
        close ();
        return false;
    }

    //  The only message type that needs special care are long and zcopy messages.
    if (_u.base.type == type_lmsg && !_u.lmsg.content->refcnt.sub (refs_)) {
        //  We used "placement new" operator to initialize the reference
        //  counter so we call the destructor explicitly now.
        _u.lmsg.content->refcnt.~atomic_counter_t ();

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

uint32_t zmq::msg_t::get_routing_id () const
{
    return _u.base.routing_id;
}

int zmq::msg_t::set_routing_id (uint32_t routing_id_)
{
    if (routing_id_) {
        _u.base.routing_id = routing_id_;
        return 0;
    }
    errno = EINVAL;
    return -1;
}

int zmq::msg_t::reset_routing_id ()
{
    _u.base.routing_id = 0;
    return 0;
}

const char *zmq::msg_t::group () const
{
    if (_u.base.group.type == group_type_long)
        return _u.base.group.lgroup.content->group;
    return _u.base.group.sgroup.group;
}

int zmq::msg_t::set_group (group_: *const c_char)
{
    size_t length = strnlen (group_, ZMQ_GROUP_MAX_LENGTH);

    return set_group (group_, length);
}

int zmq::msg_t::set_group (group_: *const c_char, length_: usize)
{
    if (length_ > ZMQ_GROUP_MAX_LENGTH) {
        errno = EINVAL;
        return -1;
    }

    if (length_ > 14) {
        _u.base.group.lgroup.type = group_type_long;
        _u.base.group.lgroup.content =
          (long_group_t *) malloc (sizeof (long_group_t));
        assert (_u.base.group.lgroup.content);
        new (&_u.base.group.lgroup.content->refcnt) zmq::atomic_counter_t ();
        _u.base.group.lgroup.content->refcnt.set (1);
        strncpy (_u.base.group.lgroup.content->group, group_, length_);
        _u.base.group.lgroup.content->group[length_] = '\0';
    } else {
        strncpy (_u.base.group.sgroup.group, group_, length_);
        _u.base.group.sgroup.group[length_] = '\0';
    }

    return 0;
}

zmq::atomic_counter_t *zmq::msg_t::refcnt ()
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
