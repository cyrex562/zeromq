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

// #include "radio.hpp"
// #include "macros.hpp"
// #include "pipe.hpp"
// #include "err.hpp"
// #include "msg.hpp"
pub struct radio_t ZMQ_FINAL : public ZmqSocketBase
{
// public:
    radio_t (ZmqContext *parent_, u32 tid_, sid_: i32);
    ~radio_t ();

    //  Implementations of virtual functions from ZmqSocketBase.
    void xattach_pipe (pipe_t *pipe_,
                       bool subscribe_to_all_ = false,
                       bool locally_initiated_ = false);
    int xsend (msg: &mut ZmqMessage);
    bool xhas_out ();
    int xrecv (msg: &mut ZmqMessage);
    bool xhas_in ();
    void xread_activated (pipe_: &mut pipe_t);
    void xwrite_activated (pipe_: &mut pipe_t);
    int xsetsockopt (option_: i32, const optval_: *mut c_void, optvallen_: usize);
    void xpipe_terminated (pipe_: &mut pipe_t);

  // private:
    //  List of all subscriptions mapped to corresponding pipes.
    typedef std::multimap<std::string, pipe_t *> subscriptions_t;
    subscriptions_t _subscriptions;

    //  List of udp pipes
    typedef std::vector<pipe_t *> udp_pipes_t;
    udp_pipes_t _udp_pipes;

    //  Distributor of messages holding the list of outbound pipes.
    dist_t _dist;

    //  Drop messages if HWM reached, otherwise return with EAGAIN
    _lossy: bool

    ZMQ_NON_COPYABLE_NOR_MOVABLE (radio_t)
};
pub struct radio_session_t ZMQ_FINAL : public session_base_t
{
// public:
    radio_session_t (io_thread_t *io_thread_,
                     connect_: bool,
                     socket_: *mut ZmqSocketBase,
                     const ZmqOptions &options_,
                     Address *addr_);
    ~radio_session_t ();

    //  Overrides of the functions from session_base_t.
    int push_msg (msg: &mut ZmqMessage);
    int pull_msg (msg: &mut ZmqMessage);
    void reset ();

  // private:
    enum
    {
        group,
        body
    } _state;

    ZmqMessage _pending_msg;

    ZMQ_NON_COPYABLE_NOR_MOVABLE (radio_session_t)
};

radio_t::radio_t (class ZmqContext *parent_, u32 tid_, sid_: i32) :
    ZmqSocketBase (parent_, tid_, sid_, true), _lossy (true)
{
    options.type = ZMQ_RADIO;
}

radio_t::~radio_t ()
{
}

void radio_t::xattach_pipe (pipe_t *pipe_,
                                 subscribe_to_all_: bool,
                                 locally_initiated_: bool)
{
    LIBZMQ_UNUSED (subscribe_to_all_);
    LIBZMQ_UNUSED (locally_initiated_);

    zmq_assert (pipe_);

    //  Don't delay pipe termination as there is no one
    //  to receive the delimiter.
    pipe_.set_nodelay ();

    _dist.attach (pipe_);

    if (subscribe_to_all_)
        _udp_pipes.push_back (pipe_);
    //  The pipe is active when attached. Let's read the subscriptions from
    //  it, if any.
    else
        xread_activated (pipe_);
}

void radio_t::xread_activated (pipe_: &mut pipe_t)
{
    //  There are some subscriptions waiting. Let's process them.
    ZmqMessage msg;
    while (pipe_.read (&msg)) {
        //  Apply the subscription to the trie
        if (msg.is_join () || msg.is_leave ()) {
            std::string group = std::string (msg.group ());

            if (msg.is_join ())
                _subscriptions.ZMQ_MAP_INSERT_OR_EMPLACE (ZMQ_MOVE (group),
                                                          pipe_);
            else {
                std::pair<subscriptions_t::iterator, subscriptions_t::iterator>
                  range = _subscriptions.equal_range (group);

                for (subscriptions_t::iterator it = range.first;
                     it != range.second; ++it) {
                    if (it.second == pipe_) {
                        _subscriptions.erase (it);
                        break;
                    }
                }
            }
        }
        msg.close ();
    }
}

void radio_t::xwrite_activated (pipe_: &mut pipe_t)
{
    _dist.activated (pipe_);
}
int radio_t::xsetsockopt (option_: i32,
                               const optval_: *mut c_void,
                               optvallen_: usize)
{
    if (optvallen_ != mem::size_of::<int>() || *static_cast<const int *> (optval_) < 0) {
        errno = EINVAL;
        return -1;
    }
    if (option_ == ZMQ_XPUB_NODROP)
        _lossy = (*static_cast<const int *> (optval_) == 0);
    else {
        errno = EINVAL;
        return -1;
    }
    return 0;
}

void radio_t::xpipe_terminated (pipe_: &mut pipe_t)
{
    for (subscriptions_t::iterator it = _subscriptions.begin (),
                                   end = _subscriptions.end ();
         it != end;) {
        if (it.second == pipe_) {
#if __cplusplus >= 201103L || (defined _MSC_VER && _MSC_VER >= 1700)
            it = _subscriptions.erase (it);
// #else
            _subscriptions.erase (it++);
// #endif
        } else {
            ++it;
        }
    }

    {
        const udp_pipes_t::iterator end = _udp_pipes.end ();
        const udp_pipes_t::iterator it =
          std::find (_udp_pipes.begin (), end, pipe_);
        if (it != end)
            _udp_pipes.erase (it);
    }

    _dist.pipe_terminated (pipe_);
}

int radio_t::xsend (msg: &mut ZmqMessage)
{
    //  Radio sockets do not allow multipart data (ZMQ_SNDMORE)
    if (msg.flags () & ZmqMessage::more) {
        errno = EINVAL;
        return -1;
    }

    _dist.unmatch ();

    const std::pair<subscriptions_t::iterator, subscriptions_t::iterator>
      range = _subscriptions.equal_range (std::string (msg.group ()));

    for (subscriptions_t::iterator it = range.first; it != range.second; ++it)
        _dist.match (it.second);

    for (udp_pipes_t::iterator it = _udp_pipes.begin (),
                               end = _udp_pipes.end ();
         it != end; ++it)
        _dist.match (*it);

    int rc = -1;
    if (_lossy || _dist.check_hwm ()) {
        if (_dist.send_to_matching (msg) == 0) {
            rc = 0; //  Yay, sent successfully
        }
    } else
        errno = EAGAIN;

    return rc;
}

bool radio_t::xhas_out ()
{
    return _dist.has_out ();
}

int radio_t::xrecv (msg: &mut ZmqMessage)
{
    //  Messages cannot be received from PUB socket.
    LIBZMQ_UNUSED (msg);
    errno = ENOTSUP;
    return -1;
}

bool radio_t::xhas_in ()
{
    return false;
}

radio_session_t::radio_session_t (io_thread_t *io_thread_,
                                       connect_: bool,
                                       ZmqSocketBase *socket_,
                                       const ZmqOptions &options_,
                                       Address *addr_) :
    session_base_t (io_thread_, connect_, socket_, options_, addr_),
    _state (group)
{
}

radio_session_t::~radio_session_t ()
{
}

int radio_session_t::push_msg (msg: &mut ZmqMessage)
{
    if (msg.flags () & ZmqMessage::command) {
        char *command_data = static_cast<char *> (msg.data ());
        const size_t data_size = msg.size ();

        group_length: i32;
        const char *group;

        ZmqMessage join_leave_msg;
        rc: i32;

        //  Set the msg type to either JOIN or LEAVE
        if (data_size >= 5 && memcmp (command_data, "\4JOIN", 5) == 0) {
            group_length = static_cast<int> (data_size) - 5;
            group = command_data + 5;
            rc = join_leave_msg.init_join ();
        } else if (data_size >= 6 && memcmp (command_data, "\5LEAVE", 6) == 0) {
            group_length = static_cast<int> (data_size) - 6;
            group = command_data + 6;
            rc = join_leave_msg.init_leave ();
        }
        //  If it is not a JOIN or LEAVE just push the message
        else
            return session_base_t::push_msg (msg);

        errno_assert (rc == 0);

        //  Set the group
        rc = join_leave_msg.set_group (group, group_length);
        errno_assert (rc == 0);

        //  Close the current command
        rc = msg.close ();
        errno_assert (rc == 0);

        //  Push the join or leave command
        *msg = join_leave_msg;
        return session_base_t::push_msg (msg);
    }
    return session_base_t::push_msg (msg);
}

int radio_session_t::pull_msg (msg: &mut ZmqMessage)
{
    if (_state == group) {
        int rc = session_base_t::pull_msg (&_pending_msg);
        if (rc != 0)
            return rc;

        const char *group = _pending_msg.group ();
        let length: i32 = static_cast<int> (strlen (group));

        //  First frame is the group
        rc = msg.init_size (length);
        errno_assert (rc == 0);
        msg.set_flags (ZmqMessage::more);
        memcpy (msg.data (), group, length);

        //  Next status is the body
        _state = body;
        return 0;
    }
    *msg = _pending_msg;
    _state = group;
    return 0;
}

void radio_session_t::reset ()
{
    session_base_t::reset ();
    _state = group;
}
