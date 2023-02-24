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
// #include "macros.hpp"
// #include "req.hpp"
// #include "err.hpp"
// #include "msg.hpp"
// #include "wire.hpp"
// #include "random.hpp"
// #include "likely.hpp"
pub struct req_t ZMQ_FINAL : public dealer_t
{
// public:
    req_t (ZmqContext *parent_, u32 tid_, sid_: i32);
    ~req_t ();

    //  Overrides of functions from ZmqSocketBase.
    int xsend (msg: &mut ZmqMessage);
    int xrecv (msg: &mut ZmqMessage);
    bool xhas_in ();
    bool xhas_out ();
    int xsetsockopt (option_: i32, const optval_: *mut c_void, optvallen_: usize);
    void xpipe_terminated (pipe_: &mut pipe_t);

  protected:
    //  Receive only from the pipe the request was sent to, discarding
    //  frames from other pipes.
    int recv_reply_pipe (msg: &mut ZmqMessage);

  // private:
    //  If true, request was already sent and reply wasn't received yet or
    //  was received partially.
    _receiving_reply: bool

    //  If true, we are starting to send/recv a message. The first part
    //  of the message must be empty message part (backtrace stack bottom).
    _message_begins: bool

    //  The pipe the request was sent to and where the reply is expected.
    pipe_t *_reply_pipe;

    //  Whether request id frames shall be sent and expected.
    _request_id_frames_enabled: bool

    //  The current request id. It is incremented every time before a new
    //  request is sent.
    u32 _request_id;

    //  If false, send() will reset its internal state and terminate the
    //  reply_pipe's connection instead of failing if a previous request is
    //  still pending.
    _strict: bool

    ZMQ_NON_COPYABLE_NOR_MOVABLE (req_t)
};
pub struct req_session_t ZMQ_FINAL : public session_base_t
{
// public:
    req_session_t (io_thread_t *io_thread_,
                   connect_: bool,
                   socket_: *mut ZmqSocketBase,
                   const ZmqOptions &options_,
                   Address *addr_);
    ~req_session_t ();

    //  Overrides of the functions from session_base_t.
    int push_msg (msg: &mut ZmqMessage);
    void reset ();

  // private:
    enum
    {
        bottom,
        request_id,
        body
    } _state;

    ZMQ_NON_COPYABLE_NOR_MOVABLE (req_session_t)
};

req_t::req_t (class ZmqContext *parent_, u32 tid_, sid_: i32) :
    dealer_t (parent_, tid_, sid_),
    _receiving_reply (false),
    _message_begins (true),
    _reply_pipe (null_mut()),
    _request_id_frames_enabled (false),
    _request_id (generate_random ()),
    _strict (true)
{
    options.type = ZMQ_REQ;
}

req_t::~req_t ()
{
}

int req_t::xsend (msg: &mut ZmqMessage)
{
    //  If we've sent a request and we still haven't got the reply,
    //  we can't send another request unless the strict option is disabled.
    if (_receiving_reply) {
        if (_strict) {
            errno = EFSM;
            return -1;
        }

        _receiving_reply = false;
        _message_begins = true;
    }

    //  First part of the request is the request routing id.
    if (_message_begins) {
        _reply_pipe = null_mut();

        if (_request_id_frames_enabled) {
            _request_id++;

            ZmqMessage id;
            int rc = id.init_size (mem::size_of::<u32>());
            memcpy (id.data (), &_request_id, mem::size_of::<u32>());
            errno_assert (rc == 0);
            id.set_flags (ZmqMessage::more);

            rc = dealer_t::sendpipe (&id, &_reply_pipe);
            if (rc != 0) {
                return -1;
            }
        }

        ZmqMessage bottom;
        int rc = bottom.init ();
        errno_assert (rc == 0);
        bottom.set_flags (ZmqMessage::more);

        rc = dealer_t::sendpipe (&bottom, &_reply_pipe);
        if (rc != 0)
            return -1;
        zmq_assert (_reply_pipe);

        _message_begins = false;

        // Eat all currently available messages before the request is fully
        // sent. This is done to avoid:
        //   REQ sends request to A, A replies, B replies too.
        //   A's reply was first and matches, that is used.
        //   An hour later REQ sends a request to B. B's old reply is used.
        ZmqMessage drop;
        while (true) {
            rc = drop.init ();
            errno_assert (rc == 0);
            rc = dealer_t::xrecv (&drop);
            if (rc != 0)
                break;
            drop.close ();
        }
    }

    bool more = (msg.flags () & ZmqMessage::more) != 0;

    int rc = dealer_t::xsend (msg);
    if (rc != 0)
        return rc;

    //  If the request was fully sent, flip the FSM into reply-receiving state.
    if (!more) {
        _receiving_reply = true;
        _message_begins = true;
    }

    return 0;
}

int req_t::xrecv (msg: &mut ZmqMessage)
{
    //  If request wasn't send, we can't wait for reply.
    if (!_receiving_reply) {
        errno = EFSM;
        return -1;
    }

    //  Skip messages until one with the right first frames is found.
    while (_message_begins) {
        //  If enabled, the first frame must have the correct request_id.
        if (_request_id_frames_enabled) {
            int rc = recv_reply_pipe (msg);
            if (rc != 0)
                return rc;

            if (unlikely (!(msg.flags () & ZmqMessage::more)
                          || msg.size () != mem::size_of::<_request_id>()
                          || *static_cast<u32 *> (msg.data ())
                               != _request_id)) {
                //  Skip the remaining frames and try the next message
                while (msg.flags () & ZmqMessage::more) {
                    rc = recv_reply_pipe (msg);
                    errno_assert (rc == 0);
                }
                continue;
            }
        }

        //  The next frame must be 0.
        // TODO: Failing this check should also close the connection with the peer!
        int rc = recv_reply_pipe (msg);
        if (rc != 0)
            return rc;

        if (unlikely (!(msg.flags () & ZmqMessage::more) || msg.size () != 0)) {
            //  Skip the remaining frames and try the next message
            while (msg.flags () & ZmqMessage::more) {
                rc = recv_reply_pipe (msg);
                errno_assert (rc == 0);
            }
            continue;
        }

        _message_begins = false;
    }

    let rc: i32 = recv_reply_pipe (msg);
    if (rc != 0)
        return rc;

    //  If the reply is fully received, flip the FSM into request-sending state.
    if (!(msg.flags () & ZmqMessage::more)) {
        _receiving_reply = false;
        _message_begins = true;
    }

    return 0;
}

bool req_t::xhas_in ()
{
    //  TODO: Duplicates should be removed here.

    if (!_receiving_reply)
        return false;

    return dealer_t::xhas_in ();
}

bool req_t::xhas_out ()
{
    if (_receiving_reply && _strict)
        return false;

    return dealer_t::xhas_out ();
}

int req_t::xsetsockopt (option_: i32,
                             const optval_: *mut c_void,
                             optvallen_: usize)
{
    const bool is_int = (optvallen_ == mem::size_of::<int>());
    int value = 0;
    if (is_int)
        memcpy (&value, optval_, mem::size_of::<int>());

    switch (option_) {
        case ZMQ_REQ_CORRELATE:
            if (is_int && value >= 0) {
                _request_id_frames_enabled = (value != 0);
                return 0;
            }
            break;

        case ZMQ_REQ_RELAXED:
            if (is_int && value >= 0) {
                _strict = (value == 0);
                return 0;
            }
            break;

        default:
            break;
    }

    return dealer_t::xsetsockopt (option_, optval_, optvallen_);
}

void req_t::xpipe_terminated (pipe_: &mut pipe_t)
{
    if (_reply_pipe == pipe_)
        _reply_pipe = null_mut();
    dealer_t::xpipe_terminated (pipe_);
}

int req_t::recv_reply_pipe (msg: &mut ZmqMessage)
{
    while (true) {
        pipe_t *pipe = null_mut();
        let rc: i32 = dealer_t::recvpipe (msg, &pipe);
        if (rc != 0)
            return rc;
        if (!_reply_pipe || pipe == _reply_pipe)
            return 0;
    }
}

req_session_t::req_session_t (io_thread_t *io_thread_,
                                   connect_: bool,
                                   ZmqSocketBase *socket_,
                                   const ZmqOptions &options_,
                                   Address *addr_) :
    session_base_t (io_thread_, connect_, socket_, options_, addr_),
    _state (bottom)
{
}

req_session_t::~req_session_t ()
{
}

int req_session_t::push_msg (msg: &mut ZmqMessage)
{
    //  Ignore commands, they are processed by the engine and should not
    //  affect the state machine.
    if (unlikely (msg.flags () & ZmqMessage::command))
        return 0;

    switch (_state) {
        case bottom:
            if (msg.flags () == ZmqMessage::more) {
                //  In case option ZMQ_CORRELATE is on, allow request_id to be
                //  transferred as first frame (would be too cumbersome to check
                //  whether the option is actually on or not).
                if (msg.size () == mem::size_of::<u32>()) {
                    _state = request_id;
                    return session_base_t::push_msg (msg);
                }
                if (msg.size () == 0) {
                    _state = body;
                    return session_base_t::push_msg (msg);
                }
            }
            break;
        case request_id:
            if (msg.flags () == ZmqMessage::more && msg.size () == 0) {
                _state = body;
                return session_base_t::push_msg (msg);
            }
            break;
        case body:
            if (msg.flags () == ZmqMessage::more)
                return session_base_t::push_msg (msg);
            if (msg.flags () == 0) {
                _state = bottom;
                return session_base_t::push_msg (msg);
            }
            break;
    }
    errno = EFAULT;
    return -1;
}

void req_session_t::reset ()
{
    session_base_t::reset ();
    _state = bottom;
}
