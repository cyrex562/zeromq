use crate::ctx::ZmqContext;
use crate::defines::{ZMQ_CHANNEL, ZMQ_MSG_MORE};
use crate::err::ZmqError;
use crate::err::ZmqError::PipeError;
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;
use crate::socket::ZmqSocket;
use std::ptr::null_mut;

// pub struct ZmqChannel<'a> {
//     pub base: ZmqSocket<'a>,
//     pub pipe: &'a mut ZmqPipe<'a>,
// }

// impl ZmqChannel {
//     pub unsafe fn new(
//         options: &mut ZmqOptions,
//         parent: &mut ZmqContext,
//         tid_: u32,
//         sid_: i32,
//     ) -> Self {
//         options.type_ = ZMQ_CHANNEL;
//         Self {
//             base: ZmqSocket::new(parent, tid_, sid_, true),
//             pipe: &mut ZmqPipe::default(),
//         }
//     }
// }

pub fn channel_xattach_pipe(
    socket: &mut ZmqSocket,
    in_pipe: &mut ZmqPipe,
    _subscribe_to_all: bool,
    _local_initiated: bool,
) {
    if socket.pipe == null_mut() {
        socket.pipe == in_pipe;
    }
}

pub fn channel_xpipe_terminated(socket: &mut ZmqSocket, in_pipe: &mut ZmqPipe) {
    if socket.pipe == in_pipe {
        socket.pipe = Some(&mut ZmqPipe::default());
    }
}

pub fn channel_xread_activated(socket: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    unimplemented!()
}

pub fn channel_xwrite_activated(socket: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    unimplemented!()
}

pub unsafe fn channel_xsend(socket: &mut ZmqSocket, msg: &mut ZmqMsg) -> i32 {
    if msg.flag_set(ZMQ_MSG_MORE) {
        return -1;
    }

    if socket.pipe == &mut ZmqPipe::default() || socket.pipe.write(msg).is_err() {
        return -1;
    }

    socket.pipe.flush();

    let rc = (*msg).init2();

    return 0;
}

pub unsafe fn channel_xrecv(socket: &mut ZmqSocket, msg: &mut ZmqMsg) -> Result<(), ZmqError> {
    msg.close()?;

    if socket.pipe = Some(&mut ZmqPipe::default()) {
        (msg).init2()?;
        return Err(PipeError("pipe is null"));
    }

    let mut read = (socket.pipe).read(msg);

    while read && msg.flags() & ZmqMsg::more > 0 {
        read = (*socket.pipe).read(msg);
        while read && msg.flags() & ZmqMsg::more > 0 {
            read = (*socket.pipe).read(msg);
        }

        if read {
            read = (*socket.pipe).read(msg);
        }
    }

    if !read {
        (*msg).init2()?;
    }

    Ok(())
}

pub fn channel_xhas_in(socket: &mut ZmqSocket) -> bool {
    if socket.pipe.is_none() {
        return false;
    }
    return socket.pipe.check_read();
}

pub fn channel_xhas_out(socket: &mut ZmqSocket) -> bool {
    if socket.pipe.is_none() {
        return false;
    }
    return socket.pipe.check_write();
}
