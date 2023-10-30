use crate::ctx::ZmqContext;
use crate::defines::{ZMQ_SUB, ZMQ_SUBSCRIBE, ZMQ_UNSUBSCRIBE};
use crate::msg::{close_and_return, ZmqMsg};
use crate::options::ZmqOptions;
use crate::xsub::ZmqXSub;

// pub struct ZmqSub<'a> {
//     pub xsub: ZmqXSub<'a>,
// }

// impl ZmqSub {
//     pub unsafe fn new(options: &mut ZmqOptions, parent_: &mut ZmqContext, tid_: u32, sid_: i32) -> Self {
//         options.type_ = ZMQ_SUB;
//         options.filter = true;
//         Self {
//             xsub: ZmqXSub::new(options, parent_, tid_, sid_),
//         }
//     }
// 
//     
// }


pub unsafe fn sub_xsetsockopt(socket: &mut ZmqSocket, option_: i32, optval_: &[u8], optvallen_: usize) -> i32 {
    if (option_ != ZMQ_SUBSCRIBE && option_ != ZMQ_UNSUBSCRIBE) {
        // errno = EINVAL;
        return -1;
    }

    //  Create the subscription message.
    // msg_t msg;
    let mut msg = ZmqMsg::default();
    // int rc;
    let mut rc = 0i32;
    let data = (optval_);
    if (option_ == ZMQ_SUBSCRIBE) {
        rc = msg.init_subscribe(optvallen_, data);
    } else {
        rc = msg.init_cancel(optvallen_, data);
    }
    // errno_assert (rc == 0);

    //  Pass it further on in the stack.
    rc = xsub_xsend(&mut msg);
    return close_and_return(&mut msg, rc);
}

// int zmq::sub_t::xsend (msg_t *)
pub unsafe fn sub_xsend(socket: &mut ZmqSocket, msg: &mut ZmqMsg) -> i32 {
    //  Override the XSUB's send.
    // errno = ENOTSUP;
    return -1;
}

// bool zmq::sub_t::xhas_out ()
pub  fn sub_xhas_out(socket: &mut ZmqSocket) -> bool {
    //  Override the XSUB's send.
    return false;
}

pub fn sub_xattach_pipe(
    socket: &mut ZmqSocket,
    pipe_: &mut ZmqPipe,
    subscribe_to_all_: bool,
    locally_initiated_: bool,
) {
    unimplemented!()
}

pub fn sub_xgetsockopt(socket: &mut ZmqSocket, option: u32) -> Result<[u8], ZmqError> {
    unimplemented!();
}

pub fn sub_xjoin(socket: &mut ZmqSocket, group: &str) -> i32 {
    unimplemented!();
}

pub fn sub_xrecv(socket: &mut ZmqSocket, msg: &mut ZmqMsg) -> i32 {
    unimplemented!()
}

pub fn sub_xhas_in(socket: &mut ZmqSocket) -> i32 {
    unimplemented!()
}

pub fn sub_xread_activated(socket: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    unimplemented!()
}

pub fn sub_xwrite_activated(socket: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    unimplemented!()
}

pub fn sub_xpipe_terminated(socket: &mut ZmqSocket, pipe: &mut ZmqPipe)
