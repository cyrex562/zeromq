use crate::ctx::ZmqContext;
use crate::defines::{ZMQ_DEALER, ZMQ_PROBE_ROUTER};
use crate::defines::err::ZmqError;
use crate::defines::err::ZmqError::SocketError;
use crate::defines::fair_queue::ZmqFairQueue;
use crate::defines::load_balancer::ZmqLoadBalancer;
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;
use crate::socket::ZmqSocket;

// pub struct ZmqDealer<'a>
// {
//     pub socket_base: ZmqSocket<'a>,
//     pub _fq: ZmqFairQueue<'a>,
//     pub _lb: ZmqLoadBalancer<'a>,
//     pub _probe_router: bool,
// }

// impl ZmqDealer {
//     pub unsafe fn new(options: &mut ZmqOptions, parent_: &mut ZmqContext, tid_: u32, sid_: i32) -> Self
//     {
//         options.type_ = ZMQ_DEALER;
//         options.can_send_hello_msg = true;
//         options.can_recv_hiccup_msg = true;
//
//         Self {
//             socket_base: ZmqSocket::new(parent_, tid_, sid_, false),
//             _fq: ZmqFairQueue::default(),
//             _lb: ZmqLoadBalancer::default(),
//             _probe_router: false,
//         }
//     }
//
//
// }

pub fn dealer_xattach_pipe(ctx: &mut ZmqContext, socket: &mut ZmqSocket, pipe_: &mut ZmqPipe, _subscribe_to_all_: bool, _locally_initiated_: bool) -> Result<(),ZmqError>
{
    if socket.probe_router {
        let mut probe_msg = ZmqMsg::default();
        probe_msg.init2 ()?;

        pipe_.write (&mut probe_msg)?;

        pipe_.flush (ctx);

        probe_msg.close ()?;

    }

    socket.fq.attach (pipe_);
    socket.lb.attach (pipe_);
    Ok(())
}

pub fn dealer_xsetsockopt(socket: &mut ZmqSocket, option_: i32, optval_: &[u8], optvallen_: usize) -> Result<(),ZmqError>
{
    // let is_int = optvallen_ == 4;
    let mut value: u32 = u32::from_le_bytes(optval_[0..4].try_into().unwrap());

    return if option_ == ZMQ_PROBE_ROUTER as i32 {
        socket.probe_router = value != 0;
        Ok(())
    } else {
        Err(SocketError("invalid option"))
    }

    // socket.xsetsockopt (option_, optval_, optvallen_)
}

pub fn dealer_xsend(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> Result<(),ZmqError> {
    socket.lb.sendpipe(msg_, &mut None)
}

pub fn dealer_xrecv(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> Result<(),ZmqError> {
    socket.lb.recvpipe(msg_, &mut None)
}

pub fn dealer_xhas_in(socket: &mut ZmqSocket) -> bool {
    socket.fq.has_in()
}

pub fn dealer_xhas_out(socket: &mut ZmqSocket) -> bool {
    socket.lb.has_out()
}

pub fn dealer_xread_activated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) -> Result<(),ZmqError> {
    socket.fq.activated(pipe_)

}

pub fn dealer_xwrite_activated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) -> Result<(),ZmqError> {
    socket.lb.activated(pipe_);
    Ok(())
}

pub fn dealer_sendpipe(socket: &mut ZmqSocket, msg_: &mut ZmqMsg, pipe_: &mut Option<&mut ZmqPipe>) -> Result<(),ZmqError> {
    socket.lb.sendpipe(msg_, pipe_)
}

pub fn dealer_recvpipe(
    ctx: &mut ZmqContext,
    socket: &mut ZmqSocket,
    msg_: &mut ZmqMsg,
    pipe_: &mut Option<&mut ZmqPipe>
) -> Result<(),ZmqError> {
    socket.fq.recvpipe(ctx, msg_, pipe_)
}

pub fn dealer_xgetsockopt(socket: &mut ZmqSocket, option: u32) -> Result<Vec<u8>, ZmqError> {
    unimplemented!();
}
pub fn dealer_xjoin(socket: &mut ZmqSocket, group: &str) -> Result<(),ZmqError> {
    unimplemented!();
}

pub fn dealer_xpipe_terminated(socket: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    unimplemented!()
}
