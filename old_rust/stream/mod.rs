use crate::ctx::ZmqContext;
use crate::defines::{ZMQ_MSG_MORE, ZMQ_STREAM_NOTIFY};
use crate::defines::err::ZmqError;
use crate::defines::fair_queue::ZmqFairQueue;
use crate::msg::ZmqMsg;
use crate::options::{do_setsockopt_int_as_bool_strict, ZmqOptions};
use crate::pipe::ZmqPipe;
use crate::socket::ZmqSocket;
use crate::utils::put_u32;

pub struct ZmqStream<'a> {
    pub base: ZmqSocket<'a>,
    pub _fq: ZmqFairQueue<'a>,
    pub _prefetched: bool,
    pub _routing_id_sent: bool,
    pub _prefetched_routing_id: ZmqMsg,
    pub _prefetched_msg: ZmqMsg,
    pub _current_out: Option<&'a mut ZmqPipe<'a>>,
    pub _more_out: bool,
    pub _next_integral_routing_id: u32,
}

impl<'a> ZmqStream<'a> {
    pub fn new(
        parent_: &mut ZmqContext,
        tid_: u32,
        sid_: i32,
    ) -> Result<Self, ZmqError> {
        let mut out = Self {
            base: ZmqSocket::new(parent_, tid_, sid_, false),
            _fq: ZmqFairQueue::new(),
            _prefetched: false,
            _routing_id_sent: false,
            _prefetched_routing_id: ZmqMsg::default(),
            _prefetched_msg: ZmqMsg::default(),
            _current_out: None,
            _more_out: false,
            _next_integral_routing_id: 0,
        };

        // options.type = ZMQ_STREAM;
        // options.raw_socket = true;

        out._prefetched_routing_id.init2()?;
        out._prefetched_msg.init2()?;

        Ok(out)
    }

    pub fn xattach_pipe(&mut self,
                        options: &mut ZmqOptions,
                        pipe_: &mut ZmqPipe,
                        subscribe_to_all: bool,
                        locally_initiated_: bool,
    ) {
        self.identify_peer(options, pipe_, locally_initiated_);
        self._fq.attach(pipe_);
    }

    pub fn xpipe_terminated(&mut self, pipe_: &mut ZmqPipe) {
        self._fq.terminated(pipe_);
        if pipe_ == self._current_out {
            self._current_out = None;
        }
    }

    pub fn xread_activated(&mut self, pipe_: &mut ZmqPipe) -> Result<(), ZmqError> {
        self._fq.activated(pipe_)
    }

    pub fn xsend(&mut self, msg_: &mut ZmqMsg) -> Result<(), ZmqError> {
        //  If this is the first part of the message it's the ID of the
        //  peer to send the message to.
        if !self._more_out {
            // zmq_assert (!_current_out);

            //  If we have malformed message (prefix with no subsequent message)
            //  then just silently ignore it.
            //  TODO: The connections should be killed instead.
            if msg_.flags() & ZMQ_MSG_MORE {
                //  Find the pipe associated with the routing id stored in the prefix.
                //  If there's no such pipe return an Error

                // TODO
                // let mut out_pipe = lookup_out_pipe(
                //     blob_t((msg_.data_mut()),
                //            msg_.size(), reference_tag_t()));

                // TODO
                // if out_pipe {
                //     self._current_out = out_pipe.pipe;
                //     if !self._current_out.check_write() {
                //         out_pipe.active = false;
                //         self._current_out = None;
                //         // errno = EAGAIN;
                //         return -1;
                //     }
                // } else {
                //     // errno = EHOSTUNREACH;
                //     return -1;
                // }
            }

            //  Expect one more message frame.
            self._more_out = true;

            (msg_).close()?;
            // errno_assert (rc == 0);
            msg_.init2()?;
            // errno_assert (rc == 0);
            return Ok(());
        }

        //  Ignore the MORE flag
        (msg_).reset_flags(ZMQ_MSG_MORE);

        //  This is the last part of the message.
        self._more_out = false;

        //  Push the message into the pipe. If there's no out pipe, just drop it.
        if self._current_out {
            // Close the remote connection if user has asked to do so
            // by sending zero length message.
            // Pending messages in the pipe will be dropped (on receiving Term- ack)
            if msg_.size() == 0 {
                self._current_out.terminate(false);
                msg_.close()?;
                // errno_assert (rc == 0);
                msg_.init2()?;
                // errno_assert (rc == 0);
                self._current_out = None;
                return Ok(());
            }
            let ok = self._current_out.write(msg_);
            if ok {
                self._current_out.flush();
            }
            self._current_out = None;
        } else {
            msg_.close()?;
            // errno_assert (rc == 0);
        }

        //  Detach the message from the data buffer.
        msg_.init2()?;
        // errno_assert (rc == 0);

        return Ok(());
    }

    pub fn xsetsockopt(&mut self,
                       options: &mut ZmqOptions,
                       option_: i32,
                       optval_: &mut [u8],
                       optvallen_: usize,
    ) -> Result<(), ZmqError> {
        return match option_ {
            ZMQ_STREAM_NOTIFY => {
                // if (optvallen_ != size_of::<i32>()) {
                //     // errno = EINVAL;
                //     return -1;
                // }
                // self._routing_id_sent = *(optval as *const i32) != 0;
                do_setsockopt_int_as_bool_strict(optval_, optvallen_, &mut options.raw_notify)
            }
            _ => {
                self.base.xsetsockopt(options, option_, optval_, optvallen_)
            }
        };
    }

    pub fn xrecv(&mut self,
                 ctx: &mut ZmqContext,
                 msg_: &mut ZmqMsg,
    ) -> Result<(), ZmqError> {
        if self._prefetched {
            if !self._routing_id_sent {
                // TODO
                // let rc = msg_.move(self._prefetched_routing_id);
                // errno_assert (rc == 0);
                self._routing_id_sent = true;
            } else {
                // TODO
                // let rc = msg_. move (self._prefetched_msg);
                // errno_assert (rc == 0);
                self._prefetched = false;
            }
            return Ok(());
        }

        // pipe_t *pipe = NULL;
        let mut pipe = ZmqPipe::default();
        self._fq.recvpipe(ctx, &mut self._prefetched_msg, &mut Some(&mut pipe))?;
        // if rc != 0 {
        //     return -1;
        // }

        // zmq_assert (pipe != NULL);
        // zmq_assert ((_prefetched_msg.flags () & msg_t::more) == 0);

        //  We have received a frame with TCP data.
        //  Rather than sending this frame, we keep it in prefetched
        //  buffer and send a frame with peer's ID.
        let mut routing_id = pipe.get_routing_id();
        msg_.close()?;
        // errno_assert (rc == 0);
        msg_.init_size(routing_id.size())?;
        // errno_assert (rc == 0);

        // forward metadata (if any)
        let mut metadata = self._prefetched_msg.metadata();
        if metadata {
            msg_.set_metadata(metadata);
        }

        // libc::memcpy(msg_.data_mut(), routing_id.data() as *const c_void, routing_id.size());
        msg_.data_mut().clone_from_slice(routing_id.data());
        msg_.set_flags(ZMQ_MSG_MORE);

        self._prefetched = true;
        self._routing_id_sent = true;

        return Ok(());
    }

    pub fn xhas_in(&mut self, ctx: &mut ZmqContext) -> bool {
        //  We may already have a message pre-fetched.
        if self._prefetched {
            return true;
        }

        //  Try to read the next message.
        //  The message, if read, is kept in the pre-fetch buffer.
        // pipe_t *pipe = NULL;
        let mut pipe: ZmqPipe;
        self._fq.recvpipe(ctx, &mut self._prefetched_msg, &mut Some(&mut pipe))?;
        // if rc != 0 {
        //     return false;
        // }

        // zmq_assert (pipe != NULL);
        // zmq_assert ((_prefetched_msg.flags () & msg_t::more) == 0);

        let routing_id = pipe.get_routing_id();
        self._prefetched_routing_id.init_size(routing_id.size())?;
        // errno_assert (rc == 0);

        // forward metadata (if any)
        let mut metadata = self._prefetched_msg.metadata();
        if metadata {
            self._prefetched_routing_id.set_metadata(metadata);
        }

        // libc::memcpy(self._prefetched_routing_id.data_mut(), routing_id.data() as *const c_void,
        //              routing_id.size());
        self._prefetched_routing_id.data_mut().clone_from_slice(routing_id.data());
        self._prefetched_routing_id.set_flags(ZMQ_MSG_MORE);

        self._prefetched = true;
        self._routing_id_sent = false;

        return true;
    }

    pub fn xhas_out(&mut self) -> bool {
        true
    }

    pub fn identify_peer(&mut self,
                         options: &mut ZmqOptions,
                         pipe_: &mut ZmqPipe,
                         locally_initiated_: bool,
    ) {
        // unsigned char buffer[5];
        let mut buffer: [u8; 5] = [0; 5];
        buffer[0] = 0;
        let mut routing_id: Vec<u8> = vec![];
        if locally_initiated_ && self.connect_routing_id_is_set() {
            let mut connect_routing_id = self.extract_connect_routing_id();
            routing_id.set(
                (connect_routing_id),
                connect_routing_id.length());
            //  Not allowed to duplicate an existing rid
            // zmq_assert (!has_out_pipe (routing_id));
        } else {
            put_u32(buffer.add(1), self._next_integral_routing_id);
            self._next_integral_routing_id += 1;
            // TODO
            // routing_id.set(&mut buffer, size_of_val(buffer));
            // libc::memcpy(options.routing_id as *mut c_void, routing_id.data() as *const c_void, routing_id.size());
            options.routing_id.clone_from_slice(&routing_id);
            options.routing_id_size = (routing_id.size());
        }
        pipe_.set_router_socket_routing_id(&mut routing_id);
        self.add_out_pipe((routing_id), pipe_);
    }
}
