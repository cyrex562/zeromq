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
// #include "xsub.hpp"
// #include "err.hpp"
pub struct xsub_t : public ZmqSocketBase
{
// public:
    xsub_t (ZmqContext *parent_, u32 tid_, sid_: i32);
    ~xsub_t () ZMQ_OVERRIDE;

  protected:
    //  Overrides of functions from ZmqSocketBase.
    void xattach_pipe (pipe_t *pipe_,
                       bool subscribe_to_all_,
                       bool locally_initiated_) ZMQ_FINAL;
    int xsetsockopt (option_: i32,
                     const optval_: *mut c_void,
                     optvallen_: usize) ZMQ_OVERRIDE;
    int xgetsockopt (option_: i32, optval_: *mut c_void, optvallen_: *mut usize) ZMQ_FINAL;
    int xsend (ZmqMessage *msg) ZMQ_OVERRIDE;
    bool xhas_out () ZMQ_OVERRIDE;
    int xrecv (ZmqMessage *msg) ZMQ_FINAL;
    bool xhas_in () ZMQ_FINAL;
    void xread_activated (pipe_t *pipe_) ZMQ_FINAL;
    void xwrite_activated (pipe_t *pipe_) ZMQ_FINAL;
    void xhiccuped (pipe_t *pipe_) ZMQ_FINAL;
    void xpipe_terminated (pipe_t *pipe_) ZMQ_FINAL;

  // private:
    //  Check whether the message matches at least one subscription.
    bool match (ZmqMessage *msg);

    //  Function to be applied to the trie to send all the subsciptions
    //  upstream.
    static void
    send_subscription (unsigned char *data, size: usize, arg_: *mut c_void);

    //  Fair queueing object for inbound pipes.
    fq_t _fq;

    //  Object for distributing the subscriptions upstream.
    dist_t _dist;

    //  The repository of subscriptions.
// #ifdef ZMQ_USE_RADIX_TREE
    radix_tree_t _subscriptions;
// #else
    trie_with_size_t _subscriptions;
// #endif

    // If true, send all unsubscription messages upstream, not just
    // unique ones
    bool _verbose_unsubs;

    //  If true, 'message' contains a matching message to return on the
    //  next recv call.
    bool _has_message;
    ZmqMessage _message;

    //  If true, part of a multipart message was already sent, but
    //  there are following parts still waiting.
    bool _more_send;

    //  If true, part of a multipart message was already received, but
    //  there are following parts still waiting.
    bool _more_recv;

    //  If true, subscribe and cancel messages are processed for the rest
    //  of multipart message.
    bool _process_subscribe;

    //  This option is enabled with ZMQ_ONLY_FIRST_SUBSCRIBE.
    //  If true, messages following subscribe/unsubscribe in a multipart
    //  message are treated as user data regardless of the first byte.
    bool _only_first_subscribe;

    ZMQ_NON_COPYABLE_NOR_MOVABLE (xsub_t)
};

xsub_t::xsub_t (class ZmqContext *parent_, u32 tid_, sid_: i32) :
    ZmqSocketBase (parent_, tid_, sid_),
    _verbose_unsubs (false),
    _has_message (false),
    _more_send (false),
    _more_recv (false),
    _process_subscribe (false),
    _only_first_subscribe (false)
{
    options.type = ZMQ_XSUB;

    //  When socket is being closed down we don't want to wait till pending
    //  subscription commands are sent to the wire.
    options.linger.store (0);

    let rc: i32 = _message.init ();
    errno_assert (rc == 0);
}

xsub_t::~xsub_t ()
{
    let rc: i32 = _message.close ();
    errno_assert (rc == 0);
}

void xsub_t::xattach_pipe (pipe_t *pipe_,
                                bool subscribe_to_all_,
                                bool locally_initiated_)
{
    LIBZMQ_UNUSED (subscribe_to_all_);
    LIBZMQ_UNUSED (locally_initiated_);

    zmq_assert (pipe_);
    _fq.attach (pipe_);
    _dist.attach (pipe_);

    //  Send all the cached subscriptions to the new upstream peer.
    _subscriptions.apply (send_subscription, pipe_);
    pipe_->flush ();
}

void xsub_t::xread_activated (pipe_t *pipe_)
{
    _fq.activated (pipe_);
}

void xsub_t::xwrite_activated (pipe_t *pipe_)
{
    _dist.activated (pipe_);
}

void xsub_t::xpipe_terminated (pipe_t *pipe_)
{
    _fq.pipe_terminated (pipe_);
    _dist.pipe_terminated (pipe_);
}

void xsub_t::xhiccuped (pipe_t *pipe_)
{
    //  Send all the cached subscriptions to the hiccuped pipe.
    _subscriptions.apply (send_subscription, pipe_);
    pipe_->flush ();
}

int xsub_t::xsetsockopt (option_: i32,
                              const optval_: *mut c_void,
                              optvallen_: usize)
{
    if (option_ == ZMQ_ONLY_FIRST_SUBSCRIBE) {
        if (optvallen_ != mem::size_of::<int>()
            || *static_cast<const int *> (optval_) < 0) {
            errno = EINVAL;
            return -1;
        }
        _only_first_subscribe = (*static_cast<const int *> (optval_) != 0);
        return 0;
    }
// #ifdef ZMQ_BUILD_DRAFT_API
    else if (option_ == ZMQ_XSUB_VERBOSE_UNSUBSCRIBE) {
        _verbose_unsubs = (*static_cast<const int *> (optval_) != 0);
        return 0;
    }
// #endif
    errno = EINVAL;
    return -1;
}

int xsub_t::xgetsockopt (option_: i32, optval_: *mut c_void, optvallen_: *mut usize)
{
    if (option_ == ZMQ_TOPICS_COUNT) {
        // make sure to use a multi-thread safe function to avoid race conditions with I/O threads
        // where subscriptions are processed:
// #ifdef ZMQ_USE_RADIX_TREE
        u64 num_subscriptions = _subscriptions.size ();
// #else
        u64 num_subscriptions = _subscriptions.num_prefixes ();
// #endif

        return do_getsockopt<int> (optval_, optvallen_,
                                   (int) num_subscriptions);
    }

    // room for future options here

    errno = EINVAL;
    return -1;
}

int xsub_t::xsend (ZmqMessage *msg)
{
    size_t size = msg->size ();
    unsigned char *data = static_cast<unsigned char *> (msg->data ());

    const bool first_part = !_more_send;
    _more_send = (msg->flags () & ZmqMessage::more) != 0;

    if (first_part) {
        _process_subscribe = !_only_first_subscribe;
    } else if (!_process_subscribe) {
        //  User message sent upstream to XPUB socket
        return _dist.send_to_all (msg);
    }

    if (msg->is_subscribe () || (size > 0 && *data == 1)) {
        //  Process subscribe message
        //  This used to filter out duplicate subscriptions,
        //  however this is already done on the XPUB side and
        //  doing it here as well breaks ZMQ_XPUB_VERBOSE
        //  when there are forwarding devices involved.
        if (!msg->is_subscribe ()) {
            data = data + 1;
            size = size - 1;
        }
        _subscriptions.add (data, size);
        _process_subscribe = true;
        return _dist.send_to_all (msg);
    }
    if (msg->is_cancel () || (size > 0 && *data == 0)) {
        //  Process unsubscribe message
        if (!msg->is_cancel ()) {
            data = data + 1;
            size = size - 1;
        }
        _process_subscribe = true;
        const bool rm_result = _subscriptions.rm (data, size);
        if (rm_result || _verbose_unsubs)
            return _dist.send_to_all (msg);
    } else
        //  User message sent upstream to XPUB socket
        return _dist.send_to_all (msg);

    int rc = msg->close ();
    errno_assert (rc == 0);
    rc = msg->init ();
    errno_assert (rc == 0);

    return 0;
}

bool xsub_t::xhas_out ()
{
    //  Subscription can be added/removed anytime.
    return true;
}

int xsub_t::xrecv (ZmqMessage *msg)
{
    //  If there's already a message prepared by a previous call to zmq_poll,
    //  return it straight ahead.
    if (_has_message) {
        let rc: i32 = msg->move (_message);
        errno_assert (rc == 0);
        _has_message = false;
        _more_recv = (msg->flags () & ZmqMessage::more) != 0;
        return 0;
    }

    //  TODO: This can result in infinite loop in the case of continuous
    //  stream of non-matching messages which breaks the non-blocking recv
    //  semantics.
    while (true) {
        //  Get a message using fair queueing algorithm.
        int rc = _fq.recv (msg);

        //  If there's no message available, return immediately.
        //  The same when error occurs.
        if (rc != 0)
            return -1;

        //  Check whether the message matches at least one subscription.
        //  Non-initial parts of the message are passed
        if (_more_recv || !options.filter || match (msg)) {
            _more_recv = (msg->flags () & ZmqMessage::more) != 0;
            return 0;
        }

        //  Message doesn't match. Pop any remaining parts of the message
        //  from the pipe.
        while (msg->flags () & ZmqMessage::more) {
            rc = _fq.recv (msg);
            errno_assert (rc == 0);
        }
    }
}

bool xsub_t::xhas_in ()
{
    //  There are subsequent parts of the partly-read message available.
    if (_more_recv)
        return true;

    //  If there's already a message prepared by a previous call to zmq_poll,
    //  return straight ahead.
    if (_has_message)
        return true;

    //  TODO: This can result in infinite loop in the case of continuous
    //  stream of non-matching messages.
    while (true) {
        //  Get a message using fair queueing algorithm.
        int rc = _fq.recv (&_message);

        //  If there's no message available, return immediately.
        //  The same when error occurs.
        if (rc != 0) {
            errno_assert (errno == EAGAIN);
            return false;
        }

        //  Check whether the message matches at least one subscription.
        if (!options.filter || match (&_message)) {
            _has_message = true;
            return true;
        }

        //  Message doesn't match. Pop any remaining parts of the message
        //  from the pipe.
        while (_message.flags () & ZmqMessage::more) {
            rc = _fq.recv (&_message);
            errno_assert (rc == 0);
        }
    }
}

bool xsub_t::match (ZmqMessage *msg)
{
    const bool matching = _subscriptions.check (
      static_cast<unsigned char *> (msg->data ()), msg->size ());

    return matching ^ options.invert_matching;
}

void xsub_t::send_subscription (unsigned char *data,
                                     size: usize,
                                     arg_: *mut c_void)
{
    pipe_t *pipe = static_cast<pipe_t *> (arg_);

    //  Create the subscription message.
    ZmqMessage msg;
    let rc: i32 = msg.init_subscribe (size, data);
    errno_assert (rc == 0);

    //  Send it to the pipe.
    const bool sent = pipe->write (&msg);
    //  If we reached the SNDHWM, and thus cannot send the subscription, drop
    //  the subscription message instead. This matches the behaviour of
    //  zmq_setsockopt(ZMQ_SUBSCRIBE, ...), which also drops subscriptions
    //  when the SNDHWM is reached.
    if (!sent)
        msg.close ();
}
