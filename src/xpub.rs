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

// #include "xpub.hpp"
// #include "pipe.hpp"
// #include "err.hpp"
// #include "msg.hpp"
// #include "macros.hpp"
// #include "generic_mtrie_impl.hpp"
pub struct xpub_t : public ZmqSocketBase
{
// public:
    xpub_t (ZmqContext *parent_, u32 tid_, sid_: i32);
    ~xpub_t () ZMQ_OVERRIDE;

    //  Implementations of virtual functions from ZmqSocketBase.
    void xattach_pipe (pipe_t *pipe_,
                       bool subscribe_to_all_ = false,
                       bool locally_initiated_ = false) ZMQ_OVERRIDE;
    int xsend (ZmqMessage *msg) ZMQ_FINAL;
    bool xhas_out () ZMQ_FINAL;
    int xrecv (ZmqMessage *msg) ZMQ_OVERRIDE;
    bool xhas_in () ZMQ_OVERRIDE;
    void xread_activated (pipe_t *pipe_) ZMQ_FINAL;
    void xwrite_activated (pipe_t *pipe_) ZMQ_FINAL;
    int
    xsetsockopt (option_: i32, const optval_: *mut c_void, optvallen_: usize) ZMQ_FINAL;
    int xgetsockopt (option_: i32, optval_: *mut c_void, optvallen_: *mut usize) ZMQ_FINAL;
    void xpipe_terminated (pipe_t *pipe_) ZMQ_FINAL;

  // private:
    //  Function to be applied to the trie to send all the subscriptions
    //  upstream.
    static void send_unsubscription (mtrie_t::prefix_t data,
                                     size: usize,
                                     xpub_t *self_);

    //  Function to be applied to each matching pipes.
    static void mark_as_matching (pipe_t *pipe_, xpub_t *self_);

    //  List of all subscriptions mapped to corresponding pipes.
    mtrie_t _subscriptions;

    //  List of manual subscriptions mapped to corresponding pipes.
    mtrie_t _manual_subscriptions;

    //  Distributor of messages holding the list of outbound pipes.
    dist_t _dist;

    // If true, send all subscription messages upstream, not just
    // unique ones
    bool _verbose_subs;

    // If true, send all unsubscription messages upstream, not just
    // unique ones
    bool _verbose_unsubs;

    //  True if we are in the middle of sending a multi-part message.
    bool _more_send;

    //  True if we are in the middle of receiving a multi-part message.
    bool _more_recv;

    //  If true, subscribe and cancel messages are processed for the rest
    //  of multipart message.
    bool _process_subscribe;

    //  This option is enabled with ZMQ_ONLY_FIRST_SUBSCRIBE.
    //  If true, messages following subscribe/unsubscribe in a multipart
    //  message are treated as user data regardless of the first byte.
    bool _only_first_subscribe;

    //  Drop messages if HWM reached, otherwise return with EAGAIN
    bool _lossy;

    //  Subscriptions will not bed added automatically, only after calling set option with ZMQ_SUBSCRIBE or ZMQ_UNSUBSCRIBE
    bool _manual;

    //  Send message to the last pipe, only used if xpub is on manual and after calling set option with ZMQ_SUBSCRIBE
    bool _send_last_pipe;

    //  Function to be applied to match the last pipe.
    static void mark_last_pipe_as_matching (pipe_t *pipe_, xpub_t *self_);

    //  Last pipe that sent subscription message, only used if xpub is on manual
    pipe_t *_last_pipe;

    // Pipes that sent subscriptions messages that have not yet been processed, only used if xpub is on manual
    std::deque<pipe_t *> _pending_pipes;

    //  Welcome message to send to pipe when attached
    ZmqMessage _welcome_msg;

    //  List of pending (un)subscriptions, ie. those that were already
    //  applied to the trie, but not yet received by the user.
    std::deque<Blob> _pending_data;
    std::deque<ZmqMetadata *> _pending_metadata;
    std::deque<unsigned char> _pending_flags;

    ZMQ_NON_COPYABLE_NOR_MOVABLE (xpub_t)
};

xpub_t::xpub_t (class ZmqContext *parent_, u32 tid_, sid_: i32) :
    ZmqSocketBase (parent_, tid_, sid_),
    _verbose_subs (false),
    _verbose_unsubs (false),
    _more_send (false),
    _more_recv (false),
    _process_subscribe (false),
    _only_first_subscribe (false),
    _lossy (true),
    _manual (false),
    _send_last_pipe (false),
    _pending_pipes (),
    _welcome_msg ()
{
    _last_pipe = NULL;
    options.type = ZMQ_XPUB;
    _welcome_msg.init ();
}

xpub_t::~xpub_t ()
{
    _welcome_msg.close ();
    for (std::deque<ZmqMetadata *>::iterator it = _pending_metadata.begin (),
                                            end = _pending_metadata.end ();
         it != end; ++it)
        if (*it && (*it)->drop_ref ())
            LIBZMQ_DELETE (*it);
}

void xpub_t::xattach_pipe (pipe_t *pipe_,
                                bool subscribe_to_all_,
                                bool locally_initiated_)
{
    LIBZMQ_UNUSED (locally_initiated_);

    zmq_assert (pipe_);
    _dist.attach (pipe_);

    //  If subscribe_to_all_ is specified, the caller would like to subscribe
    //  to all data on this pipe, implicitly.
    if (subscribe_to_all_)
        _subscriptions.add (NULL, 0, pipe_);

    // if welcome message exists, send a copy of it
    if (_welcome_msg.size () > 0) {
        ZmqMessage copy;
        copy.init ();
        let rc: i32 = copy.copy (_welcome_msg);
        errno_assert (rc == 0);
        const bool ok = pipe_->write (&copy);
        zmq_assert (ok);
        pipe_->flush ();
    }

    //  The pipe is active when attached. Let's read the subscriptions from
    //  it, if any.
    xread_activated (pipe_);
}

void xpub_t::xread_activated (pipe_t *pipe_)
{
    //  There are some subscriptions waiting. Let's process them.
    ZmqMessage msg;
    while (pipe_->read (&msg)) {
        ZmqMetadata *metadata = msg.metadata ();
        unsigned char *msg_data = static_cast<unsigned char *> (msg.data ()),
                      *data = NULL;
        size_t size = 0;
        bool subscribe = false;
        bool is_subscribe_or_cancel = false;
        bool notify = false;

        const bool first_part = !_more_recv;
        _more_recv = (msg.flags () & ZmqMessage::more) != 0;

        if (first_part || _process_subscribe) {
            //  Apply the subscription to the trie
            if (msg.is_subscribe () || msg.is_cancel ()) {
                data = static_cast<unsigned char *> (msg.command_body ());
                size = msg.command_body_size ();
                subscribe = msg.is_subscribe ();
                is_subscribe_or_cancel = true;
            } else if (msg.size () > 0 && (*msg_data == 0 || *msg_data == 1)) {
                data = msg_data + 1;
                size = msg.size () - 1;
                subscribe = *msg_data == 1;
                is_subscribe_or_cancel = true;
            }
        }

        if (first_part)
            _process_subscribe =
              !_only_first_subscribe || is_subscribe_or_cancel;

        if (is_subscribe_or_cancel) {
            if (_manual) {
                // Store manual subscription to use on termination
                if (!subscribe)
                    _manual_subscriptions.rm (data, size, pipe_);
                else
                    _manual_subscriptions.add (data, size, pipe_);

                _pending_pipes.push_back (pipe_);
            } else {
                if (!subscribe) {
                    const mtrie_t::rm_result rm_result =
                      _subscriptions.rm (data, size, pipe_);
                    //  TODO reconsider what to do if rm_result == mtrie_t::not_found
                    notify =
                      rm_result != mtrie_t::values_remain || _verbose_unsubs;
                } else {
                    const bool first_added =
                      _subscriptions.add (data, size, pipe_);
                    notify = first_added || _verbose_subs;
                }
            }

            //  If the request was a new subscription, or the subscription
            //  was removed, or verbose mode or manual mode are enabled, store it
            //  so that it can be passed to the user on next recv call.
            if (_manual || (options.type == ZMQ_XPUB && notify)) {
                //  ZMTP 3.1 hack: we need to support sub/cancel commands, but
                //  we can't give them back to userspace as it would be an API
                //  breakage since the payload of the message is completely
                //  different. Manually craft an old-style message instead.
                //  Although with other transports it would be possible to simply
                //  reuse the same buffer and prefix a 0/1 byte to the topic, with
                //  inproc the subscribe/cancel command string is not present in
                //  the message, so this optimization is not possible.
                //  The pushback makes a copy of the data array anyway, so the
                //  number of buffer copies does not change.
                Blob notification (size + 1);
                if (subscribe)
                    *notification.data () = 1;
                else
                    *notification.data () = 0;
                memcpy (notification.data () + 1, data, size);

                _pending_data.push_back (ZMQ_MOVE (notification));
                if (metadata)
                    metadata->add_ref ();
                _pending_metadata.push_back (metadata);
                _pending_flags.push_back (0);
            }
        } else if (options.type != ZMQ_PUB) {
            //  Process user message coming upstream from xsub socket,
            //  but not if the type is PUB, which never processes user
            //  messages
            _pending_data.push_back (Blob (msg_data, msg.size ()));
            if (metadata)
                metadata->add_ref ();
            _pending_metadata.push_back (metadata);
            _pending_flags.push_back (msg.flags ());
        }

        msg.close ();
    }
}

void xpub_t::xwrite_activated (pipe_t *pipe_)
{
    _dist.activated (pipe_);
}

int xpub_t::xsetsockopt (option_: i32,
                              const optval_: *mut c_void,
                              optvallen_: usize)
{
    if (option_ == ZMQ_XPUB_VERBOSE || option_ == ZMQ_XPUB_VERBOSER
        || option_ == ZMQ_XPUB_MANUAL_LAST_VALUE || option_ == ZMQ_XPUB_NODROP
        || option_ == ZMQ_XPUB_MANUAL || option_ == ZMQ_ONLY_FIRST_SUBSCRIBE) {
        if (optvallen_ != mem::size_of::<int>()
            || *static_cast<const int *> (optval_) < 0) {
            errno = EINVAL;
            return -1;
        }
        if (option_ == ZMQ_XPUB_VERBOSE) {
            _verbose_subs = (*static_cast<const int *> (optval_) != 0);
            _verbose_unsubs = false;
        } else if (option_ == ZMQ_XPUB_VERBOSER) {
            _verbose_subs = (*static_cast<const int *> (optval_) != 0);
            _verbose_unsubs = _verbose_subs;
        } else if (option_ == ZMQ_XPUB_MANUAL_LAST_VALUE) {
            _manual = (*static_cast<const int *> (optval_) != 0);
            _send_last_pipe = _manual;
        } else if (option_ == ZMQ_XPUB_NODROP)
            _lossy = (*static_cast<const int *> (optval_) == 0);
        else if (option_ == ZMQ_XPUB_MANUAL)
            _manual = (*static_cast<const int *> (optval_) != 0);
        else if (option_ == ZMQ_ONLY_FIRST_SUBSCRIBE)
            _only_first_subscribe = (*static_cast<const int *> (optval_) != 0);
    } else if (option_ == ZMQ_SUBSCRIBE && _manual) {
        if (_last_pipe != NULL)
            _subscriptions.add ((unsigned char *) optval_, optvallen_,
                                _last_pipe);
    } else if (option_ == ZMQ_UNSUBSCRIBE && _manual) {
        if (_last_pipe != NULL)
            _subscriptions.rm ((unsigned char *) optval_, optvallen_,
                               _last_pipe);
    } else if (option_ == ZMQ_XPUB_WELCOME_MSG) {
        _welcome_msg.close ();

        if (optvallen_ > 0) {
            let rc: i32 = _welcome_msg.init_size (optvallen_);
            errno_assert (rc == 0);

            unsigned char *data =
              static_cast<unsigned char *> (_welcome_msg.data ());
            memcpy (data, optval_, optvallen_);
        } else
            _welcome_msg.init ();
    } else {
        errno = EINVAL;
        return -1;
    }
    return 0;
}

int xpub_t::xgetsockopt (option_: i32, optval_: *mut c_void, optvallen_: *mut usize)
{
    if (option_ == ZMQ_TOPICS_COUNT) {
        // make sure to use a multi-thread safe function to avoid race conditions with I/O threads
        // where subscriptions are processed:
        return do_getsockopt<int> (optval_, optvallen_,
                                   (int) _subscriptions.num_prefixes ());
    }

    // room for future options here

    errno = EINVAL;
    return -1;
}

static void stub (mtrie_t::prefix_t data, size: usize, arg_: *mut c_void)
{
    LIBZMQ_UNUSED (data);
    LIBZMQ_UNUSED (size);
    LIBZMQ_UNUSED (arg_);
}

void xpub_t::xpipe_terminated (pipe_t *pipe_)
{
    if (_manual) {
        //  Remove the pipe from the trie and send corresponding manual
        //  unsubscriptions upstream.
        _manual_subscriptions.rm (pipe_, send_unsubscription, this, false);
        //  Remove pipe without actually sending the message as it was taken
        //  care of by the manual call above. subscriptions is the real mtrie,
        //  so the pipe must be removed from there or it will be left over.
        _subscriptions.rm (pipe_, stub, static_cast<void *> (NULL), false);

        // In case the pipe is currently set as last we must clear it to prevent
        // subscriptions from being re-added.
        if (pipe_ == _last_pipe) {
            _last_pipe = NULL;
        }
    } else {
        //  Remove the pipe from the trie. If there are topics that nobody
        //  is interested in anymore, send corresponding unsubscriptions
        //  upstream.
        _subscriptions.rm (pipe_, send_unsubscription, this, !_verbose_unsubs);
    }

    _dist.pipe_terminated (pipe_);
}

void xpub_t::mark_as_matching (pipe_t *pipe_, xpub_t *self_)
{
    self_->_dist.match (pipe_);
}

void xpub_t::mark_last_pipe_as_matching (pipe_t *pipe_, xpub_t *self_)
{
    if (self_->_last_pipe == pipe_)
        self_->_dist.match (pipe_);
}

int xpub_t::xsend (ZmqMessage *msg)
{
    const bool msg_more = (msg->flags () & ZmqMessage::more) != 0;

    //  For the first part of multi-part message, find the matching pipes.
    if (!_more_send) {
        // Ensure nothing from previous failed attempt to send is left matched
        _dist.unmatch ();

        if (unlikely (_manual && _last_pipe && _send_last_pipe)) {
            _subscriptions.match (static_cast<unsigned char *> (msg->data ()),
                                  msg->size (), mark_last_pipe_as_matching,
                                  this);
            _last_pipe = NULL;
        } else
            _subscriptions.match (static_cast<unsigned char *> (msg->data ()),
                                  msg->size (), mark_as_matching, this);
        // If inverted matching is used, reverse the selection now
        if (options.invert_matching) {
            _dist.reverse_match ();
        }
    }

    int rc = -1; //  Assume we fail
    if (_lossy || _dist.check_hwm ()) {
        if (_dist.send_to_matching (msg) == 0) {
            //  If we are at the end of multi-part message we can mark
            //  all the pipes as non-matching.
            if (!msg_more)
                _dist.unmatch ();
            _more_send = msg_more;
            rc = 0; //  Yay, sent successfully
        }
    } else
        errno = EAGAIN;
    return rc;
}

bool xpub_t::xhas_out ()
{
    return _dist.has_out ();
}

int xpub_t::xrecv (ZmqMessage *msg)
{
    //  If there is at least one
    if (_pending_data.empty ()) {
        errno = EAGAIN;
        return -1;
    }

    // User is reading a message, set last_pipe and remove it from the deque
    if (_manual && !_pending_pipes.empty ()) {
        _last_pipe = _pending_pipes.front ();
        _pending_pipes.pop_front ();

        // If the distributor doesn't know about this pipe it must have already
        // been terminated and thus we can't allow manual subscriptions.
        if (_last_pipe != NULL && !_dist.has_pipe (_last_pipe)) {
            _last_pipe = NULL;
        }
    }

    int rc = msg->close ();
    errno_assert (rc == 0);
    rc = msg->init_size (_pending_data.front ().size ());
    errno_assert (rc == 0);
    memcpy (msg->data (), _pending_data.front ().data (),
            _pending_data.front ().size ());

    // set metadata only if there is some
    if (ZmqMetadata *metadata = _pending_metadata.front ()) {
        msg->set_metadata (metadata);
        // Remove ref corresponding to vector placement
        metadata->drop_ref ();
    }

    msg->set_flags (_pending_flags.front ());
    _pending_data.pop_front ();
    _pending_metadata.pop_front ();
    _pending_flags.pop_front ();
    return 0;
}

bool xpub_t::xhas_in ()
{
    return !_pending_data.is_empty();
}

void xpub_t::send_unsubscription (mtrie_t::prefix_t data,
                                       size: usize,
                                       xpub_t *self_)
{
    if (self_->options.type != ZMQ_PUB) {
        //  Place the unsubscription to the queue of pending (un)subscriptions
        //  to be retrieved by the user later on.
        Blob unsub (size + 1);
        *unsub.data () = 0;
        if (size > 0)
            memcpy (unsub.data () + 1, data, size);
        self_->_pending_data.ZMQ_PUSH_OR_EMPLACE_BACK (ZMQ_MOVE (unsub));
        self_->_pending_metadata.push_back (NULL);
        self_->_pending_flags.push_back (0);

        if (self_->_manual) {
            self_->_last_pipe = NULL;
            self_->_pending_pipes.push_back (NULL);
        }
    }
}
