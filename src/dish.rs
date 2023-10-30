use crate::ctx::ZmqContext;
use crate::defines::{ZMQ_DISH, ZMQ_GROUP_MAX_LENGTH, ZmqSubscriptions};
use crate::dist::ZmqDist;
use crate::fair_queue::ZmqFairQueue;
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;
use crate::socket::ZmqSocket;

pub struct ZmqDish<'a> {
    pub socket_base: ZmqSocket<'a>,
    pub _fq: ZmqFairQueue<'a>,
    pub _dist: ZmqDist<'a>,
    pub _subscriptions: ZmqSubscriptions,
    pub _has_message: bool,
    pub _message: ZmqMsg,
}

impl ZmqDish {
    pub unsafe fn new(options: &mut ZmqOptions, parent_: &mut ZmqContext, tid_: u32, sid_: i32) -> Self {
        options.type_ = ZMQ_DISH;
        options.linger = 0;
        let mut out = Self {
            socket_base: ZmqSocket::new(parent_, tid_, sid_, false),
            _fq: ZmqFairQueue::new(),
            _dist: dist_t::new(),
            _subscriptions: ZmqSubscriptions::new(),
            _has_message: false,
            _message: ZmqMsg::new(),
        };

        let rc = out._message.init2();

        out
    }
}

pub fn dish_xsetsockopt(socket: &mut ZmqSocket, option_: i32, optval_: &[u8], optvallen_: usize) -> i32 {
    unimplemented!()
}


pub fn dish_xattach_pipe(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe, subscribe_to_all_: bool, locally_initiated_: bool) {
    socket.fq.attach(pipe_);
    socket.dist.attach(pipe_);
    socket.send_subscriptions(pipe_);
}

pub fn dish_xread_activated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    socket.fq.activated(pipe_);
}

pub fn dish_xwrite_activated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    socket.dist.activated(pipe_);
}

pub fn dish_xpipe_terminated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    socket.fq.terminated(pipe_);
    socket.dist.terminated(pipe_);
}

pub fn dish_xhiccuped(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    socket.send_subscriptions(pipe_)
}

pub fn dish_xjoin(socket: &mut ZmqSocket, group_: &str) -> i32 {
    if group_.len() > ZMQ_GROUP_MAX_LENGTH {
        return -1;
    }

    socket.subscriptions.insert(group_.to_string());

    let mut msg = ZmqMsg::new();
    let mut rc = msg.init_join();

    rc = msg.set_group(group_);
    ;

    rc = socket.dist.send_to_all(&mut msg);

    let mut rc2 = msg.close();

    rc
}

pub fn dish_xleave(socket: &mut ZmqSocket, group_: &str) -> i32 {
    if group_.len() > ZMQ_GROUP_MAX_LENGTH {
        return -1;
    }

    socket.subscriptions.remove(group_);

    let mut msg = ZmqMsg::new();
    let mut rc = msg.init_leave();

    rc = msg.set_group(group_);
    ;

    rc = socket.dist.send_to_all(&mut msg);

    let mut rc2 = msg.close();

    rc
}

pub fn dish_xsend(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> i32 {
    unimplemented!()
}

pub fn dish_xhas_out(socket: &mut ZmqSocket) -> bool {
    true
}

pub unsafe fn xrecv(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> i32 {
    if socket.has_message {
        let mut rc = msg_.move_(socket.message);
        socket.has_message = false;
        return 0;
    }

    socket.xxrecv(msg_)
}

pub unsafe fn dish_xxrecv(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> i32 {
    loop {
        let mut rc = socket.fq.recv(msg_);
        if rc < 0 {
            return -1;
        }

        let mut count = 0;
        for x in socket.subscriptions.iter() {
            if x == msg_.group() {
                count += 1;
            }
        }
        if count == 0 {
            break;
        }
    }

    0
}

pub fn dish_xhas_in(socket: &mut ZmqSocket) -> bool {
    if socket.has_message {
        return true;
    }

    let mut rc = socket.xxrecv(&mut socket.message);
    if rc < 0 {
        return false;
    }

    socket.has_message = true;
    true
}

pub unsafe fn dish_send_subscriptions(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    for it in socket.subscriptions.iter_mut() {
        let mut msg = ZmqMsg::new();
        let mut rc = msg.init_join();

        rc = msg.set_group(it.as_str());

        pipe_.write(&mut msg);
        // rc = self._dist.send_to_pipe(&mut msg, pipe_);
        // let mut rc2 = msg.close();
    }

    pipe_.flush();
}

pub fn dish_xgetsockopt(socket: &mut ZmqSocket, option: u32) -> Result<[u8], ZmqError> {
    unimplemented!();
}

pub fn dish_xrecv(socket: &mut ZmqSocket, msg: &mut ZmqMsg) -> i32 {
    unimplemented!()
}
