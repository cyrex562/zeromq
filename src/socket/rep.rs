use crate::defines::{ZMQ_MSG_MORE, ZMQ_REP};
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::socket::ZmqSocket;

pub struct ZmqRep {
    pub router: router_t,
    pub _sending_reply: bool,
    pub _request_begins: bool,
}

impl ZmqRep {
    pub fn new(options: &mut ZmqOptions, ctx: &mut crate::ctx::ZmqContext, sid: i32) -> Self {
        let router = router_t::new(ctx, tid, sid);
        options.type_ = ZMQ_REP;
        Self {
            router,
            _sending_reply: false,
            _request_begins: true,
        }
    }
}

pub fn rep_xsetsockopt(
    socket: &mut ZmqSocket,
    option_: i32,
    optval_: &[u8],
    optvallen_: usize,
) -> i32 {
    unimplemented!()
}

pub fn rep_xattach_pipe(
    socket: &mut ZmqSocket,
    pipe_: &mut ZmqPipe,
    subscribe_to_all_: bool,
    locally_initiated_: bool,
) {
    unimplemented!()
}

pub fn rep_xsend(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> i32 {
    //  If we are in the middle of receiving a request, we cannot send reply.
    if !socket._sending_reply {
        // errno = EFSM;
        return -1;
    }

    let more = flag_set(msg_.flags(), ZMQ_MSG_MORE);

    //  Push message to the reply pipe.
    let rc = socket.router.xsend(msg_);
    if rc != 0 {
        return rc;
    }

    //  If the reply is complete flip the FSM back to request receiving state.
    if !more {
        socket._sending_reply = false;
    }

    return 0;
}

// int zmq::rep_t::xrecv (msg_t *msg_)
pub unsafe fn rep_xrecv(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> i32 {
    //  If we are in middle of sending a reply, we cannot receive next request.
    if socket._sending_reply {
        // errno = EFSM;
        return -1;
    }

    //  First thing to do when receiving a request is to copy all the labels
    //  to the reply pipe.
    if (socket._request_begins) {
        loop {
            let rc = socket.router.xrecv(msg_);
            if rc != 0 {
                return rc;
            }

            if msg_.flags() & ZMQ_MSG_MORE {
                //  Empty message part delimits the traceback stack.
                let bottom = (msg_.size() == 0);

                //  Push it to the reply pipe.
                rc = socket.router.xsend(msg_);
                // errno_assert (rc == 0);

                if bottom {
                    break;
                }
            } else {
                //  If the traceback stack is malformed, discard anything
                //  already sent to pipe (we're at end of invalid message).
                rc = socket.router.rollback();
                // errno_assert (rc == 0);
            }
        }
        socket._request_begins = false;
    }

    //  Get next message part to return to the user.
    let rc = socket.router.xrecv(msg_);
    if rc != 0 {
        return rc;
    }

    //  If whole request is read, flip the FSM to reply-sending state.
    if !(msg_.flags() & ZMQ_MSG_MORE) {
        socket._sending_reply = true;
        socket._request_begins = true;
    }

    return 0;
}

// bool zmq::rep_t::xhas_in ()
pub fn rep_xhas_in(socket: &mut ZmqSocket) -> bool {
    if socket._sending_reply {
        return false;
    }

    return socket.router.xhas_in();
}

// bool zmq::rep_t::xhas_out ()
pub fn rep_xhas_out(socket: &mut ZmqSocket) -> bool {
    if !socket._sending_reply {
        return false;
    }

    return socket.xhas_out();
}

pub fn rep_xgetsockopt(socket: &mut ZmqSocket, option: u32) -> Result<[u8], ZmqError> {
    unimplemented!();
}

pub fn rep_xjoin(socket: &mut ZmqSocket, group: &str) -> i32 {
    unimplemented!();
}

pub fn rep_xread_activated(socket: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    unimplemented!()
}

pub fn rep_xwrite_activated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    unimplemented!()
}

pub fn rep_xpipe_terminated(socket: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    unimplemented!()
}
