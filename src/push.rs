use libc::option;
use crate::ctx::ZmqContext;
use crate::defines::ZMQ_PUSH;
use crate::load_balancer::ZmqLoadBalancer;
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;
use crate::socket_base::ZmqSocket;

pub struct ZmqPush<'a> {
    pub socket_base: ZmqSocket<'a>,
    pub _lb: ZmqLoadBalancer,
}

impl ZmqPush {
    pub unsafe fn new(options: &mut ZmqOptions, parent_: &mut ZmqContext, tid_: u32, sid: i32) -> Self {
        options.type_ = ZMQ_PUSH;
        Self {
            socket_base: ZmqSocket::new(parent_, tid_, sid, false),
            _lb: ZmqLoadBalancer::new(),
        }
    }
    

}


 pub unsafe fn push_xattach_pipe(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe, subscribe_to_all_: bool, locally_initiated_: bool) {
        pipe_.set_nodelay();
        socket._lb.attach(pipe_);
    }

    pub unsafe fn push_xwrite_activated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
        socket._lb.activated(pipe_)
    }

    pub unsafe fn push_xpipe_terminated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
        socket._lb.pipe_terminated(pipe_);
    }

    pub unsafe fn push_xsend(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> i32 {
        socket._lb.send(msg_)
    }

    pub unsafe fn push_xhas_out(socket: &mut ZmqSocket) -> bool {
        socket._lb.has_out()
    }
