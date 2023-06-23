use crate::context::ZmqContext;
use crate::defines::ZMQ_GATHER;
use crate::fair_queue::ZmqFq;
use crate::message::{ZmqMessage, ZMQ_MSG_MORE};
use crate::pipe::ZmqPipe;
use crate::socket::ZmqSocket;

// #[derive(Default, Debug, Clone)]
// pub struct ZmqGather {
//     //
//     //     ZmqGather (ZmqContext *parent_, tid: u32, sid_: i32);
//     //     ~ZmqGather ();
//
//     //
//     //  Overrides of functions from ZmqSocketBase.
//     // void xattach_pipe (pipe: &mut ZmqPipe,
//     //                    subscribe_to_all_: bool,
//     //                    locally_initiated_: bool);
//     // int xrecv (msg: &mut ZmqMessage);
//     // bool xhas_in ();
//     // void xread_activated (pipe: &mut ZmqPipe);
//     // void xpipe_terminated (pipe: &mut ZmqPipe);
//
//     //
//     //  Fair queueing object for inbound pipes.
//     pub fair_queue: ZmqFq,
//     pub socket_base: ZmqSocket,
//     // // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqGather)
// }

pub fn gather_xread_activated(sock: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    sock.fair_queue.activated(pipe);
}

pub fn gather_xpipe_terminated(sock: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    sock.fair_queue.pipe_terminated(pipe);
}

pub fn gather_xrecv(sock: &mut ZmqSocket, msg: &mut ZmqMessage) -> anyhow::Result<()> {
    let mut rc = sock.fair_queue.recvpipe(msg, None);

    // Drop any messages with more flag
    while msg.flags() & ZMQ_MSG_MORE != 0 {
        // drop all frames of the current multi-frame message
        sock.fair_queue.recvpipe(msg, None)?;

        while msg.flags() & ZMQ_MSG_MORE != 0 {
            sock.fair_queue.recvpipe(msg, None)?;
        }

        // get the new message
        // if rc == 0 {
        //     rc = self.fair_queue.recvpipe(msg, None)?;
        // }
        sock.fair_queue.recvpipe(msg, None)?;
    }

    Ok(())
}

pub fn gather_xhas_in(sock: &mut ZmqSocket) -> bool {
    return sock.fair_queue.has_in();
}

// ZmqGather::~ZmqGather ()
// {
// }
