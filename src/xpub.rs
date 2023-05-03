/*
    Copyright (c) 2007-2016 Contributors as noted in the AUTHORS file

    This file is part of libzmq, the ZeroMQ core engine in C+= 1.

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

use std::collections::VecDeque;
use std::mem;
use std::ptr::null_mut;

use libc::{EAGAIN, EINVAL};
use trie_rs::TrieBuilder;

use crate::context::ZmqContext;
use crate::defines::{
    ZMQ_ONLY_FIRST_SUBSCRIBE, ZMQ_PUB, ZMQ_TOPICS_COUNT, ZMQ_UNSUBSCRIBE, ZMQ_XPUB,
    ZMQ_XPUB_MANUAL, ZMQ_XPUB_MANUAL_LAST_VALUE, ZMQ_XPUB_NODROP, ZMQ_XPUB_VERBOSE,
    ZMQ_XPUB_VERBOSER, ZMQ_XPUB_WELCOME_MSG,
};
use crate::dist::ZmqDist;
use crate::message::ZMQ_MSG_MORE;
use crate::metadata::ZmqMetadata;
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;
use crate::socket_base::ZmqSocketBase;
use crate::utils::copy_bytes;
use crate::ZmqDecoderInterface::ZmqMessage;

// #include "xpub.hpp"
// #include "pipe.hpp"
// #include "err.hpp"
// #include "msg.hpp"
// #include "macros.hpp"
// #include "generic_mtrie_impl.hpp"
// : public ZmqSocketBase
pub struct XPub {
    pub base: ZmqSocketBase,
    //  List of all subscriptions mapped to corresponding pipes.
    // mtrie_t _subscriptions;
    pub _subscriptions: TrieBuilder<Vec<u8>>,
    //  List of manual subscriptions mapped to corresponding pipes.
    // mtrie_t _manual_subscriptions;
    pub _manual_subscriptions: TrieBuilder<Vec<u8>>,
    //  Distributor of messages holding the list of outbound pipes.
    // ZmqDist _dist;
    pub _dist: ZmqDist,
    // If true, send all subscription messages upstream, not just
    // unique ones
    pub _verbose_subs: bool,
    // If true, send all unsubscription messages upstream, not just
    // unique ones
    pub _verbose_unsubs: bool,
    //  True if we are in the middle of sending a multi-part message.
    pub _more_send: bool,
    //  True if we are in the middle of receiving a multi-part message.
    pub _more_recv: bool,
    //  If true, subscribe and cancel messages are processed for the rest
    //  of multipart message.
    pub _process_subscribe: bool,
    //  This option is enabled with ZMQ_ONLY_FIRST_SUBSCRIBE.
    //  If true, messages following subscribe/unsubscribe in a multipart
    //  message are treated as user data regardless of the first byte.
    pub _only_first_subscribe: bool,
    //  Drop messages if HWM reached, otherwise return with EAGAIN
    pub _lossy: bool,
    //  Subscriptions will not bed added automatically, only after calling set option with ZMQ_SUBSCRIBE or ZMQ_UNSUBSCRIBE
    pub _manual: bool,
    //  Send message to the last pipe, only used if xpub is on manual and after calling set option with ZMQ_SUBSCRIBE
    pub _send_last_pipe: bool,
    //  Function to be applied to match the last pipe.
    //  Last pipe that sent subscription message, only used if xpub is on manual
    // ZmqPipe *_last_pipe;
    pub _last_pipe: Option<ZmqPipe>,
    // Pipes that sent subscriptions messages that have not yet been processed, only used if xpub is on manual
    // std::deque<ZmqPipe *> _pending_pipes;
    pub _pending_pipes: VecDeque<ZmqPipe>,
    //  Welcome message to send to pipe when attached
    // ZmqMessage _welcome_msg;
    pub _welcome_msg: ZmqMessage,
    //  List of pending (un)subscriptions, ie. those that were already
    //  applied to the trie, but not yet received by the user.
    // std::deque<Blob> _pending_data;
    pub _pending_data: VecDeque<Vec<u8>>,
    // std::deque<ZmqMetadata *> _pending_metadata;
    pub _pending_metadata: VecDeque<Vec<u8>>,
    // std::deque<unsigned char> _pending_flags;
    pub _pending_flags: VecDeque<u8>,
    // ZMQ_NON_COPYABLE_NOR_MOVABLE (XPub)
}

impl XPub {
    //
    // XPub (ZmqContext *parent_, tid: u32, sid_: i32);
    pub fn new(parent: &mut ZmqContext, options: &mut ZmqOptions, tid: u32, sid_: i32) -> Self {
        // ZmqSocketBase (parent_, tid, sid_),
        // _verbose_subs (false),
        // _verbose_unsubs (false),
        // _more_send (false),
        // _more_recv (false),
        // _process_subscribe (false),
        // _only_first_subscribe (false),
        // _lossy (true),
        // _manual (false),
        // _send_last_pipe (false),
        // _pending_pipes (),
        // _welcome_msg ()
        //     _last_pipe = null_mut();
        //     options.type = ZMQ_XPUB;
        //     _welcome_msg.init ();
        let base = ZmqSocketBase::new(parent, options, tid, sid_, false);
        options.type_ = ZMQ_XPUB as i32;
        let mut out = Self {
            base: base,
            ..Default::default()
        };
        out._welcome_msg.init();
        out
    }

    // ~XPub () ;

    //  Implementations of virtual functions from ZmqSocketBase.
    // void xattach_pipe (pipe: &mut ZmqPipe,
    pub fn xattach_pipe(
        &mut self,
        options: &mut ZmqOptions,
        pipe: &mut ZmqPipe,
        subscribe_to_all_: bool,
        locally_initiated_: bool,
    ) {
        // LIBZMQ_UNUSED (locally_initiated_);

        // zmq_assert (pipe);
        self._dist.attach(pipe);

        //  If subscribe_to_all_ is specified, the caller would like to subscribe
        //  to all data on this pipe, implicitly.
        if (subscribe_to_all_) {
            self._subscriptions.add(b"", 0, pipe);
        }

        // if welcome message exists, send a copy of it
        if (self._welcome_msg.size() > 0) {
            let copy = ZmqMessage::default();
            copy.init();
            // let rc: i32 = copy.copy (_welcome_msg);
            copy = self._welcome_msg.clone();
            // errno_assert (rc == 0);
            let ok = pipe.write(&mut copy);
            // zmq_assert (ok);
            pipe.flush();
        }

        //  The pipe is active when attached. Let's read the subscriptions from
        //  it, if any.
        self.xread_activated(options, pipe);
    }

    // bool subscribe_to_all_ = false,

    // bool locally_initiated_ = false) ;

    // int xsend (msg: &mut ZmqMessage) ;
    pub fn xsend(&mut self, options: &mut ZmqOptions, msg: &mut ZmqMessage) -> i32 {
        let msg_more = (msg.flags() & ZMQ_MSG_MORE) != 0;

        //  For the first part of multi-part message, find the matching pipes.
        if (!self._more_send) {
            // Ensure nothing from previous failed attempt to send is left matched
            self._dist.unmatch();

            if (self._manual && self._last_pipe.is_some() && self._send_last_pipe) {
                self._subscriptions.match_(
                    (msg.data()),
                    msg.size(),
                    mark_last_pipe_as_matching,
                    this,
                );
                self._last_pipe = None;
            } else {
                self._subscriptions
                    .match_((msg.data()), msg.size(), mark_as_matching, self);
            }
            // If inverted matching is used, reverse the selection now
            if (options.invert_matching) {
                self._dist.reverse_match();
            }
        }

        let mut rc = -1; //  Assume we fail
        if (self._lossy || self._dist.check_hwm()) {
            if (self._dist.send_to_matching(msg) == 0) {
                //  If we are at the end of multi-part message we can mark
                //  all the pipes as non-matching.
                if (!msg_more) {
                    self._dist.unmatch();
                }
                self._more_send = msg_more;
                rc = 0; //  Yay, sent successfully
            }
        } else {
            errno = EAGAIN;
        }
        return rc;
    }

    // bool xhas_out () ;
    pub fn xhas_out() -> bool {
        return _dist.has_out();
    }

    // int xrecv (msg: &mut ZmqMessage) ;
    pub fn xrecv(&mut self, msg: &mut ZmqMessage) -> i32 {
        //  If there is at least one
        if (self._pending_data.empty()) {
            errno = EAGAIN;
            return -1;
        }

        // User is reading a message, set last_pipe and remove it from the deque
        if (self._manual && !self._pending_pipes.empty()) {
            self._last_pipe = self._pending_pipes.front().cloned();
            self._pending_pipes.pop_front();

            // If the distributor doesn't know about this pipe it must have already
            // been terminated and thus we can't allow manual subscriptions.
            if (self._last_pipe.is_some() && !self._dist.has_pipe(&mut self._last_pipe.unwrap())) {
                self._last_pipe = None;
            }
        }

        let rc = msg.close();
        // errno_assert(rc == 0);
        rc = msg.init_size(self._pending_data.front().size());
        // errno_assert(rc == 0);
        copy_bytes(
            msg.data(),
            0,
            self._pending_data.front().data(),
            0,
            self._pending_data.front().size(),
        );

        // set metadata only if there is some
        let metadata = self._pending_metadata.front();
        if (metadata.is_some()) {
            msg.set_metadata(metadata.unwrap());
            // Remove ref corresponding to vector placement
            metadata.unwrap().drop_ref();
        }

        msg.set_flags(self._pending_flags.front());
        self._pending_data.pop_front();
        self._pending_metadata.pop_front();
        self._pending_flags.pop_front();
        return 0;
    }

    // bool xhas_in () ;
    pub fn xhas_in(&mut self) -> bool {
        return !self._pending_data.is_empty();
    }

    // void xread_activated (pipe: &mut ZmqPipe) ;

    pub fn xread_activated(&mut self, options: &mut ZmqOptions, pipe: &mut ZmqPipe) {
        //  There are some subscriptions waiting. Let's process them.
        let mut msg = ZmqMessage::default();
        while (pipe.read(&mut msg)) {
            // ZmqMetadata *metadata = msg.metadata ();
            let metadata = msg.metadata();
            let mut msg_data = msg.data_mut(); // (msg.data ()),
            *data = null_mut();
            let mut size = 0;
            let mut subscribe = false;
            let mut is_subscribe_or_cancel = false;
            let mut notify = false;

            let first_part = !_more_recv;
            _more_recv = (msg.flags() & ZMQ_MSG_MORE) != 0;

            if (first_part || _process_subscribe) {
                //  Apply the subscription to the trie
                if (msg.is_subscribe() || msg.is_cancel()) {
                    data = (msg.command_body());
                    size = msg.command_body_size();
                    subscribe = msg.is_subscribe();
                    is_subscribe_or_cancel = true;
                } else if (msg.size() > 0 && (*msg_data == 0 || *msg_data == 1)) {
                    data = msg_data + 1;
                    size = msg.size() - 1;
                    subscribe = *msg_data == 1;
                    is_subscribe_or_cancel = true;
                }
            }

            if (first_part) {
                self._process_subscribe = !self._only_first_subscribe || is_subscribe_or_cancel;
            }

            if (is_subscribe_or_cancel) {
                if (_manual) {
                    // Store manual subscription to use on termination
                    if (!subscribe) {
                        self._manual_subscriptions.rm(data, size, pipe);
                    } else {
                        self._manual_subscriptions.add(data, size, pipe);
                    }

                    self._pending_pipes.push_back(pipe.clone());
                } else {
                    if (!subscribe) {
                        let rm_result = self._subscriptions.rm(data, size, pipe);
                        //  TODO reconsider what to do if rm_result == mtrie_t::not_found
                        notify = rm_result != mtrie_t::values_remain || self._verbose_unsubs;
                    } else {
                        let first_added = self._subscriptions.add(data, size, pipe);
                        notify = first_added || self._verbose_subs;
                    }
                }

                //  If the request was a new subscription, or the subscription
                //  was removed, or verbose mode or manual mode are enabled, store it
                //  so that it can be passed to the user on next recv call.
                if (self._manual || (options.type_ == ZMQ_XPUB && notify)) {
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
                    let notification: Vec<u8> = Vec::with_capacity(size + 1);
                    if (subscribe) {
                        *notification.data() = 1;
                    } else {
                        *notification.data() = 0;
                    }
                    copy_bytes(notification.data(), 1, data, 0, size);

                    self._pending_data.push_back(ZMQ_MOVE(notification));
                    if (metadata) {
                        metadata.add_ref();
                    }
                    self._pending_metadata.push_back(metadata);
                    self._pending_flags.push_back(0);
                }
            } else if (options.type_ != ZMQ_PUB) {
                //  Process user message coming upstream from xsub socket,
                //  but not if the type is PUB, which never processes user
                //  messages
                self._pending_data.push_back(Blob(msg_data, msg.size()));
                if (metadata) {
                    metadata.add_ref();
                }
                self._pending_metadata.push_back(metadata);
                self._pending_flags.push_back(msg.flags());
            }

            msg.close();
        }
    }

    // void xwrite_activated (pipe: &mut ZmqPipe) ;
    pub fn xwrite_activated(&mut self, pipe: &mut ZmqPipe) {
        self._dist.activated(pipe);
    }

    // int xsetsockopt (option_: i32, const optval_: &mut [u8], optvallen_: usize) ;

    pub fn xsetsockopt(&mut self, option_: i32, optval_: &mut [u8], optvallen_: usize) -> i32 {
        if (option_ == ZMQ_XPUB_VERBOSE
            || option_ == ZMQ_XPUB_VERBOSER
            || option_ == ZMQ_XPUB_MANUAL_LAST_VALUE
            || option_ == ZMQ_XPUB_NODROP
            || option_ == ZMQ_XPUB_MANUAL
            || option_ == ZMQ_ONLY_FIRST_SUBSCRIBE)
        {
            if (optvallen_ != mem::size_of::<int>() || i32::from_le_bytes(optval_.clone()) < 0) {
                errno = EINVAL;
                return -1;
            }
            if (option_ == ZMQ_XPUB_VERBOSE) {
                self._verbose_subs = ((optval_) != 0);
                self._verbose_unsubs = false;
            } else if (option_ == ZMQ_XPUB_VERBOSER) {
                self._verbose_subs = ((optval_) != 0);
                self._verbose_unsubs = self._verbose_subs;
            } else if (option_ == ZMQ_XPUB_MANUAL_LAST_VALUE) {
                self._manual = ((optval_) != 0);
                self._send_last_pipe = self._manual;
            } else if (option_ == ZMQ_XPUB_NODROP) {
                self._lossy = ((optval_) == 0);
            } else if (option_ == ZMQ_XPUB_MANUAL) {
                self._manual = ((optval_) != 0);
            } else if (option_ == ZMQ_ONLY_FIRST_SUBSCRIBE) {
                self._only_first_subscribe = ((optval_) != 0);
            }
        } else if (option_ == ZMQ_SUBSCRIBE && self._manual) {
            if (self._last_pipe != null_mut()) {
                self._subscriptions
                    .add(optval_, optvallen_, self._last_pipe.clone());
            }
        } else if (option_ == ZMQ_UNSUBSCRIBE && self._manual) {
            if (self._last_pipe != null_mut()) {
                self._subscriptions
                    .rm(optval_, optvallen_, self._last_pipe.clone());
            }
        } else if (option_ == ZMQ_XPUB_WELCOME_MSG) {
            self._welcome_msg.close();

            if (optvallen_ > 0) {
                let rc: i32 = self._welcome_msg.init_size(optvallen_);
                // errno_assert(rc == 0);

                let data = (self._welcome_msg.data_mut());
                copy_bytes(data, 0, optval_, 0, optvallen_);
            } else {
                self._welcome_msg.init();
            }
        } else {
            errno = EINVAL;
            return -1;
        }
        return 0;
    }

    // int xgetsockopt (option_: i32, optval_: &mut [u8], optvallen_: *mut usize) ;
    pub fn xgetsockopt(&mut self, option_: i32, optval_: &mut [u8], optvallen_: *mut usize) -> i32 {
        if (option_ == ZMQ_TOPICS_COUNT) {
            // make sure to use a multi-thread safe function to avoid race conditions with I/O threads
            // where subscriptions are processed:
            return do_getsockopt(optval_, optvallen_, self._subscriptions.num_prefixes());
        }

        // room for future options here

        errno = EINVAL;
        return -1;
    }

    // void xpipe_terminated (pipe: &mut ZmqPipe) ;
    pub fn xpipe_terminated(&mut self, pipe: &mut ZmqPipe) {
        if (self._manual) {
            //  Remove the pipe from the trie and send corresponding manual
            //  unsubscriptions upstream.
            self._manual_subscriptions
                .rm(pipe, send_unsubscription, this, false);
            //  Remove pipe without actually sending the message as it was taken
            //  care of by the manual call above. subscriptions is the real mtrie,
            //  so the pipe must be removed from there or it will be left over.
            self._subscriptions.rm(pipe, stub, (null_mut()), false);

            // In case the pipe is currently set as last we must clear it to prevent
            // subscriptions from being re-added.
            if (pipe == self._last_pipe) {
                self._last_pipe = None;
            }
        } else {
            //  Remove the pipe from the trie. If there are topics that nobody
            //  is interested in anymore, send corresponding unsubscriptions
            //  upstream.
            self._subscriptions
                .rm(pipe, send_unsubscription, this, !self._verbose_unsubs);
        }

        self._dist.pipe_terminated(pipe);
    }

    //
    //  Function to be applied to the trie to send all the subscriptions
    //  upstream.
    // static void send_unsubscription (mtrie_t::prefix_t data,
    // size: usize,
    // XPub *self_);
    pub fn send_unsubscription(&mut self, data: TrieBuilder<Vec<u8>>, size: usize) {
        if (self.options.type_ != ZMQ_PUB) {
            //  Place the unsubscription to the queue of pending (un)subscriptions
            //  to be retrieved by the user later on.
            // TODO:
            // Blob unsub (size + 1);
            // *unsub.data () = 0;
            // if (size > 0) {
            //     copy_bytes(unsub.data_mut(), 1, data, 0,size);
            // }
            self._pending_data.ZMQ_PUSH_OR_EMPLACE_BACK(ZMQ_MOVE(unsub));
            // self._pending_metadata.push_back ();
            self._pending_flags.push_back(0);

            if (self._manual) {
                self._last_pipe = None;
                // self._pending_pipes.push_back (null_mut());
            }
        }
    }

    //  Function to be applied to each matching pipes.
    // static void mark_as_matching (pipe: &mut ZmqPipe, XPub *self_);
    pub fn mark_as_matching(&mut self, pipe: &mut ZmqPipe) {
        self._dist.match_(pipe);
    }

    // static void mark_last_pipe_as_matching (pipe: &mut ZmqPipe, XPub *self_);
    pub fn mark_last_pipe_as_matching(&mut self, pipe: &mut ZmqPipe) {
        if (self._last_pipe == pipe) {
            self._dist.match_(pipe);
        }
    }
}

// XPub::~XPub ()
// {
//     _welcome_msg.close ();
//     for (std::deque<ZmqMetadata *>::iterator it = _pending_metadata.begin (),
//                                             end = _pending_metadata.end ();
//          it != end; += 1it)
//         if (*it && (*it)->drop_ref ())
//             LIBZMQ_DELETE (*it);
// }

// pub fn  stub (mtrie_t::prefix_t data, size: usize, arg_: &mut [u8])
// {
//     LIBZMQ_UNUSED (data);
//     LIBZMQ_UNUSED (size);
//     LIBZMQ_UNUSED (arg_);
// }
