use anyhow::bail;
use libc::{EAGAIN, EINVAL};

use crate::content::ZmqContent;
use crate::context::ZmqContext;
use crate::defines::ZMQ_DGRAM;
use crate::message::{ZmqMessage, ZMQ_MSG_MORE};
use crate::pipe::ZmqPipe;
use crate::socket::ZmqSocket;

#[derive(Copy, Clone, Debug)]
pub struct ZmqDgram<'a> {
    pub pipe: Option<&'a mut ZmqPipe>,
    pub _more_out: bool,
    pub socket_base: ZmqSocket<'a>,
}

// int xsend (msg: &mut ZmqMessage);
pub fn dgram_xsend(sock: &mut ZmqSocket, msg: &mut ZmqMessage) -> anyhow::Result<()> {
    // If there's no out pipe, just drop it.
    if sock.pipe.is_none() {
        msg.close()?;
        // errno_assert (rc == 0);
    }

    //  If this is the first part of the message it's the ID of the
    //  peer to send the message to.
    if (!sock._more_out) {
        if (!(msg.flags() & ZMQ_MSG_MORE)) {
            errno = EINVAL;
            bail!("EINVAL");
        }
    } else {
        //  dgram messages are two part only, reject part if more is set
        if (msg.flags() & ZMQ_MSG_MORE) {
            errno = EINVAL;
            bail!("EINVAL");
        }
    }

    // Push the message into the pipe.
    if (!unsafe { sock.pipe.write(msg) }) {
        errno = EAGAIN;
        bail!("EAGAIN");
    }

    if (!(msg.flags() & ZMQ_MSG_MORE)) {
        sock.pipe.flush();
    }

    // flip the more flag
    sock._more_out = !sock._more_out;

    //  Detach the message from the data buffer.
    msg.init2()?;
    // errno_assert(rc == 0);

    Ok(())
}

// int xrecv (msg: &mut ZmqMessage);
pub fn dgram_xrecv(sock: &mut ZmqSocket, msg: &mut ZmqMessage) -> i32 {
    //  Deallocate old content of the message.
    let mut rc = msg.close();
    // errno_assert (rc == 0);

    if sock.pipe.is_none() || !sock.pipe.unwrap().read(msg) {
        //  Initialise the output parameter to be a 0-byte message.
        msg.init2();
        // errno_assert(rc == 0);

        errno = EAGAIN;
        return -1;
    }

    return 0;
}

// bool xhas_in ();
pub fn dgram_xhas_in(sock: &mut ZmqSocket) -> bool {
    if sock.pipe.is_none() {
        return false;
    }

    return sock.pipe.unwrap().check_read();
}

// bool xhas_out ();

// void xread_activated (pipe: &mut ZmqPipe);
pub fn dgram_xread_activated(sock: &mut ZmqSocket, pipe: *mut ZmqPipe) {
    //  There's just one pipe. No lists of active and inactive pipes.
    //  There's nothing to do here.
    unimplemented!("xread_activated")
}

// void xwrite_activated (pipe: &mut ZmqPipe);
pub fn dgram_xwrite_activated(sock: &mut ZmqSocket, pipe: *mut ZmqPipe) {
    //  There's just one pipe. No lists of active and inactive pipes.
    //  There's nothing to do here.
    unimplemented!("xwrite_activated")
}

// void xpipe_terminated (pipe: &mut ZmqPipe);
pub fn dgram_xpipe_terminated(sock: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    if pipe == sock.pipe {
        sock.pipe = None;
    }
}

pub fn dgram_xhas_out(sock: &mut ZmqSocket) -> bool {
    if sock.pipe.is_none() {
        return false;
    }

    return sock.pipe.check_write();
}
