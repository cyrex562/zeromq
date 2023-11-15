use crate::ctx::ZmqContext;
use crate::defines::{ZMQ_MSG_MORE, ZMQ_ONLY_FIRST_SUBSCRIBE, ZMQ_PUB, ZMQ_SUBSCRIBE, ZMQ_TOPICS_COUNT, ZMQ_UNSUBSCRIBE, ZMQ_XPUB, ZMQ_XPUB_MANUAL, ZMQ_XPUB_MANUAL_LAST_VALUE, ZMQ_XPUB_NODROP, ZMQ_XPUB_VERBOSE, ZMQ_XPUB_VERBOSER, ZMQ_XPUB_WELCOME_MSG};
use crate::defines::err::ZmqError;
use crate::defines::mtrie::ZmqMtrie;
use crate::err::ZmqError;
use crate::err::ZmqError::SocketError;
use crate::msg::ZmqMsg;
use crate::options::{do_getsockopt, ZmqOptions};
use crate::pipe::ZmqPipe;
use crate::socket::ZmqSocket;

// pub struct ZmqXPub<'a> {
//     pub socket_base: ZmqSocket<'a>,
//     pub subscriptions: ZmqMtrie,
//     pub manual_subscriptions: ZmqMtrie,
//     pub dist: ZmqDist<'a>,
//     pub verbose_unsubs: bool,
//     pub more_send: bool,
//     pub more_recv: bool,
//     pub process_subscribe: bool,
//     pub only_first_subscribe: bool,
//     pub lossy: bool,
//     pub manual: bool,
//     pub send_last_pipe: bool,
//     pub last_pipe: Option<&'a mut ZmqPipe<'a>>,
//     pub pending_pipes: VecDeque<&'a mut ZmqPipe<'a>>,
//     pub welcome_msg: ZmqMsg<'a>,
//     pub pending_data: VecDeque<Vec<u8>>,
//     pub pending_metadata: VecDeque<&'a mut ZmqMetadata>,
//     pub pending_flags: VecDeque<u8>,
// }

// impl ZmqXPub {
//     pub unsafe fn new(options: &mut ZmqOptions, parent_: &mut ZmqContext, tid_: u32, sid_: i32) -> Self {
//         options.type_ = ZMQ_XPUB;
//         let mut out = Self {
//             socket_base: ZmqSocket::new(parent_, tid_, sid_, false),
//             subscriptions: GenericMtrie::new(),
//             manual_subscriptions: GenericMtrie::new(),
//             dist: ZmqDist::new(),
//             verbose_unsubs: false,
//             more_send: false,
//             more_recv: false,
//             process_subscribe: false,
//             only_first_subscribe: false,
//             lossy: false,
//             manual: false,
//             send_last_pipe: false,
//             last_pipe: None,
//             pending_pipes: Default::default(),
//             welcome_msg: ZmqMsg::default(),
//             pending_data: Default::default(),
//             pending_metadata: Default::default(),
//             pending_flags: Default::default(),
//         };
//         out.welcome_msg.init2();
//         out
//     }
// }


pub fn xpub_xattach_pipe(ctx: &mut ZmqContext, socket: &mut ZmqSocket, options: &mut ZmqOptions, pipe_: &mut ZmqPipe, subscribe_to_all_: bool, locally_initiated_: bool) -> Result<(),ZmqError> {
    socket.dist.attatch(pipe_);
    if subscribe_to_all_ {
        socket.subscriptions.add(None, 0, pipe_);
    }

    if socket.welcome_msg.size() > 0 {
        let mut copy: ZmqMsg = ZmqMsg::new();
        copy.init2()?;
        copy.copy(&mut socket.welcome_msg)?;
        pipe_.write(&mut copy)?;
        pipe_.flush(ctx);
    }

    socket.xread_activated(ctx, options, pipe_)
}

pub fn xpub_xread_activated(ctx: &mut ZmqContext, socket: &mut ZmqSocket, options: &mut ZmqOptions, pipe_: &mut ZmqPipe) -> Result<(),ZmqError> {
    //  There are some subscriptions waiting. Let's process them.
    // msg_t msg;
    let mut msg: ZmqMsg::new();
    while pipe_.read(ctx, &msg) {
        let mut metadata = msg.metadata();
        let mut msg_data = msg.data();
        let mut data: &mut [u8] = &mut [0u8; 1];

        let mut size = 0usize;
        let mut subscribe = false;
        let mut is_subscribe_or_cancel = false;
        let mut notify = false;

        let mut first_part = !socket.more_recv;
        socket.more_recv = msg.flag_set(ZMQ_MSG_MORE);

        if first_part || socket.process_subscribe {
            //  Apply the subscription to the trie
            if msg.is_subscribe() || msg.is_cancel() {
                data = (msg.command_body());
                size = msg.command_body_size();
                subscribe = msg.is_subscribe();
                is_subscribe_or_cancel = true;
            } else if msg.size() > 0 && (*msg_data == 0 || *msg_data == 1) {
                data = msg_data + 1;
                size = msg.size() - 1;
                subscribe = *msg_data == 1;
                is_subscribe_or_cancel = true;
            }
        }

        if first_part {
            socket.process_subscribe = !socket.only_first_subscribe || is_subscribe_or_cancel;
        }

        if is_subscribe_or_cancel {
            if socket.manual {
                // Store manual subscription to use on termination
                if !subscribe {
                    socket.manual_subscriptions.rm(data, size, pipe_);
                } else {
                    socket.manual_subscriptions.add(Some(data), size, pipe_);
                }

                socket.pending_pipes.push_back(pipe_);
            } else {
                if !subscribe {
                    let mut rm_result = socket.subscriptions.rm(data, size, pipe_);
                    //  TODO reconsider what to do if rm_result == mtrie_t::not_found
                    notify = rm_result != ZmqMtrie::values_remain || socket.verbose_unsubs;
                } else {
                    let first_added = socket.subscriptions.add(data, size, pipe_);
                    notify = first_added || socket.verbose_subs;
                }
            }

            //  If the request was a new subscription, or the subscription
            //  was removed, or verbose mode or manual mode are enabled, store it
            //  so that it can be passed to the user on next recv call.
            if socket.manual || (options.socket_type == ZMQ_XPUB && notify) {
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
                // blob_t notification (size + 1);
                let mut notification: Vec<u8> = Vec::with_capacity(size + 1);
                if subscribe {
                    notification.data()[0] = 1;
                } else {
                    notification.data()[0] = 0;
                }
                // libc::memcpy(notification.data_mut().add(1) as *mut c_void, data as *const c_void, size);
                notification.data_mut()[1..].copy_from_slice(data);

                socket.pending_data.push_back(notification);
                if metadata {
                    metadata.add_ref();
                }
                socket.pending_metadata.push_back(metadata);
                socket.pending_flags.push_back(0);
            }
        } else if options.socket_type != ZMQ_PUB {
            //  Process user message coming upstream from xsub socket,
            //  but not if the type is PUB, which never processes user
            //  messages
            socket.pending_data.push_back(msg_data);
            if metadata {
                metadata.add_ref();
            }
            socket.pending_metadata.push_back(metadata);
            socket.pending_flags.push_back(msg.flags());
        }

        msg.close();
    }

    Ok(())
}

pub fn xpub_xwrite_activated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    socket.dist.activated(pipe_)
}

pub fn xpub_xsetsockopt(
    socket: &mut ZmqSocket,
    option_: i32,
    optval_: &[u8],
    optvallen_: usize
) -> Result<(),ZmqError> {
    if option_ == ZMQ_XPUB_VERBOSE || option_ == ZMQ_XPUB_VERBOSER || option_ == ZMQ_XPUB_MANUAL_LAST_VALUE || option_ == ZMQ_XPUB_NODROP || option_ == ZMQ_XPUB_MANUAL || option_ == ZMQ_ONLY_FIRST_SUBSCRIBE {
        if optvallen_ != 4 || (optval_[0]) < 0 {
            // errno = EINVAL;
            return Err(SocketError("EINVAL"));
        }
        if option_ == ZMQ_XPUB_VERBOSE {
            socket.verbose_subs = ((optval_[0]) != 0);
            socket.verbose_unsubs = false;
        } else if option_ == ZMQ_XPUB_VERBOSER {
            socket.verbose_subs = ((optval_[0]) != 0);
            socket.verbose_unsubs = socket.verbose_subs;
        } else if option_ == ZMQ_XPUB_MANUAL_LAST_VALUE {
            socket.manual = ((optval_[0]) != 0);
            socket.send_last_pipe = socket.manual;
        } else if option_ == ZMQ_XPUB_NODROP {
            socket.lossy = ((optval_[0]) == 0);
        } else if option_ == ZMQ_XPUB_MANUAL {
            socket.manual = ((optval_[0]) != 0);
        } else if option_ == ZMQ_ONLY_FIRST_SUBSCRIBE {
            socket.only_first_subscribe = ((optval_[0]) != 0);
        }
    } else if option_ == ZMQ_SUBSCRIBE && socket.manual {
        if socket.last_pipe.is_some() {
            socket.subscriptions.add(optval_, optvallen_, &mut socket.last_pipe);
        }
    } else if option_ == ZMQ_UNSUBSCRIBE && socket.manual {
        if socket.last_pipe.is_some() {
            socket.subscriptions.rm(optval_, optvallen_, &mut socket.last_pipe);
        }
    } else if option_ == ZMQ_XPUB_WELCOME_MSG {
        socket.welcome_msg.close()?;

        if optvallen_ > 0 {
            socket.welcome_msg.init_size(optvallen_)?;
            // errno_assert (rc == 0);

            let data = (socket.welcome_msg.data_mut());
            // libc::memcpy(data, optval_.as_ptr() as *const c_void, optvallen_);
            data.clone_from_slice(optval_);
        } else { socket.welcome_msg.init2()?; }
    } else {
        // errno = EINVAL;
        return Err(SocketError("EINVAL"));
    }
    return Ok(())
}


pub fn xpub_xgetsockopt(socket: &mut ZmqSocket, option_: i32) -> Result<[u8],ZmqError> {
    if option_ == ZMQ_TOPICS_COUNT {
        return  do_getsockopt(option_);
        // return if rc == 0 {
        //     *optvallen_ = socket.subscriptions.count();
        //     Ok(rc)
        // } else {
        //     Err(SocketError("EINVAL"))
        // }
    }
    return Err(SocketError("EINVAL"));
}

pub fn xpub_stub(socket: &mut ZmqSocket, data_: &mut [u8], size_: usize, arg_: &mut [u8]) {
    unimplemented!()
}

pub fn xpub_xpipe_terminated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    if socket.manual {
        //  Remove the pipe from the trie and send corresponding manual
        //  unsubscriptions upstream.
        socket.manual_subscriptions.rm(pipe_, socket.send_unsubscription, socket, false);
        //  Remove pipe without actually sending the message as it was taken
        //  care of by the manual call above. subscriptions is the real mtrie,
        //  so the pipe must be removed from there or it will be left over.
        socket.subscriptions.rm(pipe_, socket.stub, None, false);

        // In case the pipe is currently set as last we must clear it to prevent
        // subscriptions from being re-added.
        if pipe_ == socket.last_pipe {
            socket.last_pipe = None;
        }
    } else {
        //  Remove the pipe from the trie. If there are topics that nobody
        //  is interested in anymore, send corresponding unsubscriptions
        //  upstream.
        socket.subscriptions.rm(pipe_, socket.send_unsubscription, socket, !socket.verbose_unsubs);
    }

    socket.dist.pipe_terminated(pipe_);
}

pub fn xpub_mark_as_matching(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe, other: &mut ZmqSocket) {
    other.dist.match_(pipe_);
}

pub fn xpub_mark_last_pipe_as_matching(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe, other_: &mut ZmqSocket) {
    if other_.last_pipe.unwrap() == pipe_ {
        other_.dist.match_(pipe_);
    }
}

pub fn xpub_xsend(socket: &mut ZmqSocket, options: &mut ZmqOptions, msg_: &mut ZmqMsg) -> Result<(),ZmqError> {
    let mut msg_more = msg_.flag_set(ZMQ_MSG_MORE) != 0;

    //  For the first part of multi-part message, find the matching pipes.
    if !socket.more_send {
        // Ensure nothing from previous failed attempt to send is left matched
        socket.dist.unmatch();

        if socket.manual && socket.last_pipe.is_some() && socket.send_last_pipe {
            socket.subscriptions.match_((msg_.data_mut()),
                                        msg_.size(), socket.mark_last_pipe_as_matching,
                                        socket);
            socket.last_pipe = None;
        } else {
            socket.subscriptions.match_((msg_.data_mut()),
                                        msg_.size(), socket.mark_as_matching, socket);
        }
        // If inverted matching is used, reverse the selection now
        if options.invert_matching {
            socket.dist.reverse_match();
        }
    }

    let mut rc = -1; //  Assume we fail
    if socket.lossy || socket.dist.check_hwm() {
        if socket.dist.send_to_matching(msg_) == 0 {
            //  If we are at the end of multi-part message we can mark
            //  all the pipes as non-matching.
            if !msg_more {
                socket.dist.unmatch();
            }
            socket.more_send = msg_more;
            rc = 0; //  Yay, sent successfully
        }
    } else {
        // errno = EAGAIN;
    }
    return rc;
}

pub fn xpub_xhas_out(socket: &mut ZmqSocket) -> bool {
    socket.dist.has_out()
}

pub fn xpub_xrecv(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> Result<(),ZmqError> {
    //  If there is at least one
    if socket.pending_data.empty() {
        // errno = EAGAIN;
        return -1;
    }

    // User is reading a message, set last_pipe and remove it from the deque
    if socket.manual && !socket.pending_pipes.empty() {
        socket.last_pipe = Some(*socket.pending_pipes.front().unwrap());
        socket.pending_pipes.pop_front();

        // If the distributor doesn't know about this pipe it must have already
        // been Terminated and thus we can't allow manual subscriptions.
        if socket.last_pipe != None && !socket.dist.has_pipe(socket.last_pipe.unwrap()) {
            socket.last_pipe = None;
        }
    }

    let mut rc = msg_.close();
    // errno_assert (rc == 0);
    rc = msg_.init_size(socket.pending_data.front().size());
    // errno_assert (rc == 0);
    // libc::memcpy(msg_.data_mut(), socket.pending_data.front().data(),
    //              socket.pending_data.front().size());
    msg_.data_mut().clone_from_slice(socket.pending_data.front().unwrap());

    // set metadata only if there is some
    let metadata = socket.pending_metadata.front();
    if metadata.is_some() {
        msg_.set_metadata(*metadata.unwrap());
        // Remove ref corresponding to vector placement
        metadata.drop_ref();
    }

    msg_.set_flags(*socket.pending_flags.front().unwrap());
    socket.pending_data.pop_front();
    socket.pending_metadata.pop_front();
    socket.pending_flags.pop_front();
    return 0;
}

pub fn xpub_xhas_in(socket: &mut ZmqSocket) -> bool {
    !socket.pending_data.empty()
}

pub fn xpub_send_unsubscription(options: &ZmqOptions, socket: &mut ZmqSocket, data_: &[u8], size_: usize, other_: &mut ZmqSocket) {
    if options.socket_type != ZMQ_PUB {
        //  Place the unsubscription to the queue of pending (un)subscriptions
        //  to be retrieved by the user later on.
        // blob_t unsub (size_ + 1);
        let unsub: Vec<u8> = Vec::with_capacity(size_ + 1);
        unsub.data()[0] = 0;
        let data_ptr = &mut unsub[1..];
        data_ptr.copy_from_slice(data_);
        // if (size_ > 0) {
        //     libc::memcpy(unsub.data().add(1), data_, size_);
        // }
        other_.pending_data.ZMQ_PUSH_OR_EMPLACE_BACK((unsub));
        // other_.pending_metadata.push_back(None);
        // other_.pending_flags.push_back(0);

        if other_.manual {
            other_.last_pipe = None;
            // other_.pending_pipes.push_back(None);
        }
    }
}

pub fn xpub_xjoin(socket: &mut ZmqSocket, group: &str) -> Result<(),ZmqError> {
    unimplemented!();
}
