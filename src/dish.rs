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

use libc::{EFAULT, EINVAL, ENOTSUP};

use crate::address::ZmqAddress;
use crate::context::ZmqContext;
use crate::defines::{ZMQ_DISH, ZMQ_GROUP_MAX_LENGTH};
use crate::dish::DishSessionState::{body, group};
use crate::dist::ZmqDist;
use crate::fq::{self, ZmqFq};
use crate::message::{ZmqMessage, ZMQ_MSG_COMMAND, ZMQ_MSG_MORE};
use crate::pipe::ZmqPipe;
use crate::session_base::ZmqSessionBase;
use crate::socket_base::ZmqSocketBase;
use crate::thread_context::ZmqThreadContext;
use crate::utils::copy_bytes;

// #include "macros.hpp"
// #include "dish.hpp"
// #include "err.hpp"
#[derive(Default, Debug, Clone)]
pub struct ZmqDish {
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
    pub socket_base: ZmqSocketBase,
}

impl ZmqDish {
    // ZmqDish (ZmqContext *parent_, tid: u32, sid_: i32);
    // ZmqDish::ZmqDish (parent: &mut ZmqContext, tid: u32, sid_: i32) :
    // ZmqSocketBase (parent_, tid, sid_, true), _has_message (false)
    pub fn new(parent_: &mut ZmqContext, tid: u32, sid_: i32) -> Self {
        parent_.type_ = ZMQ_DISH as i32;

        //  When socket is being closed down we don't want to wait till pending
        //  subscription commands are sent to the wire.
        parent_.linger.store(0, Ordering::Relaxed);

        let mut out = Self {
            ..Default::default()
        };

        out._message.init2();
        // errno_assert (rc == 0);
        out
    }
    // ~ZmqDish ();
    // void xattach_pipe (pipe: &mut ZmqPipe,
    //     subscribe_to_all_: bool,
    //     locally_initiated_: bool);
    // ZmqDish::~ZmqDish ()
    // {
    //     let rc: i32 = _message.close ();
    //     errno_assert (rc == 0);
    // }
    pub fn xattach_pipe(pipe: &mut ZmqPipe, subscribe_to_all_: bool, locally_initiated_: bool) {
        // LIBZMQ_UNUSED (subscribe_to_all_);
        // LIBZMQ_UNUSED (locally_initiated_);

        zmq_assert(pipe);
        fair_queue.attach(pipe);
        _dist.attach(pipe);

        //  Send all the cached subscriptions to the new upstream peer.
        send_subscriptions(pipe);
    }

    // int xsend (msg: &mut ZmqMessage);
    pub fn xsend(&mut self, msg: &mut ZmqMessage) -> i32 {
        // LIBZMQ_UNUSED (msg);
        errno = ENOTSUP;
        return -1;
    }

    // bool xhas_out ();
    pub fn xhas_out(&mut self) -> bool {
        //  Subscription can be added/removed anytime.
        return true;
    }

    // int xrecv (msg: &mut ZmqMessage);
    pub fn xrecv(&mut self, msg: &mut ZmqMessage) -> i32 {
        //  If there's already a message prepared by a previous call to zmq_poll,
        //  return it straight ahead.
        if (self._has_message) {
            // TODO
            // let rc: i32 = msg = self._message;

            // errno_assert (rc == 0);
            self._has_message = false;
            return 0;
        }
        return self.xxrecv(msg);
    }

    // bool xhas_in ();
    pub fn xhas_in(&mut self) -> bool {
        //  If there's already a message prepared by a previous call to zmq_poll,
        //  return straight ahead.
        if (self._has_message) {
            return true;
        }

        let rc: i32 = self.xxrecv(&mut self._message);
        if (rc != 0) {
            // errno_assert (errno == EAGAIN);
            return false;
        }

        //  Matching message found
        self._has_message = true;
        return true;
    }

    // void xread_activated (pipe: &mut ZmqPipe);
    pub fn xread_activated(&mut self, pipe: &mut ZmqPipe) {
        self.fair_queue.activated(pipe);
    }

    // void xwrite_activated (pipe: &mut ZmqPipe);
    pub fn xwrite_activated(&mut self, pipe: &mut ZmqPipe) {
        self._dist.activated(pipe);
    }

    // void xhiccuped (pipe: &mut ZmqPipe);
    pub fn xhiccuped(&mut self, pipe: &mut ZmqPipe) {
        //  Send all the cached subscriptions to the hiccuped pipe.
        send_subscriptions(pipe);
    }

    // void xpipe_terminated (pipe: &mut ZmqPipe);
    pub fn xpipe_terminated(&mut self, pipe: &mut ZmqPipe) {
        self.fair_queue.pipe_terminated(pipe);
        self._dist.pipe_terminated(pipe);
    }

    // int xjoin (group_: &str);
    pub fn xjoin(&mut self, group_: &str) -> i32 {
        // const std::string group = std::string (group_);

        if (group_.len() > ZMQ_GROUP_MAX_LENGTH) {
            errno = EINVAL;
            return -1;
        }

        //  User cannot join same group twice
        if (!self._subscriptions.insert(group_.to_string()).second) {
            errno = EINVAL;
            return -1;
        }
        let mut msg = ZmqMessage::default();
        let mut rc = msg.init_join();
        // errno_assert (rc == 0);

        rc = msg.set_group(group_);
        // errno_assert (rc == 0);

        let mut err = 0;
        rc = self._dist.send_to_all(&mut msg);
        if (rc != 0) {
            err = errno;
        }
        msg.close();
        // errno_assert (rc2 == 0);
        if (rc != 0) {
            errno = err;
        }
        return rc;
    }

    // int xleave (group_: &str);
    pub fn xleave(&mut self, group_: &str) -> i32 {
        // const std::string group = std::string (group_);

        if (group_.len() > ZMQ_GROUP_MAX_LENGTH) {
            errno = EINVAL;
            return -1;
        }

        if (0 == self._subscriptions.erase(group_)) {
            errno = EINVAL;
            return -1;
        }
        let mut msg = ZmqMessage::default();
        let mut rc = msg.init_leave();
        // errno_assert (rc == 0);

        rc = msg.set_group(group_);
        // errno_assert (rc == 0);

        let mut err = 0;
        rc = self._dist.send_to_all(&mut msg);
        if (rc != 0) {
            err = errno;
        }
        msg.close();
        // errno_assert (rc2 == 0);
        if (rc != 0) {
            errno = err;
        }
        return rc;
    }

    // int xxrecv (msg: &mut ZmqMessage);
    pub fn xxrecv(&mut self, msg: &mut ZmqMessage) -> i32 {
        loop {
            //  Get a message using fair queueing algorithm.
            let rc: i32 = fair_queue.recv(msg);

            //  If there's no message available, return immediately.
            //  The same when error occurs.
            if (rc != 0) {
                return -1;
            }

            //  Skip non matching messages
            if !(0 == _subscriptions.count(std::string(msg.group()))) {
                break;
            }
        }

        //  Found a matching message
        return 0;
    }

    // void send_subscriptions (pipe: &mut ZmqPipe);
    pub fn send_subscriptions(&mut self, pipe: &mut ZmqPipe) {
        // for (subscriptions_t::iterator it = _subscriptions.begin (),
        //     end = _subscriptions.end ();
        // it != end; += 1it)
        for it in _subscriptions.iter() {
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
}

pub enum DishSessionState {
    group,
    body,
}

pub struct DishSession {
    // public:
    // private:
    pub _group_msg: ZmqMessage,
    // // ZMQ_NON_COPYABLE_NOR_MOVABLE (DishSession)
    pub session_base: ZmqSessionBase,
}

impl DishSession {
    // DishSession (ZmqIoThread *io_thread_,
    //     connect_: bool,
    //     socket: *mut ZmqSocketBase,
    //     options: &ZmqOptions,
    //     Address *addr_);
    // DishSession::DishSession (ZmqIoThread *io_thread_,
    //     connect_: bool,
    //     ZmqSocketBase *socket,
    //     options: &ZmqOptions,
    //     Address *addr_) :
    // ZmqSessionBase (io_thread_, connect_, socket, options_, addr_),
    // _state (group)
    pub fn new(
        io_thread: &mut ZmqThreadContext,
        connect_: bool,
        socket: &mut ZmqSocketbase,
        ctx: &mut ZmqContext,
        addr: &mut ZmqAddress,
    ) -> Self {
        DishSession {
            session_base: ZmqSessionBase::new(cx, io_thread, connect_, socket, addr),
            _group_msg: ZmqMessage::default(),
        }
    }

    // ~DishSession ();
    // DishSession::~DishSession ()
    // {
    // }

    //  Overrides of the functions from ZmqSessionBase.

    // int push_msg (msg: &mut ZmqMessage);

    pub fn push_msg(&mut self, msg: &mut ZmqMessage) -> i32 {
        if (self._state == group) {
            if ((msg.flags() & ZMQ_MSG_MORE) != ZMQ_MSG_MORE) {
                errno = EFAULT;
                return -1;
            }

            if (msg.size() > ZMQ_GROUP_MAX_LENGTH) {
                errno = EFAULT;
                return -1;
            }

            self._group_msg = msg.clone();
            self._state = body;

            msg.init2();
            // errno_assert (rc == 0);
            return 0;
        }
        let group_setting = msg.group();
        let mut rc: i32;
        if (group_setting[0] != 0) {
            // goto has_group;
        }

        //  Set the message group
        rc = msg.set_group2(self._group_msg.data(), self._group_msg.size());
        // errno_assert (rc == 0);

        //  We set the group, so we don't need the group_msg anymore
        self._group_msg.close();
        // errno_assert (rc == 0);
        // has_group:
        //  Thread safe socket doesn't support multipart messages
        if ((msg.flags() & ZMQ_MSG_MORE) == ZMQ_MSG_MORE) {
            errno = EFAULT;
            return -1;
        }

        //  Push message to dish socket
        rc = self.push_msg(msg);

        if (rc == 0) {
            self._state = group;
        }

        return rc;
    }

    // int pull_msg (msg: &mut ZmqMessage);
    pub fn pull_msg(&mut self, msg: &mut ZmqMessage) -> i32 {
        let rc = self.socket_base.pull_msg(msg);

        if (rc != 0) {
            return rc;
        }

        if (!msg.is_join() && !msg.is_leave()) {
            return rc;
        }

        let group_length: i32 = msg.group().len() as i32;

        let mut command: ZmqMessage = ZmqMessage::default();
        let mut offset: i32;

        if (msg.is_join()) {
            rc = command.init_size((group_length + 5) as usize);
            errno_assert(rc == 0);
            offset = 5;
            copy_bytes(command.data_mut(), 0, b"\x04JOIN", 0, 5);
        } else {
            rc = command.init_size((group_length + 6) as usize);
            errno_assert(rc == 0);
            offset = 6;
            copy_bytes(command.data_mut(), 0, b"\x05LEAVE", 0, 6);
        }

        command.set_flags(ZMQ_MSG_COMMAND);
        let mut command_data = (command.data_mut());

        //  Copy the group
        copy_bytes(
            command_data,
            offset,
            msg.group().as_bytes(),
            0,
            group_length,
        );

        //  Close the join message
        rc = msg.close();
        errno_assert(rc == 0);

        *msg = command;

        return 0;
    }

    // void reset ();

    pub fn reset(&mut self) {
        self.session_base.reset();
        self._state = group;
    }
}
