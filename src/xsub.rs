use crate::ctx::ZmqContext;
use crate::defines::{MSG_MORE, ZMQ_ONLY_FIRST_SUBSCRIBE, ZMQ_TOPICS_COUNT, ZMQ_XSUB, ZMQ_XSUB_VERBOSE_UNSUBSCRIBE};
use crate::dist::ZmqDist;
use crate::fair_queue::ZmqFairQueue;
use crate::msg::ZmqMsg;
use crate::options::{do_getsockopt, ZmqOptions};
use crate::pipe::ZmqPipe;
use crate::radix_tree::ZmqRadixTree;
use crate::socket_base::ZmqSocket;

// pub struct ZmqXSub<'a> {
//     pub socket_base: ZmqSocket<'a>,
//     pub _fq: ZmqFairQueue,
//     pub _dist: ZmqDist,
//     pub _subscriptions: ZmqRadixTree,
//     pub _verbose_unsubs: bool,
//     pub _has_message: bool,
//     pub _message: ZmqMsg,
//     pub _more_send: bool,
//     pub _more_recv: bool,
//     pub _process_subscribe: bool,
//     pub _only_first_subscribe: bool,
// }

// impl ZmqXSub {
//     pub unsafe fn new(options: &mut ZmqOptions, parent_: &mut ZmqContext, tid_: u32, sid_: i32) -> Self {
//         let mut out = Self {
//             socket_base: ZmqSocket::new(parent_, tid_, sid_, false),
//             _fq: ZmqFairQueue::default(),
//             _dist: ZmqDist::default(),
//             _subscriptions: ZmqRadixTree::default(),
//             _verbose_unsubs: false,
//             _has_message: false,
//             _message: ZmqMsg::default(),
//             _more_send: false,
//             _more_recv: false,
//             _process_subscribe: false,
//             _only_first_subscribe: false,
//         };
//         options.type_ = ZMQ_XSUB;
//         options.linger.store(0);
//         out._message.init2();
//         out
//     }
// 
//     
// }


pub unsafe fn xsub_xattach_pipe(
    socket: &mut ZmqSocket,
    pipe_: &mut ZmqPipe,
    subscribe_to_all_: bool,
    locally_initiated_: bool,
) {
    socket._fq.attach(pipe_);
    socket._dist.attach(pipe_);

    //  Send all the cached subscriptions to the new upstream peer.
    socket._subscriptions.apply(socket.send_subscription, pipe_);
    pipe_.flush();
}

pub unsafe fn xsub_xread_activated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    socket._fq.activated(pipe_);
}

pub unsafe fn xsub_xwrite_activated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    socket._dist.activated(pipe_);
}

pub unsafe fn xsub_xpipe_terminated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    socket._fq.terminated(pipe_);
    socket._dist.terminated(pipe_);
}

pub unsafe fn xsub_xhiccuped(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    //  Send all the cached subscriptions to the hiccuped pipe.
    socket._subscriptions.apply(socket.send_subscription, pipe_);
    pipe_.flush();
}

pub unsafe fn xsub_xsetsockopt(socket: &mut ZmqSocket, option_: i32, optval_: &[u8], optvallen_: usize) -> i32 {
    let opt_val_i32: i32 = i32::from_le_bytes([optval_[0], optval_[1], optval_[2], optval_[3]]);
    if (option_ == ZMQ_ONLY_FIRST_SUBSCRIBE) {
        if (optvallen_ != 4 || (opt_val_i32) < 0) {
            // errno = EINVAL;
            return -1;
        }
        socket._only_first_subscribe = (opt_val_i32 != 0);
        return 0;
    }
    // #ifdef ZMQ_BUILD_DRAFT_API
    else if (option_ == ZMQ_XSUB_VERBOSE_UNSUBSCRIBE) {
        socket._verbose_unsubs = (opt_val_i32 != 0);
        return 0;
    }
    // #endif
    //     errno = EINVAL;
    return -1;
}

pub unsafe fn xsub_xgetsockopt(
    socket: &mut ZmqSocket,
    option_: i32,
    optval_: &mut [u8],
    optvallen_: &mut usize,
) -> i32 {
    if option_ == ZMQ_TOPICS_COUNT {
        // make sure to use a multi-thread safe function to avoid race conditions with I/O threads
        // where subscriptions are processed:
        // #ifdef ZMQ_USE_RADIX_TREE
        let mut num_subscriptions = socket._subscriptions.size();
        // #else
        //         uint64_t num_subscriptions = _subscriptions.num_prefixes ();
        // #endif

        return do_getsockopt(optval_, optvallen_, num_subscriptions);
    }

    // room for future options here

    // errno = EINVAL;
    return -1;
}

pub unsafe fn xsub_xsend(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> i32 {
    let mut size = msg_.size();
    let mut data = (msg_.data_mut());

    let first_part = !socket._more_send;
    socket._more_send = msg_.flag_set(MSG_MORE);

    if (first_part) {
        socket._process_subscribe = !socket._only_first_subscribe;
    } else if (!socket._process_subscribe) {
        //  User message sent upstream to XPUB socket
        return socket._dist.send_to_all(msg_);
    }

    if (msg_.is_subscribe() || (size > 0 && *data == 1)) {
        //  Process subscribe message
        //  This used to filter out duplicate subscriptions,
        //  however this is already Done on the XPUB side and
        //  doing it here as well breaks ZMQ_XPUB_VERBOSE
        //  when there are forwarding devices involved.
        if (!msg_.is_subscribe()) {
            data = data.add(1);
            size = size - 1;
        }
        socket._subscriptions.add(data, size);
        socket._process_subscribe = true;
        return socket._dist.send_to_all(msg_);
    }
    if (msg_.is_cancel() || (size > 0 && *data == 0)) {
        //  Process unsubscribe message
        if (!msg_.is_cancel()) {
            data = data.add(1);
            size = size - 1;
        }
        socket._process_subscribe = true;
        let rm_result = socket._subscriptions.rm(data, size);
        if (rm_result || socket._verbose_unsubs) {
            return socket._dist.send_to_all(msg_);
        }
    } else {
        //  User message sent upstream to XPUB socket
        return socket._dist.send_to_all(msg_);
    }

    let rc = msg_.close();
    // errno_assert (rc == 0);
    rc = msg_.init2();
    // errno_assert (rc == 0);

    return 0;
}

pub fn xsub_xhas_out(socket: &mut ZmqSocket) -> bool {
    true
}

pub unsafe fn xsub_xrecv(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> i32 {
    //  If there's already a message prepared by a previous call to zmq_poll,
    //  return it straight ahead.
    if (socket._has_message) {
        let rc = msg_. move (socket._message);
        // errno_assert (rc == 0);
        socket._has_message = false;
        socket._more_recv = msg_.flag_set(MSG_MORE);
        return 0;
    }

    //  TODO: This can result in infinite loop in the case of continuous
    //  stream of non-matching messages which breaks the non-blocking recv
    //  semantics.
    loop {
        //  Get a message using fair queueing algorithm.
        let mut rc = socket._fq.recv(msg_);

        //  If there's no message available, return immediately.
        //  The same when Error occurs.
        if (rc != 0) {
            return -1;
        }

        //  Check whether the message matches at least one subscription.
        //  Non-initial parts of the message are passed
        if (socket._more_recv || !socket.options.filter || socket.match_(msg_)) {
            socket._more_recv = msg_.flag_set(MSG_MORE);
            return 0;
        }

        //  Message doesn't match. Pop any remaining parts of the message
        //  from the pipe.
        while msg_.flag_set(MSG_MORE) {
            rc = socket._fq.recv(msg_);
            // errno_assert (rc == 0);
        }
    }
}

pub unsafe fn xsub_xhas_in(socket: &mut ZmqSocket) -> bool {
    //  There are subsequent parts of the partly-read message available.
    if (socket._more_recv) {
        return true;
    }

    //  If there's already a message prepared by a previous call to zmq_poll,
    //  return straight ahead.
    if (socket._has_message) {
        return true;
    }

    //  TODO: This can result in infinite loop in the case of continuous
    //  stream of non-matching messages.
    loop {
        //  Get a message using fair queueing algorithm.
        let mut rc = socket._fq.recv(socket: &mut ZmqSocket._message);

        //  If there's no message available, return immediately.
        //  The same when Error occurs.
        if (rc != 0) {
            // errno_assert (errno == EAGAIN);
            return false;
        }

        //  Check whether the message matches at least one subscription.
        if (!socket.options.filter || socket.match_(&socket._message)) {
            socket._has_message = true;
            return true;
        }

        //  Message doesn't match. Pop any remaining parts of the message
        //  from the pipe.
        while (socket._message.flags() & ZmqMsg::more) {
            rc = socket._fq.recv(socket: &mut ZmqSocket._message);
            // errno_assert (rc == 0);
        }
    }
}

pub unsafe fn xsub_match_(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> bool {
    let matching = socket._subscriptions.check(
        (msg_.data_mut()), msg_.size());

    return matching ^ socket.options.invert_matching;
}

pub unsafe fn xsub_send_subscription(socket: &mut ZmqSocket, data_: &mut [u8], size_: usize, arg_: &mut [u8]) {
    let mut pipe = arg_.as_mut_ptr() as *mut ZmqPipe;

    //  Create the subscription message.
    let mut msg = ZmqMsg::default();
    let rc = msg.init_subscribe(size_, data_);
    // errno_assert (rc == 0);

    //  Send it to the pipe.
    let sent = (*pipe).write(&mut msg);
    //  If we reached the SNDHWM, and thus cannot send the subscription, drop
    //  the subscription message instead. This matches the behaviour of
    //  zmq_setsockopt(ZMQ_SUBSCRIBE, ...), which also drops subscriptions
    //  when the SNDHWM is reached.
    if (!sent) {
        msg.close();
    }
}
