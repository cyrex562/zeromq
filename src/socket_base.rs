use std::collections::HashMap;
use std::ffi::c_void;
use std::mem::size_of_val;
use std::ptr::null_mut;
use libc::{clock_t, EAGAIN, EINTR, EINVAL};
use crate::add;
use crate::address::address_t;
use crate::array::{array_item_t, array_t};
use crate::blob::blob_t;
use crate::command::command_t;
use crate::ctx::ctx_t;
use crate::defines::{handle_t, ZMQ_DEALER, ZMQ_DGRAM, ZMQ_DISH, ZMQ_DONTWAIT, ZMQ_EVENT_CLOSE_FAILED, ZMQ_EVENT_CLOSED, ZMQ_EVENT_CONNECT_DELAYED, ZMQ_EVENT_CONNECT_RETRIED, ZMQ_EVENT_CONNECTED, ZMQ_EVENT_DISCONNECTED, ZMQ_EVENT_LISTENING, ZMQ_EVENTS, ZMQ_FD, ZMQ_LAST_ENDPOINT, ZMQ_LINGER, ZMQ_POLLIN, ZMQ_POLLOUT, ZMQ_PUB, ZMQ_RADIO, ZMQ_RCVMORE, ZMQ_RECONNECT_STOP_AFTER_DISCONNECT, ZMQ_REQ, ZMQ_SNDMORE, ZMQ_SUB, ZMQ_THREAD_SAFE};
use crate::endpoint::{endpoint_uri_pair_t, make_unconnected_connect_endpoint_pair};
use crate::fd::fd_t;
use crate::fq::pipes_t;
use crate::i_mailbox::i_mailbox;
use crate::i_poll_events::i_poll_events;
use crate::mailbox::mailbox_t;
use crate::mailbox_safe::mailbox_safe_t;
use crate::msg::{MSG_MORE, msg_t};
use crate::mutex::mutex_t;
use crate::object::object_t;
use crate::options::{do_getsockopt, get_effective_conflate_option, options_t};
use crate::own::own_t;
use crate::pipe::{i_pipe_events, pipe_t, pipepair};
use crate::poller::poller_t;
use crate::session_base::session_base_t;
use crate::signaler::signaler_t;
use crate::tcp_address::tcp_address_t;
use crate::udp_address::udp_address_t;
use crate::utils::get_errno;

pub type endpoint_pipe_t<'a> = (&'a mut own_t<'a>, &'a mut pipe_t<'a>);
pub type endpoints_t<'a> = HashMap<String, endpoint_pipe_t<'a>>;

pub type map_t<'a> = HashMap<String, &'a mut pipe_t<'a>>;

#[derive(Default)]
pub struct inprocs_t<'a> {
    pub _inprocs: map_t<'a>,
}

impl inprocs_t {
    pub fn emplace(&mut self, endpoint_uri_: &str, pipe_: &mut pipe_t) {
        self._inprocs.insert(endpoint_uri_.to_string(), pipe_);
    }

    pub unsafe fn erase_pipes(&mut self, endpoint_uri_str: &str) -> i32 {
        for (k, v) in self._inprocs.iter_mut() {
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
pub struct socket_base_t<'a> {
    pub own: own_t<'a>,
    pub array_item: array_item_t,
    pub poll_events: dyn i_poll_events,
    pub pipe_events: dyn i_pipe_events,
    pub _mailbox: Option<&'a mut dyn i_mailbox>,
    pub _pipes: pipes_t,
    pub _poller: Option<&'a mut poller_t>,
    pub _handle: Option<&'a mut handle_t>,
    pub _last_tsc: u64,
    pub _ticks: i32,
    pub _rcvmore: bool,
    pub _clock: clock_t,
    pub _monitor_socket: *mut c_void,
    pub _monitor_events: i64,
    pub _last_endpoint: String,
    pub _thread_safe: bool,
    pub _reaper_signaler: Option<signaler_t>,
    pub _monitor_sync: mutex_t,
    pub _disconnected: bool,
    pub _sync: mutex_t,
    pub _endpoints: endpoints_t<'a>,
    pub _inprocs: inprocs_t<'a>,
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

    pub unsafe fn new(parent_: &mut ctx_t, tid_: u32, sid_: i32, thread_safe_: bool) -> Self {
        let mut out = Self {
            own: own_t::new(parent_, tid_),
            array_item: array_item_t::new(),
            poll_events: null_mut(),
            pipe_events: null_mut(),
            _mailbox: None,
            _pipes: array_t::default(),
            _poller: None,
            _handle: None,
            _last_tsc: 0,
            _ticks: 0,
            _rcvmore: false,
            _clock: 0,
            _monitor_socket: null_mut(),
            _monitor_events: 0,
            _last_endpoint: "".to_string(),
            _thread_safe: false,
            _reaper_signaler: None,
            _monitor_sync: mutex_t::new(),
            _disconnected: false,
            _sync: mutex_t::new(),
            _endpoints: Default::default(),
            _inprocs: inprocs_t::default(),
            _tag: 0,
            _ctx_terminated: false,
            _destroyed: false,
        };
        // TODO
        // options.socket_id = sid_;
        //     options.ipv6 = (parent_->get (ZMQ_IPV6) != 0);
        //     options.linger.store (parent_->get (ZMQ_BLOCKY) ? -1 : 0);
        //     options.zero_copy = parent_->get (ZMQ_ZERO_COPY_RECV) != 0;

        if out._thread_safe {
            out._mailbox = Some(&mut mailbox_safe_t::new());
        } else {
            out._mailbox = mailbox_t::new();
        }
        out
    }

    pub fn get_peer_state(&mut self, routing_id: *const c_void, routing_id_size_: usize) -> i32 {
        unimplemented!()
    }

    pub fn get_mailbox(&mut self) -> &mut Option<&mut dyn i_mailbox> {
        &mut self._mailbox
    }

    pub fn parse_uri(&mut self, uri_: &str, protocol_: &mut String, path_: &mut String) -> i32 {
        let mut uri = String::from(uri_);
        let pos = uri.find("://");
        if pos.is_none() {
            return -1;
        }

        *protocol_ = uri_[0..pos.unwrap()].to_string();
        *path_ = uri_[pos.unwrap() + 3..].to_string();

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

    pub unsafe fn attach_pipe(&mut self, pipe_: &mut pipe_t, subscribe_to_all_: bool, locally_initiated_: bool) {
        (*pipe_).set_event_sink(self);
        self._pipes.push_back(pipe_);
        self.xattach_pipe(pipe_, subscribe_to_all_, locally_initiated_);

        if self.is_terminating() {
            self.register_term_acks(1);
            (*pipe_).terminate(false);
        }
    }

    pub unsafe fn setsockopt(&mut self, options: &mut options_t, option_: i32, optval_: *const c_void, optvallen_: usize) -> i32 {
        if self._ctx_terminated {
            return -1;
        }

        let mut rc = self.xsetsockopt(option_, optval_, optvallen_);
        if rc == 0 {
            return 0;
        }

        rc = self.own.options.setsockopt(option_, optval_, optvallen_);
        self.update_pipe_options(options, option_);
        rc
    }

    pub unsafe fn getsockopt(&mut self, option_: i32, optval_: *mut c_void, optvallen_: *mut usize) -> i32 {
        if (self._ctx_terminated) {
            // errno = ETERM;
            return -1;
        }

        //  First, check whether specific socket type overloads the option.
        let mut rc = self.xgetsockopt(option_, optval_, optvallen_);
        if rc == 0 || get_errno() != EINVAL {
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

            return do_getsockopt(
                optval_, optvallen_,
                ((self._mailbox)).get_fd());
        }

        if (option_ == ZMQ_EVENTS) {
            let rc = self.process_commands(0, false);
            if rc != 0 && (get_errno() == EINTR || get_errno() == ETERM) {
                return -1;
            }
            // errno_assert (rc == 0);

            return do_getsockopt(optval_, optvallen_,
                                 (if self.has_out() { ZMQ_POLLOUT } else { 0 }) | (if self.has_in() { ZMQ_POLLIN } else { 0 }));
        }

        if (option_ == ZMQ_LAST_ENDPOINT) {
            return do_getsockopt(optval_, optvallen_, &self._last_endpoint);
        }

        if (option_ == ZMQ_THREAD_SAFE) {
            return do_getsockopt(optval_, optvallen_, if self._thread_safe { 1 } else { 0 });
        }

        return self.own.options.getsockopt(option_, optval_, optvallen_);
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

    pub unsafe fn bind(&mut self, endpoint_uri_: &str) -> i32 {
        if self._ctx_terminated {
            return -1;
        }

        let mut rc = self.process_commands(0, false);
        if rc != 0 {
            return -1;
        }

        let mut protocol = String::new();
        let mut address = String::new();
        if self.parse_uri(endpoint_uri_, &mut protocol, &mut address) || self.check_protocol(&protocol) {
            return -1;
        }

        if protocol == "udp" {
            if !self.own.options.type_ == ZMQ_DGRAM  || !self.own.options.type_ == ZMQ_DISH  {
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

    pub unsafe fn connect(&mut self, endpoint_uri_: &str) -> i32 {
        self.connect_internal(endpoint_uri_)
    }

    pub unsafe fn connect_internal(&mut self, endpoint_uri_: &str) -> i32 {
        if ((self._ctx_terminated)) {
            // errno = ETERM;
            return -1;
        }

        //  Process pending commands, if any.
        let mut rc = self.process_commands(0, false);
        if ((rc != 0)) {
            return -1;
        }

        //  Parse endpoint_uri_ string.
        // std::string protocol;
        let mut protocol = String::new();
        let mut address = String::new();
        // std::string address;
        if (self.parse_uri(endpoint_uri_, &mut protocol, &mut address) || self.check_protocol(&protocol)) {
            return -1;
        }

        //     if (protocol == protocol_name::inproc) {
        //         //  TODO: inproc connect is specific with respect to creating pipes
        //         //  as there's no 'reconnect' functionality implemented. Once that
        //         //  is in place we should follow generic pipe creation algorithm.
        //
        //         //  Find the peer endpoint.
        //         const endpoint_t peer = find_endpoint (endpoint_uri_);
        //
        //         // The total HWM for an inproc connection should be the sum of
        //         // the binder's HWM and the connector's HWM.
        //         const int sndhwm = peer.socket == NULL ? options.sndhwm
        //                            : options.sndhwm != 0 && peer.options.rcvhwm != 0
        //                              ? options.sndhwm + peer.options.rcvhwm
        //                              : 0;
        //         const int rcvhwm = peer.socket == NULL ? options.rcvhwm
        //                            : options.rcvhwm != 0 && peer.options.sndhwm != 0
        //                              ? options.rcvhwm + peer.options.sndhwm
        //                              : 0;
        //
        //         //  Create a bi-directional pipe to connect the peers.
        //         object_t *parents[2] = {this, peer.socket == NULL ? this : peer.socket};
        //         pipe_t *new_pipes[2] = {NULL, NULL};
        //
        //         const bool conflate = get_effective_conflate_option (options);
        //
        //         int hwms[2] = {conflate ? -1 : sndhwm, conflate ? -1 : rcvhwm};
        //         bool conflates[2] = {conflate, conflate};
        //         rc = pipepair (parents, new_pipes, hwms, conflates);
        //         if (!conflate) {
        //             new_pipes[0]->set_hwms_boost (peer.options.sndhwm,
        //                                           peer.options.rcvhwm);
        //             new_pipes[1]->set_hwms_boost (options.sndhwm, options.rcvhwm);
        //         }
        //
        //         // errno_assert (rc == 0);
        //
        //         if (!peer.socket) {
        //             //  The peer doesn't exist yet so we don't know whether
        //             //  to send the routing id message or not. To resolve this,
        //             //  we always send our routing id and drop it later if
        //             //  the peer doesn't expect it.
        //             send_routing_id (new_pipes[0], options);
        //
        // // #ifdef ZMQ_BUILD_DRAFT_API
        //             //  If set, send the hello msg of the local socket to the peer.
        //             if (options.can_send_hello_msg && options.hello_msg.size () > 0) {
        //                 send_hello_msg (new_pipes[0], options);
        //             }
        // // #endif
        //
        //             const endpoint_t endpoint = {this, options};
        //             pend_connection (std::string (endpoint_uri_), endpoint, new_pipes);
        //         } else {
        //             //  If required, send the routing id of the local socket to the peer.
        //             if (peer.options.recv_routing_id) {
        //                 send_routing_id (new_pipes[0], options);
        //             }
        //
        //             //  If required, send the routing id of the peer to the local socket.
        //             if (options.recv_routing_id) {
        //                 send_routing_id (new_pipes[1], peer.options);
        //             }
        //
        // // #ifdef ZMQ_BUILD_DRAFT_API
        //             //  If set, send the hello msg of the local socket to the peer.
        //             if (options.can_send_hello_msg && options.hello_msg.size () > 0) {
        //                 send_hello_msg (new_pipes[0], options);
        //             }
        //
        //             //  If set, send the hello msg of the peer to the local socket.
        //             if (peer.options.can_send_hello_msg
        //                 && peer.options.hello_msg.size () > 0) {
        //                 send_hello_msg (new_pipes[1], peer.options);
        //             }
        //
        //             if (peer.options.can_recv_disconnect_msg
        //                 && peer.options.disconnect_msg.size () > 0)
        //                 new_pipes[0]->set_disconnect_msg (peer.options.disconnect_msg);
        // // #endif
        //
        //             //  Attach remote end of the pipe to the peer socket. Note that peer's
        //             //  seqnum was incremented in find_endpoint function. We don't need it
        //             //  increased here.
        //             send_bind (peer.socket, new_pipes[1], false);
        //         }
        //
        //         //  Attach local end of the pipe to this socket object.
        //         attach_pipe (new_pipes[0], false, true);
        //
        //         // Save last endpoint URI
        //         _last_endpoint.assign (endpoint_uri_);
        //
        //         // remember inproc connections for disconnect
        //         _inprocs.emplace (endpoint_uri_, new_pipes[0]);
        //
        //         options.connected = true;
        //         return 0;
        //     }
        let is_single_connect = (self.own.options.type_ == ZMQ_DEALER  || self.own.options.type_ == ZMQ_SUB  || self.own.options.type_ == ZMQ_PUB  || self.own.options.type_ == ZMQ_REQ );
        if ((is_single_connect)) {
            if (0 != self._endpoints.count()) {
                // There is no valid use for multiple connects for SUB-PUB nor
                // DEALER-ROUTER nor REQ-REP. Multiple connects produces
                // nonsensical results.
                return 0;
            }
        }

        //  Choose the I/O thread to run the session in.
        let mut io_thread = self.choose_io_thread(self.own.options.affinity);
        if (!io_thread) {
            // errno = EMTHREAD;
            return -1;
        }

        // address_t *paddr = new (std::nothrow) address_t (protocol, address, this->get_ctx ());
        // alloc_assert (paddr);
        let mut paddr = address_t::new(&mut protocol, &mut address, self.get_ctx());

        //  Resolve address (if needed by the protocol)
        if (protocol == "tcp") {
            //  Do some basic sanity checks on tcp:// address syntax
            //  - hostname starts with digit or letter, with embedded '-' or '.'
            //  - IPv6 address may contain hex chars and colons.
            //  - IPv6 link local address may contain % followed by interface name / zone_id
            //    (Reference: https://tools.ietf.org/html/rfc4007)
            //  - IPv4 address may contain decimal digits and dots.
            //  - Address must end in ":port" where port is *, or numeric
            //  - Address may contain two parts separated by ':'
            //  Following code is quick and dirty check to catch obvious errors,
            //  without trying to be fully accurate.
            // const char *check = address.c_str ();
            let check = address.clone();
            // if (isalnum (*check) || isxdigit (*check) || *check == '['
            //     || *check == ':') {
            //     check++;
            //     while (isalnum (*check) || isxdigit (*check) || *check == '.'
            //            || *check == '-' || *check == ':' || *check == '%'
            //            || *check == ';' || *check == '[' || *check == ']'
            //            || *check == '_' || *check == '*') {
            //         check++;
            //     }
            // }
            //  Assume the worst, now look for success
            rc = -1;
            //  Did we reach the end of the address safely?
            // if (*check == 0) {
            //     //  Do we have a valid port string? (cannot be '*' in connect
            //     check = strrchr (address.c_str (), ':');
            //     if (check) {
            //         check++;
            //         if (*check && (isdigit (*check)))
            //             rc = 0; //  Valid
            //     }
            // }
            if (rc == -1) {
                // errno = EINVAL;
                // LIBZMQ_DELETE (paddr);
                return -1;
            }
            //  Defer resolution until a socket is opened
            paddr.resolved.tcp_addr = tcp_address_t::default();
        }
        // // #ifdef ZMQ_HAVE_WS
        // // #ifdef ZMQ_HAVE_WSS
        //     else if (protocol == protocol_name::ws || protocol == protocol_name::wss) {
        //         if (protocol == protocol_name::wss) {
        //             paddr->resolved.wss_addr = new (std::nothrow) wss_address_t ();
        //             alloc_assert (paddr->resolved.wss_addr);
        //             rc = paddr->resolved.wss_addr->resolve (address.c_str (), false,
        //                                                     options.ipv6);
        //         } else
        // // #else
        //     else if (protocol == protocol_name::ws) {
        // #endif
        //         {
        //             paddr->resolved.ws_addr = new (std::nothrow) ws_address_t ();
        //             alloc_assert (paddr->resolved.ws_addr);
        //             rc = paddr->resolved.ws_addr->resolve (address.c_str (), false,
        //                                                    options.ipv6);
        //         }
        //
        //         if (rc != 0) {
        //             // LIBZMQ_DELETE (paddr);
        //             return -1;
        //         }
        //     }
        // #endif

        // #if defined ZMQ_HAVE_IPC
        //     else if (protocol == protocol_name::ipc) {
        //         paddr->resolved.ipc_addr = new (std::nothrow) ipc_address_t ();
        //         alloc_assert (paddr->resolved.ipc_addr);
        //         int rc = paddr->resolved.ipc_addr->resolve (address.c_str ());
        //         if (rc != 0) {
        //             LIBZMQ_DELETE (paddr);
        //             return -1;
        //         }
        //     }
        // #endif

        if (protocol == "udp") {
            if (self.own.options.type_ != ZMQ_RADIO) {
                // errno = ENOCOMPATPROTO;
                // LIBZMQ_DELETE (paddr);
                return -1;
            }

            paddr.resolved.udp_addr = udp_address_t::new(); //new (std::nothrow) udp_address_t ();
            // alloc_assert (paddr->resolved.udp_addr);
            rc = paddr.resolved.udp_addr.resolve(&mut address, false,
                                                 self.own.options.ipv6);
            if (rc != 0) {
                // LIBZMQ_DELETE (paddr);
                return -1;
            }
        }

        // TBD - Should we check address for ZMQ_HAVE_NORM???

        // #ifdef ZMQ_HAVE_OPENPGM
        //     if (protocol == protocol_name::pgm || protocol == protocol_name::epgm) {
        //         struct pgm_addrinfo_t *res = NULL;
        //         uint16_t port_number = 0;
        //         int rc =
        //           pgm_socket_t::init_address (address.c_str (), &res, &port_number);
        //         if (res != NULL)
        //             pgm_freeaddrinfo (res);
        //         if (rc != 0 || port_number == 0) {
        //             return -1;
        //         }
        //     }
        // #endif
        // #if defined ZMQ_HAVE_TIPC
        //     else if (protocol == protocol_name::tipc) {
        //         paddr->resolved.tipc_addr = new (std::nothrow) tipc_address_t ();
        //         alloc_assert (paddr->resolved.tipc_addr);
        //         int rc = paddr->resolved.tipc_addr->resolve (address.c_str ());
        //         if (rc != 0) {
        //             LIBZMQ_DELETE (paddr);
        //             return -1;
        //         }
        //         const sockaddr_tipc *const saddr =
        //           reinterpret_cast<const sockaddr_tipc *> (
        //             paddr->resolved.tipc_addr->addr ());
        //         // Cannot connect to random Port Identity
        //         if (saddr->addrtype == TIPC_ADDR_ID
        //             && paddr->resolved.tipc_addr->is_random ()) {
        //             LIBZMQ_DELETE (paddr);
        //             errno = EINVAL;
        //             return -1;
        //         }
        //     }
        // #endif
        // #if defined ZMQ_HAVE_VMCI
        //     else if (protocol == protocol_name::vmci) {
        //         paddr->resolved.vmci_addr =
        //           new (std::nothrow) vmci_address_t (this->get_ctx ());
        //         alloc_assert (paddr->resolved.vmci_addr);
        //         int rc = paddr->resolved.vmci_addr->resolve (address.c_str ());
        //         if (rc != 0) {
        //             LIBZMQ_DELETE (paddr);
        //             return -1;
        //         }
        //     }
        // #endif

        //  Create session.
        let mut session = session_base_t::create(io_thread, true, self, &self.own.options, paddr);
        // errno_assert (session);

        //  PGM does not support subscription forwarding; ask for all data to be
        //  sent to this pipe. (same for NORM, currently?)
        // #if defined ZMQ_HAVE_OPENPGM && defined ZMQ_HAVE_NORM
        //     const bool subscribe_to_all =
        //       protocol == protocol_name::pgm || protocol == protocol_name::epgm
        //       || protocol == protocol_name::norm || protocol == protocol_name::udp;
        // #elif defined ZMQ_HAVE_OPENPGM
        //     const bool subscribe_to_all = protocol == protocol_name::pgm
        //                                   || protocol == protocol_name::epgm
        //                                   || protocol == protocol_name::udp;
        // #elif defined ZMQ_HAVE_NORM
        //     const bool subscribe_to_all =
        //       protocol == protocol_name::norm || protocol == protocol_name::udp;
        // #else
        let subscribe_to_all = protocol == "udp";
        // #endif
        //     pipe_t *newpipe = NULL;
        let mut newpipe: &mut pipe_t;

        if self.own.options.immediate != 1 || subscribe_to_all {
            //  Create a bi-directional pipe.
            // object_t *parents[2] = {this, session};
            let mut parents: [&mut object_t; 2] = [&mut self as &mut object_t, session as &mut object_t];
            // pipe_t *new_pipes[2] = {NULL, NULL};
            let mut new_pipes: [Option<&mut pipe_t>; 2] = [None, None];

            let conflate = get_effective_conflate_option(&self.own.options);

            let mut hwms: [i32; 2] = [
                if conflate { -1 } else { self.own.options.sndhwm },
                if conflate { -1 } else { self.own.options.rcvhwm }
            ];
            let mut conflates: [bool; 2] = [conflate, conflate];
            rc = pipepair(parents, &mut new_pipes, hwms, conflates);
            // errno_assert (rc == 0);

            //  Attach local end of the pipe to the socket object.
            self.attach_pipe(new_pipes[0].unwrap(), subscribe_to_all, true);
            newpipe = new_pipes[0].unwrap();

            //  Attach remote end of the pipe to the session object later on.
            session.attach_pipe(new_pipes[1].unwrap());
        }

        //  Save last endpoint URI
        self._last_endpoint = paddr.to_string();

        self.add_endpoint(make_unconnected_connect_endpoint_pair(endpoint_uri_),
                          (&mut session), newpipe);
        return 0;
    }

    pub unsafe fn resolve_tcp_addr(&mut self, endpoint_uri_pair_: &mut String, tcp_address_: &mut String) -> String {
        // The resolved last_endpoint is used as a key in the endpoints map.
        // The address passed by the user might not match in the TCP case due to
        // IPv4-in-IPv6 mapping (EG: tcp://[::ffff:127.0.0.1]:9999), so try to
        // resolve before giving up. Given at this stage we don't know whether a
        // socket is connected or bound, try with both.
        if self._endpoints.find(endpoint_uri_pair_) == self._endpoints.end() {
            // tcp_address_t *tcp_addr = new (std::nothrow) tcp_address_t ();
            // alloc_assert (tcp_addr);
            let mut tcp_addr = tcp_address_t::new();
            let mut rc = tcp_addr.resolve(tcp_address_, false, self.own.options.ipv6);

            if rc == 0 {
                tcp_addr.to_string(endpoint_uri_pair_);
                if self._endpoints.find(endpoint_uri_pair_) == self._endpoints.end() {
                    rc = tcp_addr.resolve(tcp_address_, true, self.own.options.ipv6);
                    if rc == 0 {
                        tcp_addr.to_string(endpoint_uri_pair_);
                    }
                }
            }
            // LIBZMQ_DELETE (tcp_addr);
        }
        return endpoint_uri_pair_.clone();
    }

    pub unsafe fn add_endpoint(&mut self, endpoint_pair_: endpoint_uri_pair_t, endpoint_: &mut own_t, pipe_: &mut pipe_t) {
        //  Activate the session. Make it a child of this socket.
        self.launch_child(endpoint_);
        self._endpoints.insert(endpoint_pair_.identifier().clone(),
                               endpoint_pipe_t(endpoint_, pipe_));

        if (pipe_ != null_mut()) {
            (pipe_).set_endpoint_pair(endpoint_pair_);
        }
    }

    pub unsafe fn term_endpoint(&mut self, endpoint_uri_: &str) -> i32 {
        //  Check whether the context hasn't been shut down yet.
        if ((self._ctx_terminated)) {
            // errno = ETERM;
            return -1;
        }

        //  Check whether endpoint address passed to the function is valid.
        if ((!endpoint_uri_)) {
            // errno = EINVAL;
            return -1;
        }

        //  Process pending commands, if any, since there could be pending unprocessed process_own()'s
        //  (from launch_child() for example) we're asked to terminate now.
        let rc = self.process_commands(0, false);
        if ((rc != 0)) {
            return -1;
        }

        //  Parse endpoint_uri_ string.
        // std::string uri_protocol;
        let mut uri_protocol = String::new();
        // std::string uri_path;
        let mut uri_path = String::new();
        if (self.parse_uri(endpoint_uri_, &mut uri_protocol, &mut uri_path) || self.check_protocol(&uri_protocol)) {
            return -1;
        }

        let mut endpoint_uri_str = (endpoint_uri_).to_string();

        // Disconnect an inproc socket
        // if (uri_protocol == protocol_name::inproc) {
        //     return unregister_endpoint (endpoint_uri_str, this) == 0
        //         ? 0
        //         : _inprocs.erase_pipes (endpoint_uri_str);
        // }

        let resolved_endpoint_uri = if uri_protocol == "tcp" { resolve_tcp_addr(endpoint_uri_str, uri_path.c_str()) } else { endpoint_uri_str };

        //  Find the endpoints range (if any) corresponding to the endpoint_uri_pair_ string.
        // const std::pair<endpoints_t::iterator, endpoints_t::iterator> range =
        // _endpoints.equal_range (resolved_endpoint_uri);
        // if (range.first == range.second) {
        //     errno = ENOENT;
        //     return -1;
        // }

        // for (endpoints_t::iterator it = range.first; it != range.second; ++it) {
        // //  If we have an associated pipe, terminate it.
        // if (it->second.second != NULL)
        // it->second.second->terminate (false);
        // term_child (it->second.first);
        for it in self._endpoints.iter_mut() {
            if it.0 == resolved_endpoint_uri {
                it.1.terminate(false);
                self.term_child(it.1)
            }
        }
        self._endpoints.remove(&resolved_endpoint_uri);

        // _endpoints.erase (range.first, range.second);

        if (self.own.options.reconnect_stop & ZMQ_RECONNECT_STOP_AFTER_DISCONNECT) {
            self._disconnected = true;
        }

        return 0;
    }

    pub unsafe fn send(&mut self, msg_: &mut msg_t, flags_: i32) -> i32 {
        //  Check whether the context hasn't been shut down yet.
        if ((self._ctx_terminated)) {
            // errno = ETERM;
            return -1;
        }

        //  Check whether message passed to the function is valid.
        if ((msg_.is_none() || !msg_.check())) {
            // errno = EFAULT;
            return -1;
        }

        //  Process pending commands, if any.
        let mut rc = self.process_commands(0, true);
        if ((rc != 0)) {
            return -1;
        }

        //  Clear any user-visible flags that are set on the message.
        msg_.reset_flags(MSG_MORE);

        //  At this point we impose the flags on the message.
        if (flags_ & ZMQ_SNDMORE) {
            msg_.set_flags(MSG_MORE);
        }

        msg_.reset_metadata();

        //  Try to send the message using method in each socket class
        rc = self.xsend(msg_);
        if (rc == 0) {
            return 0;
        }
        //  Special case for ZMQ_PUSH: -2 means pipe is dead while a
        //  multi-part send is in progress and can't be recovered, so drop
        //  silently when in blocking mode to keep backward compatibility.
        if ((rc == -2)) {
            if (!((flags_ & ZMQ_DONTWAIT != 0) || self.own.options.sndtimeo == 0)) {
                rc = msg_.close();
                // errno_assert (rc == 0);
                rc = msg_.init2();
                // errno_assert (rc == 0);
                return 0;
            }
        }
        // if ((errno != EAGAIN)) {
        //     return -1;
        // }

        //  In case of non-blocking send we'll simply propagate
        //  the error - including EAGAIN - up the stack.
        if ((flags_ & ZMQ_DONTWAIT != 0) || self.own.options.sndtimeo == 0) {
            return -1;
        }

        //  Compute the time when the timeout should occur.
        //  If the timeout is infinite, don't care.
        let mut timeout = self.own.options.sndtimeo;
        let end = if timeout < 0 { 0 } else { (self._clock.now_ms() + timeout) };

        //  Oops, we couldn't send the message. Wait for the next
        //  command, process it and try to send the message again.
        //  If timeout is reached in the meantime, return EAGAIN.
        loop {
            if ((self.process_commands(timeout, false) != 0)) {
                return -1;
            }
            rc = self.xsend(msg_);
            if (rc == 0) {
                break;
            }
            // if ((errno != EAGAIN)) {
            //     return -1;
            // }
            if (timeout > 0) {
                timeout = (end - self._clock.now_ms());
                if (timeout <= 0) {
                    // errno = EAGAIN;
                    return -1;
                }
            }
        }

        return 0;
    }

    pub unsafe fn recv(&mut self, msg_: &mut msg_t, flags_: i32) -> i32 {
        //  Check whether the context hasn't been shut down yet.
        if ((self._ctx_terminated)) {
            // errno = ETERM;
            return -1;
        }

        //  Check whether message passed to the function is valid.
        if ((msg_.is_none() || !msg_.check())) {
            // errno = EFAULT;
            return -1;
        }

        //  Once every inbound_poll_rate messages check for signals and process
        //  incoming commands. This happens only if we are not polling altogether
        //  because there are messages available all the time. If poll occurs,
        //  ticks is set to zero and thus we avoid this code.
        //
        //  Note that 'recv' uses different command throttling algorithm (the one
        //  described above) from the one used by 'send'. This is because counting
        //  ticks is more efficient than doing RDTSC all the time.
        self._ticks += 1;
        if (self._ticks == self._inbound_poll_rate) {
            if ((self.process_commands(0, false) != 0)) {
                return -1;
            }
            self._ticks = 0;
        }

        //  Get the message.
        let mut rc = self.xrecv(msg_);
        if ((rc != 0 && get_errno() != EAGAIN)) {
            return -1;
        }

        //  If we have the message, return immediately.
        if (rc == 0) {
            self.extract_flags(msg_);
            return 0;
        }

        //  If the message cannot be fetched immediately, there are two scenarios.
        //  For non-blocking recv, commands are processed in case there's an
        //  activate_reader command already waiting in a command pipe.
        //  If it's not, return EAGAIN.
        if ((flags_ & ZMQ_DONTWAIT != 0) || self.own.options.rcvtimeo == 0) {
            if ((self.process_commands(0, false) != 0)) {
                return -1;
            }
            self._ticks = 0;

            rc = self.xrecv(msg_);
            if (rc < 0) {
                return rc;
            }
            self.extract_flags(msg_);

            return 0;
        }

        //  Compute the time when the timeout should occur.
        //  If the timeout is infinite, don't care.
        let mut timeout = self.own.options.rcvtimeo;
        let mut end = if timeout < 0 { 0 } else { (self._clock.now_ms() + timeout) };

        //  In blocking scenario, commands are processed over and over again until
        //  we are able to fetch a message.
        let mut block = (self._ticks != 0);
        while (true) {
            if ((self.process_commands(if block { timeout } else { 0 }, false) != 0)) {
                return -1;
            }
            rc = self.xrecv(msg_);
            if (rc == 0) {
                self._ticks = 0;
                break;
            }
            if ((get_errno() != EAGAIN)) {
                return -1;
            }
            block = true;
            if (timeout > 0) {
                timeout = (end - self._clock.now_ms());
                if (timeout <= 0) {
                    // errno = EAGAIN;
                    return -1;
                }
            }
        }

        self.extract_flags(msg_);
        return 0;
    }

    pub unsafe fn close(&mut self) -> i32 {
        // scoped_optional_lock_t sync_lock (_thread_safe ? &_sync : NULL);

        //  Remove all existing signalers for thread safe sockets
        if (self._thread_safe) {
            ((self._mailbox)).clear_signalers();
        }

        //  Mark the socket as dead
        self._tag = 0xdeadbeef;


        //  Transfer the ownership of the socket from this application thread
        //  to the reaper thread which will take care of the rest of shutdown
        //  process.
        self.send_reap(self);

        return 0;
    }

    pub fn has_in(&mut self) -> bool {
        self.xhas_in()
    }

    pub fn has_out(&mut self) -> bool {
        self.xhas_out()
    }

    pub unsafe fn start_reaping(&mut self, poller_: &mut poller_t) {
        //  Plug the socket to the reaper thread.
        self._poller = Some(poller_);

        // fd_t fd;
        let mut fd: fd_t = -1;

        if (!self._thread_safe) {
            fd = ((self._mailbox)).get_fd();
        } else {
            // scoped_optional_lock_t sync_lock (_thread_safe ? &_sync : NULL);

            // _reaper_signaler = new (std::nothrow) signaler_t ();
            self._reaper_signaler = Some(signaler_t::new());
            // zmq_assert (_reaper_signaler);

            //  Add signaler to the safe mailbox
            fd = self._reaper_signaler.get_fd();
            ((self._mailbox)).add_signaler(self._reaper_signaler);

            //  Send a signal to make sure reaper handle existing commands
            self._reaper_signaler.send();
        }

        self._handle = self._poller.add_fd(fd, self);
        self._poller.set_pollin(self._handle);

        //  Initialise the termination and check whether it can be deallocated
        //  immediately.
        self.terminate();
        self.check_destroy();
    }

    pub unsafe fn process_commands(&mut self, timeout_: i32, throttle_: bool) -> i32 {
        if (timeout_ == 0) {
            //  If we are asked not to wait, check whether we haven't processed
            //  commands recently, so that we can throttle the new commands.

            //  Get the CPU's tick counter. If 0, the counter is not available.
            let tsc = clock_t::rdtsc();

            //  Optimised version of command processing - it doesn't have to check
            //  for incoming commands each time. It does so only if certain time
            //  elapsed since last command processing. Command delay varies
            //  depending on CPU speed: It's ~1ms on 3GHz CPU, ~2ms on 1.5GHz CPU
            //  etc. The optimisation makes sense only on platforms where getting
            //  a timestamp is a very cheap operation (tens of nanoseconds).
            if (tsc && throttle_) {
                //  Check whether TSC haven't jumped backwards (in case of migration
                //  between CPU cores) and whether certain time have elapsed since
                //  last command processing. If it didn't do nothing.
                if (tsc >= self._last_tsc && tsc - self._last_tsc <= _max_command_delay) {
                    return 0;
                }
                self._last_tsc = tsc;
            }
        }

        //  Check whether there are any commands pending for this thread.
        let mut cmd = command_t::new();
        let mut rc = self._mailbox.recv(&cmd, timeout_);

        // if (rc != 0 && errno == EINTR)
        //     return -1;

        //  Process all available commands.
        while (rc == 0 || get_errno() == EINTR) {
            if (rc == 0) {
                cmd.destination.process_command(cmd);
            }
            rc = self._mailbox.recv(&cmd, 0);
        }

        // zmq_assert (errno == EAGAIN);

        if (self._ctx_terminated) {
            // errno = ETERM;
            return -1;
        }

        return 0;
    }

    pub unsafe fn process_stop(&mut self) {
        self.stop_monitor(false);
        self._ctx_terminated = true;
    }

    pub unsafe fn process_bind(&mut self, pipe_: &mut pipe_t) {
        self.attach_pipe(pipe_, false, false);
    }

    pub unsafe fn process_term(&mut self, linger_: i32) {
        //  Unregister all inproc endpoints associated with this socket.
        //  Doing this we make sure that no new pipes from other sockets (inproc)
        //  will be initiated.
        self.unregister_endpoints(self);

        //  Ask all attached pipes to terminate.
        // for (pipes_t::size_type i = 0, size = _pipes.size (); i != size; ++i)
        for i in 0..self._pipes.len() {
            //  Only inprocs might have a disconnect message set
            self._pipes[i].send_disconnect_msg();
            self._pipes[i].terminate(false);
        }
        self.register_term_acks((self._pipes.size()));

        //  Continue the termination process immediately.
        self.own.process_term(linger_);
    }

    pub unsafe fn process_term_endpoint(&mut self, endpoint_: &mut String) {
        self.term_endpoint(endpoint_);
    }

    pub unsafe fn process_pipe_stats_publish(&mut self, outbound_queue_count_: u64, inbound_queue_count_: u64, endpoint_pair_: &mut endpoint_uri_pair_t) {
        let mut values: [u64; 2] = [outbound_queue_count_, inbound_queue_count_];
        self.event(endpoint_pair_, &values, 2, ZMQ_EVENT_PIPES_STATS);
    }

    pub unsafe fn query_pipes_stats(&mut self) -> i32 {
        {
            // scoped_lock_t lock (_monitor_sync);
            if !(self._monitor_events & ZMQ_EVENT_PIPES_STATS) {
                // errno = EINVAL;
                return -1;
            }
        }
        if (self._pipes.size() == 0) {
            // errno = EAGAIN;
            return -1;
        }
        // for (pipes_t::size_type i = 0, size = _pipes.size (); i != size; ++i)
        for i in 0..self._pipes.size() {
            self._pipes[i].send_stats_to_peer(self as *mut own_t);
        }

        return 0;
    }

    pub fn update_pipe_options(&mut self, options: &mut options_t, option_: i32) {
        if (option_ == ZMQ_SNDHWM || option_ == ZMQ_RCVHWM) {
            // for (pipes_t::size_type i = 0, size = _pipes.size (); i != size; ++i)
            for i in 0..self._pipes.size() {
                self._pipes[i].set_hwms(options.rcvhwm, options.sndhwm);
                self._pipes[i].send_hwms_to_peer(options.sndhwm, options.rcvhwm);
            }
        }
    }

    pub fn process_destroy(&mut self) {
        self._destroyed = true;
    }

    pub fn xsetsockopt(&mut self, a: i32, b: *const c_void, c: usize) -> i32 {
        // self.setsockopt(a, b, c)
        return -1;
    }

    pub fn xgetsockopt(&mut self, a: i32, b: *mut c_void, c: *mut usize) -> i32 {
        // self.getsockopt(a, b, c)
        return -1;
    }

    pub fn xhas_out(&mut self) -> bool {
        return false;
    }

    pub fn xsend(&mut self, msg_: *mut msg_t) -> i32 {
        return -1;
    }

    pub fn xhas_in(&mut self) -> bool {
        return false;
    }

    pub fn xjoin(&mut self, group: &str) -> i32 {
        return -1;
    }

    pub fn xleave(&mut self, group_: &str) -> i32 {
        return -1;
    }

    pub fn xrecv(&mut self, msg_: *mut msg_t) -> i32 {
        return -1;
    }

    pub fn xread_activated(&mut self, pipe_: *mut pipe_t) {
        unimplemented!()
    }

    pub fn xwrite_activated(&mut self, pipe_: *mut pipe_t) {
        unimplemented!()
    }

    pub fn xhiccuped(&mut self, pipe_: *mut pipe_t) {
        unimplemented!()
    }

    pub unsafe fn in_event(&mut self) {
        {
            // scoped_optional_lock_t sync_lock (_thread_safe ? &_sync : NULL);

            //  If the socket is thread safe we need to unsignal the reaper signaler
            if (self._thread_safe) {
                self._reaper_signaler.recv();
            }

            self.process_commands(0, false);
        }
        self.check_destroy();
    }

    pub unsafe fn out_event(&mut self) {
        unimplemented!()
    }

    pub unsafe fn timer_event(&mut self) {
        unimplemented!()
    }

    pub unsafe fn check_destroy(&mut self) {
        //  If the object was already marked as destroyed, finish the deallocation.
        if (self._destroyed) {
            //  Remove the socket from the reaper's poller.
            self._poller.rm_fd(self._handle);

            //  Remove the socket from the context.
            self.destroy_socket(self);

            //  Notify the reaper about the fact.
            self.send_reaped();

            //  Deallocate.
            self.process_destroy();
        }
    }

    pub unsafe fn read_activated(&mut self, pipe_: *mut pipe_t) {
        self.xread_activated(pipe_);
    }

    pub unsafe fn write_activated(&mut self, pipe_: *mut pipe_t) {
        self.xwrite_activated(pipe_);
    }

    pub unsafe fn pipe_terminated(&mut self, pipe_: *mut pipe_t) {
        //  Notify the specific socket type about the pipe termination.
        self.xpipe_terminated(pipe_);

        // Remove pipe from inproc pipes
        self._inprocs.erase_pipe(pipe_);

        //  Remove the pipe from the list of attached pipes and confirm its
        //  termination if we are already shutting down.
        self._pipes.erase(pipe_);

        // Remove the pipe from _endpoints (set it to NULL).
        let identifier = pipe_.get_endpoint_pair().identifier();
        if !identifier.empty() {
            // std::pair<endpoints_t::iterator, endpoints_t::iterator> range;
            // range = _endpoints.equal_range (identifier);

            // for (endpoints_t::iterator it = range.first; it != range.second; ++it)
            for it in self._endpoints.iter_mut() {
                // if (it.1.1 == pipe_) {
                //     it.1.1 = NULL;
                //     break;
                // }
            }
        }

        if (self.is_terminating()) {
            self.unregister_term_ack();
        }
    }

    pub unsafe fn extract_flags(&mut self, msg_: * msg_t) {
        //  Test whether routing_id flag is valid for this socket type.
        if ((msg_.flags() & msg_t::routing_id)) {
            // zmq_assert(options.recv_routing_id)
        };

        //  Remove MORE flag.
        self._rcvmore = (msg_.flags() & MSG_MORE) != 0;
    }

    pub unsafe fn monitor(&mut self, endpoint_: &str, events_: u64, event_version_: i32, type_: i32) -> i32 {
        if ((self._ctx_terminated)) {
            // errno = ETERM;
            return -1;
        }

        //  Event version 1 supports only first 16 events.
        if ((event_version_ == 1 && events_ >> 16 != 0)) {
            // errno = EINVAL;
            return -1;
        }

        //  Support deregistering monitoring endpoints as well
        if (endpoint_ == null_mut()) {
            self.stop_monitor();
            return 0;
        }
        //  Parse endpoint_uri_ string.
        // std::string protocol;
        let mut protocol = String::new();
        // std::string address;
        let mut address = String::new();
        if (self.parse_uri(endpoint_, &mut protocol, &mut address) || self.check_protocol(&protocol)) {
            return -1;
        }

        //  Event notification only supported over inproc://
        // if (protocol != protocol_name::inproc) {
        //     // errno = EPROTONOSUPPORT;
        //     return -1;
        // }

        // already monitoring. Stop previous monitor before starting new one.
        if (self._monitor_socket != null_mut()) {
            self.stop_monitor(true);
        }

        // Check if the specified socket type is supported. It must be a
        // one-way socket types that support the SNDMORE flag.
        // switch (type_) {
        //     case ZMQ_PAIR:
        //         break;
        //     case ZMQ_PUB:
        //         break;
        //     case ZMQ_PUSH:
        //         break;
        //     default:
        //         errno = EINVAL;
        //         return -1;
        // }

        //  Register events to monitor
        self._monitor_events = events_ as i64;
        self.own.options.monitor_event_version = event_version_;
        //  Create a monitor socket of the specified type.
        self._monitor_socket = zmq_socket(self.get_ctx(), type_);
        if (self._monitor_socket == null_mut()) {
            return -1;
        }

        //  Never block context termination on pending event messages
        let mut linger = 0;
        let mut rc = zmq_setsockopt(self._monitor_socket, ZMQ_LINGER, &linger, 4);
        if (rc == -1) {
            self.stop_monitor(false);
        }

        //  Spawn the monitor socket endpoint
        rc = zmq_bind(self._monitor_socket, endpoint_);
        if (rc == -1) {
            self.stop_monitor(false);
        }
        return rc;
    }

    pub unsafe fn event_connected(&mut self, endpoint_uri_pair_: &endpoint_uri_pair_t, err_: i32) {
        let mut values: [u64; 1] = [err_ as u64];
        self.event(endpoint_uri_pair_, &values, 1, ZMQ_EVENT_CONNECTED as u64);
    }

    pub unsafe fn event_connect_delayed(&mut self, endpoint_uri_pair_: &endpoint_uri_pair_t, err_: i32) {
        let mut values: [u64; 1] = [err_ as u64];
        self.event(endpoint_uri_pair_, &values, 1, ZMQ_EVENT_CONNECT_DELAYED as u64);
    }

    pub unsafe fn event_connect_retried(&mut self, endpoint_uri_pair_: &endpoint_uri_pair_t, err_: i32) {
        let mut values: [u64; 1] = [err_ as u64];
        self.event(endpoint_uri_pair_, &values, 1, ZMQ_EVENT_CONNECT_RETRIED as u64);
    }

    pub unsafe fn event_listening(&mut self, endpoint_uri_pair_: &endpoint_uri_pair_t, fd_: fd_t) {
        let mut values: [u64; 1] = [fd_ as u64];
        self.event(endpoint_uri_pair_, &values, 1, ZMQ_EVENT_LISTENING as u64);
    }

    pub unsafe fn event_bind_failed(&mut self, endpoint_uri_pair_: &endpoint_uri_pair_t, err_: i32) {
        let mut values: [u64; 1] = [err_ as u64];
        self.event(endpoint_uri_pair_, &values, 1, ZMQ_EVENT_BIND_FAILED);
    }

    pub unsafe fn event_accepted(&mut self, endpoint_uri_pair_: &endpoint_uri_pair_t, fd_: fd_t) {
        let mut values: [u64; 1] = [fd_ as u64];
        self.event(endpoint_uri_pair_, &values, 1, ZMQ_EVENT_ACCEPTED);
    }

    pub unsafe fn event_accept_failed(&mut self, endpoint_uri_pair_: &endpoint_uri_pair_t, err_: i32) {
        let mut values: [u64; 1] = [err_ as u64];
        self.event(endpoint_uri_pair_, &values, 1, ZMQ_EVENT_ACCEPT_FAILED);
    }

    pub unsafe fn event_closed(&mut self, endpoint_uri_pair_: &endpoint_uri_pair_t, fd_: fd_t) {
        let mut values: [u64; 1] = [fd_ as u64];
        self.event(endpoint_uri_pair_, &values, 1, ZMQ_EVENT_CLOSED as u64);
    }

    pub unsafe fn event_close_failed(&mut self, endpoint_uri_pair_: &endpoint_uri_pair_t, err_: i32) {
        let mut values: [u64; 1] = [err_ as u64];
        self.event(endpoint_uri_pair_, &values, 1, ZMQ_EVENT_CLOSE_FAILED as u64);
    }

    pub unsafe fn event_disconnected(&mut self, endpoint_uri_pair_: &endpoint_uri_pair_t, fd_: fd_t) {
        let mut values: [u64; 1] = [fd_ as u64];
        self.event(endpoint_uri_pair_, &values, 1, ZMQ_EVENT_DISCONNECTED as u64);
    }

    pub unsafe fn event_handshake_failed_no_detail(&mut self, endpoint_pair_: &endpoint_uri_pair_t, err_: i32) {
        let mut values: [u64; 1] = [err_ as u64];
        self.event(endpoint_pair_, &values, 1, ZMQ_EVENT_HANDSHAKE_FAILED_NO_DETAIL);
    }

    pub unsafe fn event_handshake_failed_protocol(&mut self, endpoint_pair_: &endpoint_uri_pair_t, err_: i32, protocol_: &str) {
        let mut values: [u64; 2] = [err_ as u64, protocol_ as u64];
        self.event(endpoint_pair_, &values, 2, ZMQ_EVENT_HANDSHAKE_FAILED_PROTOCOL);
    }

    pub unsafe fn event_handshake_failed_auth(&mut self, endpoint_pair_: &endpoint_uri_pair_t, err_: i32, protocol_: &str) {
        let mut values: [u64; 2] = [err_ as u64, protocol_ as u64];
        self.event(endpoint_pair_, &values, 2, ZMQ_EVENT_HANDSHAKE_FAILED_AUTH);
    }

    pub unsafe fn event_handshake_succeeded(&mut self, endpoint_pair_: &endpoint_uri_pair_t, err_: i32) {
        let mut values: [u64; 1] = [err_ as u64];
        self.event(endpoint_pair_, &values, 1, ZMQ_EVENT_HANDSHAKE_SUCCEEDED);
    }

    pub unsafe fn event(&mut self, endpoint_uri_pair_: &endpoint_uri_pair_t, values_: &[u64], values_count_: u64, type_: u64) {
        if (self._monitor_events & type_) {
            self.monitor_event(type_, values_, values_count_, endpoint_uri_pair_);
        }
    }

    pub unsafe fn monitor_event(&mut self, event_: u64, values_: &[u64], values_count_: u64, endpoint_uri_pair_: endpoint_uri_pair_t) {
        // this is a private method which is only called from
        // contexts where the _monitor_sync mutex has been locked before

        if (self._monitor_socket) {
            // zmq_msg_t msg;
            let mut msg = zmq_msg_t::new();

            match (self.own.options.monitor_event_version) {
                1 => {
                    //  The API should not allow to activate unsupported events
                    // zmq_assert (event_ <= std::numeric_limits<uint16_t>::max ());
                    //  v1 only allows one value
                    // zmq_assert (values_count_ == 1);
                    // zmq_assert (values_[0]
                    //             <= std::numeric_limits<uint32_t>::max ());

                    //  Send event and value in first frame
                    let event = (event_) as u16;
                    let value = (values_[0]) as u32;
                    zmq_msg_init_size(&msg, 2 + 4);
                    let mut data = (zmq_msg_data(&msg));
                    //  Avoid dereferencing uint32_t on unaligned address
                    libc::memcpy(data + 0, &event as *const c_void, 2);
                    libc::memcpy(data + 2, &value as *const c_void, 4);
                    zmq_msg_send(&msg, self._monitor_socket, ZMQ_SNDMORE);

                    let endpoint_uri = endpoint_uri_pair_.identifier();

                    //  Send address in second frame
                    zmq_msg_init_size(&msg, endpoint_uri.size());
                    libc::memcpy(zmq_msg_data(&msg), endpoint_uri.as_ptr() as *const c_void,
                                 endpoint_uri.size());
                    zmq_msg_send(&msg, self._monitor_socket, 0);
                }
                2 => {
                    //  Send event in first frame (64bit unsigned)
                    zmq_msg_init_size(&msg, size_of_val(&event_));
                    libc::memcpy(zmq_msg_data(&msg), &event_ as *const c_void, 2);
                    zmq_msg_send(&msg, self._monitor_socket, ZMQ_SNDMORE);

                    //  Send number of values that will follow in second frame
                    zmq_msg_init_size(&msg, size_of_val(&values_count_));
                    libc::memcpy(zmq_msg_data(&msg), &values_count_ as *const c_void,
                                 size_of_val(&values_count_));
                    zmq_msg_send(&msg, self._monitor_socket, ZMQ_SNDMORE);

                    //  Send values in third-Nth frames (64bit unsigned)
                    // for (uint64_t i = 0; i < values_count_; ++i)
                    for i in 0..values_count_ {
                        zmq_msg_init_size(&msg, size_of_val(&values_[i]));
                        libc::memcpy(zmq_msg_data(&msg), &values_[i],
                                     size_of_val(&values_[i]));
                        zmq_msg_send(&msg, self._monitor_socket, ZMQ_SNDMORE);
                    }

                    //  Send local endpoint URI in second-to-last frame (string)
                    zmq_msg_init_size(&msg, endpoint_uri_pair_.local.size());
                    libc::memcpy(zmq_msg_data(&msg), endpoint_uri_pair_.local.c_str(),
                                 endpoint_uri_pair_.local.size());
                    zmq_msg_send(&msg, self._monitor_socket, ZMQ_SNDMORE);

                    //  Send remote endpoint URI in last frame (string)
                    zmq_msg_init_size(&msg, endpoint_uri_pair_.remote.size());
                    libc::memcpy(zmq_msg_data(&msg), endpoint_uri_pair_.remote.c_str(),
                                 endpoint_uri_pair_.remote.size());
                    zmq_msg_send(&msg, self._monitor_socket, 0);
                }
                _ => {}
            }
        }
    }

    pub unsafe fn stop_monitor(&mut self, send_monitor_stopped_event_: bool) {
        // this is a private method which is only called from
        // contexts where the _monitor_sync mutex has been locked before

        if (self._monitor_socket) {
            if ((self._monitor_events & ZMQ_EVENT_MONITOR_STOPPED)
                && send_monitor_stopped_event_) {
                // uint64_t values[1] = {0};
                let mut values: [u64; 1] = [0];
                self.monitor_event (ZMQ_EVENT_MONITOR_STOPPED, &values, 1,
                               endpoint_uri_pair_t::new());
            }
            zmq_close (self._monitor_socket);
            self._monitor_socket = null_mut();
            self._monitor_events = 0;
        }
    }

    pub unsafe fn is_disconnected(&mut self) -> bool {
        self._disconnected
    }
}

impl routing_socket_base_t {
    pub unsafe fn new(parent_: &mut ctx_t, tid_: u32, sid_: i32) -> Self{
        Self {
            base: socket_base_t::new(parent_, tid_, sid_, false),
            _out_pipes: HashMap::new(),
            _connect_routing_id: String::new(),
        }
    }

    pub unsafe fn xsetsockopt(&mut self, option_: i32, optval: *const c_void, optvallen_: usize) -> i32 {
        if (option_ == ZMQ_CONNECT_ROUTING_ID) {
            if (optvallen_ > 255) {
                // errno = EINVAL;
                return -1;
            }
            self._connect_routing_id = String::from_raw_parts(optval as *mut u8, optvallen_, optvallen_);
            return 0;
        }

        return self.base.xsetsockopt(option_, optval, optvallen_);
    }

    pub unsafe fn xwrite_activated(&mut self, pipe_: &mut pipe_t) {
        for it in self._out_pipes.iter_mut() {
            if it.1.pipe == *pipe_ {
                it.1.active = true;
                break;
            }
        }
    }

    pub unsafe fn extract_connect_routing_id(&mut self) -> String {
        // std::string res = ZMQ_MOVE (_connect_routing_id);
        let res = self._connect_routing_id.clone();
        self._connect_routing_id.clear ();
        return res;
    }

    pub unsafe fn connect_routing_id_is_set(&mut self) -> bool {
        return !self._connect_routing_id.empty();
    }

    pub unsafe fn add_out_pipe(&mut self, routing_id_: blob_t, pipe_: &mut pipe_t) {
        //  Add the record into output pipes lookup table
        // const out_pipe_t outpipe = {pipe_, true};
        let outpipe = out_pipe_t {
            pipe: pipe_.clone(),
            active: true,
        };
        let ok =
          self._out_pipes.insert ((routing_id_.clone()), outpipe);
    }

    pub unsafe fn has_out_pipe(&mut self, routing_id_: blob_t) -> bool {
        return self._out_pipes.contains_key(&routing_id_);
    }

    pub unsafe fn lookup_out_pipe(&mut self, routing_id_: blob_t) -> *mut out_pipe_t {
        // out_pipes_t::iterator it = _out_pipes.find (routing_id_);
        // if (it == _out_pipes.end ())
        //     return NULL;
        // return &it.1;
        return self._out_pipes.get_mut(&routing_id_).unwrap();
    }

    pub unsafe fn erase_out_pipe(&mut self, pipe_: &mut pipe_t) {
        let erased = self._out_pipes.erase(pipe_.get_routing_id());
    }

    pub unsafe fn try_erase_out_pipe(&mut self, routing_id_: &blob_t) -> out_pipe_t {
        // out_pipes_t::iterator it = _out_pipes.find (routing_id_);
        // if (it == _out_pipes.end ())
        //     return out_pipe_t ();
        // out_pipe_t outpipe = it.1;
        // _out_pipes.erase (it);
        // return outpipe;
        return self._out_pipes.remove(routing_id_).unwrap();
    }
}

pub struct out_pipe_t<'a> {
    pub pipe: pipe_t<'a>,
    pub active: bool,
}

pub struct routing_socket_base_t<'a> {
    pub base: socket_base_t<'a>,
    pub _out_pipes: HashMap<blob_t, out_pipe_t<'a>>,
    pub _connect_routing_id: String,
}
