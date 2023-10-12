use std::collections::VecDeque;
use std::ffi::c_void;
use std::ptr::null_mut;
use libc::{EINVAL, memcpy};
use crate::blob::blob_t;
use crate::ctx::ctx_t;
use crate::defines::{ZMQ_ONLY_FIRST_SUBSCRIBE, ZMQ_PUB, ZMQ_SUBSCRIBE, ZMQ_TOPICS_COUNT, ZMQ_UNSUBSCRIBE, ZMQ_XPUB, ZMQ_XPUB_MANUAL, ZMQ_XPUB_MANUAL_LAST_VALUE, ZMQ_XPUB_NODROP, ZMQ_XPUB_VERBOSE, ZMQ_XPUB_VERBOSER, ZMQ_XPUB_WELCOME_MSG};
use crate::dist::dist_t;
use crate::generic_mtrie::generic_mtrie_t;
use crate::metadata::metadata_t;
use crate::msg::{more, msg_t};
use crate::mtrie::mtrie_t;
use crate::options::{do_getsockopt, options_t};
use crate::pipe::pipe_t;
use crate::socket_base::socket_base_t;

pub struct xpub_t<'a> {
    pub socket_base: socket_base_t<'a>,
    pub _subscriptions: mtrie_t,
    pub _manual_subscriptions: mtrie_t,
    pub _dist: dist_t,
    pub _verbose_unsubs: bool,
    pub _more_send: bool,
    pub _more_recv: bool,
    pub _process_subscribe: bool,
    pub _only_first_subscribe: bool,
    pub _lossy: bool,
    pub _manual: bool,
    pub _send_last_pipe: bool,
    pub _last_pipe: Option<&'a mut pipe_t<'a>>,
    pub _pending_pipes: VecDeque<&'a mut pipe_t<'a>>,
    pub _welcome_msg: msg_t,
    pub _pending_data: VecDeque<blob_t>,
    pub _pending_metadata: VecDeque<&'a mut metadata_t>,
    pub _pending_flags: VecDeque<u8>,
}

impl xpub_t {
    pub unsafe fn new(options: &mut options_t, parent_: &mut ctx_t, tid_: u32, sid_: i32) -> Self {
        options.type_ = ZMQ_XPUB;
        let mut out = Self {
            socket_base: socket_base_t::new(parent_, tid_, sid_),
            _subscriptions: generic_mtrie_t::new(),
            _manual_subscriptions: generic_mtrie_t::new(),
            _dist: dist_t::new(),
            _verbose_unsubs: false,
            _more_send: false,
            _more_recv: false,
            _process_subscribe: false,
            _only_first_subscribe: false,
            _lossy: false,
            _manual: false,
            _send_last_pipe: false,
            _last_pipe: None,
            _pending_pipes: Default::default(),
            _welcome_msg: msg_t::default(),
            _pending_data: Default::default(),
            _pending_metadata: Default::default(),
            _pending_flags: Default::default(),
        };
        out._welcome_msg.init2();
        out
    }

    pub unsafe fn xattach_pipe(&mut self, pipe_: &mut pipe_t, subscribe_to_all_: bool, locally_initiated_: bool) -> Self {
        self._dist.attatch(pipe_);
        if subscribe_to_all_ {
            self._subscriptions.add(None, 0, pipe_);
        }

        if self._welcome_msg.size() > 0 {
            let mut copy: msg_t = msg_t::new();
            copy.init2();
            copy.copy(&self._welcome_msg);
            let ok = pipe_.write(&mut copy);
            pipe_.flush();
        }

        self.xread_activated(pipe_)
    }

    pub unsafe fn xread_activated(&mut self, options: &mut options_t, pipe_: &mut pipe_t) {
        //  There are some subscriptions waiting. Let's process them.
        // msg_t msg;
        let mut msg: msg_t::new();
        while (pipe_.read(&msg)) {
            let mut metadata = msg.metadata();
            let mut msg_data = (msg.data());
            let mut data: *mut u8 = null_mut();

            let mut size = 0usize;
            let mut subscribe = false;
            let mut is_subscribe_or_cancel = false;
            let mut notify = false;

            let mut first_part = !self._more_recv;
            self._more_recv = msg.flag_set(more);

            if (first_part || self._process_subscribe) {
                //  Apply the subscription to the trie
                if (msg.is_subscribe() || msg.is_cancel()) {
                    data = (msg.command_body());
                    size = msg.command_body_size();
                    subscribe = msg.is_subscribe();
                    is_subscribe_or_cancel = true;
                } else if msg.size() > 0 && (*msg_data == 0 || *msg_data == 1) {
                    data = msg_data + 1;
                    size = msg.size() - 1;
                    subscribe = *msg_data == 1;
                    is_subscribe_or_cancel = true;
                }
            }

            if (first_part) {
                self._process_subscribe = !self._only_first_subscribe || is_subscribe_or_cancel;
            }

            if (is_subscribe_or_cancel) {
                if (self._manual) {
                    // Store manual subscription to use on termination
                    if (!subscribe) {
                        self._manual_subscriptions.rm(data, size, pipe_);
                    } else {
                        self._manual_subscriptions.add(Some(data), size, pipe_);
                    }

                    self._pending_pipes.push_back(pipe_);
                } else {
                    if (!subscribe) {
                        let mut rm_result = self._subscriptions.rm(data, size, pipe_);
                        //  TODO reconsider what to do if rm_result == mtrie_t::not_found
                        notify = rm_result != mtrie_t::values_remain || self._verbose_unsubs;
                    } else {
                        let first_added = self._subscriptions.add(data, size, pipe_);
                        notify = first_added || self._verbose_subs;
                    }
                }

                //  If the request was a new subscription, or the subscription
                //  was removed, or verbose mode or manual mode are enabled, store it
                //  so that it can be passed to the user on next recv call.
                if (self._manual || (options.type_ == ZMQ_XPUB && notify)) {
                    //  ZMTP 3.1 hack: we need to support sub/cancel commands, but
                    //  we can't give them back to userspace as it would be an API
                    //  breakage since the payload of the message is completely
                    //  different. Manually craft an old-style message instead.
                    //  Although with other transports it would be possible to simply
                    //  reuse the same buffer and prefix a 0/1 byte to the topic, with
                    //  inproc the subscribe/cancel command string is not present in
                    //  the message, so this optimization is not possible.
                    //  The pushback makes a copy of the data array anyway, so the
                    //  number of buffer copies does not change.
                    // blob_t notification (size + 1);
                    let mut notification = blob_t::new2(size + 1);
                    if (subscribe) {
                        notification.data()[0] = 1;
                    } else {
                        notification.data()[0] = 0;
                    }
                    libc::memcpy(notification.data_mut().add(1) as *mut c_void, data as *const c_void, size);

                    self._pending_data.push_back(notification);
                    if (metadata) {
                        metadata.add_ref();
                    }
                    self._pending_metadata.push_back(metadata);
                    self._pending_flags.push_back(0);
                }
            } else if (options.type_ != ZMQ_PUB) {
                //  Process user message coming upstream from xsub socket,
                //  but not if the type is PUB, which never processes user
                //  messages
                self._pending_data.push_back(blob_t::new3(msg_data, msg.size()));
                if (metadata) {
                    metadata.add_ref();
                }
                self._pending_metadata.push_back(metadata);
                self._pending_flags.push_back(msg.flags());
            }

            msg.close();
        }

        pub unsafe fn xwrite_activated(&mut self, pipe_: &mut pipe_t) {
            self._dist.activated(pipe_)
        }

        pub unsafe fn xsetsockopt(&mut self, option_: i32, optval_: &[u8], optvallen_: usize) -> i32 {
            if (option_ == ZMQ_XPUB_VERBOSE || option_ == ZMQ_XPUB_VERBOSER || option_ == ZMQ_XPUB_MANUAL_LAST_VALUE || option_ == ZMQ_XPUB_NODROP || option_ == ZMQ_XPUB_MANUAL || option_ == ZMQ_ONLY_FIRST_SUBSCRIBE) {
                if (optvallen_ != 4 || (optval_[0]) < 0) {
                    // errno = EINVAL;
                    return -1;
                }
                if (option_ == ZMQ_XPUB_VERBOSE) {
                    self._verbose_subs = ((optval_[0]) != 0);
                    self._verbose_unsubs = false;
                } else if (option_ == ZMQ_XPUB_VERBOSER) {
                    self._verbose_subs = ((optval_[0]) != 0);
                    self._verbose_unsubs = self._verbose_subs;
                } else if (option_ == ZMQ_XPUB_MANUAL_LAST_VALUE) {
                    self._manual = ((optval_[0]) != 0);
                    self._send_last_pipe = self._manual;
                } else if (option_ == ZMQ_XPUB_NODROP) {
                    self._lossy = ((optval_[0]) == 0);
                } else if (option_ == ZMQ_XPUB_MANUAL) {
                    self._manual = ((optval_[0]) != 0);
                } else if (option_ == ZMQ_ONLY_FIRST_SUBSCRIBE) {
                    self._only_first_subscribe = ((optval_[0]) != 0);
                }
            } else if (option_ == ZMQ_SUBSCRIBE && self._manual) {
                if (self._last_pipe.is_some()) {
                    self._subscriptions.add(optval_, optvallen_, self._last_pipe);
                }
            } else if (option_ == ZMQ_UNSUBSCRIBE && self._manual) {
                if (self._last_pipe.is_some())
                self._subscriptions.rm(optval_, optvallen_, self._last_pipe);
            } else if (option_ == ZMQ_XPUB_WELCOME_MSG) {
                self._welcome_msg.close();

                if (optvallen_ > 0) {
                    let rc = self._welcome_msg.init_size(optvallen_);
                    // errno_assert (rc == 0);

                    let data = (self._welcome_msg.data());
                    libc::memcpy(data, optval_.as_ptr() as *const c_void, optvallen_);
                } else self._welcome_msg.init();
            } else {
                // errno = EINVAL;
                return -1;
            }
            return 0;
        }
    }

    pub unsafe fn xgetsockopt(&mut self, option_: i32, optval_: &mut [u8], optvallen_: &mut usize) -> i32 {
        if option_ == ZMQ_TOPICS_COUNT {
            return do_getsockopt(optval_.as_ptr() as *mut c_void, optvallen_, self._subscriptions._num_prefixes);
        }
        return -1;
    }


}
