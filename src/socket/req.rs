use std::mem::size_of_val;
use crate::ctx::ZmqContext;
use crate::defines::{ZMQ_MSG_MORE, ZMQ_REQ_CORRELATE, ZMQ_REQ_RELAXED};
use crate::defines::err::ZmqError;
use crate::err::ZmqError;
use crate::err::ZmqError::SocketError;
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;
use crate::socket::dealer::{dealer_xpipe_terminated, dealer_xsetsockopt};
use crate::socket::ZmqSocket;

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

pub fn req_xattach_pipe(
    socket: &mut ZmqSocket,
    pipe_: &mut ZmqPipe,
    subscribe_to_all_: bool,
    locally_initiated_: bool,
) {
    unimplemented!()
}

pub fn req_xsend(ctx: &mut ZmqContext, options: &mut ZmqOptions, socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> Result<(),ZmqError> {
    //  If we've sent a request and we still haven't got the reply,
    //  we can't send another request unless the strict option is disabled.
    if socket.receiving_reply {
        if socket.strict {
            // errno = EFSM;
            return Err(SocketError("EFSM"));
        }

        socket.receiving_reply = false;
        socket.message_begins = true;
    }

    //  First part of the request is the request routing id.
    if socket.message_begins {
        socket.reply_pipe = None;

        if socket.request_id_frames_enabled {
            socket.request_id += 1;

            // msg_t id;
            let mut id = ZmqMsg::default();
            id.init_size(4)?;
            // libc::memcpy(id.data_mut(), &socket._request_id, 4);
            id.data_mut().clone_from_slice(&socket.request_id.to_be_bytes());
            // errno_assert (rc == 0);
            id.set_flags(ZmqMsg::more);

            socket.sendpipe(&id, &socket.reply_pipe)?;
            // if rc != 0 {
            //     return -1;
            // }
        }

        // msg_t bottom;
        let mut bottom = ZmqMsg::default();
        bottom.init2()?;
        // errno_assert (rc == 0);
        bottom.set_flags(ZmqMsg::more);

        socket.sendpipe(&mut bottom, &socket.reply_pipe)?;
        // if rc != 0 {
        //     return -1;
        // }
        // zmq_assert (_reply_pipe);

        socket.message_begins = false;

        // Eat all currently available messages before the request is fully
        // sent. This is Done to avoid:
        //   REQ sends request to A, A replies, B replies too.
        //   A's reply was first and matches, that is used.
        //   An hour later REQ sends a request to B. B's old reply is used.
        // msg_t drop;
        let mut drop = ZmqMsg::default();
        loop {
            drop.init2()?;
            // errno_assert (rc == 0);
            socket.xrecv(ctx, options, &mut drop)?;
            drop.close()?;
        }
    }

    let more = (msg_.flags() & ZmqMsg::more) != 0;

    socket.xsend(ctx, options, msg_)?;

    //  If the request was fully sent, flip the FSM into reply-receiving state.
    if !more {
        socket.receiving_reply = true;
        socket.message_begins = true;
    }

    Ok(())
}

// int zmq::req_t::xrecv (msg_t *msg_)
pub fn req_xrecv(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> Result<(),ZmqError> {
    //  If request wasn't send, we can't wait for reply.
    if !socket.receiving_reply {
        // errno = EFSM;
        return Err(SocketError("EFSM"));
    }

    //  Skip messages until one with the right first frames is found.
    while socket.message_begins {
        //  If enabled, the first frame must have the correct request_id.
        if socket.request_id_frames_enabled {
            let rc = socket.recv_reply_pipe(msg_);
            if rc != 0 {
                return rc;
            }

            if (msg_.flags() & ZMQ_MSG_MORE) == 0
                || msg_.size() != size_of_val(&socket.request_id)
                || msg_.data_mut() != socket.request_id
            {
                //  Skip the remaining frames and try the next message
                while msg_.flags() & ZMQ_MSG_MORE {
                    rc = socket.recv_reply_pipe(msg_);
                    // errno_assert (rc == 0);
                }
                continue;
            }
        }

        //  The next frame must be 0.
        // TODO: Failing this check should also close the connection with the peer!
        let mut rc = socket.recv_reply_pipe(msg_);
        if rc != 0 {
            return rc;
        }

        if (msg_.flags() & ZMQ_MSG_MORE) == 0 || msg_.size() != 0 {
            //  Skip the remaining frames and try the next message
            while msg_.flags() & ZmqMsg::more {
                rc = socket.recv_reply_pipe(msg_);
                // errno_assert (rc == 0);
            }
            continue;
        }

        socket.message_begins = false;
    }

    let rc = socket.recv_reply_pipe(msg_);
    if rc != 0 {
        return rc;
    }

    //  If the reply is fully received, flip the FSM into request-sending state.
    if !(msg_.flags() & ZMQ_MSG_MORE) {
        socket.receiving_reply = false;
        socket.message_begins = true;
    }

    return Ok(());
}

// bool zmq::req_t::xhas_in ()
pub fn req_xhas_in(socket: &mut ZmqSocket) -> bool {
    //  TODO: Duplicates should be removed here.

    if !socket.receiving_reply {
        return false;
    }

    return socket.xhas_in();
}

// bool zmq::req_t::xhas_out ()
pub fn req_xhas_out(socket: &mut ZmqSocket) -> bool {
    if socket.receiving_reply && socket.strict {
        return false;
    }

    return socket.xhas_out();
}

// int zmq::req_t::xsetsockopt (int option_,
//                          const void *optval_,
//                          size_t optvallen_)
pub fn req_xsetsockopt(
    socket: &mut ZmqSocket,
    option_: i32,
    optval_: &[u8],
    optvallen_: usize,
) -> Result<(),ZmqError> {
    let is_int = (optvallen_ == 4);
    let mut value = 0;
    if is_int {
        // libc::memcpy(&value, optval_, 4);
        value = i32::from_le_bytes(optval_[0..4].try_into().unwrap());
    }

    match option_ {
        ZMQ_REQ_CORRELATE => {
            if is_int && value >= 0 {
                socket.request_id_frames_enabled = (value != 0);
                return Ok(());
            }
        }
        ZMQ_REQ_RELAXED => {
            if is_int && value >= 0 {
                socket.strict = (value == 0);
                return Ok(());
            }
        }
        _ => {}
    }

    return dealer_xsetsockopt(socket, option_, optval_, optvallen_);
}

// void zmq::req_t::xpipe_terminated (pipe_t *pipe_)
pub fn req_xpipe_terminated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    if socket.reply_pipe.unwrap() == pipe_ {
        socket.reply_pipe = None;
    }
    dealer_xpipe_terminated(socket, pipe_);
}

pub fn req_recv_reply_pipe(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> Result<(),ZmqError> {
    loop {
        let mut pipe: ZmqPipe = ZmqPipe::default();
        socket.recvpipe(msg_, &mut Some(&mut pipe))?;
        // if rc != 0 {
        //     return rc;
        // }
        if socket.reply_pipe.is_none() || pipe == *socket.reply_pipe.unwrap() {
            return Ok(());
        }
    }
}

pub fn req_xgetsockopt(socket: &mut ZmqSocket, option: u32) -> Result<[u8], ZmqError> {
    unimplemented!();
}

pub fn req_xjoin(socket: &mut ZmqSocket, group: &str) -> Result<(),ZmqError> {
    unimplemented!();
}

pub fn req_xread_activated(socket: &mut ZmqSocket, pipe: &mut ZmqPipe) -> Result<(),ZmqError> {
    unimplemented!()
}

pub fn req_xwrite_activated(socket: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    unimplemented!()
}
