use std::collections::VecDeque;
use std::mem;
use std::ptr::null_mut;

use anyhow::bail;

use crate::context::ZmqContext;
use crate::defines::ZMQ_DEALER;
use crate::lb::LoadBalancer;
use crate::message::ZmqMessage;
use crate::pipe::ZmqPipe;
use crate::socket::ZmqSocket;
use crate::utils::copy_bytes;

pub fn dealer_xsetsockopt(
    sock: &mut ZmqSocket,
    option_: i32,
    optval_: &mut [u8],
    optvallen_: usize,
) -> anyhow::Result<()> {
    let is_int = (optvallen_ == mem::size_of::<int>());
    let mut value = 0;
    if (is_int) {
        let mut val_bytes: [u8; 4] = [0; 4];
        // memcpy (&value, optval_, mem::size_of::<int>());
        copy_bytes(&mut val_bytes, 0, optval_, 0, 4);
        value = i32::from_le_bytes(val_bytes);
    }

    match option_ {
        ZMQ_PROBE_ROUTER => {
            if is_int && value >= 0 {
                probe_router = (value != 0);
                return Ok(());
            }
        }
        // break;
        _ => {} // break;
    }

    // errno = EINVAL;
    // return -1;
    bail!("EINVAL")
}

// int xsend (msg: &mut ZmqMessage) ;
pub fn dealer_xsend(sock: &mut ZmqSocket, msg: &mut ZmqMessage) -> i32 {
    return sock.sendpipe(msg, null_mut());
}

// int xrecv (msg: &mut ZmqMessage) ;
pub fn dealer_xrecv(sock: &mut ZmqSocket, msg: &mut ZmqMessage) -> i32 {
    return sock.recvpipe(msg, null_mut());
}

// bool xhas_in () ;
pub fn dealer_xhas_in(sock: &mut ZmqSocket) -> bool {
    return sock.fair_queue.has_in();
}

// bool xhas_out () ;
pub fn dealer_xhas_out(sock: &mut ZmqSocket) -> bool {
    return load_balance.has_out();
}

// void xread_activated (pipe: &mut ZmqPipe) ;
pub fn dealer_xread_activated(sock: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    sock.fair_queue.activated(pipe);
}

// void xwrite_activated (pipe: &mut ZmqPipe) ;
pub fn dealer_xwrite_activated(sock: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    sock.load_balance.activated(pipe);
}

// void xpipe_terminated (pipe: &mut ZmqPipe) ;
pub fn dealer_xpipe_terminated(sock: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    sock.fair_queue.pipe_terminated(pipe);
    sock.load_balance.pipe_terminated(pipe);
}

//  Send and recv - knowing which pipe was used.

// int sendpipe (msg: &mut ZmqMessage ZmqPipe **pipe);
pub fn dealer_sendpipe(sock: &mut ZmqSocket, msg: &mut ZmqMessage, pipe: *mut *mut ZmqPipe) -> i32 {
    return sock.load_balance.sendpipe(msg, pipe);
}

pub fn dealer_recvpipe(sock: &mut ZmqSocket, msg: &mut ZmqMessage, pipe: *mut *mut ZmqPipe) -> i32 {
    return sock.fair_queue.recvpipe(msg, pipe);
}
