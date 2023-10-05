use std::collections::HashMap;
use std::ffi::c_void;
use std::ptr::null_mut;
use libc::{clock_t, EINTR, EINVAL};
use crate::add;
use crate::address::address_t;
use crate::array::{array_item_t, array_t};
use crate::blob::blob_t;
use crate::ctx::ctx_t;
use crate::defines::{handle_t, ZMQ_DGRAM, ZMQ_DISH, ZMQ_EVENTS, ZMQ_FD, ZMQ_LAST_ENDPOINT, ZMQ_POLLIN, ZMQ_POLLOUT, ZMQ_RCVMORE, ZMQ_THREAD_SAFE};
use crate::fq::pipes_t;
use crate::i_mailbox::i_mailbox;
use crate::i_poll_events::i_poll_events;
use crate::mailbox::mailbox_t;
use crate::mailbox_safe::mailbox_safe_t;
use crate::mutex::mutex_t;
use crate::options::do_getsockopt;
use crate::own::own_t;
use crate::pipe::{i_pipe_events, pipe_t};
use crate::poller::poller_t;
use crate::signaler::signaler_t;
use crate::utils::get_errno;

pub type endpoint_pipe_t = (*mut own_t, *mut pipe_t);
pub type endpoints_t = HashMap<String, endpoint_pipe_t>;

pub type map_t = HashMap<String,*mut pipe_t>;

#[derive(Default)]
pub struct inprocs_t {
    pub _inprocs: map_t,
}

impl inprocs_t {
    pub fn emplace(&mut self, endpoint_uri_: &str, pipe_: *mut pipe_t) {
        self._inprocs.insert(endpoint_uri_.to_string(), pipe_);
    }

    pub unsafe fn erase_pipes(&mut self, endpoint_uri_str: &str) -> i32 {
        for (k,v) in self._inprocs.iter_mut() {
            if k == endpoint_uri_str {
                v.send_disconnect_msg();
                v.terminate(true)
            }
        }
        self._inprocs.remove(endpoint_uri_str);
        return 0;
    }
}

#[derive(PartialEq)]
pub struct socket_base_t
{
    pub own: own_t,
    pub array_item: array_item_t,
    pub poll_events: i_poll_events,
    pub pipe_events: i_pipe_events,
    pub _mailbox: *mut i_mailbox,
    pub _pipes: pipes_t,
    pub _poller: *mut poller_t,
    pub _handle: *mut handle_t,
    pub _last_tsc: u64,
    pub _ticks: i32,
    pub _rcvmore: bool,
    pub _clock: clock_t,
    pub _monitor_socket: *mut c_void,
    pub _monitor_events: i64,
    pub _last_endpoint: String,
    pub _thread_safe: bool,
    pub _reaper_signaler: *mut signaler_t,
    pub _monitor_sync: mutex_t,
    pub _disconnected: bool,
    pub _sync: mutex_t,
    pub _endpoints: endpoints_t,
    pub _inprocs: inprocs_t,
    pub _tag: u32,
    pub _ctx_terminated: bool,
    pub _destroyed: bool,
}

impl socket_base_t {
    pub fn check_tag(&self) -> bool {
        return self._tag == 0xbaddecaf;
    }

    pub fn is_thread_safe(&self) -> bool {
        self._thread_safe
    }

    pub fn create(type_: i32, parent_: *mut ctx_t, tid_: u32, sid_: i32) -> *mut Self {
        todo!()
        // socket_base_t *s = NULL;
        //     switch (type_) {
        //         case ZMQ_PAIR:
        //             s = new (std::nothrow) pair_t (parent_, tid_, sid_);
        //             break;
        //         case ZMQ_PUB:
        //             s = new (std::nothrow) pub_t (parent_, tid_, sid_);
        //             break;
        //         case ZMQ_SUB:
        //             s = new (std::nothrow) sub_t (parent_, tid_, sid_);
        //             break;
        //         case ZMQ_REQ:
        //             s = new (std::nothrow) req_t (parent_, tid_, sid_);
        //             break;
        //         case ZMQ_REP:
        //             s = new (std::nothrow) rep_t (parent_, tid_, sid_);
        //             break;
        //         case ZMQ_DEALER:
        //             s = new (std::nothrow) dealer_t (parent_, tid_, sid_);
        //             break;
        //         case ZMQ_ROUTER:
        //             s = new (std::nothrow) router_t (parent_, tid_, sid_);
        //             break;
        //         case ZMQ_PULL:
        //             s = new (std::nothrow) pull_t (parent_, tid_, sid_);
        //             break;
        //         case ZMQ_PUSH:
        //             s = new (std::nothrow) push_t (parent_, tid_, sid_);
        //             break;
        //         case ZMQ_XPUB:
        //             s = new (std::nothrow) xpub_t (parent_, tid_, sid_);
        //             break;
        //         case ZMQ_XSUB:
        //             s = new (std::nothrow) xsub_t (parent_, tid_, sid_);
        //             break;
        //         case ZMQ_STREAM:
        //             s = new (std::nothrow) stream_t (parent_, tid_, sid_);
        //             break;
        //         case ZMQ_SERVER:
        //             s = new (std::nothrow) server_t (parent_, tid_, sid_);
        //             break;
        //         case ZMQ_CLIENT:
        //             s = new (std::nothrow) client_t (parent_, tid_, sid_);
        //             break;
        //         case ZMQ_RADIO:
        //             s = new (std::nothrow) radio_t (parent_, tid_, sid_);
        //             break;
        //         case ZMQ_DISH:
        //             s = new (std::nothrow) dish_t (parent_, tid_, sid_);
        //             break;
        //         case ZMQ_GATHER:
        //             s = new (std::nothrow) gather_t (parent_, tid_, sid_);
        //             break;
        //         case ZMQ_SCATTER:
        //             s = new (std::nothrow) scatter_t (parent_, tid_, sid_);
        //             break;
        //         case ZMQ_DGRAM:
        //             s = new (std::nothrow) dgram_t (parent_, tid_, sid_);
        //             break;
        //         case ZMQ_PEER:
        //             s = new (std::nothrow) peer_t (parent_, tid_, sid_);
        //             break;
        //         case ZMQ_CHANNEL:
        //             s = new (std::nothrow) channel_t (parent_, tid_, sid_);
        //             break;
        //         default:
        //             errno = EINVAL;
        //             return NULL;
        //     }
        //
        //     alloc_assert (s);
        //
        //     if (s->_mailbox == NULL) {
        //         s->_destroyed = true;
        //         LIBZMQ_DELETE (s);
        //         return NULL;
        //     }
        //
        //     return s;
    }

    pub unsafe fn new(parent_: *mut ctx_t, tid_: u32, sid_: i32, thread_safe_: bool) -> Self {
        let mut out = Self {
            own: own_t::new(parent_, tid_),
            array_item: array_item_t::new(),
            poll_events: null_mut(),
            pipe_events: null_mut(),
            _mailbox: null_mut(),
            _pipes: array_t::default(),
            _poller: null_mut(),
            _handle: null_mut(),
            _last_tsc: 0,
            _ticks: 0,
            _rcvmore: false,
            _clock: 0,
            _monitor_socket: null_mut(),
            _monitor_events: 0,
            _last_endpoint: "".to_string(),
            _thread_safe: false,
            _reaper_signaler: null_mut(),
            _monitor_sync: mutex_t::new(),
            _disconnected: false,
            _sync: mutex_t::new(),
            _endpoints: Default::default(),
            _inprocs: inprocs_t::default(),
            _tag: 0,
            _ctx_terminated: false,
            _destroyed: false,
        }
        ;
        // TODO
        // options.socket_id = sid_;
        //     options.ipv6 = (parent_->get (ZMQ_IPV6) != 0);
        //     options.linger.store (parent_->get (ZMQ_BLOCKY) ? -1 : 0);
        //     options.zero_copy = parent_->get (ZMQ_ZERO_COPY_RECV) != 0;

        if out._thread_safe {
            out._mailbox = &mut mailbox_safe_t::new();
        } else {
            out._mailbox = mailbox_t::new();
        }
        out
    }

    pub fn get_peer_state(&mut self, routing_id: *const c_void, routing_id_size_: usize) -> i32 {
        unimplemented!()
    }

    pub fn get_mailbox(&mut self) -> *mut dyn i_mailbox {
        self._mailbox
    }

    pub fn parse_uri(&mut self, uri_: &str, protocol_: &mut String, path_: &mut String) -> i32 {
        let mut uri = String::from(uri_);
        let pos = uri.find("://");
        if pos.is_none() {
            return -1;
        }

       *protocol_ = uri_[0..pos.unwrap()].to_string();
        *path_ = uri_[pos.unwrap()+3..].to_string();

        if protocol_.is_empty() || path_.is_empty() {
            return -1;
        }

        return 0;
    }

    pub fn check_protocol(&mut self, protocol_: &str) -> i32 {
        let protocols = ["tcp", "udp"];
        if protocols.contains(&protocol_) {
            return 0;
        }
        return -1;
    }

    pub unsafe fn attach_pipe(&mut self, pipe_: *mut pipe_t, subscribe_to_all_: bool, locally_initiated_: bool) {
        (*pipe_).set_event_sink(self)
        self._pipes.push_back(pipe_);
        self.xattach_pipe(pipe_, subscribe_to_all_, locally_initiated_);

        if self.is_terminating() {
            self.register_term_acks(1);
            (*pipe_).terminate(false);
        }
    }

    pub unsafe fn setsockopt(&mut self, option_: i32, optval_: *const c_void, optvallen_: usize) -> i32 {
        if self._ctx_terminated {
            return -1;
        }

        let mut rc = self.xsetsockopt(option_, optval_, optvallen_);
        if rc == 0 {
            return 0;
        }

        rc = self.own.options.setsockopt(option_, optval_, optvallen_);
        self.update_pipe_options(option_);
        rc
    }

    pub unsafe fn getsockopt(&mut self, option_: i32, optval_: *mut c_void, optvallen_: *mut usize) -> i32 {
        if ((self._ctx_terminated)) {
            // errno = ETERM;
            return -1;
        }

        //  First, check whether specific socket type overloads the option.
        let mut rc = self.xgetsockopt (option_, optval_, optvallen_);
        if (rc == 0 || get_errno() != EINVAL) {
            return rc;
        }

        if (option_ == ZMQ_RCVMORE) {
            return do_getsockopt(optval_, optvallen_, if self._rcvmore { 1 } else { 0 });
        }

        if (option_ == ZMQ_FD) {
            if (self._thread_safe) {
                // thread safe socket doesn't provide file descriptor
                // errno = EINVAL;
                return -1;
            }

            return do_getsockopt (
              optval_, optvallen_,
              ( (self._mailbox)).get_fd ());
        }

        if (option_ == ZMQ_EVENTS) {
            let rc = self.process_commands (0, false);
            if rc != 0 && (get_errno() == EINTR || get_errno() == ETERM)
            {
                return -1;
            }
            // errno_assert (rc == 0);

            return do_getsockopt (optval_, optvallen_,
                                       (if self.has_out () { ZMQ_POLLOUT } else { 0 })
                                         | (if self.has_in () { ZMQ_POLLIN } else { 0 }));
        }

        if (option_ == ZMQ_LAST_ENDPOINT) {
            return do_getsockopt (optval_, optvallen_, &self._last_endpoint);
        }

        if (option_ == ZMQ_THREAD_SAFE) {
            return do_getsockopt (optval_, optvallen_, if self._thread_safe { 1 } else { 0 });
        }

        return self.own.options.getsockopt (option_, optval_, optvallen_);
    }

    pub fn join(&mut self, group_: &str) -> i32 {
        self.xjoin(group_)
    }

    pub fn leave(&mut self, group_: &str) -> i32 {
        self.xleave(group_)
    }

    pub fn addsignaler(&mut self, s_: *mut signaler_t) {
        self._mailbox.add_signaler(s_);
    }

    pub fn bind(&mut self, endpoint_uri_: &str) -> i32 {
        if self._ctx_terminated {
            return -1;
        }

        let mut rc = self.process_commands(0,false);
        if rc != 0 {
            return -1;
        }

        let mut protocol = String::new();
        let mut address = String::new();
        if self.parse_uri(endpoint_uri_, &mut protocol, &mut address) || self.check_protocol(&protocol) {
            return -1;
        }

        if protocol == "udp" {
            if !self.own.options.type_ == ZMQ_DGRAM || !self.own.options.type_ == ZMQ_DISH {
                return -1;
            }

            let io_thread = self.choose_io_thread(self.own.options.affinity);
            if io_thread.is_null() {
                return -1;
            }

            let mut paddr = address_t::default();
            paddr.address = address;
            paddr.protocol = protocol;
            paddr.parent == self.get_ctx();

        }

        return 0;
    }
}

pub struct out_pipe_t {
    pub pipe: pipe_t,
    pub active: bool,
}

pub struct routing_socket_base_t
{
    pub base: socket_base_t,
    pub _out_pipes: HashMap<blob_t, out_pipe_t>,
    pub _connect_routing_id: String,
}
