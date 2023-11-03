use crate::defines::ZMQ_MSG_MORE;
use crate::msg::ZmqMsg;
use crate::pipe::ZmqPipe;
use crate::socket::ZmqSocket;

pub struct ZmqDgram<'a> {
    pub socket_base: ZmqSocket<'a>,
    pub _pipe: Option<&'a mut ZmqPipe<'a>>,
    pub _more_out: bool,
}

impl ZmqDgram {
    pub unsafe fn new(options: &mut options_t,
                      parent_: &mut ctx_t,
                      tid_: u32,
                      sid_: i32) -> ZmqDgram {
        ZmqDgram {
            socket_base: ZmqSocket::new(parent_, tid_, sid_, false),
            _pipe: None,
            _more_out: false,
        }
    }


}

pub unsafe fn dgram_xattach_pipe(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe, subscribe_to_all: bool, locally_initiated: bool) {
    if socket.pipe.is_none() {
        socket.pipe = Some(pipe_);
    } else {
        pipe_.terminate(false);
    }
}

pub unsafe fn dgram_xpipe_terminated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    if socket.pipe.is_some() && socket.pipe.unwrap() == pipe_ {
        socket.pipe = None;
    }
}

pub unsafe fn dgram_xread_activated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    unimplemented!()
}

pub unsafe fn dgram_xwrite_activated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    unimplemented!()
}

pub unsafe fn xsend(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> i32 {
    // If there's no out pipe, just drop it.
    if (!socket.pipe) {
        let mut rc = msg_.close();
        // errno_assert (rc == 0);
        return -1;
    }

    //  If this is the first part of the message it's the ID of the
    //  peer to send the message to.
    if (!socket.more_out) {
        if (!(msg_.flags() & ZMQ_MSG_MORE != 0)) {
            // errno = EINVAL;
            return -1;
        }
    } else {
        //  dgram messages are two part only, reject part if more is set
        if (msg_.flags() & ZMQ_MSG_MORE) {
            // errno = EINVAL;
            return -1;
        }
    }

    // Push the message into the pipe.
    if (!socket.pipe.write(msg_)) {
        // errno = EAGAIN;
        return -1;
    }

    if (!(msg_.flags() & ZMQ_MSG_MORE))
    socket.pipe.flush();

    // flip the more flag
    socket.more_out = !socket.more_out;

    //  Detach the message from the data buffer.
    let rc = msg_.init();
    // errno_assert (rc == 0);

    return 0;
}

pub unsafe fn dgram_xrecv(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> i32 {
    //  Deallocate old content of the message.
    let mut rc = msg_.close();
    // errno_assert (rc == 0);

    if (!socket.pipe || !socket.pipe.read(msg_)) {
        //  Initialise the output parameter to be a 0-byte message.
        rc = msg_.init2();
        // errno_assert (rc == 0);

        // errno = EAGAIN;
        return -1;
    }

    return 0;
}

pub  fn dgram_xhas_in(socket: &mut ZmqSocket) -> bool {
    if (socket.pipe.is_none()) {
        return false;
    }

    return socket.pipe.check_read();
}

pub  fn dgram_xhas_out(socket: &mut ZmqSocket) -> bool {
    if (socket.pipe.is_none()) {
        return false;
    }

    return socket.pipe.check_write();
}

pub fn dgram_xsetsockopt(socket: &mut ZmqSocket, option_: i32, optval_: &[u8], optvallen_: usize) -> i32 {
    unimplemented!()
}
pub fn dgram_xgetsockopt(socket: &mut ZmqSocket, option: u32) -> Result<[u8], ZmqError> {
    unimplemented!();
}

pub fn dgram_xjoin(socket: &mut ZmqSocket, group: &str) -> i32 {
    unimplemented!();
}

pub fn dgram_xsend(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> i32 {
    unimplemented!()
}

pub fn dgram_xpipe_terminated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    unimplemented!()
}
