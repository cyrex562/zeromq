use std::collections::HashMap;
use std::ffi::c_void;
use std::ptr::null_mut;
use libc::{clock_t, EINTR, EINVAL};
use crate::add;
use crate::address::address_t;
use crate::array::{array_item_t, array_t};
use crate::blob::blob_t;
use crate::ctx::ctx_t;
use crate::defines::{handle_t, ZMQ_DEALER, ZMQ_DGRAM, ZMQ_DISH, ZMQ_EVENTS, ZMQ_FD, ZMQ_LAST_ENDPOINT, ZMQ_POLLIN, ZMQ_POLLOUT, ZMQ_PUB, ZMQ_RADIO, ZMQ_RCVMORE, ZMQ_REQ, ZMQ_SUB, ZMQ_THREAD_SAFE};
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
use crate::udp_address::udp_address_t;
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

    pub fn connect(&mut self, endpoint_uri_: &str) -> i32 {
        self.connect_internal(endpoint_uri_)
    }

    pub unsafe fn connect_internal(&mut self, endpoint_uri_: &str) -> i32
    {
         if ((self._ctx_terminated)) {
            // errno = ETERM;
            return -1;
        }

        //  Process pending commands, if any.
        let mut rc = self.process_commands (0, false);
        if ( (rc != 0)) {
            return -1;
        }

        //  Parse endpoint_uri_ string.
        // std::string protocol;
        let mut protocol = String::new();
        let mut address = String::new();
        // std::string address;
        if (self.parse_uri (endpoint_uri_, &mut protocol, &mut address)
            || self.check_protocol (&protocol)) {
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
        let is_single_connect =
          (self.own.options.type_ == ZMQ_DEALER || self.own.options.type_ == ZMQ_SUB
           || self.own.options.type_ == ZMQ_PUB || self.own.options.type_ == ZMQ_REQ);
        if ((is_single_connect)) {
            if (0 != self._endpoints.count (endpoint_uri_)) {
                // There is no valid use for multiple connects for SUB-PUB nor
                // DEALER-ROUTER nor REQ-REP. Multiple connects produces
                // nonsensical results.
                return 0;
            }
        }

        //  Choose the I/O thread to run the session in.
        let mut io_thread = self.choose_io_thread (self.own.options.affinity);
        if (!io_thread) {
            // errno = EMTHREAD;
            return -1;
        }

        // address_t *paddr = new (std::nothrow) address_t (protocol, address, this->get_ctx ());
        // alloc_assert (paddr);
        let mut paddr = address_t::new(protocol, address, self.get_ctx());

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
            paddr->resolved.tcp_addr = NULL;
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
            rc = paddr.resolved.udp_addr.resolve (address, false,
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
        session_base_t *session =
          session_base_t::create (io_thread, true, this, options, paddr);
        errno_assert (session);

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
        const bool subscribe_to_all = protocol == protocol_name::udp;
    // #endif
        pipe_t *newpipe = NULL;

        if (options.immediate != 1 || subscribe_to_all) {
            //  Create a bi-directional pipe.
            object_t *parents[2] = {this, session};
            pipe_t *new_pipes[2] = {NULL, NULL};

            const bool conflate = get_effective_conflate_option (options);

            int hwms[2] = {conflate ? -1 : options.sndhwm,
                           conflate ? -1 : options.rcvhwm};
            bool conflates[2] = {conflate, conflate};
            rc = pipepair (parents, new_pipes, hwms, conflates);
            errno_assert (rc == 0);

            //  Attach local end of the pipe to the socket object.
            attach_pipe (new_pipes[0], subscribe_to_all, true);
            newpipe = new_pipes[0];

            //  Attach remote end of the pipe to the session object later on.
            session->attach_pipe (new_pipes[1]);
        }

        //  Save last endpoint URI
        paddr->to_string (_last_endpoint);

        add_endpoint (make_unconnected_connect_endpoint_pair (endpoint_uri_),
                      static_cast<own_t *> (session), newpipe);
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
