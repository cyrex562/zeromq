use crate::ctx::ctx_t;
use crate::defines::{
    ZMQ_ONLY_FIRST_SUBSCRIBE, ZMQ_TOPICS_COUNT, ZMQ_XSUB, ZMQ_XSUB_VERBOSE_UNSUBSCRIBE,
};
use crate::dist::dist_t;
use crate::fq::fq_t;
use crate::msg::{more, msg_t};
use crate::options::{do_getsockopt, options_t};
use crate::pipe::pipe_t;
use crate::radix_tree::radix_tree_t;
use crate::socket_base::socket_base_t;

pub struct xsub_t<'a> {
    pub socket_base: socket_base_t<'a>,
    pub _fq: fq_t,
    pub _dist: dist_t,
    pub _subscriptions: radix_tree_t,
    pub _verbose_unsubs: bool,
    pub _has_message: bool,
    pub _message: msg_t,
    pub _more_send: bool,
    pub _more_recv: bool,
    pub _process_subscribe: bool,
    pub _only_first_subscribe: bool,
}

impl xsub_t {
    pub unsafe fn new(options: &mut options_t, parent_: &mut ctx_t, tid_: u32, sid_: i32) -> Self {
        let mut out = Self {
            socket_base: socket_base_t::new(parent_, tid_, sid_, false),
            _fq: fq_t::default(),
            _dist: dist_t::default(),
            _subscriptions: radix_tree_t::default(),
            _verbose_unsubs: false,
            _has_message: false,
            _message: msg_t::default(),
            _more_send: false,
            _more_recv: false,
            _process_subscribe: false,
            _only_first_subscribe: false,
        };
        options.type_ = ZMQ_XSUB;
        options.linger.store(0);
        out._message.init2();
        out
    }

    pub unsafe fn xattach_pipe(
        &mut self,
        pipe_: &mut pipe_t,
        subscribe_to_all_: bool,
        locally_initiated_: bool,
    ) {
        self._fq.attach(pipe_);
        self._dist.attach(pipe_);

        //  Send all the cached subscriptions to the new upstream peer.
        self._subscriptions.apply(self.send_subscription, pipe_);
        pipe_.flush();
    }

    pub unsafe fn xread_activated(&mut self, pipe_: &mut pipe_t) {
        self._fq.activated(pipe_);
    }

    pub unsafe fn xwrite_activated(&mut self, pipe_: &mut pipe_t) {
        self._dist.activated(pipe_);
    }

    pub unsafe fn xpipe_terminated(&mut self, pipe_: &mut pipe_t) {
        self._fq.terminated(pipe_);
        self._dist.terminated(pipe_);
    }

    pub unsafe fn xhiccuped(&mut self, pipe_: &mut pipe_t) {
        //  Send all the cached subscriptions to the hiccuped pipe.
        self._subscriptions.apply(self.send_subscription, pipe_);
        pipe_.flush();
    }

    pub unsafe fn xsetsockopt(&mut self, option_: i32, optval_: &[u8], optvallen_: usize) -> i32 {
        let opt_val_i32: i32 = i32::from_le_bytes([optval_[0], optval_[1], optval_[2], optval_[3]]);
        if (option_ == ZMQ_ONLY_FIRST_SUBSCRIBE) {
            if (optvallen_ != 4 || (opt_val_i32) < 0) {
                // errno = EINVAL;
                return -1;
            }
            self._only_first_subscribe = (opt_val_i32 != 0);
            return 0;
        }
        // #ifdef ZMQ_BUILD_DRAFT_API
        else if (option_ == ZMQ_XSUB_VERBOSE_UNSUBSCRIBE) {
            self._verbose_unsubs = (opt_val_i32 != 0);
            return 0;
        }
        // #endif
        //     errno = EINVAL;
        return -1;
    }

    pub unsafe fn xgetsockopt(
        &mut self,
        option_: i32,
        optval_: &mut [u8],
        optvallen_: &mut usize,
    ) -> i32 {
        if option_ == ZMQ_TOPICS_COUNT {
            // make sure to use a multi-thread safe function to avoid race conditions with I/O threads
            // where subscriptions are processed:
            // #ifdef ZMQ_USE_RADIX_TREE
            let mut num_subscriptions = self._subscriptions.size();
            // #else
            //         uint64_t num_subscriptions = _subscriptions.num_prefixes ();
            // #endif

            return do_getsockopt(optval_, optvallen_, num_subscriptions);
        }

        // room for future options here

        // errno = EINVAL;
        return -1;
    }

    pub unsafe fn xsend(&mut self, msg_: &mut msg_t) -> i32 {
        let mut size = msg_.size();
        let mut data = (msg_.data());

        let first_part = !self._more_send;
        self._more_send = msg_.flag_set(more);

        if (first_part) {
            self._process_subscribe = !self._only_first_subscribe;
        } else if (!self._process_subscribe) {
            //  User message sent upstream to XPUB socket
            return self._dist.send_to_all(msg_);
        }

        if (msg_.is_subscribe() || (size > 0 && *data == 1)) {
            //  Process subscribe message
            //  This used to filter out duplicate subscriptions,
            //  however this is already done on the XPUB side and
            //  doing it here as well breaks ZMQ_XPUB_VERBOSE
            //  when there are forwarding devices involved.
            if (!msg_.is_subscribe()) {
                data = data.add(1);
                size = size - 1;
            }
            self._subscriptions.add(data, size);
            self._process_subscribe = true;
            return self._dist.send_to_all(msg_);
        }
        if (msg_.is_cancel() || (size > 0 && *data == 0)) {
            //  Process unsubscribe message
            if (!msg_.is_cancel()) {
                data = data.add(1);
                size = size - 1;
            }
            self._process_subscribe = true;
            let rm_result = self._subscriptions.rm(data, size);
            if (rm_result || self._verbose_unsubs) {
                return self._dist.send_to_all(msg_);
            }
        } else {
            //  User message sent upstream to XPUB socket
            return self._dist.send_to_all(msg_);
        }

        let rc = msg_.close();
        // errno_assert (rc == 0);
        rc = msg_.init2();
        // errno_assert (rc == 0);

        return 0;
    }

    pub fn xhas_out(&mut self) -> bool {
        true
    }

    pub unsafe fn xrecv(&mut self, msg_: &mut msg_t) -> i32 {
        //  If there's already a message prepared by a previous call to zmq_poll,
        //  return it straight ahead.
        if (self._has_message) {
            let rc = msg_.move (self._message);
            // errno_assert (rc == 0);
            self._has_message = false;
            self._more_recv = msg_.flag_set(more);
            return 0;
        }

        //  TODO: This can result in infinite loop in the case of continuous
        //  stream of non-matching messages which breaks the non-blocking recv
        //  semantics.
        loop {
            //  Get a message using fair queueing algorithm.
            let mut rc = self._fq.recv (msg_);

            //  If there's no message available, return immediately.
            //  The same when error occurs.
            if (rc != 0) {
                return -1;
            }

            //  Check whether the message matches at least one subscription.
            //  Non-initial parts of the message are passed
            if (self._more_recv || !self.options.filter || self.match_(msg_)) {
                self._more_recv = msg_.flag_set(more);
                return 0;
            }

            //  Message doesn't match. Pop any remaining parts of the message
            //  from the pipe.
            while msg_.flag_set(more) {
                rc = self._fq.recv (msg_);
                // errno_assert (rc == 0);
            }
        }
    }

    pub unsafe fn xhas_in(&mut self) -> bool {
            //  There are subsequent parts of the partly-read message available.
        if (self._more_recv) {
            return true;
        }

        //  If there's already a message prepared by a previous call to zmq_poll,
        //  return straight ahead.
        if (self._has_message) {
            return true;
        }

        //  TODO: This can result in infinite loop in the case of continuous
        //  stream of non-matching messages.
        loop {
            //  Get a message using fair queueing algorithm.
            let mut rc = self._fq.recv (&mut self._message);

            //  If there's no message available, return immediately.
            //  The same when error occurs.
            if (rc != 0) {
                // errno_assert (errno == EAGAIN);
                return false;
            }

            //  Check whether the message matches at least one subscription.
            if (!self.options.filter || self.match_(&self._message)) {
                self._has_message = true;
                return true;
            }

            //  Message doesn't match. Pop any remaining parts of the message
            //  from the pipe.
            while (self._message.flags () & msg_t::more) {
                rc = self._fq.recv (&mut self._message);
                // errno_assert (rc == 0);
            }
        }
    }
    
    pub unsafe fn match_(&mut self, msg_: &mut msg_t) -> bool {
        let matching = self._subscriptions.check (
           (msg_.data ()), msg_.size ());
    
        return matching ^ self.options.invert_matching;
    }
    
    pub unsafe fn send_subscription(&mut self, data_: &mut [u8], size_: usize, arg_: &mut [u8]) {
        let mut pipe = arg_.as_mut_ptr() as *mut pipe_t;

        //  Create the subscription message.
        let mut msg = msg_t::default();
        let rc = msg.init_subscribe (size_, data_);
        // errno_assert (rc == 0);
    
        //  Send it to the pipe.
        let sent = (*pipe).write (&mut msg);
        //  If we reached the SNDHWM, and thus cannot send the subscription, drop
        //  the subscription message instead. This matches the behaviour of
        //  zmq_setsockopt(ZMQ_SUBSCRIBE, ...), which also drops subscriptions
        //  when the SNDHWM is reached.
        if (!sent) {
            msg.close();
        }
    }
}
