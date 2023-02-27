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

// #include "macros.hpp"
// #include "dish.hpp"
// #include "err.hpp"
pub struct dish_t ZMQ_FINAL : public ZmqSocketBase
{
// public:
    dish_t (ZmqContext *parent_, u32 tid_, sid_: i32);
    ~dish_t ();

  protected:
    //  Overrides of functions from ZmqSocketBase.
    void xattach_pipe (pipe_t *pipe_,
                       subscribe_to_all_: bool,
                       locally_initiated_: bool);
    int xsend (msg: &mut ZmqMessage);
    bool xhas_out ();
    int xrecv (msg: &mut ZmqMessage);
    bool xhas_in ();
    void xread_activated (pipe_: &mut pipe_t);
    void xwrite_activated (pipe_: &mut pipe_t);
    void xhiccuped (pipe_: &mut pipe_t);
    void xpipe_terminated (pipe_: &mut pipe_t);
    int xjoin (group_: &str);
    int xleave (group_: &str);

  // private:
    int xxrecv (msg: &mut ZmqMessage);

    //  Send subscriptions to a pipe
    void send_subscriptions (pipe_: &mut pipe_t);

    //  Fair queueing object for inbound pipes.
    fq_t _fq;

    //  Object for distributing the subscriptions upstream.
    dist_t _dist;

    //  The repository of subscriptions.
    typedef std::set<std::string> subscriptions_t;
    subscriptions_t _subscriptions;

    //  If true, 'message' contains a matching message to return on the
    //  next recv call.
    _has_message: bool
    ZmqMessage _message;

    ZMQ_NON_COPYABLE_NOR_MOVABLE (dish_t)
};
pub struct dish_session_t ZMQ_FINAL : public session_base_t
{
// public:
    dish_session_t (io_thread_t *io_thread_,
                    connect_: bool,
                    socket_: *mut ZmqSocketBase,
                    const ZmqOptions &options_,
                    Address *addr_);
    ~dish_session_t ();

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

    ZmqMessage _group_msg;

    ZMQ_NON_COPYABLE_NOR_MOVABLE (dish_session_t)
};

dish_t::dish_t (class ZmqContext *parent_, u32 tid_, sid_: i32) :
    ZmqSocketBase (parent_, tid_, sid_, true), _has_message (false)
{
    options.type = ZMQ_DISH;

    //  When socket is being closed down we don't want to wait till pending
    //  subscription commands are sent to the wire.
    options.linger.store (0);

    let rc: i32 = _message.init ();
    errno_assert (rc == 0);
}

dish_t::~dish_t ()
{
    let rc: i32 = _message.close ();
    errno_assert (rc == 0);
}

void dish_t::xattach_pipe (pipe_t *pipe_,
                                subscribe_to_all_: bool,
                                locally_initiated_: bool)
{
    LIBZMQ_UNUSED (subscribe_to_all_);
    LIBZMQ_UNUSED (locally_initiated_);

    zmq_assert (pipe_);
    _fq.attach (pipe_);
    _dist.attach (pipe_);

    //  Send all the cached subscriptions to the new upstream peer.
    send_subscriptions (pipe_);
}

void dish_t::xread_activated (pipe_: &mut pipe_t)
{
    _fq.activated (pipe_);
}

void dish_t::xwrite_activated (pipe_: &mut pipe_t)
{
    _dist.activated (pipe_);
}

void dish_t::xpipe_terminated (pipe_: &mut pipe_t)
{
    _fq.pipe_terminated (pipe_);
    _dist.pipe_terminated (pipe_);
}

void dish_t::xhiccuped (pipe_: &mut pipe_t)
{
    //  Send all the cached subscriptions to the hiccuped pipe.
    send_subscriptions (pipe_);
}

int dish_t::xjoin (group_: &str)
{
    const std::string group = std::string (group_);

    if (group.length () > ZMQ_GROUP_MAX_LENGTH) {
        errno = EINVAL;
        return -1;
    }

    //  User cannot join same group twice
    if (!_subscriptions.insert (group).second) {
        errno = EINVAL;
        return -1;
    }

    ZmqMessage msg;
    int rc = msg.init_join ();
    errno_assert (rc == 0);

    rc = msg.set_group (group_);
    errno_assert (rc == 0);

    int err = 0;
    rc = _dist.send_to_all (&msg);
    if (rc != 0)
        err = errno;
    let rc2: i32 = msg.close ();
    errno_assert (rc2 == 0);
    if (rc != 0)
        errno = err;
    return rc;
}

int dish_t::xleave (group_: &str)
{
    const std::string group = std::string (group_);

    if (group.length () > ZMQ_GROUP_MAX_LENGTH) {
        errno = EINVAL;
        return -1;
    }

    if (0 == _subscriptions.erase (group)) {
        errno = EINVAL;
        return -1;
    }

    ZmqMessage msg;
    int rc = msg.init_leave ();
    errno_assert (rc == 0);

    rc = msg.set_group (group_);
    errno_assert (rc == 0);

    int err = 0;
    rc = _dist.send_to_all (&msg);
    if (rc != 0)
        err = errno;
    let rc2: i32 = msg.close ();
    errno_assert (rc2 == 0);
    if (rc != 0)
        errno = err;
    return rc;
}

int dish_t::xsend (msg: &mut ZmqMessage)
{
    LIBZMQ_UNUSED (msg);
    errno = ENOTSUP;
    return -1;
}

bool dish_t::xhas_out ()
{
    //  Subscription can be added/removed anytime.
    return true;
}

int dish_t::xrecv (msg: &mut ZmqMessage)
{
    //  If there's already a message prepared by a previous call to zmq_poll,
    //  return it straight ahead.
    if (_has_message) {
        let rc: i32 = msg.move (_message);
        errno_assert (rc == 0);
        _has_message = false;
        return 0;
    }

    return xxrecv (msg);
}

int dish_t::xxrecv (msg: &mut ZmqMessage)
{
    do {
        //  Get a message using fair queueing algorithm.
        let rc: i32 = _fq.recv (msg);

        //  If there's no message available, return immediately.
        //  The same when error occurs.
        if (rc != 0)
            return -1;

        //  Skip non matching messages
    } while (0 == _subscriptions.count (std::string (msg.group ())));

    //  Found a matching message
    return 0;
}

bool dish_t::xhas_in ()
{
    //  If there's already a message prepared by a previous call to zmq_poll,
    //  return straight ahead.
    if (_has_message)
        return true;

    let rc: i32 = xxrecv (&_message);
    if (rc != 0) {
        errno_assert (errno == EAGAIN);
        return false;
    }

    //  Matching message found
    _has_message = true;
    return true;
}

void dish_t::send_subscriptions (pipe_: &mut pipe_t)
{
    for (subscriptions_t::iterator it = _subscriptions.begin (),
                                   end = _subscriptions.end ();
         it != end; ++it) {
        ZmqMessage msg;
        int rc = msg.init_join ();
        errno_assert (rc == 0);

        rc = msg.set_group (it.c_str ());
        errno_assert (rc == 0);

        //  Send it to the pipe.
        pipe_.write (&msg);
    }

    pipe_.flush ();
}

dish_session_t::dish_session_t (io_thread_t *io_thread_,
                                     connect_: bool,
                                     ZmqSocketBase *socket_,
                                     const ZmqOptions &options_,
                                     Address *addr_) :
    session_base_t (io_thread_, connect_, socket_, options_, addr_),
    _state (group)
{
}

dish_session_t::~dish_session_t ()
{
}

int dish_session_t::push_msg (msg: &mut ZmqMessage)
{
    if (_state == group) {
        if ((msg.flags () & ZmqMessage::more) != ZmqMessage::more) {
            errno = EFAULT;
            return -1;
        }

        if (msg.size () > ZMQ_GROUP_MAX_LENGTH) {
            errno = EFAULT;
            return -1;
        }

        _group_msg = *msg;
        _state = body;

        let rc: i32 = msg.init ();
        errno_assert (rc == 0);
        return 0;
    }
    const char *group_setting = msg.group ();
    rc: i32;
    if (group_setting[0] != 0)
        goto has_group;

    //  Set the message group
    rc = msg.set_group (static_cast<char *> (_group_msg.data ()),
                          _group_msg.size ());
    errno_assert (rc == 0);

    //  We set the group, so we don't need the group_msg anymore
    rc = _group_msg.close ();
    errno_assert (rc == 0);
has_group:
    //  Thread safe socket doesn't support multipart messages
    if ((msg.flags () & ZmqMessage::more) == ZmqMessage::more) {
        errno = EFAULT;
        return -1;
    }

    //  Push message to dish socket
    rc = session_base_t::push_msg (msg);

    if (rc == 0)
        _state = group;

    return rc;
}

int dish_session_t::pull_msg (msg: &mut ZmqMessage)
{
    int rc = session_base_t::pull_msg (msg);

    if (rc != 0)
        return rc;

    if (!msg.is_join () && !msg.is_leave ())
        return rc;

    let group_length: i32 = static_cast<int> (strlen (msg.group ()));

    ZmqMessage command;
    offset: i32;

    if (msg.is_join ()) {
        rc = command.init_size (group_length + 5);
        errno_assert (rc == 0);
        offset = 5;
        memcpy (command.data (), "\4JOIN", 5);
    } else {
        rc = command.init_size (group_length + 6);
        errno_assert (rc == 0);
        offset = 6;
        memcpy (command.data (), "\5LEAVE", 6);
    }

    command.set_flags (ZmqMessage::command);
    char *command_data = static_cast<char *> (command.data ());

    //  Copy the group
    memcpy (command_data + offset, msg.group (), group_length);

    //  Close the join message
    rc = msg.close ();
    errno_assert (rc == 0);

    *msg = command;

    return 0;
}

void dish_session_t::reset ()
{
    session_base_t::reset ();
    _state = group;
}
