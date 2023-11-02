use crate::ctx::ZmqContext;
use crate::defines::{MSG_MORE, ZMQ_SERVER};
use crate::defines::fair_queue::ZmqFairQueue;
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::out_pipe::{ZmqOutpipe, ZmqOutPipes};
use crate::pipe::ZmqPipe;
use crate::socket::ZmqSocket;

// pub struct ZmqServer<'a> {
//     pub socket_base: ZmqSocket<'a>,
//     pub _fq: ZmqFairQueue,
//     //  Acceptable inbound pipes.
//     pub _out_pipes: ZmqOutPipes,
//     //  Outbound pipes indexed by peer id.
//     pub _next_routing_id: u32, //  Next routing id to assign.
// }
// 
// impl ZmqServer {
//     pub unsafe fn new(options: &mut ZmqOptions, parent_: &mut ZmqContext, tid_: u32, sid_: i32) -> ZmqServer {
//         options.type_ = ZMQ_SERVER;
//         options.can_send_hello_msg = true;
//         options.can_recv_disconnect_msg = true;
//         Self {
//             socket_base: ZmqSocket::new(parent_, tid_, sid_),
//             _fq: ZmqFairQueue::default(),
//             _out_pipes: ZmqOutPipes::default(),
//             _next_routing_id: 0,
//         }
//     }
// 
//     
// }



pub unsafe fn server_xattach_pipe(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe, subscribe_to_all_: bool, locally_initiated_: bool) {
    let mut routing_id = socket.next_routing_id += 1;
    if (!routing_id) {
        routing_id = socket.next_routing_id += 1;
    } //  Never use Routing ID zero

    pipe_.set_server_socket_routing_id(routing_id);
    //  Add the record into output pipes lookup table
    // outpipe_t outpipe = {pipe_, true};
    let outpipe = ZmqOutpipe {
        pipe: pipe_,
        active: true,
    };
    let ok = socket.out_pipes.ZMQ_MAP_INSERT_OR_EMPLACE(routing_id, outpipe).second;
    // zmq_assert (ok);

    socket.fq.attach(pipe_);
}

pub unsafe fn server_xpipe_terminated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    // const out_pipes_t::iterator it = _out_pipes.find (pipe_->get_server_socket_routing_id ());
    let it = socket.out_pipes.find(pipe_.get_server_socket_routing_id());
    // zmq_assert (it != _out_pipes.end ());

    // _out_pipes.erase (it);
    socket.out_pipes.remove(it);

    socket.fq.pipe_terminated(pipe_);
}

pub unsafe fn server_xread_activated(socket: &mut ZmqSocket, pipe_: &mut ZmqPipe) {
    socket.fq.read_activated(pipe_);
}

pub unsafe fn server_xwrite_activated(socket: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    let end = socket.out_pipes.iter_mut().last().unwrap();

    let mut it: (&u32, &mut ZmqOutpipe);
    for i in 0..socket.out_pipes.len() {
        it = socket.out_pipes.iter_mut().nth(i).unwrap();
        if it.1.pipe == pipe {
            it.1.active = true;
            break;
        }
    }
}

pub fn server_xsend(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> i32 {
    //  SERVER sockets do not allow multipart data (ZMQ_SNDMORE)
    if msg_.flag_set(MSG_MORE) {
        // errno = EINVAL;
        return -1;
    }
    //  Find the pipe associated with the routing stored in the message.
    let mut routing_id = msg_.get_routing_id();
    let it = socket.out_pipes.iter_mut().find(routing_id).unwrap();

    if (it != socket.out_pipes.iter_mut().end()) {
        if (!it.1.pipe.check_write()) {
            it.1.active = false;
            // errno = EAGAIN;
            return -1;
        }
    } else {
        // errno = EHOSTUNREACH;
        return -1;
    }

    //  Message might be delivered over inproc, so we reset routing id
    let mut rc = msg_.reset_routing_id();
    // errno_assert (rc == 0);

    let ok = it.1.pipe.write(msg_);
    if ((!ok)) {
        // Message failed to send - we must close it ourselves.
        rc = msg_.close();
        // errno_assert (rc == 0);
    } else it.1.pipe.flush();

    //  Detach the message from the data buffer.
    rc = msg_.init2();
    // errno_assert (rc == 0);

    return 0;
}

pub unsafe fn server_xrecv(socket: &mut ZmqSocket, msg_: &mut ZmqMsg) -> i32 {
    // pipe_t *pipe = NULL;
    let mut pipe= ZmqPipe::default();
    let mut rc = socket.fq.recvpipe (msg_, &mut Some(&mut pipe));

    // Drop any messages with more flag
    // while (rc == 0 && msg_->flags () & msg_t::more)
    while rc == 0 && msg_.flag_set(MSG_MORE)
    {
        // drop all frames of the current multi-frame message
        rc = socket.fq.recvpipe (msg_, &mut None);

        // while (rc == 0 && msg_->flags () & msg_t::more)
        while rc == 0 && msg_.flag_set(MSG_MORE)
        {
            rc = socket.fq.recvpipe(msg_, &mut None);
        }

        // get the new message
        if (rc == 0) {
            rc = socket.fq.recvpipe(msg_, &mut Some(&mut pipe));
        }
    }

    if (rc != 0) {
        return rc;
    }

    // zmq_assert (pipe != NULL);

    let routing_id = pipe.get_server_socket_routing_id ();
    msg_.set_routing_id (routing_id as i32);

    return 0;
}

pub fn server_xhas_in (socket: &mut ZmqSocket) -> bool
{
    return socket.fq.has_in ();
}

pub fn server_xhas_out(socket: &mut ZmqSocket) -> bool {
    true
}


pub fn server_xsetsockopt(socket: &mut ZmqSocket, option_: i32, optval_: &[u8], optvallen_: usize) -> i32 {
    unimplemented!()
}

pub fn server_xgetsockopt(socket: &mut ZmqSocket, option: u32) -> Result<[u8], ZmqError> {
    unimplemented!();
}

pub fn server_xjoin(socket: &mut ZmqSocket, group: &str) -> i32 {
    unimplemented!();
}
