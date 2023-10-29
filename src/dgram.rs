use crate::defines::MSG_MORE;
use crate::msg::ZmqMsg;
use crate::pipe::ZmqPipe;
use crate::socket_base::ZmqSocket;

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
    if socket._pipe.is_none() {
        socket._pipe = Some(pipe_);
    } else {
        pipe_.terminate(false);
    }
}

pub unsafe fn dgram_xpipe_terminated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    if socket._pipe.is_some() && socket._pipe.unwrap() == pipe_ {
        socket._pipe = None;
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
    if (!socket._pipe) {
        let mut rc = msg_.close();
        // errno_assert (rc == 0);
        return -1;
    }

    //  If this is the first part of the message it's the ID of the
    //  peer to send the message to.
    if (!socket._more_out) {
        if (!(msg_.flags() & MSG_MORE != 0)) {
            // errno = EINVAL;
            return -1;
        }
    } else {
        //  dgram messages are two part only, reject part if more is set
        if (msg_.flags() & MSG_MORE) {
            // errno = EINVAL;
            return -1;
        }
    }

    // Push the message into the pipe.
    if (!socket._pipe.write(msg_)) {
        // errno = EAGAIN;
        return -1;
    }

    if (!(msg_.flags() & MSG_MORE))
    socket._pipe.flush();

    // flip the more flag
    socket._more_out = !socket._more_out;

    //  Detach the message from the data buffer.
    let rc = msg_.init();
    // errno_assert (rc == 0);

    return 0;
}

pub unsafe fn dgram_xrecv(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> i32 {
    //  Deallocate old content of the message.
    let mut rc = msg_.close();
    // errno_assert (rc == 0);

    if (!socket._pipe || !socket._pipe.read(msg_)) {
        //  Initialise the output parameter to be a 0-byte message.
        rc = msg_.init2();
        // errno_assert (rc == 0);

        // errno = EAGAIN;
        return -1;
    }

    return 0;
}

pub unsafe fn dgram_xhas_in(socket: &mut ZmqSocket) -> bool {
    if (socket._pipe.is_none()) {
        return false;
    }

    return socket._pipe.check_read();
}

pub unsafe fn dgram_xhas_out(socket: &mut ZmqSocket) -> bool {
    if (socket._pipe.is_none()) {
        return false;
    }

    return socket._pipe.check_write();
}
