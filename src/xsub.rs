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

use std::mem;
use anyhow::bail;

use bincode::{deserialize, options};
use libc::EINVAL;

use crate::defines::{ZMQ_ONLY_FIRST_SUBSCRIBE, ZMQ_TOPICS_COUNT, ZMQ_XSUB_VERBOSE_UNSUBSCRIBE};
use crate::dist::ZmqDist;
use crate::fair_queue::ZmqFq;
use crate::message::{ZmqMessage, ZMQ_MSG_MORE};
use crate::pipe::ZmqPipe;
use crate::socket::{ZmqSocket};


// #[derive(Default, Debug, Clone)]
// pub struct XSub {
//     //
//     //
//     //  Overrides of functions from ZmqSocketBase.
//     //
//     //  Fair queueing object for inbound pipes.
//     // ZmqFq fair_queue;
//     pub fair_queue: ZmqFq,
//
//     //  Object for distributing the subscriptions upstream.
//     // ZmqDist _dist;
//     pub _dist: ZmqDist,
//
//     //  The repository of subscriptions.
//     // #ifdef ZMQ_USE_RADIX_TREE
//     //     radix_tree_t _subscriptions;
//     pub _subscriptions: radix_tree_t,
//     // #else
//     //     trie_with_size_t _subscriptions;
//     // #endif
//
//     // If true, send all unsubscription messages upstream, not just
//     // unique ones
//     pub _verbose_unsubs: bool,
//
//     //  If true, 'message' contains a matching message to return on the
//     //  next recv call.
//     pub _has_message: bool,
//     pub _message: ZmqMessage,
//
//     //  If true, part of a multipart message was already sent, but
//     //  there are following parts still waiting.
//     pub _more_send: bool,
//
//     //  If true, part of a multipart message was already received, but
//     //  there are following parts still waiting.
//     pub _more_recv: bool,
//     //  If true, subscribe and cancel messages are processed for the rest
//     //  of multipart message.
//     pub _process_subscribe: bool,
//
//     //  This option is enabled with ZMQ_ONLY_FIRST_SUBSCRIBE.
//     //  If true, messages following subscribe/unsubscribe in a multipart
//     //  message are treated as user data regardless of the first byte.
//     pub _only_first_subscribe: bool,
//
//     // // ZMQ_NON_COPYABLE_NOR_MOVABLE (XSub)
//     pub socket_base: ZmqSocket,
// }



// int xsetsockopt (option_: i32,
//                  const optval_: *mut c_void,
//                  optvallen_: usize) ;
pub fn xsetsockopt(sock: &mut ZmqSocket, option_: i32, opt_val: &mut [u8], optvallen_: usize) -> anyhow::Result<()> {
    if option_ == ZMQ_ONLY_FIRST_SUBSCRIBE {
        let opt_val_int = i32::from_le_bytes([opt_val[0], opt_val[1], opt_val[2], opt_val[3]]);
        if optvallen_ != mem::size_of::<i32>() || opt_val_int < 0 {
            // errno = EINVAL;
            // return -1;
            bail!("EINVAL");
        }
        sock._only_first_subscribe = opt_val_int != 0;
        return Ok(())
    }
    // #ifdef ZMQ_BUILD_DRAFT_API
    else if option_ == ZMQ_XSUB_VERBOSE_UNSUBSCRIBE {
        let opt_val_int = i32::from_le_bytes([opt_val[0], opt_val[1], opt_val[2], opt_val[3]]);
        sock._verbose_unsubs = opt_val_int != 0;
        return Ok(());
    }
    // #endif
    // errno = EINVAL;
    // return -1;
    bail!("EINVAL");
}

// int xgetsockopt (option_: i32, optval_: *mut c_void, optvallen_: *mut usize) ;
pub fn xgetsockopt(sock: &mut ZmqSocket, option_: i32, optval_: &mut [u8], optvallen_: &mut usize) -> anyhow::Result<()> {
    if option_ == ZMQ_TOPICS_COUNT {
        // make sure to use a multi-thread safe function to avoid race conditions with I/O threads
        // where subscriptions are processed:
        // #ifdef ZMQ_USE_RADIX_TREE
        let num_subscriptions = sock._subscriptions.size();
        // #else
        //         u64 num_subscriptions = _subscriptions.num_prefixes ();
        // #endif

        // TODO
        // return do_getsockopt(optval_, optvallen_, num_subscriptions);
    }

    // room for future options here

    // errno = EINVAL;
    // return -1;
    bail!("EINVAL");
}

// int xsend (ZmqMessage *msg) ;

pub fn xsend(sock: &mut ZmqSocket, msg: &mut ZmqMessage) -> i32 {
    let mut size = msg.size();
    let mut data = msg.data().unwrap().first_mut().unwrap();

    let first_part = !sock._more_send;
    sock._more_send = (msg.flags() & ZMQ_MSG_MORE) != 0;

    if first_part {
        sock._process_subscribe = !sock._only_first_subscribe;
    } else if (!sock._process_subscribe) {
        //  User message sent upstream to XPUB socket
        return sock._dist.send_to_all(msg);
    }

    if msg.is_subscribe() || (size > 0 && *data == 1) {
        //  Process subscribe message
        //  This used to filter out duplicate subscriptions,
        //  however this is already Done on the XPUB side and
        //  doing it here as well breaks ZMQ_XPUB_VERBOSE
        //  when there are forwarding devices involved.
        if !msg.is_subscribe() {
            data = data + 1;
            size = size - 1;
        }
        sock._subscriptions.add(data, size);
        sock._process_subscribe = true;
        return sock._dist.send_to_all(msg);
    }
    if msg.is_cancel() || (size > 0 && *data == 0) {
        //  Process unsubscribe message
        if !msg.is_cancel() {
            data = data + 1;
            size = size - 1;
        }
        sock._process_subscribe = true;
        let rm_result = sock._subscriptions.rm(data, size);
        if (rm_result || sock._verbose_unsubs) {
            return sock._dist.send_to_all(msg);
        }
    } else {
        //  User message sent upstream to XPUB socket
        return sock._dist.send_to_all(msg);
    }

    let mut rc = msg.close();
    // errno_assert (rc == 0);
    rc = msg.init2();
    // errno_assert (rc == 0);

    return 0;
}

// bool xhas_out () ;
pub fn xhas_out(sock: &mut ZmqSocket) -> bool {
    //  Subscription can be added/removed anytime.
    return true;
}

// int xrecv (ZmqMessage *msg) ;
pub fn xrecv(sock: &mut ZmqSocket, msg: &mut ZmqMessage) -> i32 {
    //  If there's already a message prepared by a previous call to zmq_poll,
    //  return it straight ahead.
    if sock._has_message {
        // let rc: i32 = msg.move (self._message);
        sock._message = msg.clone();
        // errno_assert (rc == 0);
        sock._has_message = false;
        sock._more_recv = (msg.flags() & ZMQ_MSG_MORE) != 0;
        return 0;
    }

    //  TODO: This can result in infinite loop in the case of continuous
    //  stream of non-matching messages which breaks the non-blocking recv
    //  semantics.
    loop {
        //  Get a message using fair queueing algorithm.
        let rc = sock.fair_queue.recv(msg);

        //  If there's no message available, return immediately.
        //  The same when error occurs.
        if (rc != 0) {
            return -1;
        }

        //  Check whether the message matches at least one subscription.
        //  Non-initial parts of the message are passed
        if sock._more_recv || !options.filter || sock.match_(msg) {
            sock._more_recv = (msg.flags() & ZMQ_MSG_MORE) != 0;
            return 0;
        }

        //  Message doesn't match. Pop any remaining parts of the message
        //  from the pipe.
        while msg.flags() & ZMQ_MSG_MORE {
            sock.fair_queue.recv(msg);
            // errno_assert(rc == 0);
        }
    }
}

// bool xhas_in () ;
pub fn xhas_in(sock: &mut ZmqSocket) -> bool {
    //  There are subsequent parts of the partly-read message available.
    if sock._more_recv {
        return true;
    }

    //  If there's already a message prepared by a previous call to zmq_poll,
    //  return straight ahead.
    if (sock._has_message) {
        return true;
    }

    //  TODO: This can result in infinite loop in the case of continuous
    //  stream of non-matching messages.
    loop {
        //  Get a message using fair queueing algorithm.
        let mut rc = sock.fair_queue.recv(&mut sock._message);

        //  If there's no message available, return immediately.
        //  The same when error occurs.
        if (rc != 0) {
            // errno_assert (errno == EAGAIN);
            return false;
        }

        //  Check whether the message matches at least one subscription.
        if !options.filter || sock.match_(&mut sock._message) {
            sock._has_message = true;
            return true;
        }

        //  Message doesn't match. Pop any remaining parts of the message
        //  from the pipe.
        while sock._message.flags() & ZMQ_MSG_MORE {
            rc = sock.fair_queue.recv(&mut sock._message);
            // errno_assert (rc == 0);
        }
    }
}

// void xread_activated (ZmqPipe *pipe_) ;
pub fn xread_activated(sock: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    sock.fair_queue.activated(pipe);
}

// void xwrite_activated (ZmqPipe *pipe_) ;
pub fn xwrite_activated(sock: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    sock._dist.activated(pipe);
}

// void xhiccuped (ZmqPipe *pipe_) ;
pub fn xhiccuped(sock: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    //  Send all the cached subscriptions to the hiccuped pipe.
    sock._subscriptions.apply(send_subscription, pipe);
    sock.pipe.flush();
}

// void xpipe_terminated (ZmqPipe *pipe_) ;
pub fn xpipe_terminated(sock: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    sock.fair_queue.pipe_terminated(pipe);
    sock._dist.pipe_terminated(pipe);
}

//  Check whether the message matches at least one subscription.
// bool match (ZmqMessage *msg);
pub fn match_(sock: &mut ZmqSocket, msg: &mut ZmqMessage) -> bool {
    let matching = sock._subscriptions.check((msg.data()), msg.size());

    return matching ^ options.invert_matching;
}

//  Function to be applied to the trie to send all the subsciptions
//  upstream.
// static void
// send_subscription (unsigned char *data, size: usize, arg_: *mut c_void);
pub fn send_subscription(
    sock: &mut ZmqSocket,
    data: &mut [u8],
    size: usize,
    arg_: &mut [u8],
) -> anyhow::Result<()> {
    // ZmqPipe *pipe = static_cast<ZmqPipe *> (arg_);
    let mut pipe: ZmqPipe = deserialize(arg_)?;

    //  Create the subscription message.
    let mut msg = ZmqMessage::default();
    msg.init_subscribe(size, data)?;
    // errno_assert (rc == 0);

    //  Send it to the pipe.
    let sent = pipe.write(&mut msg);
    //  If we reached the SNDHWM, and thus cannot send the subscription, drop
    //  the subscription message instead. This matches the behaviour of
    //  zmq_setsockopt(ZMQ_SUBSCRIBE, ...), which also drops subscriptions
    //  when the SNDHWM is reached.
    if !sent {
        msg.close();
    }
    Ok(())
}
