use crate::ctx::ZmqContext;
use crate::defines::{MSG_MORE, ZMQ_PAIR};
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;
use crate::socket_base::ZmqSocket;

pub struct ZmqPair<'a> {
    pub socket_base: ZmqSocket<'a>,
    pub _pipe: Option<&'a mut ZmqPipe<'a>>,
}

impl ZmqPair {
    pub unsafe fn new(options: &mut ZmqOptions, parent_: &mut ZmqContext, tid_: u32, sid_: i32) -> ZmqPair {
        let mut out = Self {
            socket_base: ZmqSocket::new(parent_, tid_, sid_),
            _pipe: ZmqPipe::default(),
        };
        options.type_ = ZMQ_PAIR;
        out
    }


}

 pub unsafe fn pair_xattach_pipe(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe, subscribe_to_all_: bool, locally_initiated_: bool) {
    if socket._pipe.is_none() {
        socket._pipe = Some(pipe_);
    } else {
        socket._pipe.as_mut().unwrap().terminate(false);
    }
}

pub unsafe fn pair_xpipe_terminated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    if pipe_ == socket._pipe {
        socket._pipe = None;
    }
}

pub unsafe fn pair_xread_activated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    unimplemented!()
}

pub unsafe fn pair_xwrite_activated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    unimplemented!()
}

pub unsafe fn pair_xsend(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> i32 {
    if (!socket._pipe || !socket._pipe.write (msg_)) {
        // errno = EAGAIN;
        return -1;
    }

    if msg_.flag_clear(MSG_MORE) == true{
    socket._pipeflush ();}

    //  Detach the original message from the data buffer.
    let rc = msg_.init2 ();
    // errno_assert (rc == 0);

    return 0;
}

pub unsafe fn pair_xrecv(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> i32 {
    //  Deallocate old content of the message.
    let rc = msg_.close ();
    // errno_assert (rc == 0);

    if (!socket._pipe.is_none() || !socket._pipe.read (msg_)) {
        //  Initialise the output parameter to be a 0-byte message.
        rc = msg_.init2();
        // errno_assert (rc == 0);

        // errno = EAGAIN;
        return -1;
    }
    return 0;
}

pub unsafe fn pair_xhas_in (socket: &mut ZmqSocket) -> bool
{
    if (socket._pipe.is_none()) {
        return false;
    }

    return socket._pipe.check_read ();
}

pub unsafe fn pair_xhas_out (socket: &mut ZmqSocket) -> bool
{
    if (socket._pipe.is_none()) {
        return false;
    }

    return socket._pipe.check_write ();
}
