use crate::defines::err::ZmqError;
use crate::defines::err::ZmqError::{PipeError, SocketError};
use crate::defines::ZMQ_MSG_MORE;
use crate::msg::ZmqMsg;
use crate::pipe::ZmqPipe;
use crate::socket::ZmqSocket;


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
    if socket.pipe == None {
        socket.pipe == Some(in_pipe);
    }
}

pub fn channel_xpipe_terminated(socket: &mut ZmqSocket, in_pipe: &mut ZmqPipe) {
    if socket.pipe.unwrap() == in_pipe {
        socket.pipe = None;
    }
}

pub fn channel_xread_activated(_socket: &mut ZmqSocket, _pipe: &mut ZmqPipe) {
    unimplemented!()
}

pub fn channel_xwrite_activated(_socket: &mut ZmqSocket, _pipe: &mut ZmqPipe) {
    unimplemented!()
}

pub fn channel_xsend(socket: &mut ZmqSocket, msg: &mut ZmqMsg) -> Result<(),ZmqError> {
    if msg.flag_set(ZMQ_MSG_MORE) {
        return Err(SocketError("msg more flag is set"));
    }

    if socket.pipe.is_none() || socket.pipe.unwrap().write(msg).is_err() {
        return Err(SocketError("pipe is null"));
    }

    socket.pipe.flush();

    (msg).init2()?;

    Ok(())
}

pub fn channel_xrecv(socket: &mut ZmqSocket, msg: &mut ZmqMsg) -> Result<(), ZmqError> {
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
