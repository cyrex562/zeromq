use crate::ctx::ZmqContext;

use crate::defines::blob::ZmqBlob;
use crate::defines::{MSG_MORE, ZMQ_NOTIFY_CONNECT, ZMQ_NOTIFY_DISCONNECT, ZMQ_POLLOUT, ZMQ_PROBE_ROUTER, ZMQ_ROUTER_HANDOVER, ZMQ_ROUTER_MANDATORY, ZMQ_ROUTER_NOTIFY, ZMQ_ROUTER_RAW};
use crate::defines::err::ZmqError;
use crate::defines::err::ZmqError::SocketError;
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;
use crate::socket::ZmqSocket;
use crate::utils::put_u32;

// pub struct ZmqRouter<'a> {
//     pub routing_socket_base: ZmqRoutingSocketBase<'a>,
//     pub _fq: ZmqFairQueue<'a>,
//     pub _prefetched: bool,
//     pub _routing_id_sent: bool,
//     pub _prefetched_id: ZmqMsg<'a>,
//     pub _prefetched_msg: ZmqMsg<'a>,
//     pub _current_in: &'a mut ZmqPipe<'a>,
//     pub _terminate_current_in: bool,
//     pub _more_in: bool,
//     pub _anonymous_pipes: HashSet<&'a mut ZmqPipe<'a>>,
//     pub _current_out: &'a mut ZmqPipe<'a>,
//     pub _more_out: bool,
//     pub _next_integral_routing_id: u32,
//     pub _mandatory: bool,
//     pub _raw_socket: bool,
//     pub _probe_router: bool,
//     pub _handover: bool,
// }

// impl ZmqRouter {
//     pub unsafe fn new(options: &mut ZmqOptions, parent_: &mut ZmqContext, tid_: u32, sid_: i32) -> Self {
//         options.type_ = ZMQ_ROUTER;
//         options.recv_routing_id = true;
//         options.raw_socket = false;
//         options.can_send_hello_msg = true;
//         options.can_recv_disconnect_msg = true;
//         let mut out = Self {
//             routing_socket_base: ZmqRoutingSocketBase::new(parent_, tid_, sid_),
//             _fq: ZmqFairQueue::default(),
//             _prefetched: false,
//             _routing_id_sent: false,
//             _prefetched_id: Default::default(),
//             _prefetched_msg: Default::default(),
//             _current_in: &mut Default::default(),
//             _terminate_current_in: false,
//             _more_in: false,
//             _anonymous_pipes: Default::default(),
//             _current_out: &mut Default::default(),
//             _more_out: false,
//             _next_integral_routing_id: 0,
//             _mandatory: false,
//             _raw_socket: false,
//             _probe_router: false,
//             _handover: false,
//         };
//         out._prefetched_id.init2();
//         out._prefetched_msg.init2();
//         out
//     }
// 
//     
// }


// void zmq::router_t::xattach_pipe (pipe_t *pipe_,
//                               bool subscribe_to_all_,
//                               bool locally_initiated_)
pub fn router_xattach_pipe(ctx: &mut ZmqContext, socket: &mut ZmqSocket, pipe_: &mut ZmqPipe, subscribe_to_all_: bool, locally_initiated_: bool) {
    // LIBZMQ_UNUSED (subscribe_to_all_);

    // zmq_assert (pipe_);

    if socket.probe_router {
        // msg_t probe_msg;
        let mut probe_msg = ZmqMsg::default();
        let mut rc = probe_msg.init2();
        // errno_assert (rc == 0);

        rc = pipe_.write(&mut probe_msg);
        // zmq_assert (rc) is not applicable here, since it is not a bug.
        // LIBZMQ_UNUSED (rc);

        pipe_.flush(ctx);

        rc = probe_msg.close();
        // errno_assert (rc == 0);
    }

    let routing_id_ok = socket.identify_peer(pipe_, locally_initiated_);
    if routing_id_ok {
        socket.fq.attach(pipe_);
    } else {
        socket.anonymous_pipes.insert(pipe_);
    }
}

// int zmq::router_t::xsetsockopt (int option_,
//                             const void *optval_,
//                             size_t optvallen_)
pub fn router_xsetsockopt(socket: &mut ZmqSocket, options: &mut ZmqOptions, option_: i32, optval_: &mut [u8], optvallen_: usize) -> Result<(),ZmqError> {
    // const bool is_int = (optvallen_ == sizeof (int));
    let is_int = optvallen_ == 4;
    let mut value = 0;
    if is_int {
        // libc::memcpy(&value, optval_, 4);
        value = i32::from_le_bytes(optval_[0..4].try_into().unwrap());
    }

    match option_ {
        ZMQ_ROUTER_RAW => {
            if is_int && value >= 0 {
                socket.raw_socket = (value != 0);
                if socket.raw_socket {
                    options.recv_routing_id = false;
                    options.raw_socket = true;
                }
                return Ok(());
            }
        }

        ZMQ_ROUTER_MANDATORY => {
            if is_int && value >= 0 {
                socket.mandatory = (value != 0);
                return Ok(());
            }
        }

        ZMQ_PROBE_ROUTER => {
            if is_int && value >= 0 {
                socket.probe_router = (value != 0);
                return Ok(());
            }
        }

        ZMQ_ROUTER_HANDOVER => {
            if is_int && value >= 0 {
                socket.handover = (value != 0);
                return Ok(());
            }
        }


        // #ifdef ZMQ_BUILD_DRAFT_API
        ZMQ_ROUTER_NOTIFY => {
            if is_int && value >= 0 && value <= (ZMQ_NOTIFY_CONNECT | ZMQ_NOTIFY_DISCONNECT) as i32 {
                options.router_notify = value;
                return Ok(());
            }
        }

        // #endif

        _ => {
            return socket.xsetsockopt(options, option_, optval_,
                                                     optvallen_);
        }
    }
    // errno = EINVAL;
    return Err(SocketError("EINVAL"));
}

// void zmq::router_t::xpipe_terminated (pipe_t *pipe_)
pub fn router_xpipe_terminated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    if 0 == socket.anonymous_pipes.erase(pipe_) {
        socket.erase_out_pipe(pipe_);
        socket.fq.pipe_terminated(pipe_);
        pipe_.rollback();
        if pipe_ == socket.current_out {
            socket.current_out = None;
        }
    }
}

// void zmq::router_t::xread_activated (pipe_t *pipe_)
pub fn router_xread_activated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) -> Result<(),ZmqError> {
    // const std::set<pipe_t *>::iterator it = _anonymous_pipes.find (pipe_);
    let it = socket.anonymous_pipes.iter_mut().find(|&x| *x == pipe_);
    if it.is_none() {
        socket.fq.activated(pipe_)?;
    } else {
        let routing_id_ok = socket.identify_peer(pipe_, false);
        if routing_id_ok {
            socket.anonymous_pipes.erase(it);
            socket.fq.attach(pipe_);
        }
    }

    Ok(())
}

// int zmq::router_t::xsend (msg_t *msg_)
pub fn router_xsend(options: &ZmqOptions, socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> Result<(),ZmqError> {
    //  If this is the first part of the message it's the ID of the
    //  peer to send the message to.
    if !socket.more_out {
        // zmq_assert (!_current_out);

        //  If we have malformed message (prefix with no subsequent message)
        //  then just silently ignore it.
        //  TODO: The connections should be killed instead.
        if msg_.flags() & MSG_MORE {
            socket.more_out = true;

            //  Find the pipe associated with the routing id stored in the prefix.
            //  If there's no such pipe just silently ignore the message, unless
            //  router_mandatory is set.
            // let mut out_pipe = socket.lookup_out_pipe(
            //     ZmqBlob::new3((msg_.data_mut()), msg_.size()));
            let mut out_pipe = socket.lookup_out_pipe(vec![]);

            if out_pipe {
                socket.current_out = Some(out_pipe.pipe);

                // Check whether pipe is closed or not
                if !socket.current_out.check_write() {
                    // Check whether pipe is full or not
                    let pipe_full = !socket.current_out.check_hwm();
                    out_pipe.active = false;
                    socket.current_out = None;

                    if socket.mandatory {
                        socket.more_out = false;
                        if pipe_full {
                            // errno = EAGAIN;
                        } else {
                            // errno = EHOSTUNREACH;
                        }
                        return Err(SocketError("EAGAIN"));
                    }
                }
            } else if socket.mandatory {
                socket.more_out = false;
                // errno = EHOSTUNREACH;
                return Err(SocketError("EHOSTUNREACH"));
            }
        }

        msg_.close()?;
        // errno_assert (rc == 0);
        msg_.init2()?;
        // errno_assert (rc == 0);
        return Ok(());
    }

    //  Ignore the MORE flag for raw-sock or assert?
    if options.raw_socket {
        msg_.reset_flags(MSG_MORE as u8);
    }

    //  Check whether this is the last part of the message.
    socket.more_out = (msg_.flags() & MSG_MORE) != 0;

    //  Push the message into the pipe. If there's no out pipe, just drop it.
    if socket.current_out {
        // Close the remote connection if user has asked to do so
        // by sending zero length message.
        // Pending messages in the pipe will be dropped (on receiving Term- ack)
        if socket.raw_socket && msg_.size() == 0 {
            socket.current_out.terminate(false);
            let mut rc = msg_.close();
            // errno_assert (rc == 0);
            rc = msg_.init2();
            // errno_assert (rc == 0);
            socket.current_out = None;
            return Ok(());
        }

        let ok = socket.current_out.write(msg_);
        if !ok {
            // Message failed to send - we must close it ourselves.
            let rc = msg_.close();
            // errno_assert (rc == 0);
            // HWM was checked before, so the pipe must be gone. Roll back
            // messages that were piped, for example REP labels.
            socket.current_out.rollback();
            socket.current_out = None;
        } else {
            if !socket.more_out {
                socket.current_out.flush();
                socket.current_out = None;
            }
        }
    } else {
        let rc = msg_.close();
        // errno_assert (rc == 0);
    }

    //  Detach the message from the data buffer.
    let rc = msg_.init2();
    // errno_assert (rc == 0);

    return Ok(());
}

// int zmq::router_t::xrecv (msg_t *msg_)
pub fn router_xrecv(ctx: &mut ZmqContext, socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> Result<(),ZmqError> {
    if socket.prefetched {
        if !socket.routing_id_sent {
            // TODO
            // let rc = msg_. move (socket.prefetched_id);
            // errno_assert (rc == 0);
            socket.routing_id_sent = true;
        } else {
            // TODO
            // let rc = msg_.move(socket.prefetched_msg);
            // errno_assert (rc == 0);
            socket.prefetched = false;
        }
        socket.more_in = (msg_.flags() & ZmqMsg::more) != 0;

        if !socket.more_in {
            if socket.terminate_current_in {
                socket.current_in.terminate(true);
                socket.terminate_current_in = false;
            }
            socket.current_in = None;
        }
        Ok(())
    }

    // pipe_t *pipe = NULL;
    // let mut pipe: Option<&mut ZmqPipe> = None;
    let mut pipe: Option<&mut ZmqPipe> = Some(&mut ZmqPipe::default());
    socket.fq.recvpipe(ctx, msg_, &mut pipe)?;

    //  It's possible that we receive peer's routing id. That happens
    //  after reconnection. The current implementation assumes that
    //  the peer always uses the same routing id.
    while msg_.is_routing_id() {
        socket.fq.recvpipe(ctx, msg_, &mut pipe)?;
    }

    // if rc != 0 {
    //     return -1;
    // }

    // zmq_assert (pipe != NULL);

    //  If we are in the middle of reading a message, just return the next part.
    if socket.more_in {
        socket.more_in = (msg_.flags() & ZmqMsg::more) != 0;

        if !socket.more_in {
            if socket.terminate_current_in {
                socket.current_in.terminate(true);
                socket.terminate_current_in = false;
            }
            socket.current_in = None;
        }
    } else {
        //  We are at the beginning of a message.
        //  Keep the message part we have in the prefetch buffer
        //  and return the ID of the peer instead.
        // TODO
        // rc = socket.prefetched_msg. move (*msg_);
        // errno_assert (rc == 0);
        socket.prefetched = true;
        socket.current_in = pipe;

        let routing_id = pipe.unwrap().get_routing_id();
        msg_.init_size(routing_id.size())?;
        // errno_assert (rc == 0);
        // libc::memcpy(msg_.data_mut(), routing_id.data(), routing_id.size());
        msg_.data_mut().clone_from_slice(routing_id.data());
        msg_.set_flags(ZmqMsg::more);
        if socket.prefetched_msg.metadata() {
            msg_.set_metadata(socket.prefetched_msg.metadata());
        }
        socket.routing_id_sent = true;
    }

    Ok(())
}

// int zmq::router_t::rollback ()
pub fn router_rollback(socket: &mut ZmqSocket) -> Result<(),ZmqError> {
    if socket.current_out {
        socket.current_out.rollback();
        socket.current_out = None;
        socket.more_out = false;
    }
    return Ok(());
}

// bool zmq::router_t::xhas_in ()
pub  fn router_xhas_in(ctx: &mut ZmqContext, socket: &mut ZmqSocket) -> bool {
    //  If we are in the middle of reading the messages, there are
    //  definitely more parts available.
    if socket.more_in {
        return true;
    }

    //  We may already have a message pre-fetched.
    if socket.prefetched {
        return true;
    }

    //  Try to read the next message.
    //  The message, if read, is kept in the pre-fetch buffer.
    let mut pipe: Option<&mut ZmqPipe> = None;
    socket.fq.recvpipe(ctx, &mut socket.prefetched_msg, &mut pipe)?;

    //  It's possible that we receive peer's routing id. That happens
    //  after reconnection. The current implementation assumes that
    //  the peer always uses the same routing id.
    //  TODO: handle the situation when the peer changes its routing id.
    while socket.prefetched_msg.is_routing_id() {
        socket.fq.recvpipe(ctx, &mut socket.prefetched_msg, &mut pipe)?;
    }

    // if rc != 0 {
    //     return false;
    // }

    // zmq_assert (pipe != NULL);

    let routing_id = pipe.unwrap().get_routing_id();
    socket.prefetched_id.init_size(routing_id.size())?;
    // errno_assert (rc == 0);
    // libc::memcpy(socket._prefetched_id.data_mut(), routing_id.data(), routing_id.size());
    socket.prefetched_id.data_mut().clone_from_slice(routing_id.data());
    socket.prefetched_id.set_flags(MSG_MORE as u8);
    if socket.prefetched_msg.metadata() {
        socket.prefetched_id.set_metadata(socket.prefetched_msg.metadata());
    }

    socket.prefetched = true;
    socket.routing_id_sent = false;
    socket.current_in = pipe;

    return true;
}

pub fn router_check_pipe_hwm(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) -> bool {
    return pipe_.check_hwm();
}

// bool zmq::router_t::xhas_out ()
pub  fn router_xhas_out(socket: &mut ZmqSocket) -> bool {
    //  In theory, ROUTER socket is always Ready for writing (except when
    //  MANDATORY is set). Whether actual attempt to write succeeds depends
    //  on which pipe the message is going to be routed to.

    if !socket.mandatory {
        return true;
    }

    return socket.any_of_out_pipes(socket.check_pipe_hwm);
}

// int zmq::router_t::get_peer_state (const void *routing_id_,
//                                size_t routing_id_size_) const
pub fn router_get_peer_state(socket: &mut ZmqSocket, routing_id_: &[u8], routing_id_size_: usize) -> Result<i32,ZmqError> {
    let mut res: i32 = 0;

    // TODO remove the const_cast, see comment in lookup_out_pipe
    // let routing_id_blob = ZmqBlob::new3(
    //     routing_id_,
    //     routing_id_size_);
    let out_pipe = socket.lookup_out_pipe(vec![]);
    if !out_pipe {
        // errno = EHOSTUNREACH;
        return Err(SocketError("EHOSTUNREACH"));
    }

    if out_pipe.pipe.check_hwm() {
        res |= ZMQ_POLLOUT;
    }

    /** \todo does it make any sense to check the inpipe as well? */

    return Ok(res);
}

// bool zmq::router_t::identify_peer (pipe_t *pipe_, bool locally_initiated_)
pub fn router_identify_peer(ctx: &mut ZmqContext, socket: &mut ZmqSocket, options: &mut ZmqOptions, pipe_: &mut ZmqPipe, locally_initiated_: bool) -> bool {
    // msg_t msg;
    let mut msg = ZmqMsg::default();
    // blob_t routing_id;
    let mut routing_id = vec![];

    if locally_initiated_ && socket.connect_routing_id_is_set() {
        let connect_routing_id = socket.extract_connect_routing_id();
        routing_id.set(connect_routing_id.c_str(),
                       connect_routing_id.length());
        //  Not allowed to duplicate an existing rid
        // zmq_assert (!has_out_pipe (routing_id));
    } else if options.raw_socket { //  Always assign an integral routing id for raw-socket
        // unsigned char buf[5];
        let mut buf = [0u8; 5];
        buf[0] = 0;
        put_u32(&mut buf[1..], socket.next_integral_routing_id);
        socket.next_integral_routing_id += 1;
        routing_id.set(buf, 5);
    } else if !options.raw_socket {
        //  Pick up handshake cases and also case where next integral routing id is set
        msg.init2()?;
        pipe_.read(ctx, &mut msg)?;
        // if (!ok) {
        //     return false;
        // }

        if msg.size() == 0 {
            //  Fall back on the auto-generation
            // unsigned char buf[5];
            let mut buf = [0u8; 5];
            buf[0] = 0;
            put_u32(&mut buf[1..], socket.next_integral_routing_id);           socket.next_integral_routing_id += 1;
            routing_id.set(buf, 5);
            msg.close()?;
        } else {
            routing_id.set((msg.data_mut()),
                           msg.size());
            msg.close()?;

            //  Try to remove an existing routing id entry to allow the new
            //  connection to take the routing id.
            let existing_outpipe = socket.lookup_out_pipe(routing_id);

            if existing_outpipe {
                if !socket.handover {
                    //  Ignore peers with duplicate ID
                    return false;
                }

                //  We will allow the new connection to take over this
                //  routing id. Temporarily assign a new routing id to the
                //  existing pipe so we can terminate it asynchronously.
                let mut buf = [0u8; 5];
                buf[0] = 0;
                put_u32(&mut buf[1..], socket.next_integral_routing_id);
                socket.next_integral_routing_id += 1;
                // let mut new_routing_id = ZmqBlob::new3(&buf, 5);
                let mut new_routing_id: Vec<u8> = vec![];

                let old_pipe = existing_outpipe.pipe;

                socket.erase_out_pipe(old_pipe);
                old_pipe.set_router_socket_routing_id(&mut new_routing_id);
                socket.add_out_pipe(&mut new_routing_id, old_pipe);

                if old_pipe == socket.current_in {
                    socket.terminate_current_in = true;
                } else {
                    old_pipe.terminate(ctx, true);
                }
            }
        }
    }

    pipe_.set_router_socket_routing_id(&mut routing_id);
    socket.add_out_pipe(&routing_id, pipe_);

    return true;
}


pub fn router_xgetsockopt(socket: &mut ZmqSocket, option: u32) -> Result<Vec<u8>, ZmqError> {
    unimplemented!();
}

pub fn router_xjoin(socket: &mut ZmqSocket, group: &str) -> Result<(),ZmqError> {
    unimplemented!();
}

pub fn router_xwrite_activated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    unimplemented!()
}
