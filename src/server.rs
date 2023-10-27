use std::collections::HashMap;
use crate::ctx::ZmqContext;
use crate::defines::{MSG_MORE, ZMQ_SERVER};
use crate::fair_queue::ZmqFairQueue;
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;
use crate::socket_base::ZmqSocketBase;

pub struct ZmqOutpipe<'a> {
    pub pipe: &'a mut ZmqPipe<'a>,
    pub active: bool,
}

pub type ZmqOutPipes = HashMap<u32, ZmqOutpipe>;

pub struct ZmqServer<'a> {
    pub socket_base: ZmqSocketBase<'a>,
    pub _fq: ZmqFairQueue,
    //  Acceptable inbound pipes.
    pub _out_pipes: ZmqOutPipes,
    //  Outbound pipes indexed by peer id.
    pub _next_routing_id: u32, //  Next routing id to assign.
}

impl ZmqServer {
    pub unsafe fn new(options: &mut ZmqOptions, parent_: &mut ZmqContext, tid_: u32, sid_: i32) -> ZmqServer {
        options.type_ = ZMQ_SERVER;
        options.can_send_hello_msg = true;
        options.can_recv_disconnect_msg = true;
        Self {
            socket_base: ZmqSocketBase::new(parent_, tid_, sid_),
            _fq: ZmqFairQueue::default(),
            _out_pipes: ZmqOutPipes::default(),
            _next_routing_id: 0,
        }
    }

    pub unsafe fn xattach_pipe(&mut self, pipe_: &mut ZmqPipe, subscribe_to_all_: bool, locally_initiated_: bool) {
        let mut routing_id = self._next_routing_id += 1;
        if (!routing_id) {
            routing_id = self._next_routing_id += 1;
        } //  Never use Routing ID zero

        pipe_.set_server_socket_routing_id(routing_id);
        //  Add the record into output pipes lookup table
        // outpipe_t outpipe = {pipe_, true};
        let outpipe = ZmqOutpipe {
            pipe: pipe_,
            active: true,
        };
        let ok = self._out_pipes.ZMQ_MAP_INSERT_OR_EMPLACE(routing_id, outpipe).second;
        // zmq_assert (ok);

        self._fq.attach(pipe_);
    }

    pub unsafe fn xpipe_terminated(&mut self, pipe_: &mut ZmqPipe) {
        // const out_pipes_t::iterator it = _out_pipes.find (pipe_->get_server_socket_routing_id ());
        let it = self._out_pipes.find(pipe_.get_server_socket_routing_id());
        // zmq_assert (it != _out_pipes.end ());

        // _out_pipes.erase (it);
        self._out_pipes.remove(it);

        self._fq.pipe_terminated(pipe_);
    }

    pub unsafe fn xread_activated(&mut self, pipe_: &mut ZmqPipe) {
        self._fq.read_activated(pipe_);
    }

    pub unsafe fn xwrite_activated(&mut self, pipe: &mut ZmqPipe) {
        let end = self._out_pipes.iter_mut().last().unwrap();

        let mut it: (&u32, &mut ZmqOutpipe);
        for i in 0..self._out_pipes.len() {
            it = self._out_pipes.iter_mut().nth(i).unwrap();
            if it.1.pipe == pipe {
                it.1.active = true;
                break;
            }
        }
    }

    pub unsafe fn xsend(&mut self, msg_: &mut ZmqMsg) -> i32 {
        //  SERVER sockets do not allow multipart data (ZMQ_SNDMORE)
        if msg_.flag_set(MSG_MORE) {
            // errno = EINVAL;
            return -1;
        }
        //  Find the pipe associated with the routing stored in the message.
        let mut routing_id = msg_.get_routing_id();
        let it = self._out_pipes.iter_mut().find(routing_id).unwrap();

        if (it != self._out_pipes.iter_mut().end()) {
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

    pub unsafe fn xrecv(&mut self, msg_: &mut ZmqMsg) -> i32 {
        // pipe_t *pipe = NULL;
        let mut pipe= ZmqPipe::default();
        let mut rc = self._fq.recvpipe (msg_, &mut Some(&mut pipe));

        // Drop any messages with more flag
        // while (rc == 0 && msg_->flags () & msg_t::more)
        while rc == 0 && msg_.flag_set(MSG_MORE)
        {
            // drop all frames of the current multi-frame message
            rc = self._fq.recvpipe (msg_, &mut None);

            // while (rc == 0 && msg_->flags () & msg_t::more)
            while rc == 0 && msg_.flag_set(MSG_MORE)
            {
                rc = self._fq.recvpipe(msg_, &mut None);
            }

            // get the new message
            if (rc == 0) {
                rc = self._fq.recvpipe(msg_, &mut Some(&mut pipe));
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

    pub fn xhas_in (&mut self) -> bool
    {
        return self._fq.has_in ();
    }

    pub fn xhas_out(&mut self) -> bool {
        true
    }
}
