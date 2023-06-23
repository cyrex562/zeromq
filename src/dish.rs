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

use std::collections::HashSet;
use std::sync::atomic::Ordering;
use anyhow::bail;

use libc::{EFAULT, EINVAL, ENOTSUP, pipe};

use crate::address::ZmqAddress;
use crate::context::ZmqContext;
use crate::defines::{ZMQ_DISH, ZMQ_GROUP_MAX_LENGTH};
use crate::dish_session::DishSessionState::{body, group};
use crate::dist::ZmqDist;
use crate::fair_queue::{self, ZmqFq};
use crate::message::{ZMQ_MSG_COMMAND, ZMQ_MSG_MORE, ZmqMessage};
use crate::pipe::ZmqPipe;
use crate::session_base::ZmqSessionBase;
use crate::socket::ZmqSocket;
use crate::thread_context::ZmqThreadContext;
use crate::utils::copy_bytes;

// #include "macros.hpp"
// #include "dish.hpp"
// #include "err.hpp"
#[derive(Default, Debug, Clone)]
pub struct ZmqDish<'a> {
    // public:
    // ZmqDish (ZmqContext *parent_, tid: u32, sid_: i32);
    // ~ZmqDish ();
    //   protected:
    //  Overrides of functions from ZmqSocketBase.
    // private:
    //  Send subscriptions to a pipe
    //  Fair queueing object for inbound pipes.
    // ZmqFq fair_queue;
    pub fair_queue: ZmqFq,

    //  Object for distributing the subscriptions upstream.
    // ZmqDist _dist;
    pub _dist: ZmqDist,

    //  The repository of subscriptions.
    // typedef std::set<std::string> subscriptions_t;
    // subscriptions_t _subscriptions;
    pub _subscriptions: HashSet<String>,

    //  If true, 'message' contains a matching message to return on the
    //  next recv call.
    pub _has_message: bool,
    pub _message: ZmqMessage,

    // // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqDish)
    pub socket_base: ZmqSocket<'a>,
}

// impl <'a> ZmqDish<'a> {
//     // ZmqDish (ZmqContext *parent_, tid: u32, sid_: i32);
//     // ZmqDish::ZmqDish (parent: &mut ZmqContext, tid: u32, sid_: i32) :
//     // ZmqSocketBase (parent_, tid, sid_, true), _has_message (false)
//     pub fn new(parent_: &mut ZmqContext, tid: u32, sid_: i32) -> Self {
//         parent_.type_ = ZMQ_DISH as i32;
//
//         //  When socket is being closed down we don't want to wait till pending
//         //  subscription commands are sent to the wire.
//         parent_.linger.store(0, Ordering::Relaxed);
//
//         let mut out = Self {
//             ..Default::default()
//         };
//
//         out._message.init2();
//         // errno_assert (rc == 0);
//         out
//     }
//     // ~ZmqDish ();
//     // void xattach_pipe (pipe: &mut ZmqPipe,
//     //     subscribe_to_all_: bool,
//     //     locally_initiated_: bool);
//     // ZmqDish::~ZmqDish ()
//     // {
//     //     let rc: i32 = _message.close ();
//     //     errno_assert (rc == 0);
//     // }
//     pub fn xattach_pipe(pipe: &mut ZmqPipe, subscribe_to_all_: bool, locally_initiated_: bool) {
//         // LIBZMQ_UNUSED (subscribe_to_all_);
//         // LIBZMQ_UNUSED (locally_initiated_);
//
//         // zmq_assert(pipe);
//         fair_queue.attach(pipe);
//         _dist.attach(pipe);
//
//         //  Send all the cached subscriptions to the new upstream peer.
//         send_subscriptions(pipe);
//     }
//
//
// }


pub fn dish_xsend( msg: &mut ZmqMessage) -> anyhow::Result<()> {
    unimplemented!()
}


pub fn dish_xhas_out() -> bool {
    //  Subscription can be added/removed anytime.
    return true;
}


pub fn dish_xrecv(sock: &mut ZmqSocket, msg: &mut ZmqMessage) -> anyhow::Result<()> {
    //  If there's already a message prepared by a previous call to zmq_poll,
    //  return it straight ahead.

    if (sock.has_message()) {
        // TODO
        // let rc: i32 = msg = self._message;

        // errno_assert (rc == 0);
        sock.set_has_message(false);
        Ok(())
    }
    return dish_xxrecv(sock, msg);
}

// bool xhas_in ();
pub fn dish_xhas_in(sock: &mut ZmqSocket) -> bool {
    //  If there's already a message prepared by a previous call to zmq_poll,
    //  return straight ahead.
    if sock.has_message() {
        return true;
    }

    dish_xxrecv(sock, &mut sock.get_message())?;

    //  Matching message found
    sock.has_message() = true;
    return true;
}

// void xread_activated (pipe: &mut ZmqPipe);
pub fn dish_xread_activated( sock: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    sock.fair_queue.activated(pipe);
}

// void xwrite_activated (pipe: &mut ZmqPipe);
pub fn dish_xwrite_activated( sock: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    sock.dist.activated(pipe);
}

// void xhiccuped (pipe: &mut ZmqPipe);
pub fn dish_xhiccuped( sock: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    //  Send all the cached subscriptions to the hiccuped pipe.
    dish_send_subscriptions(sock, pipe);
}

// void xpipe_terminated (pipe: &mut ZmqPipe);
pub fn dish_xpipe_terminated( sock: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    sock.fair_queue.pipe_terminated(pipe);
    sock.dist.pipe_terminated(pipe);
}

// int xjoin (group_: &str);
pub fn dish_xjoin( sock: &mut ZmqSocket, group_: &str) -> anyhow::Result<()> {
    // const std::string group = std::string (group_);

    if group_.len() > ZMQ_GROUP_MAX_LENGTH {
        bail!("Invalid group name");
    }

    //  User cannot join same group twice
    if !sock._subscriptions.insert(group_.to_string()).second {
        bail!("Already joined");
    }
    let mut msg = ZmqMessage::default();
    let mut rc = msg.init_join();
    // errno_assert (rc == 0);

    rc = msg.set_group(group_);
    // errno_assert (rc == 0);

    let mut err = 0;
    rc = sock._dist.send_to_all(&mut msg);
    if !rc {
        bail!("Failed to send message");
    }
    msg.close();
    Ok(())
}

// int xleave (group_: &str);
pub fn dish_xleave( sock: &mut ZmqSocket, group_: &str) -> anyhow::Result<()> {
    // const std::string group = std::string (group_);

    if (group_.len() > ZMQ_GROUP_MAX_LENGTH) {
       bail!("Invalid group name");
    }

    if (0 == sock.subscriptions.erase(group_)) {
        bail!("Not joined");
    }
    let mut msg = ZmqMessage::default();
    let mut rc = msg.init_leave();
    // errno_assert (rc == 0);

    rc = msg.set_group(group_);
    // errno_assert (rc == 0);

    let mut err = 0;
    rc = sock.dist.send_to_all(&mut msg);
    if (rc != 0) {
        bail!("Failed to send message");
    }
    msg.close();
    Ok(())
}

// int xxrecv (msg: &mut ZmqMessage);
pub fn dish_xxrecv( sock: &mut ZmqSocket, msg: &mut ZmqMessage) -> anyhow::Result<()> {
    loop {
        //  Get a message using fair queueing algorithm.
        let rc: i32 = sock.fair_queue.recv(msg);

        //  If there's no message available, return immediately.
        //  The same when error occurs.
        if (rc != 0) {
            bail!("Failed to receive message");
        }

        //  Skip non matching messages
        if !(0 == sock.subscriptions.count((msg.group()))) {
            bail!("Failed to count subscriptions");
        }
    }

    //  Found a matching message
    Ok(())
}

// void send_subscriptions (pipe: &mut ZmqPipe);
pub fn dish_send_subscriptions( sock: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    // for (subscriptions_t::iterator it = _subscriptions.begin (),
    //     end = _subscriptions.end ();
    // it != end; += 1it)
    for it in sock.subscriptions.iter() {
        let mut msg = ZmqMessage::default();
        let mut rc = msg.init_join();
        // errno_assert (rc == 0);

        rc = msg.set_group(it.c_str());
        // errno_assert (rc == 0);

        //  Send it to the pipe.
        pipe.write(&mut msg);
    }

    pipe.flush();
}
