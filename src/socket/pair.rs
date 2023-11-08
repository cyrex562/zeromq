use crate::ctx::ZmqContext;
use crate::defines::ZMQ_MSG_MORE;
use crate::err::ZmqError;
use crate::err::ZmqError::SocketError;
use crate::msg::ZmqMsg;
use crate::pipe::ZmqPipe;
use crate::socket::ZmqSocket;

// pub struct ZmqPair<'a> {
//     pub socket_base: ZmqSocket<'a>,
//     pub _pipe: Option<&'a mut ZmqPipe<'a>>,
// }
//
// impl ZmqPair {
//     pub unsafe fn new(
//         options: &mut ZmqOptions,
//         parent_: &mut ZmqContext,
//         tid_: u32,
//         sid_: i32,
//     ) -> ZmqPair {
//         let mut out = Self {
//             socket_base: ZmqSocket::new(parent_, tid_, sid_),
//             _pipe: ZmqPipe::default(),
//         };
//         options.type_ = ZMQ_PAIR;
//         out
//     }
// }

pub fn pair_xsetsockopt(
    socket: &mut ZmqSocket,
    option_: i32,
    optval_: &[u8],
    optvallen_: usize,
) -> i32 {
    unimplemented!()
}

pub fn pair_xattach_pipe(
    ctx: &mut ZmqContext,
    socket: &mut ZmqSocket,
    pipe_: &mut ZmqPipe,
    subscribe_to_all_: bool,
    locally_initiated_: bool,
) {
    if socket.pipe.is_none() {
        socket.pipe = Some(pipe_);
    } else {
        socket.pipe.as_mut().unwrap().terminate(ctx, false);
    }
}

pub unsafe fn pair_xpipe_terminated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    if pipe_ == socket.pipe {
        socket.pipe = None;
    }
}

pub fn pair_xread_activated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) -> Result<(),ZmqError> {
    unimplemented!()
}

pub unsafe fn pair_xwrite_activated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    unimplemented!()
}

pub fn pair_xsend(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> Result<(),ZmqError> {
    if socket.pipe.is_none() || socket.pipe.unwrap().write(msg_).is_err() {
        // errno = EAGAIN;
        return Err(SocketError("EAGAIN"));
    }

    if msg_.flag_clear(ZMQ_MSG_MORE) == true {
        socket._pipeflush();
    }

    //  Detach the original message from the data buffer.
    msg_.init2()?;
    // errno_assert (rc == 0);

    Ok(())
}

pub fn pair_xrecv(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> Result<(),ZmqError> {
    //  Deallocate old content of the message.
    msg_.close()?;
    // errno_assert (rc == 0);

    if !socket.pipe.is_none() || !socket.pipe.read(msg_) {
        //  Initialise the output parameter to be a 0-byte message.
        msg_.init2()?;
        // errno_assert (rc == 0);

        // errno = EAGAIN;
        return Err(SocketError("EAGAIN"));
    }
    return Ok(());
}

pub fn pair_xhas_in(socket: &mut ZmqSocket) -> bool {
    if socket.pipe.is_none() {
        return false;
    }

    return socket.pipe.check_read();
}

pub fn pair_xhas_out(socket: &mut ZmqSocket) -> bool {
    if socket.pipe.is_none() {
        return false;
    }

    return socket.pipe.check_write();
}

pub fn pair_xgetsockopt(socket: &mut ZmqSocket, option: u32) -> Result<[u8], ZmqError> {
    unimplemented!();
}

pub fn pair_xjoin(socket: &mut ZmqSocket, group: &str) -> i32 {
    unimplemented!();
}
