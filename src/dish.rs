use std::collections::HashSet;
use std::sync::atomic::Ordering;

use anyhow::bail;
use libc::{pipe, EFAULT, EINVAL, ENOTSUP};

use crate::address::ZmqAddress;
use crate::context::ZmqContext;
use crate::defines::{ZMQ_DISH, ZMQ_GROUP_MAX_LENGTH};
use crate::dish_session::DishSessionState::{Body, Group};
use crate::dist::ZmqDist;
use crate::fair_queue::{self, ZmqFq};
use crate::message::{ZmqMessage, ZMQ_MSG_COMMAND, ZMQ_MSG_MORE};
use crate::pipe::ZmqPipe;
use crate::session_base::ZmqSessionBase;
use crate::socket::ZmqSocket;
use crate::thread_context::ZmqThreadContext;
use crate::utils::copy_bytes;

#[derive(Default, Debug, Clone)]
pub struct ZmqDish<'a> {
    pub fair_queue: ZmqFq,
    //  Object for distributing the subscriptions upstream.
    pub _dist: ZmqDist,
    //  The repository of subscriptions.
    pub _subscriptions: HashSet<String>,
    //  If true, 'message' contains a matching message to return on the
    //  next recv call.
    pub _has_message: bool,
    pub _message: ZmqMessage,
    pub socket_base: ZmqSocket<'a>,
}

pub fn dish_xsend(msg: &mut ZmqMessage) -> anyhow::Result<()> {
    unimplemented!()
}

pub fn dish_xhas_out() -> bool {
    //  Subscription can be added/removed anytime.
    return true;
}

pub fn dish_xrecv(sock: &mut ZmqSocket, msg: &mut ZmqMessage) -> anyhow::Result<()> {
    //  If there's already a message prepared by a previous call to zmq_poll,
    //  return it straight ahead.

    if sock.has_message() {
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
pub fn dish_xread_activated(sock: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    sock.fair_queue.activated(pipe);
}

// void xwrite_activated (pipe: &mut ZmqPipe);
pub fn dish_xwrite_activated(sock: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    sock.dist.activated(pipe);
}

// void xhiccuped (pipe: &mut ZmqPipe);
pub fn dish_xhiccuped(sock: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    //  Send all the cached subscriptions to the hiccuped pipe.
    dish_send_subscriptions(sock, pipe);
}

// void xpipe_terminated (pipe: &mut ZmqPipe);
pub fn dish_xpipe_terminated(sock: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    sock.fair_queue.pipe_terminated(pipe);
    sock.dist.pipe_terminated(pipe);
}

// int xjoin (group_: &str);
pub fn dish_xjoin(sock: &mut ZmqSocket, group_: &str) -> anyhow::Result<()> {
    // const std::string Group = std::string (group_);

    if group_.len() > ZMQ_GROUP_MAX_LENGTH {
        bail!("Invalid Group name");
    }

    //  User cannot join same Group twice
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
pub fn dish_xleave(sock: &mut ZmqSocket, group_: &str) -> anyhow::Result<()> {
    // const std::string Group = std::string (group_);

    if (group_.len() > ZMQ_GROUP_MAX_LENGTH) {
        bail!("Invalid Group name");
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
pub fn dish_xxrecv(sock: &mut ZmqSocket, msg: &mut ZmqMessage) -> anyhow::Result<()> {
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
pub fn dish_send_subscriptions(sock: &mut ZmqSocket, pipe: &mut ZmqPipe) {
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
