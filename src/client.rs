use anyhow::anyhow;

use crate::fair_queue::ZmqFq;
use crate::lb::LoadBalancer;
use crate::message::{ZmqMessage, ZMQ_MSG_MORE};
use crate::pipe::ZmqPipe;
use crate::socket::ZmqSocket;

#[derive(Default, Debug, Clone)]
pub struct ZmqClient<'a> {
    //
    //  Messages are fair-queued from inbound pipes. And load-balanced to
    //  the outbound pipes.
    pub fair_queue: ZmqFq,
    pub load_balancer: LoadBalancer,
    pub base: ZmqSocket<'a>, // // ZMQ_NON_COPYABLE_NOR_MOVABLE (client_t)
}

// fn client_xattach_pipe(
//
//     sock: &mut ZmqSocket,
//     pipe: &mut ZmqPipe,
//     subscribe_to_all: bool,
//     locally_initiated: bool,
// ) {
//     // LIBZMQ_UNUSED(subscribe_to_all_);
//     // LIBZMQ_UNUSED(locally_initiated_);
//
//     // zmq_assert(pipe_);
//
//     self.fq.attach(pipe);
//     self.lb.attach(pipe);
// }

pub fn client_xsend(sock: &mut ZmqSocket, msg: &mut ZmqMessage) -> anyhow::Result<()> {
    //  CLIENT sockets do not allow multipart data (ZMQ_SNDMORE)
    if (msg.flags() & ZMQ_MSG_MORE) {
        // errno = EINVAL;
        // return -1;
        return Err(anyhow!(
            "EINVAL: client sockets do not allow multipart dart (ZMQ_SNDMORE)"
        ));
    }
    sock.lb.sendpipe(msg, None)
}

pub fn client_xrecv(sock: &mut ZmqSocket, msg: &mut ZmqMessage) -> anyhow::Result<()> {
    let mut rc = sock.fq.recvpipe(msg, None);

    // Drop any messages with more flag
    while msg.flag_set(ZMQ_MSG_MORE) {
        // drop all frames of the current multi-frame message
        sock.fq.recvpipe(msg, None)?;

        while msg.flags() & ZMQ_MSG_MORE != 0 {
            sock.fq.recvpipe(msg, None)?;
        }

        // get the new message
        // if (rc == 0) {
        //     fair_queue.recvpipe(msg, null_mut());
        // }
        sock.fq.recvpipe(msg, None)?;
    }

    Ok(())
}

pub fn client_xhas_in(sock: &mut ZmqSocket) -> bool {
    return sock.fq.has_in();
}

pub fn client_xhas_out(sock: &mut ZmqSocket) -> bool {
    return sock.lb.has_out();
}

pub fn client_xread_activated(sock: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    sock.fq.activated(pipe);
}

pub fn client_xwrite_activated(sock: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    sock.lb.activated(pipe);
}

pub fn client_xpipe_terminated(sock: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    sock.fq.pipe_terminated(pipe);
    sock.lb.pipe_terminated(pipe);
}
