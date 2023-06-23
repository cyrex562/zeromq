use crate::message::{ZmqMessage, ZMQ_MSG_MORE};
use crate::pipe::ZmqPipe;
use crate::socket::{ZmqSocket};
use crate::socket_base_ops::ZmqSocketBaseOps;
use anyhow::anyhow;
use libc::socket;
use crate::context::ZmqContext;

// fn channel_xattach_pipe(
//
//     skt_base: &mut ZmqSocket,
//     in_pipe: &mut ZmqPipe,
//     subscribe_to_all: bool,
//     locally_initiated: bool,
// ) {
//     // LIBZMQ_UNUSED (subscribe_to_all_);
//     // LIBZMQ_UNUSED (locally_initiated_);
//
//     // zmq_assert (pipe_ != null_mut());
//
//     //  ZMQ_PAIR socket can only be connected to a single peer.
//     //  The socket rejects any further connection requests.
//     // if (pipe == null_mut())
//     // pipe = pipe_;
//     // else
//     // pipe_.terminate (false);
//     // }
//     if self.pipe.is_none() {
//         self.pipe = Some(in_pipe.clone());
//     } else {
//         in_pipe.terminate(false);
//     }
// }

// fn channel_xpipe_terminated(sock: &mut ZmqSocket, pipe: &mut ZmqPipe) {
//     if (pipe == self.pipe.unwrap()) {
//         self.pipe = None;
//     }
// }

pub fn channel_xwrite_activated(sock: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    //  There's just one pipe. No lists of active and inactive pipes.
    //  There's nothing to do here.
    unimplemented!()
}

pub fn channel_xsend(sock: &mut ZmqSocket, pipe: &mut ZmqPipe, msg: &mut ZmqMessage) -> anyhow::Result<()> {
    //  CHANNEL sockets do not allow multipart data (ZMQ_SNDMORE)
    if (msg.flags() & ZMQ_MSG_MORE) {
        // errno = EINVAL;
        // return -1;
        return Err(anyhow!(
                "invalid state: channel sockets do not allow multipart data"
            ));
    }
    pipe.flush();

    //  Detach the original message from the data buffer.
    msg.init2()?;
    // errno_assert (rc == 0);

    Ok(())
}

pub fn channel_xrecv(sock: &mut ZmqSocket, msg: &mut ZmqMessage) -> anyhow::Result<()> {
    //  Deallocate old content of the message.
    let mut rc = msg.close();
    // errno_assert(rc == 0);

    // if (pipe.is_none()) {
    //     //  Initialise the output parameter to be a 0-byte message.
    //     rc = msg.init2();
    //     // errno_assert(rc == 0);
    //     return Err(anyhow!("error EAGAIN"));
    // }

    // Drop any messages with more flag
    let mut read = pipe.read(msg);
    while read && msg.flags_set(ZMQ_MSG_MORE) {
        // drop all frames of the current multi-frame message
        read = pipe.read(msg);
        while read && msg.flags_set(ZMQ_MSG_MORE) {
            read = pipe.read(msg);
        }

        // get the new message
        if (read) {
            read = pipe.read(msg);
        }
    }

    if (!read) {
        //  Initialise the output parameter to be a 0-byte message.
        rc = msg.init2();
        // errno_assert(rc == 0);
        return Err(anyhow!("EAGAIN"));
        // errno = EAGAIN;
        // return -1;
    }

    // return 0;
    Ok(())
}

pub fn channel_xhas_in(sock: &mut ZmqSocket, pipe: &mut ZmqPipe) -> bool {
    return pipe.check_read();
}

pub fn channel_xhas_out(sock: &mut ZmqSocket, pipe: &mut ZmqPipe) -> bool {
    return pipe.check_write();
}