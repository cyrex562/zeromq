use crate::ctx::ZmqContext;
use crate::defines::{MSG_MORE, ZMQ_SCATTER};
use crate::load_balancer::ZmqLoadBalancer;
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;
use crate::socket_base::ZmqSocket;

// pub struct ZmqScatter {
//     pub socket_base: ZmqSocket,
//     pub _lb: ZmqLoadBalancer,
// }

// impl ZmqScatter {
//     pub unsafe fn new(options: &mut ZmqOptions, parent_: &mut ZmqContext, tid_: u32, sid_: i32) -> Self {
//         options.type_ = ZMQ_SCATTER;
//         Self {
//             socket_base: ZmqSocket::new(parent_, tid_, sid_, false),
//             _lb: ZmqLoadBalancer::new(),
//         }
//     }
// 
//    
// }

 pub unsafe fn scatter_xattach_pipe(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe, subscribe_to_all: bool, locally_initiated_: bool) {
    pipe_.set_nodelay();
    socket._lb.attach(pipe_);
}

pub unsafe fn scatter_xwrite_activated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    socket._lb.activated(pipe_);
}

pub unsafe fn scatter_xpipe_terminated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    socket._lb.terminated(pipe_);
}

pub unsafe fn scatter_xsend(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> i32 {
    //  SCATTER sockets do not allow multipart data (ZMQ_SNDMORE)
    if (msg_.flags () & MSG_MORE) {
        // errno = EINVAL;
        return -1;
    }

    return socket._lb.send (msg_);
}

// bool zmq::scatter_t::xhas_out ()
pub unsafe fn scatter_xhas_out(socket: &mut ZmqSocket) -> bool
{
    return socket._lb.has_out ();
}
