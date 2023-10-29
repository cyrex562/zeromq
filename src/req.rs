use crate::dealer::ZmqDealer;
use crate::defines::{MSG_MORE, ZMQ_REQ, ZMQ_REQ_CORRELATE, ZMQ_REQ_RELAXED};
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;

// pub struct ZmqReq {
//     // pub dealer: ZmqDealer,
//     pub _request_id: u32,
//     pub _strict: bool,
// }
// 
// impl ZmqReq {
//     pub fn new(options: &mut ZmqOptions, parent_: &mut ctx_t, tid_: u32, sid_: i32) -> Self {
//         options.type_ = ZMQ_REQ;
// 
//         Self {
//             dealer: ZmqDealer::new(options, parent_, tid_, sid_),
//             _request_id: 0,
//             _strict: false,
//         }
//     }
// 
//     
// }

pub unsafe fn req_xsend(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> i32 {
    //  If we've sent a request and we still haven't got the reply,
    //  we can't send another request unless the strict option is disabled.
    if (socket._receiving_reply) {
        if (socket._strict) {
            // errno = EFSM;
            return -1;
        }

        socket._receiving_reply = false;
        socket._message_begins = true;
    }

    //  First part of the request is the request routing id.
    if (socket._message_begins) {
        socket._reply_pipe = None;

        if (socket._request_id_frames_enabled) {
            socket._request_id += 1;

            // msg_t id;
            let mut id = ZmqMsg::default();
            let mut rc = id.init_size(4);
            libc::memcpy(id.data_mut(), &socket._request_id, 4);
            // errno_assert (rc == 0);
            id.set_flags(ZmqMsg::more);

            rc = socket.dealer.sendpipe(&id, &_reply_pipe);
            if (rc != 0) {
                return -1;
            }
        }

        // msg_t bottom;
        let mut bottom = ZmqMsg::default();
        let mut rc = bottom.init2();
        // errno_assert (rc == 0);
        bottom.set_flags(ZmqMsg::more);

        rc = socket.dealer.sendpipe(&mut bottom, &socket._reply_pipe);
        if (rc != 0) {
            return -1;
        }
        // zmq_assert (_reply_pipe);

        socket._message_begins = false;

        // Eat all currently available messages before the request is fully
        // sent. This is Done to avoid:
        //   REQ sends request to A, A replies, B replies too.
        //   A's reply was first and matches, that is used.
        //   An hour later REQ sends a request to B. B's old reply is used.
        // msg_t drop;
        let mut drop = ZmqMsg::default();
        loop {
            rc = drop.init2();
            // errno_assert (rc == 0);
            rc = socket.dealer.xrecv(&mut drop);
            if (rc != 0) {
                break;
            }
            drop.close();
        }
    }

    let more = (msg_.flags() & ZmqMsg::more) != 0;

    let rc = socket.dealer.xsend(msg_);
    if (rc != 0) {
        return rc;
    }

    //  If the request was fully sent, flip the FSM into reply-receiving state.
    if (!more) {
        socket._receiving_reply = true;
        socket._message_begins = true;
    }

    return 0;
}

// int zmq::req_t::xrecv (msg_t *msg_)
pub unsafe fn req_xrecv(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> i32 {
    //  If request wasn't send, we can't wait for reply.
    if (!socket._receiving_reply) {
        // errno = EFSM;
        return -1;
    }

    //  Skip messages until one with the right first frames is found.
    while (socket._message_begins) {
        //  If enabled, the first frame must have the correct request_id.
        if (socket._request_id_frames_enabled) {
            let rc = socket.recv_reply_pipe(msg_);
            if (rc != 0) {
                return rc;
            }

            if !(msg_.flags() & MSG_MORE)
                || msg_.size() != size_of_val(&socket._request_id)
                || msg_.data_mut() != socket._request_id
            {
                //  Skip the remaining frames and try the next message
                while (msg_.flags() & MSG_MORE) {
                    rc = socket.recv_reply_pipe(msg_);
                    // errno_assert (rc == 0);
                }
                continue;
            }
        }

        //  The next frame must be 0.
        // TODO: Failing this check should also close the connection with the peer!
        let mut rc = socket.recv_reply_pipe(msg_);
        if (rc != 0) {
            return rc;
        }

        if (!(msg_.flags() & MSG_MORE) || msg_.size() != 0) {
            //  Skip the remaining frames and try the next message
            while (msg_.flags() & ZmqMsg::more) {
                rc = socket.recv_reply_pipe(msg_);
                // errno_assert (rc == 0);
            }
            continue;
        }

        socket._message_begins = false;
    }

    let rc = socket.recv_reply_pipe(msg_);
    if (rc != 0) {
        return rc;
    }

    //  If the reply is fully received, flip the FSM into request-sending state.
    if (!(msg_.flags() & MSG_MORE)) {
        socket._receiving_reply = false;
        socket._message_begins = true;
    }

    return 0;
}

// bool zmq::req_t::xhas_in ()
pub unsafe fn req_xhas_in(&mut self) -> bool {
    //  TODO: Duplicates should be removed here.

    if (!socket._receiving_reply) {
        return false;
    }

    return socket.dealer.xhas_in();
}

// bool zmq::req_t::xhas_out ()
pub unsafe fn req_xhas_out(&mut self) -> bool {
    if (socket._receiving_reply && socket._strict) {
        return false;
    }

    return socket.dealer.xhas_out();
}

// int zmq::req_t::xsetsockopt (int option_,
//                          const void *optval_,
//                          size_t optvallen_)
pub unsafe fn req_xsetsockopt(socket: &mut ZmqSocket, option_: i32, optval_: &[u8], optvallen_: usize) -> i32 {
    let is_int = (optvallen_ == 4);
    let mut value = 0;
    if (is_int) {
        libc::memcpy(&value, optval_, 4);
    }

    match option_ {
        ZMQ_REQ_CORRELATE => {
            if (is_int && value >= 0) {
                socket._request_id_frames_enabled = (value != 0);
                return 0;
            }
        }
        ZMQ_REQ_RELAXED => {
            if (is_int && value >= 0) {
                socket._strict = (value == 0);
                return 0;
            }
        }
        _ => {}
    }

    return ZmqDealer::xsetsockopt(option_, optval_, optvallen_);
}

// void zmq::req_t::xpipe_terminated (pipe_t *pipe_)
pub unsafe fn req_xpipe_terminated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    if (socket._reply_pipe == pipe_) {
        socket._reply_pipe = None;
    }
    ZmqDealer::xpipe_terminated(pipe_);
}

pub unsafe fn req_recv_reply_pipe(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> i32 {
    loop {
        let mut pipe: ZmqPipe = ZmqPipe::default();
        let rc = scoekt.dealer.recvpipe(msg_, &mut Some(&mut pipe));
        if (rc != 0) {
            return rc;
        }
        if (!socket._reply_pipe || pipe == socket._reply_pipe) {
            return 0;
        }
    }
}
