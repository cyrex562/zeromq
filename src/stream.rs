use std::ffi::c_void;
use std::ptr::null_mut;
use crate::ctx::ctx_t;
use crate::fq::fq_t;
use crate::msg::msg_t;
use crate::options::do_setsockopt_int_as_bool_strict;
use crate::pipe::pipe_t;
use crate::socket_base::routing_socket_base_t;

pub struct stream_t<'a> {
    pub base: routing_socket_base_t,
    pub _fq: fq_t,
    pub _prefetched: bool,
    pub _routing_id_sent: bool,
    pub _prefetched_routing_id: msg_t,
    pub _prefetched_msg: msg_t,
    pub _current_out: Option<&'a mut pipe_t>,
    pub _more_out: bool,
    pub _next_integral_routing_id: u32,
}

impl stream_t {
    pub unsafe fn new(parent_: *mut ctx_t, tid_: u32, sid_: i32) -> Self {
        let mut out = Self {
            base: routing_socket_base_t::new(parent , tid_, sid_),
            _fq: fq_t::new(),
            _prefetched: false,
            _routing_id_sent: false,
            _prefetched_routing_id: msg_t::new(),
            _prefetched_msg: msg_t::new(),
            _current_out: None,
            _more_out: false,
            _next_integral_routing_id: 0,
        };

        // options.type = ZMQ_STREAM;
        // options.raw_socket = true;

        out._prefetched_routing_id.init2();
        out._prefetched_msg.init2();

        out
    }

    pub unsafe fn xattach_pipe(&mut self, pipe_: *mut pipe_t, subscribe_to_all: bool, locally_initiated_: bool)
    {
        self.identify_peer (pipe_, locally_initiated_);
        self._fq.attach (pipe_);
    }

    pub unsafe fn xpipe_terminated(&mut self, pipe_: *mut pipe_t)
    {
        self._fq.terminated (pipe_);
        if pipe_ == self._current_out {
            self._current_out = None;
        }
    }

    pub unsafe fn xread_activated(&mut self, pipe_: *mut pipe_t)
    {
        self._fq.activated (pipe_);
    }

    pub unsafe fn xsend(&mut self, msg_: &mut msg_t) -> i32 {
        //  If this is the first part of the message it's the ID of the
        //  peer to send the message to.
        if (!self._more_out) {
            // zmq_assert (!_current_out);

            //  If we have malformed message (prefix with no subsequent message)
            //  then just silently ignore it.
            //  TODO: The connections should be killed instead.
            if (msg_.flags() & msg_t::more) {
                //  Find the pipe associated with the routing id stored in the prefix.
                //  If there's no such pipe return an error

                let mut out_pipe = lookup_out_pipe(
                    blob_t((msg_.data()),
                           msg_.size(), reference_tag_t()));

                if (out_pipe) {
                    self._current_out = out_pipe.pipe;
                    if (!self._current_out.check_write()) {
                        out_pipe.active = false;
                        self._current_out = None;
                        // errno = EAGAIN;
                        return -1;
                    }
                } else {
                    // errno = EHOSTUNREACH;
                    return -1;
                }
            }

            //  Expect one more message frame.
            self._more_out = true;

            let mut rc = (msg_).close();
            // errno_assert (rc == 0);
            rc = (msg_).init2();
            // errno_assert (rc == 0);
            return 0;
        }

        //  Ignore the MORE flag
        (msg_).reset_flags(msg_t::more);

        //  This is the last part of the message.
        self._more_out = false;

        //  Push the message into the pipe. If there's no out pipe, just drop it.
        if self._current_out {
            // Close the remote connection if user has asked to do so
            // by sending zero length message.
            // Pending messages in the pipe will be dropped (on receiving term- ack)
            if msg_.size() == 0 {
                self._current_out.terminate(false);
                let mut rc = msg_.close();
                // errno_assert (rc == 0);
                rc = msg_.init2();
                // errno_assert (rc == 0);
                self._current_out = None;
                return 0;
            }
            let ok = self._current_out.write(msg_);
            if ((ok)) {
                self._current_out.flush();
            }
            self._current_out = None;
        } else {
            let rc = msg_.close();
            // errno_assert (rc == 0);
        }

        //  Detach the message from the data buffer.
        let rc = msg_.init();
        // errno_assert (rc == 0);

        return 0;
    }

    pub unsafe fn  xsetsockopt(&mut self, option_: i32, optval_: *const c_void, optvallen_: usize) -> i32 {
        match option_ {
            ZMQ_STREAM_NOTIFY => {
                // if (optvallen_ != size_of::<i32>()) {
                //     // errno = EINVAL;
                //     return -1;
                // }
                // self._routing_id_sent = *(optval as *const i32) != 0;
                return  do_setsockopt_int_as_bool_strict(optval_, optvallen_, &options.raw_notify)
            }
            _ => {
                return self.base.xsetsockopt(option_, optval_, optvallen_);
            }
        }
    }

    pub unsafe fn xrecv(&mut self, msg_: &mut msg_t) -> i32 {
        if (self._prefetched) {
            if (!self._routing_id_sent) {
                let rc = msg_.move(self._prefetched_routing_id);
                // errno_assert (rc == 0);
                self._routing_id_sent = true;
            } else {
                let rc = msg_.move(self._prefetched_msg);
                // errno_assert (rc == 0);
                self._prefetched = false;
            }
            return 0;
        }

        // pipe_t *pipe = NULL;
        let mut pipe = pipe_t::new();
        let rc = self._fq.recvpipe (&mut _prefetched_msg, &mut pipe);
        if (rc != 0) {
            return -1;
        }

        zmq_assert (pipe != NULL);
        zmq_assert ((_prefetched_msg.flags () & msg_t::more) == 0);

        //  We have received a frame with TCP data.
        //  Rather than sending this frame, we keep it in prefetched
        //  buffer and send a frame with peer's ID.
        const blob_t &routing_id = pipe->get_routing_id ();
        rc = msg_->close ();
        errno_assert (rc == 0);
        rc = msg_->init_size (routing_id.size ());
        errno_assert (rc == 0);

        // forward metadata (if any)
        metadata_t *metadata = _prefetched_msg.metadata ();
        if (metadata)
            msg_->set_metadata (metadata);

        memcpy (msg_->data (), routing_id.data (), routing_id.size ());
        msg_->set_flags (msg_t::more);

        _prefetched = true;
        _routing_id_sent = true;

        return 0;
    }
}
