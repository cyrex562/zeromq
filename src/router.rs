use std::collections::HashSet;

use libc::MSG_MORE;

use crate::blob::{blob_t, reference_tag_t};
use crate::ctx::ctx_t;
use crate::defines::{ZMQ_POLLOUT, ZMQ_PROBE_ROUTER, ZMQ_ROUTER, ZMQ_ROUTER_HANDOVER, ZMQ_ROUTER_MANDATORY, ZMQ_ROUTER_NOTIFY, ZMQ_ROUTER_RAW};
use crate::fq::fq_t;
use crate::msg::msg_t;
use crate::options::options_t;
use crate::pipe::pipe_t;
use crate::socket_base::routing_socket_base_t;
use crate::utils::put_u32;

pub struct router_t<'a> {
    pub routing_socket_base: routing_socket_base_t<'a>,
    pub _fq: fq_t,
    pub _prefetched: bool,
    pub _routing_id_sent: bool,
    pub _prefetched_id: msg_t,
    pub _prefetched_msg: msg_t,
    pub _current_in: &'a mut pipe_t<'a>,
    pub _terminate_current_in: bool,
    pub _more_in: bool,
    pub _anonymous_pipes: HashSet<&'a mut pipe_t<'a>>,
    pub _current_out: &'a mut pipe_t<'a>,
    pub _more_out: bool,
    pub _next_integral_routing_id: u32,
    pub _mandatory: bool,
    pub _raw_socket: bool,
    pub _probe_router: bool,
    pub _handover: bool,
}

impl router_t {
    pub unsafe fn new(options: &mut options_t, parent_: &mut ctx_t, tid_: u32, sid_: i32) -> Self {
        options.type_ = ZMQ_ROUTER;
        options.recv_routing_id = true;
        options.raw_socket = false;
        options.can_send_hello_msg = true;
        options.can_recv_disconnect_msg = true;
        let mut out = Self {
            routing_socket_base: routing_socket_base_t::new(parent_, tid_, sid_),
            _fq: fq_t::default(),
            _prefetched: false,
            _routing_id_sent: false,
            _prefetched_id: Default::default(),
            _prefetched_msg: Default::default(),
            _current_in: &mut Default::default(),
            _terminate_current_in: false,
            _more_in: false,
            _anonymous_pipes: Default::default(),
            _current_out: &mut Default::default(),
            _more_out: false,
            _next_integral_routing_id: 0,
            _mandatory: false,
            _raw_socket: false,
            _probe_router: false,
            _handover: false,
        };
        out._prefetched_id.init2();
        out._prefetched_msg.init2();
        out
    }

    // void zmq::router_t::xattach_pipe (pipe_t *pipe_,
    //                               bool subscribe_to_all_,
    //                               bool locally_initiated_)
    pub unsafe fn xattach_pipe(&mut self, pipe_: &mut pipe_t, subscribe_to_all_: bool, locally_initiated_: bool) {
        // LIBZMQ_UNUSED (subscribe_to_all_);

        // zmq_assert (pipe_);

        if (self._probe_router) {
            // msg_t probe_msg;
            let mut probe_msg = msg_t::default();
            let mut rc = probe_msg.init2();
            // errno_assert (rc == 0);

            rc = pipe_.write(&mut probe_msg);
            // zmq_assert (rc) is not applicable here, since it is not a bug.
            // LIBZMQ_UNUSED (rc);

            pipe_.flush();

            rc = probe_msg.close();
            // errno_assert (rc == 0);
        }

        let routing_id_ok = self.identify_peer(pipe_, locally_initiated_);
        if (routing_id_ok) {
            self._fq.attach(pipe_);
        } else {
            self._anonymous_pipes.insert(pipe_);
        }
    }

    // int zmq::router_t::xsetsockopt (int option_,
    //                             const void *optval_,
    //                             size_t optvallen_)
    pub unsafe fn xsetsockopt(&mut self, options: &mut options_t, option_: i32, optval_: &[u8], optvallen_: usize) -> i32 {
        // const bool is_int = (optvallen_ == sizeof (int));
        let is_int = optvallen_ == 4;
        let mut value = 0;
        if (is_int) {
            libc::memcpy(&value, optval_, 4);
        }

        match option_ {
            ZMQ_ROUTER_RAW => {
                if (is_int && value >= 0) {
                    self._raw_socket = (value != 0);
                    if (self._raw_socket) {
                        options.recv_routing_id = false;
                        options.raw_socket = true;
                    }
                    return 0;
                }
            }

            ZMQ_ROUTER_MANDATORY => {
                if (is_int && value >= 0) {
                    self._mandatory = (value != 0);
                    return 0;
                }
            }

            ZMQ_PROBE_ROUTER => {
                if (is_int && value >= 0) {
                    self._probe_router = (value != 0);
                    return 0;
                }
            }

            ZMQ_ROUTER_HANDOVER => {
                if (is_int && value >= 0) {
                    self._handover = (value != 0);
                    return 0;
                }
            }


            // #ifdef ZMQ_BUILD_DRAFT_API
            ZMQ_ROUTER_NOTIFY => {
                if (is_int && value >= 0 && value <= (ZMQ_NOTIFY_CONNECT | ZMQ_NOTIFY_DISCONNECT)) {
                    options.router_notify = value;
                    return 0;
                }
            }

            // #endif

            _ => {
                return routing_socket_base_t::xsetsockopt(option_, optval_,
                                                          optvallen_);
            }
        }
        // errno = EINVAL;
        return -1;
    }

    // void zmq::router_t::xpipe_terminated (pipe_t *pipe_)
    pub unsafe fn xpipe_terminated(&mut self, pipe_: &mut pipe_t) {
        if 0 == self._anonymous_pipes.erase(pipe_) {
            self.erase_out_pipe(pipe_);
            self._fq.pipe_terminated(pipe_);
            pipe_.rollback();
            if pipe_ == self._current_out {
                self._current_out = None;
            }
        }
    }

    // void zmq::router_t::xread_activated (pipe_t *pipe_)
    pub unsafe fn xread_activated(&mut self, pipe_: &mut pipe_t) {
        // const std::set<pipe_t *>::iterator it = _anonymous_pipes.find (pipe_);
        let it = self._anonymous_pipes.iter_mut().find(|&x| *x == pipe_);
        if (it.is_none()) {
            _fq.activated(pipe_);
        } else {
            let routing_id_ok = self.identify_peer(pipe_, false);
            if (routing_id_ok) {
                self._anonymous_pipes.erase(it);
                self._fq.attach(pipe_);
            }
        }
    }

    // int zmq::router_t::xsend (msg_t *msg_)
    pub unsafe fn xsend(&mut self, msg_: &mut msg_t) -> i32 {
        //  If this is the first part of the message it's the ID of the
        //  peer to send the message to.
        if (!self._more_out) {
            // zmq_assert (!_current_out);

            //  If we have malformed message (prefix with no subsequent message)
            //  then just silently ignore it.
            //  TODO: The connections should be killed instead.
            if (msg_.flags() & MSG_MORE) {
                self._more_out = true;

                //  Find the pipe associated with the routing id stored in the prefix.
                //  If there's no such pipe just silently ignore the message, unless
                //  router_mandatory is set.
                let mut out_pipe = self.lookup_out_pipe(
                    blob_t::new3((msg_.data()),
                                 msg_.size(), reference_tag_t::default()));

                if (out_pipe) {
                    self._current_out = out_pipe.pipe;

                    // Check whether pipe is closed or not
                    if (!self._current_out.check_write()) {
                        // Check whether pipe is full or not
                        let pipe_full = !self._current_out.check_hwm();
                        out_pipe.active = false;
                        self._current_out = None;

                        if (self._mandatory) {
                            self._more_out = false;
                            if (pipe_full) {
                                // errno = EAGAIN;
                            } else {
                                // errno = EHOSTUNREACH;
                            }
                            return -1;
                        }
                    }
                } else if (self._mandatory) {
                    self._more_out = false;
                    // errno = EHOSTUNREACH;
                    return -1;
                }
            }

            let mut rc = msg_.close();
            // errno_assert (rc == 0);
            rc = msg_.init2();
            // errno_assert (rc == 0);
            return 0;
        }

        //  Ignore the MORE flag for raw-sock or assert?
        if (self.options.raw_socket) {
            msg_.reset_flags(MSG_MORE as u8);
        }

        //  Check whether this is the last part of the message.
        self._more_out = (msg_.flags() & MSG_MORE) != 0;

        //  Push the message into the pipe. If there's no out pipe, just drop it.
        if (self._current_out) {
            // Close the remote connection if user has asked to do so
            // by sending zero length message.
            // Pending messages in the pipe will be dropped (on receiving term- ack)
            if (self._raw_socket && msg_.size() == 0) {
                self._current_out.terminate(false);
                let mut rc = msg_.close();
                // errno_assert (rc == 0);
                rc = msg_.init2();
                // errno_assert (rc == 0);
                self._current_out = None;
                return 0;
            }

            let ok = self._current_out.write(msg_);
            if ((!ok)) {
                // Message failed to send - we must close it ourselves.
                let rc = msg_.close();
                // errno_assert (rc == 0);
                // HWM was checked before, so the pipe must be gone. Roll back
                // messages that were piped, for example REP labels.
                self._current_out.rollback();
                self._current_out = None;
            } else {
                if (!self._more_out) {
                    self._current_out.flush();
                    self._current_out = None;
                }
            }
        } else {
            let rc = msg_.close();
            // errno_assert (rc == 0);
        }

        //  Detach the message from the data buffer.
        let rc = msg_.init2();
        // errno_assert (rc == 0);

        return 0;
    }

    // int zmq::router_t::xrecv (msg_t *msg_)
    pub unsafe fn xrecv(&mut self, msg_: &mut msg_t) -> i32 {
        if (self._prefetched) {
            if (!self._routing_id_sent) {
                let rc = msg_. move (self._prefetched_id);
                // errno_assert (rc == 0);
                self._routing_id_sent = true;
            } else {
                let rc = msg_. move (self._prefetched_msg);
                // errno_assert (rc == 0);
                self._prefetched = false;
            }
            self._more_in = (msg_.flags() & msg_t::more) != 0;

            if (!self._more_in) {
                if (self._terminate_current_in) {
                    self._current_in.terminate(true);
                    self._terminate_current_in = false;
                }
                self._current_in = None;
            }
            return 0;
        }

        // pipe_t *pipe = NULL;
        let mut pipe: Option<&mut pipe_t> = None;
        let rc = self._fq.recvpipe(msg_, &mut pipe);

        //  It's possible that we receive peer's routing id. That happens
        //  after reconnection. The current implementation assumes that
        //  the peer always uses the same routing id.
        while (rc == 0 && msg_.is_routing_id()) {
            rc = self._fq.recvpipe(msg_, &mut pipe);
        }

        if (rc != 0) {
            return -1;
        }

        // zmq_assert (pipe != NULL);

        //  If we are in the middle of reading a message, just return the next part.
        if (self._more_in) {
            self._more_in = (msg_.flags() & msg_t::more) != 0;

            if (!self._more_in) {
                if (self._terminate_current_in) {
                    self._current_in.terminate(true);
                    self._terminate_current_in = false;
                }
                self._current_in = None;
            }
        } else {
            //  We are at the beginning of a message.
            //  Keep the message part we have in the prefetch buffer
            //  and return the ID of the peer instead.
            rc = self._prefetched_msg. move (*msg_);
            // errno_assert (rc == 0);
            self._prefetched = true;
            self._current_in = pipe;

            let routing_id = pipe.unwrap().get_routing_id();
            rc = msg_.init_size(routing_id.size());
            // errno_assert (rc == 0);
            libc::memcpy(msg_.data(), routing_id.data(), routing_id.size());
            msg_.set_flags(msg_t::more);
            if (self._prefetched_msg.metadata()) {
                msg_.set_metadata(self._prefetched_msg.metadata());
            }
            self._routing_id_sent = true;
        }

        return 0;
    }

    // int zmq::router_t::rollback ()
    pub unsafe fn rollback(&mut self) -> i32 {
        if (self._current_out) {
            self._current_out.rollback();
            self._current_out = None;
            self._more_out = false;
        }
        return 0;
    }

    // bool zmq::router_t::xhas_in ()
    pub unsafe fn xhas_in(&mut self) -> bool {
        //  If we are in the middle of reading the messages, there are
        //  definitely more parts available.
        if (self._more_in) {
            return true;
        }

        //  We may already have a message pre-fetched.
        if (self._prefetched) {
            return true;
        }

        //  Try to read the next message.
        //  The message, if read, is kept in the pre-fetch buffer.
        let mut pipe: Option<&mut pipe_t> = None;
        let rc = self._fq.recvpipe(&self._prefetched_msg, &mut pipe);

        //  It's possible that we receive peer's routing id. That happens
        //  after reconnection. The current implementation assumes that
        //  the peer always uses the same routing id.
        //  TODO: handle the situation when the peer changes its routing id.
        while (rc == 0 && self._prefetched_msg.is_routing_id()) {
            rc = self._fq.recvpipe(&self._prefetched_msg, &pipe);
        }

        if (rc != 0) {
            return false;
        }

        // zmq_assert (pipe != NULL);

        let routing_id = pipe.unwrap().get_routing_id();
        rc = self._prefetched_id.init_size(routing_id.size());
        // errno_assert (rc == 0);
        libc::memcpy(self._prefetched_id.data(), routing_id.data(), routing_id.size());
        self._prefetched_id.set_flags(MSG_MORE);
        if (self._prefetched_msg.metadata()) {
            self._prefetched_id.set_metadata(self._prefetched_msg.metadata());
        }

        self._prefetched = true;
        self._routing_id_sent = false;
        self._current_in = pipe;

        return true;
    }

    pub unsafe fn check_pipe_hwm(&mut self, pipe_: &mut pipe_t) -> bool {
        return pipe_.check_hwm();
    }

    // bool zmq::router_t::xhas_out ()
    pub unsafe fn xhas_out(&mut self) -> bool {
        //  In theory, ROUTER socket is always ready for writing (except when
        //  MANDATORY is set). Whether actual attempt to write succeeds depends
        //  on which pipe the message is going to be routed to.

        if (!self._mandatory) {
            return true;
        }

        return self.any_of_out_pipes(self.check_pipe_hwm);
    }

    // int zmq::router_t::get_peer_state (const void *routing_id_,
    //                                size_t routing_id_size_) const
    pub unsafe fn get_peer_state(&mut self, routing_id_: &[u8], routing_id_size_: usize) -> i32 {
        let mut res = 0;

        // TODO remove the const_cast, see comment in lookup_out_pipe
        let routing_id_blob = blob_t::new3(
            routing_id_,
            routing_id_size_);
        let out_pipe = self.lookup_out_pipe(routing_id_blob);
        if (!out_pipe) {
            // errno = EHOSTUNREACH;
            return -1;
        }

        if (out_pipe.pipe.check_hwm()) {
            res |= ZMQ_POLLOUT;
        }

        /** \todo does it make any sense to check the inpipe as well? */

        return res;
    }

    // bool zmq::router_t::identify_peer (pipe_t *pipe_, bool locally_initiated_)
    pub unsafe fn identify_peer(&mut self, options: &mut options_t, pipe_: &mut pipe_t, locally_initiated_: bool) -> bool {
        // msg_t msg;
        let mut msg = msg_t::default();
        // blob_t routing_id;
        let mut routing_id = blob_t::default();

        if locally_initiated_ && self.connect_routing_id_is_set() {
            let connect_routing_id = self.extract_connect_routing_id();
            routing_id.set(connect_routing_id.c_str(),
                           connect_routing_id.length());
            //  Not allowed to duplicate an existing rid
            // zmq_assert (!has_out_pipe (routing_id));
        } else if (options.raw_socket) { //  Always assign an integral routing id for raw-socket
            // unsigned char buf[5];
            let mut buf = [0u8; 5];
            buf[0] = 0;
            put_u32(buf.as_mut_ptr().add(1), self._next_integral_routing_id);
            self._next_integral_routing_id += 1;
            routing_id.set(buf, 5);
        } else if (!options.raw_socket) {
            //  Pick up handshake cases and also case where next integral routing id is set
            msg.init2();
            let ok = pipe_.read(&msg);
            if (!ok) {
                return false;
            }

            if (msg.size() == 0) {
                //  Fall back on the auto-generation
                // unsigned char buf[5];
                let mut buf = [0u8; 5];
                buf[0] = 0;
                put_u32(buf + 1, self._next_integral_routing_id);
                self._next_integral_routing_id += 1;
                routing_id.set(buf, 5);
                msg.close();
            } else {
                routing_id.set((msg.data()),
                               msg.size());
                msg.close();

                //  Try to remove an existing routing id entry to allow the new
                //  connection to take the routing id.
                let existing_outpipe = self.lookup_out_pipe(routing_id);

                if (existing_outpipe) {
                    if (!self._handover) {
                        //  Ignore peers with duplicate ID
                        return false;
                    }

                    //  We will allow the new connection to take over this
                    //  routing id. Temporarily assign a new routing id to the
                    //  existing pipe so we can terminate it asynchronously.
                    let mut buf = [0u8; 5];
                    buf[0] = 0;
                    put_u32(buf.as_mut_ptr().add(1), self._next_integral_routing_id);
                    self._next_integral_routing_id += 1;
                    let mut new_routing_id = blob_t::new3(buf, 5);

                    let old_pipe = existing_outpipe.pipe;

                    self.erase_out_pipe(old_pipe);
                    old_pipe.set_router_socket_routing_id(new_routing_id);
                    self.add_out_pipe((new_routing_id), old_pipe);

                    if (old_pipe == self._current_in) {
                        self._terminate_current_in = true;
                    } else {
                        old_pipe.terminate(true);
                    }
                }
            }
        }

        pipe_.set_router_socket_routing_id(routing_id);
        self.add_out_pipe((routing_id), pipe_);

        return true;
    }
}
