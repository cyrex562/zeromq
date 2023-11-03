use std::collections::{HashMap, HashSet, VecDeque};
use std::ffi::c_void;
use std::mem::size_of_val;
use std::ptr::null_mut;

use libc::{clock_t, EAGAIN, EINTR};

use routing_socket_base::ZmqRoutingSocketBase;

use crate::address::ZmqAddress;
use crate::command::ZmqCommand;
use crate::ctx::ZmqContext;
use crate::defines::{INBOUND_POLL_RATE, MAX_COMMAND_DELAY, ZMQ_MSG_MORE, ZMQ_DEALER, ZMQ_DGRAM, ZMQ_DISH, ZMQ_DONTWAIT, ZMQ_EVENT_ACCEPT_FAILED, ZMQ_EVENT_ACCEPTED, ZMQ_EVENT_BIND_FAILED, ZMQ_EVENT_CLOSE_FAILED, ZMQ_EVENT_CLOSED, ZMQ_EVENT_CONNECT_DELAYED, ZMQ_EVENT_CONNECT_RETRIED, ZMQ_EVENT_CONNECTED, ZMQ_EVENT_DISCONNECTED, ZMQ_EVENT_HANDSHAKE_FAILED_AUTH, ZMQ_EVENT_HANDSHAKE_FAILED_NO_DETAIL, ZMQ_EVENT_HANDSHAKE_FAILED_PROTOCOL, ZMQ_EVENT_HANDSHAKE_SUCCEEDED, ZMQ_EVENT_LISTENING, ZMQ_EVENT_MONITOR_STOPPED, ZMQ_EVENT_PIPES_STATS, ZMQ_EVENTS, ZMQ_FD, ZMQ_LAST_ENDPOINT, ZMQ_LINGER, ZMQ_POLLIN, ZMQ_POLLOUT, ZMQ_PUB, ZMQ_RADIO, ZMQ_RCVHWM, ZMQ_RCVMORE, ZMQ_RECONNECT_STOP_AFTER_DISCONNECT, ZMQ_REQ, ZMQ_SNDHWM, ZMQ_SNDMORE, ZMQ_SUB, ZMQ_THREAD_SAFE, ZmqFd, ZmqHandle, ZmqSubscriptions};
use crate::defines::array::ArrayItem;
use crate::dist::ZmqDist;
use crate::endpoint::{make_unconnected_connect_endpoint_pair, ZmqEndpointUriPair};
use crate::err::ZmqError;
use crate::err::ZmqError::{InvalidContext, SocketError};
use crate::defines::fair_queue::ZmqFairQueue;
use crate::poll::i_poll_events::IPollEvents;
use crate::defines::load_balancer::ZmqLoadBalancer;
use crate::defines::mtrie::ZmqMtrie;
use crate::defines::mutex::ZmqMutex;
use crate::io::mailbox::ZmqMailbox;
use crate::io::signaler::ZmqSignaler;
use crate::metadata::ZmqMetadata;
use crate::msg::ZmqMsg;
use crate::object::obj_process_command;
use crate::options::{do_getsockopt, get_effective_conflate_option, ZmqOptions};
use crate::own::{own_process_term, ZmqOwn};
use crate::pipe::{IPipeEvents, pipepair, ZmqPipe};
use crate::pipe::out_pipe::ZmqOutPipes;
use crate::pipe::pipes::ZmqPipes;
use crate::poll::poller_base::ZmqPollerBase;
use crate::session::ZmqSession;
use crate::socket::radio::UdpPipes;
use crate::utils::get_errno;
use crate::zmq_ops::{zmq_bind, zmq_close, zmq_msg_data, zmq_msg_init_size, zmq_msg_send, zmq_setsockopt, zmq_socket};
mod channel;
mod client;
mod dealer;
mod dgram;
mod dish;
mod gather;
mod pair;
mod peer;
mod zmq_pub;
mod pull;
mod push;
mod radio;
mod rep;
mod req;
mod router;
mod routing_socket_base;
mod scatter;
mod sub;
mod xpub;
mod xsub;

pub type ZmqEndpointPipe<'a> = (&'a mut ZmqSession<'a>, &'a mut ZmqPipe<'a>);
pub type ZmqEndpoints<'a> = HashMap<String, ZmqEndpointPipe<'a>>;

pub type ZmqMap<'a> = HashMap<String, &'a mut ZmqPipe<'a>>;

pub enum ZmqSocketType {
    Client,
    Dealer,
    Dgram,
    Dish,
    Gather,
    Pair,
    Peer,
    Pub,
    Pull,
    Push,
    Radio,
    Rep,
    Req,
    Router,
    Scatter,
    Server,
    Sub,
    Xpub,
    Xsub,
}

#[derive(Default)]
pub struct ZmqInprocs<'a> {
    pub _inprocs: ZmqMap<'a>,
}

impl ZmqInprocs {
    pub fn emplace(&mut self, endpoint_uri_: &str, pipe_: &mut ZmqPipe) {
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
pub struct ZmqSocket<'a> {
    // pub dist: ZmqDist,
    // pub subscriptions: ZmqMtrie,
    // pub _has_message: bool,
    // pub _message: ZmqMsg,
    // pub _subscriptions: ZmqRadixTree,
    // pub router: ZmqRouter<'a>,
    // pub server: ZmqServer<'a>,
    pub eligible: usize,
    pub more: bool,
    pub active: usize,
    pub anonymous_pipes: HashSet<&'a mut ZmqPipe<'a>>,
    pub array_item: ArrayItem<1>,
    pub clock: clock_t,
    pub ctx_terminated: bool,
    pub current_in: &'a mut ZmqPipe<'a>,
    pub current_out: &'a mut ZmqPipe<'a>,
    pub destroyed: bool,
    pub disconnected: bool,
    pub dist: ZmqDist<'a>,
    pub endpoints: ZmqEndpoints<'a>,
    pub fq: ZmqFairQueue<'a>,
    pub handle: Option<&'a mut ZmqHandle>,
    pub handover: bool,
    pub has_message: bool,
    pub inprocs: ZmqInprocs<'a>,
    pub last_endpoint: String,
    pub last_pipe: Option<&'a mut ZmqPipe<'a>>,
    pub last_tsc: u64,
    pub lb: ZmqLoadBalancer,
    pub lossy: bool,
    pub mailbox: ZmqMailbox<'a>,
    pub mandatory: bool,
    pub manual: bool,
    pub manual_subscriptions: ZmqMtrie,
    pub matching: usize,
    pub message: ZmqMsg,
    pub monitor_events: i64,
    pub monitor_socket: Option<ZmqSocket<'a>>,
    pub monitor_sync: ZmqMutex,
    pub more_in: bool,
    pub more_out: bool,
    pub more_recv: bool,
    pub more_send: bool,
    pub next_integral_routing_id: u32,
    pub next_routing_id: u32,
    pub only_first_subscribe: bool,
    pub out_pipes: ZmqOutPipes,
    pub own: ZmqOwn<'a>,
    // pub object: ZmqObject<'a>,
    // pub terminating: bool,
    // pub sent_seqnum: ZmqAtomicCounter,
    // pub processed_seqnum: u64,
    // pub owner: Option<&'a mut ZmqOwn<'a>>,
    // pub owned: HashSet<&'a mut ZmqOwn<'a>>,
    // pub term_acks: i32,
    pub pending_data: VecDeque<Vec<u8>>,
    pub pending_flags: VecDeque<u8>,
    pub pending_metadata: VecDeque<&'a mut ZmqMetadata>,
    pub pending_pipes: VecDeque<&'a mut ZmqPipe<'a>>,
    pub pipe: Option<&'a mut ZmqPipe<'a>>,
    pub pipe_events: dyn IPipeEvents,
    pub pipes: ZmqPipes,
    pub poll_events: dyn IPollEvents,
    pub poller: Option<&'a mut ZmqPollerBase>,
    pub prefetched: bool,
    pub prefetched_id: ZmqMsg,
    pub prefetched_msg: ZmqMsg,
    pub probe_router: bool,
    pub process_subscribe: bool,
    pub raw_socket: bool,
    pub rcvmore: bool,
    pub reaper_signaler: Option<ZmqSignaler>,
    pub request_begins: bool,
    pub request_id: u32,
    pub routing_socket_base: ZmqRoutingSocketBase<'a>,
    pub send_last_pipe: bool,
    pub sending_reply: bool,
    pub socket_type: ZmqSocketType,
    pub strict: bool,
    pub subscriptions: ZmqSubscriptions,
    pub sync: ZmqMutex,
    pub tag: u32,
    pub terminate_current_in: bool,
    pub thread_safe: bool,
    pub ticks: i32,
    pub udp_pipes: UdpPipes<'a>,
    pub verbose_unsubs: bool,
    pub welcome_msg: ZmqMsg,
}

impl ZmqSocket {
    pub fn check_tag(&self) -> bool {
        return self.tag == 0xbaddecaf;
    }

    pub fn is_thread_safe(&self) -> bool {
        self.thread_safe
    }

    pub fn create(type_: i32, parent_: *mut ZmqContext, tid_: u32, sid_: i32) -> *mut Self {
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

    pub fn new(parent_: &mut ZmqContext, tid_: u32, sid_: i32, thread_safe_: bool) -> Self {
        let mut out = Self {
            eligible: 0,
            more: false,
            active: 0,
            // own: ZmqOwn::new(parent_, tid_),
            pending_data: Default::default(),
            pending_flags: Default::default(),
            pending_metadata: Default::default(),
            pending_pipes: Default::default(),
            array_item: ArrayItem::new(),
            poll_events: null_mut(),
            pipe_events: null_mut(),
            mailbox: ZmqMailbox::default(),
            mandatory: false,
            manual: false,
            manual_subscriptions: GenericMtrie::default(),
            matching: 0,
            pipes: [ZmqPipe::default(); 2],
            poller: None,
            prefetched: false,
            prefetched_id: Default::default(),
            prefetched_msg: Default::default(),
            probe_router: false,
            process_subscribe: false,
            handle: None,
            handover: false,
            last_tsc: 0,
            lb: ZmqLoadBalancer::default(),
            ticks: 0,
            udp_pipes: vec![],
            verbose_unsubs: false,
            rcvmore: false,
            clock: 0,
            monitor_socket: None,
            monitor_events: 0,
            last_endpoint: "".to_string(),
            thread_safe: false,
            reaper_signaler: None,
            request_begins: false,
            request_id: 0,
            routing_socket_base: ZmqRoutingSocketBase::default(),
            send_last_pipe: false,
            sending_reply: false,
            socket_type: ZmqSocketType::Client,
            strict: false,
            monitor_sync: ZmqMutex::new(),
            more_in: false,
            more_out: false,
            more_recv: false,
            more_send: false,
            next_integral_routing_id: 0,
            next_routing_id: 0,
            only_first_subscribe: false,
            disconnected: false,
            sync: ZmqMutex::new(),
            endpoints: Default::default(),
            inprocs: ZmqInprocs::default(),
            tag: 0,
            ctx_terminated: false,
            current_in: &mut Default::default(),
            current_out: &mut Default::default(),
            destroyed: false,
            anonymous_pipes: Default::default(),
            dist: ZmqDist::default(),
            fq: ZmqFairQueue::default(),
            has_message: false,
            last_pipe: None,
            lossy: false,
            message: Default::default(),
            out_pipes: Default::default(),
            // object: ZmqObject::default(),
            // terminating: false,
            // sent_seqnum: ZmqAtomicCounter::default(),
            // processed_seqnum: 0,
            // owner: None,
            // owned: Default::default(),
            pipe: None,
            raw_socket: false,
            subscriptions: ZmqSubscriptions::default(),
            terminate_current_in: false,
            welcome_msg: Default::default(),
            // term_acks: 0,
            own: Default::default(),
        };
        // TODO
        // options.socket_id = sid_;
        //     options.ipv6 = (parent_->get (ZMQ_IPV6) != 0);
        //     options.linger.store (parent_->get (ZMQ_BLOCKY) ? -1 : 0);
        //     options.zero_copy = parent_->get (ZMQ_ZERO_COPY_RECV) != 0;

        // if out.thread_safe {
        //     out.mailbox = ZmqMailboxSafe::new();
        // } else {
        //     out.mailbox = ZmqMailbox::new();
        // }
        out.mailbox = ZmqMailbox::new();
        out
    }

    pub fn get_peer_state(&mut self, routing_id: *const c_void, routing_id_size_: usize) -> i32 {
        unimplemented!()
    }

    pub fn get_mailbox(&mut self) -> &mut ZmqMailbox {
        &mut self.mailbox
    }

    pub fn parse_uri(&mut self, uri_: &str, protocol_: &mut String, path_: &mut String) -> Result<(), ZmqError> {
        let mut uri = String::from(uri_);
        let pos = uri.find("://");
        if pos.is_none() {
            return Err(ZmqError::ParseError("Invalid URI"));
        }

        *protocol_ = uri_[0..pos.unwrap()].to_string();
        *path_ = uri_[pos.unwrap() + 3..].to_string();

        if protocol_.is_empty() || path_.is_empty() {
            return Err(ZmqError::ParseError("Invalid URI"));
        }

        Ok(())
    }

    pub fn check_protocol(&mut self, protocol_: &str) -> bool {
        let protocols = ["tcp", "udp"];
        if protocols.contains(&protocol_) {
            return true;
        }
        return false;
    }

    pub unsafe fn attach_pipe(&mut self, options: &mut ZmqOptions, mut pipe: &mut ZmqPipe, subscribe_to_all_: bool, locally_initiated_: bool) {
        pipe.set_event_sink(self);
        self.pipes.push_back(pipe);
        // self.xattach_pipe(&pipe, subscribe_to_all_, locally_initiated_);
        match self.socket_type {
            ZmqSocketType::Client => {
                client_xattach_pipe(self, &mut pipe, subscribe_to_all_, locally_initiated_);
            }
            ZmqSocketType::Dealer => {
                dealer_xattach_pipe(self, &mut pipe, subscribe_to_all_, locally_initiated_);
            }
            ZmqSocketType::Dgram => {
                dgram_xattach_pipe(self, &mut pipe, subscribe_to_all_, locally_initiated_);
            }
            ZmqSocketType::Dish => {
                dish_xattach_pipe(self, &mut pipe, subscribe_to_all_, locally_initiated_);
            }
            ZmqSocketType::Gather => {
                gather_xattach_pipe(self, &mut pipe, subscribe_to_all_, locally_initiated_);
            }
            ZmqSocketType::Pair => {
                pair_xattach_pipe(self, &mut pipe, subscribe_to_all_, locally_initiated_);
            }
            ZmqSocketType::Peer => {
                peer_xattach_pipe(self, &mut pipe, subscribe_to_all_, locally_initiated_);
            }
            ZmqSocketType::Pub => {
                pub_xattach_pipe(self, &mut pipe, subscribe_to_all_, locally_initiated_);
            }
            ZmqSocketType::Pull => {
                pull_xattach_pipe(self, &mut pipe, subscribe_to_all_, locally_initiated_);
            }
            ZmqSocketType::Push => {
                push_xattach_pipe(self, &mut pipe, subscribe_to_all_, locally_initiated_);
            }
            ZmqSocketType::Radio => {
                radio_xattach_pipe(self, &mut pipe, subscribe_to_all_, locally_initiated_)
            }
            ZmqSocketType::Rep => {
                rep_xattach_pipe(self, &mut pipe, subscribe_to_all_, locally_initiated_);
            }
            ZmqSocketType::Req => {
                req_xattach_pipe(self, &mut pipe, subscribe_to_all_, locally_initiated_);
            }
            ZmqSocketType::Router => {
                router_xattach_pipe(self, &mut pipe, subscribe_to_all_, locally_initiated_);
            }
            ZmqSocketType::Scatter => {
                scatter_xattach_pipe(self, &mut pipe, subscribe_to_all_, locally_initiated_);
            }
            ZmqSocketType::Server => {
                server_xattach_pipe(self, &mut pipe, subscribe_to_all_, locally_initiated_);
            }
            ZmqSocketType::Sub => {
                sub_xattach_pipe(self, &mut pipe, subscribe_to_all_, locally_initiated_);
            }
            ZmqSocketType::Xpub => {
                xpub_xattach_pipe(self, options, &mut pipe, subscribe_to_all_, locally_initiated_);
            }
            ZmqSocketType::Xsub => {
                xsub_xattach_pipe(self, &mut pipe, subscribe_to_all_, locally_initiated_);
            }
        }

        if self.is_terminating() {
            self.register_term_acks(1);
            pipe.terminate(false);
        }
    }

    pub unsafe fn setsockopt(&mut self, options: &mut ZmqOptions, option_: i32, optval_: &[u8], optvallen_: usize) -> Result<(), ZmqError> {
        if self.ctx_terminated {
            return Err(InvalidContext("Context was terminated"));
        }
        // self.xsetsockopt(option_, optval_, optvallen_)?;

        match self.socket_type {
            ZmqSocketType::Client => {
                client_xsetsockopt(self, option_, optval_, optvallen_);
            }
            ZmqSocketType::Dealer => {
                dealer_xsetsockopt(self, option_, optval_, optvallen_);
            }
            ZmqSocketType::Dgram => {
                dgram_xsetsockopt(self, option_, optval_, optvallen_);
            }

            ZmqSocketType::Dish => {
                dish_xsetsockopt(self, option_, optval_, optvallen_);
            }

            ZmqSocketType::Gather => {
                gather_xsetsockopt(self, option_, optval_, optvallen_);
            }

            ZmqSocketType::Pair => {
                pair_xsetsockopt(self, option_, optval_, optvallen_);
            }

            ZmqSocketType::Peer => {
                peer_xsetsockopt(self, option_, optval_, optvallen_);
            }
            ZmqSocketType::Pub => {
                pub_xsetsockopt(self, option_, optval_, optvallen_);
            }
            ZmqSocketType::Pull => {
                pull_xsetsockopt(self, option_, optval_, optvallen_);
            }
            ZmqSocketType::Push => {
                push_xsetsockopt(self, option_, optval_, optvallen_);
            }

            ZmqSocketType::Radio => {
                radio_xsetsockopt(self, option_, optval_, optvallen_);
            }

            ZmqSocketType::Rep => {
                rep_xsetsockopt(self, option_, optval_, optvallen_);
            }

            ZmqSocketType::Req => {
                req_xsetsockopt(self, option_, optval_, optvallen_);
            }

            ZmqSocketType::Router => {
                router_xsetsockopt(self, options, option_, optval_, optvallen_);
            }

            ZmqSocketType::Scatter => {
                scatter_xsetsockopt(self, option_, optval_, optvallen_);
            }

            ZmqSocketType::Server => {
                server_xsetsockopt(self, option_, optval_, optvallen_);
            }

            ZmqSocketType::Sub => {
                sub_xsetsockopt(self, option_, optval_, optvallen_);
            }
            ZmqSocketType::Xpub => {
                xpub_xsetsockopt(self, option_, optval_, optvallen_);
            }
            ZmqSocketType::Xsub => {
                xsub_xsetsockopt(self, option_, optval_, optvallen_);
            }
        }

        options.setsockopt(option_, optval_, optvallen_)?;
        self.update_pipe_options(options, option_)?;
        Ok(())
    }

    pub unsafe fn getsockopt(&mut self, options: &ZmqOptions, option_: u32) -> Result<[u8], ZmqError> {
        if self.ctx_terminated {
            // errno = ETERM;
            return Err(InvalidContext("Context was terminated"));
        }

        //  First, check whether specific socket type overloads the option.
        // match self.xgetsockopt(option_, optval_, optvallen_) {
        //     Ok(x) => return Ok(x),
        //     Err(e) => {}
        // };

        let mut result: Result<[u8], ZmqError>;
        match self.socket_type {
            ZmqSocketType::Client => {
                result = client_xgetsockopt(self, option_);
            }
            ZmqSocketType::Dealer => {
                result = dealer_xgetsockopt(self, option_);
            }
            ZmqSocketType::Dgram => {
                result = dgram_xgetsockopt(self, option_);
            }
            ZmqSocketType::Dish => {
                result = dish_xgetsockopt(self, option_);
            }
            ZmqSocketType::Gather => {
                result = gather_xgetsockopt(self, option_);
            }
            ZmqSocketType::Pair => {
                result = pair_xgetsockopt(self, option_);
            }
            ZmqSocketType::Peer => {
                result = peer_xgetsockopt(self, option_);
            }
            ZmqSocketType::Pub => {
                result = pub_xgetsockopt(self, option_);
            }
            ZmqSocketType::Pull => {
                result = pull_xgetsockopt(self, option_);
            }
            ZmqSocketType::Push => {
                result = push_xgetsockopt(self, option_);
            }
            ZmqSocketType::Radio => {
                result = radio_xgetsockopt(self, option_);
            }
            ZmqSocketType::Rep => {
                result = rep_xgetsockopt(self, option_);
            }
            ZmqSocketType::Req => {
                result = req_xgetsockopt(self, option_);
            }
            ZmqSocketType::Router => {
                result = router_xgetsockopt(self, option_);
            }
            ZmqSocketType::Scatter => {
                result = scatter_xgetsockopt(self, option_);
            }
            ZmqSocketType::Server => {
                result = server_xgetsockopt(self, option_);
            }
            ZmqSocketType::Sub => {
                result = sub_xgetsockopt(self, option_);
            }
            ZmqSocketType::Xpub => {
                result = xpub_xgetsockopt(self, option_);
            }
            ZmqSocketType::Xsub => {
                result = xsub_xgetsockopt(self, option_);
            }
        }

        // TODO check if result is an error. if its, and its a not implemented error, then continue below. If its another error type then return it. If it is OK then return the results.

        if option_ == ZMQ_RCVMORE {
            return do_getsockopt(if self.rcvmore { 1 } else { 0 });
        }

        if option_ == ZMQ_FD {
            if self.thread_safe {
                // thread safe socket doesn't provide file descriptor
                // errno = EINVAL;
                return Err(SocketError("Socket doesn't provide file descriptor"));
            }

            return do_getsockopt((self.mailbox).get_fd());
        }

        if option_ == ZMQ_EVENTS {
            self.process_commands(options, 0, false)?;
            return do_getsockopt((if self.has_out() { ZMQ_POLLOUT } else { 0 }) | (if self.has_in() { ZMQ_POLLIN } else { 0 }));
        }

        if option_ == ZMQ_LAST_ENDPOINT {
            return do_getsockopt(&self.last_endpoint);
        }

        if option_ == ZMQ_THREAD_SAFE {
            return do_getsockopt(if self.thread_safe { 1 } else { 0 });
        }

        return options.getsockopt(option_);
    }

    pub fn join(&mut self, group_: &str) -> i32 {

        // self.xjoin(group_)
        return match self.socket_type {
            ZmqSocketType::Client => {
                client_xjoin(self, group_)
            }
            ZmqSocketType::Dealer => {
                dealer_xjoin(self, group_)
            }
            ZmqSocketType::Dgram => {
                dgram_xjoin(self, group_)
            }
            ZmqSocketType::Dish => {
                dish_xjoin(self, group_)
            }
            ZmqSocketType::Gather => {
                gather_xjoin(self, group_)
            }
            ZmqSocketType::Pair => {
                pair_xjoin(self, group_)
            }
            ZmqSocketType::Peer => {
                peer_xjoin(self, group_)
            }
            ZmqSocketType::Pub => {
                pub_xjoin(self, group_)
            }
            ZmqSocketType::Pull => {
                pull_xjoin(self, group_)
            }
            ZmqSocketType::Push => {
                push_xjoin(self, group_)
            }
            ZmqSocketType::Radio => {
                radio_xjoin(self, group_)
            }
            ZmqSocketType::Rep => {
                rep_xjoin(self, group_)
            }
            ZmqSocketType::Req => {
                req_xjoin(self, group_)
            }
            ZmqSocketType::Router => {
                router_xjoin(self, group_)
            }
            ZmqSocketType::Scatter => {
                scatter_xjoin(self, group_)
            }
            ZmqSocketType::Server => {
                server_xjoin(self, group_)
            }
            ZmqSocketType::Sub => {
                sub_xjoin(self, group_)
            }
            ZmqSocketType::Xpub => {
                xpub_xjoin(self, group_)
            }
            ZmqSocketType::Xsub => {
                xsub_xjoin(self, group_)
            }
        };
    }

    pub fn leave(&mut self, group_: &str) -> i32 {
        self.xleave(group_)
    }

    pub fn addsignaler(&mut self, signaler: &mut ZmqSignaler) {
        self.mailbox.add_signaler(signaler);
    }

    pub fn bind(&mut self, options: &ZmqOptions, endpoint_uri_: &str) -> Result<(), ZmqError> {
        if self.ctx_terminated {
            return Err(SocketError("Context was terminated"));
        }

        self.process_commands(options, 0, false)?;

        let mut protocol = String::new();
        let mut address = String::new();
        if self.parse_uri(endpoint_uri_, &mut protocol, &mut address).is_err() || self.check_protocol(&protocol) {
            return Err(ZmqError::ParseError("Invalid URI"));
        }

        if protocol == "udp" {
            if !options.type_ == ZMQ_DGRAM || !options.type_ == ZMQ_DISH {
                return Err(ZmqError::SocketError("Protocol not supported by socket type"));
            }

            let _io_thread = self.choose_io_thread(options.affinity)?;

            let mut paddr = ZmqAddress::default();
            paddr.address = address;
            paddr.protocol = protocol;
            paddr.parent == self.get_ctx();
        }

        Ok(())
    }

    pub unsafe fn connect(&mut self, options: &mut ZmqOptions, endpoint_uri: &str) -> Result<(), ZmqError> {
        self.connect_internal(options, endpoint_uri)
    }

    pub unsafe fn connect_internal(&mut self, options: &mut ZmqOptions, endpoint_uri: &str) -> Result<(), ZmqError> {
        let mut rc = 0i32;

        if self.ctx_terminated {
            return Err(SocketError("Context was terminated"));
        }

        //  Process pending commands, if any.
        self.process_commands(options, 0, false)?;

        //  Parse endpoint_uri_ string.
        let mut protocol = String::new();
        let mut address = String::new();
        // std::string address;
        if self.parse_uri(endpoint_uri, &mut protocol, &mut address).is_err() || self.check_protocol(&protocol) {
            return Err(ZmqError::ParseError("Invalid URI"));
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
        let is_single_connect = (options.type_ == ZMQ_DEALER || options.type_ == ZMQ_SUB || options.type_ == ZMQ_PUB || options.type_ == ZMQ_REQ);
        if is_single_connect {
            if 0 != self.endpoints.count() {
                // There is no valid use for multiple connects for SUB-PUB nor
                // DEALER-ROUTER nor REQ-REP. Multiple connects produces
                // nonsensical results.
                Ok(())
            }
        }

        //  Choose the I/O thread to run the session in.
        let mut io_thread = self.choose_io_thread(options.affinity);
        if (!io_thread) {
            // errno = EMTHREAD;
            return Err(SocketError("failed to get thread"));
        }

        // address_t *paddr = new (std::nothrow) address_t (protocol, address, this->get_ctx ());
        // alloc_assert (paddr);
        let mut paddr = ZmqAddress::new(&mut protocol, &mut address, self.get_ctx());

        //  Resolve address (if needed by the protocol)
        if protocol == "tcp" {
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
            // rc = -1;
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
            if rc == -1 {
                // errno = EINVAL;
                // LIBZMQ_DELETE (paddr);
                return Err(SocketError("Invalid address"));
            }
            //  Defer resolution until a socket is opened
            paddr.tcp_addr = ZmqTcpAddress::default();
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

        if protocol == "udp" {
            if options.type_ != ZMQ_RADIO {
                // errno = ENOCOMPATPROTO;
                // LIBZMQ_DELETE (paddr);
                return Err(SocketError("Protocol not supported by socket type"));
            }

            paddr.udp_addr = UdpAddress::new(); //new (std::nothrow) udp_address_t ();
            // alloc_assert (paddr->resolved.udp_addr);
            paddr.udp_addr.resolve(&mut address, false,
                                   options.ipv6)?;
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
        let mut session = ZmqSession::create(io_thread, true, self, &options, paddr);
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
        let mut newpipe: &mut ZmqPipe;

        if options.immediate != 1 || subscribe_to_all {
            //  Create a bi-directional pipe.
            // object_t *parents[2] = {this, session};
            let mut parents = (self, session);
            // pipe_t *new_pipes[2] = {NULL, NULL};
            let mut new_pipes: [Option<&mut ZmqPipe>; 2] = [None, None];

            let conflate = get_effective_conflate_option(&options);

            let mut hwms: [i32; 2] = [
                if conflate { -1 } else { options.sndhwm },
                if conflate { -1 } else { options.rcvhwm }
            ];
            let mut conflates: [bool; 2] = [conflate, conflate];
            rc = pipepair(parents, &mut new_pipes, hwms, conflates);
            // errno_assert (rc == 0);

            //  Attach local end of the pipe to the socket object.
            self.attach_pipe(options, new_pipes[0].unwrap(), subscribe_to_all, true);
            newpipe = new_pipes[0].unwrap();

            //  Attach remote end of the pipe to the session object later on.
            session.attach_pipe(new_pipes[1].unwrap());
        }

        //  Save last endpoint URI
        self.last_endpoint = paddr.to_string();

        self.add_endpoint(make_unconnected_connect_endpoint_pair(endpoint_uri),
                          (&mut session), newpipe);
        Ok(())
    }

    pub fn resolve_tcp_addr(&mut self, options: &ZmqOptions, endpoint_uri_pair_: &mut String, tcp_address_: &mut String) -> Result<String, ZmqError> {
        // The resolved last_endpoint is used as a key in the endpoints map.
        // The address passed by the user might not match in the TCP case due to
        // IPv4-in-IPv6 mapping (EG: tcp://[::ffff:127.0.0.1]:9999), so try to
        // resolve before giving up. Given at this stage we don't know whether a
        // socket is connected or bound, try with both.
        if self.endpoints.find(endpoint_uri_pair_) == self.endpoints.end() {
            // tcp_address_t *tcp_addr = new (std::nothrow) tcp_address_t ();
            // alloc_assert (tcp_addr);
            let mut tcp_addr = ZmqTcpAddress::new();
            tcp_addr.resolve(tcp_address_, false, options.ipv6)?;

            tcp_addr.to_string(endpoint_uri_pair_)?;
            if self.endpoints.find(endpoint_uri_pair_) == self.endpoints.end() {
                tcp_addr.resolve(tcp_address_, true, options.ipv6)?;
                tcp_addr.to_string(endpoint_uri_pair_)?;
            }
            // LIBZMQ_DELETE (tcp_addr);
        }
        return Ok(endpoint_uri_pair_.clone());
    }

    pub unsafe fn add_endpoint(&mut self, endpoint_pair: ZmqEndpointUriPair, endpoint: &mut ZmqSession, pipe: &mut ZmqPipe) {
        //  Activate the session. Make it a child of this socket.
        self.launch_child(endpoint);
        self.endpoints.insert(endpoint_pair.identifier().clone(),
                              (endpoint, pipe));

        if (pipe != null_mut()) {
            (pipe).set_endpoint_pair(endpoint_pair);
        }
    }

    pub fn term_endpoint(&mut self, options: &ZmqOptions, endpoint_uri_: &str) -> Result<(), ZmqError> {
        //  Check whether the context hasn't been shut down yet.
        if ((self.ctx_terminated)) {
            // errno = ETERM;
            return Err(SocketError("context was terminated"));
        }

        //  Process pending commands, if any, since there could be pending unprocessed process_own()'s
        //  (from launch_child() for example) we're asked to terminate now.
        let rc = self.process_commands(options, 0, false)?;

        //  Parse endpoint_uri_ string.
        // std::string uri_protocol;
        let mut uri_protocol = String::new();
        // std::string uri_path;
        let mut uri_path = String::new();
        if (self.parse_uri(endpoint_uri_, &mut uri_protocol, &mut uri_path).is_err() || self.check_protocol(&uri_protocol)) {
            return Err(SocketError("failed to parse URI"));
        }

        let mut endpoint_uri_str = (endpoint_uri_).to_string();

        // Disconnect an inproc socket
        // if (uri_protocol == protocol_name::inproc) {
        //     return unregister_endpoint (endpoint_uri_str, this) == 0
        //         ? 0
        //         : _inprocs.erase_pipes (endpoint_uri_str);
        // }

        let resolved_endpoint_uri = if uri_protocol == "tcp" { self.resolve_tcp_addr(options, &mut endpoint_uri_str, uri_path.c_str())? } else { endpoint_uri_str };

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
        for it in self.endpoints.iter_mut() {
            if it.0 == resolved_endpoint_uri {
                it.1.terminate(false);
                self.term_child(it.1)
            }
        }
        self.endpoints.remove(&resolved_endpoint_uri);

        // _endpoints.erase (range.first, range.second);

        if (options.reconnect_stop & ZMQ_RECONNECT_STOP_AFTER_DISCONNECT) {
            self.disconnected = true;
        }

        Ok(())
    }

    pub unsafe fn send(&mut self, options: &ZmqOptions, msg: &mut ZmqMsg, flags_: i32) -> Result<(), ZmqError> {
        //  Check whether the context hasn't been shut down yet.
        if self.ctx_terminated {
            // errno = ETERM;
            return Err(SocketError("context was terminated"));
        }

        //  Check whether message passed to the function is valid.
        if !msg.check() {
            // errno = EFAULT;
            return Err(SocketError("invalid message"));
        }

        //  Process pending commands, if any.
        self.process_commands(options, 0, true)?;

        //  Clear any user-visible flags that are set on the message.
        msg.reset_flags(ZMQ_MSG_MORE);

        //  At this point we impose the flags on the message.
        if flags_ & ZMQ_SNDMORE {
            msg.set_flags(ZMQ_MSG_MORE);
        }

        msg.reset_metadata();

        //  Try to send the message using method in each socket class
        // rc = self.xsend(msg_);
        let mut rc = self.xsend(options, msg);
        if rc == 0 {
            Ok(())
        }
        //  Special case for ZMQ_PUSH: -2 means pipe is dead while a
        //  multi-part send is in progress and can't be recovered, so drop
        //  silently when in blocking mode to keep backward compatibility.
        if rc == -2 {
            if !((flags_ & ZMQ_DONTWAIT != 0) || options.sndtimeo == 0) {
                msg.close()?;
                // errno_assert (rc == 0);
                msg.init2()?;
                // errno_assert (rc == 0);
                Ok(())
            }
        }
        // if ((errno != EAGAIN)) {
        //     return -1;
        // }

        //  In case of non-blocking send we'll simply propagate
        //  the Error - including EAGAIN - up the stack.
        if (flags_ & ZMQ_DONTWAIT != 0) || options.sndtimeo == 0 {
            return Err(SocketError("non-blocking send"));
        }

        //  Compute the time when the timeout should occur.
        //  If the timeout is infinite, don't care.
        let mut timeout = options.sndtimeo;
        let end = if timeout < 0 { 0 } else { (self.clock.now_ms() + timeout) };

        //  Oops, we couldn't send the message. Wait for the next
        //  command, process it and try to send the message again.
        //  If timeout is reached in the meantime, return EAGAIN.
        loop {
            self.process_commands(options, timeout, false)?;
            rc = self.xsend(options, msg);
            if rc == 0 {
                break;
            }
            // if ((errno != EAGAIN)) {
            //     return -1;
            // }
            if timeout > 0 {
                timeout = (end - self.clock.now_ms());
                if timeout <= 0 {
                    // errno = EAGAIN;
                    return Err(SocketError("EAGAIN"));
                }
            }
        }

        Ok(())
    }

    pub fn xsend(&mut self, options: &ZmqOptions, msg: &mut ZmqMsg) -> i32 {
        let mut rc = match self.socket_type {
            ZmqSocketType::Client => client_xsend(self, msg),
            ZmqSocketType::Dealer => dealer_xsend(self, msg),
            ZmqSocketType::Dgram => dgram_xsend(self, msg),
            ZmqSocketType::Dish => dish_xsend(self, msg),
            ZmqSocketType::Gather => gather_xsend(self, msg),
            ZmqSocketType::Pair => pair_xsend(self, msg),
            ZmqSocketType::Peer => peer_xsend(self, msg),
            ZmqSocketType::Pub => pub_xsend(self, msg),
            ZmqSocketType::Pull => pull_xsend(self, msg),
            ZmqSocketType::Push => push_xsend(self, msg),
            ZmqSocketType::Radio => radio_xsend(self, msg),
            ZmqSocketType::Rep => rep_xsend(self, msg),
            ZmqSocketType::Req => req_xsend(self, msg),
            ZmqSocketType::Router => router_xsend(self, msg),
            ZmqSocketType::Scatter => scatter_xsend(self, msg),
            ZmqSocketType::Server => server_xsend(self, msg),
            ZmqSocketType::Sub => sub_xsend(self, msg),
            ZmqSocketType::Xpub => xpub_xsend(self, options, msg),
            ZmqSocketType::Xsub => xsub_xsend(self, msg),
        };
        rc
    }

    pub unsafe fn recv(&mut self, options: &ZmqOptions, msg: &mut ZmqMsg, flags: i32) -> Result<(), ZmqError> {
        //  Check whether the context hasn't been shut down yet.
        if ((self.ctx_terminated)) {
            // errno = ETERM;
            return Err(SocketError("context was terminated"));
        }

        //  Check whether message passed to the function is valid.
        if !msg.check() {
            // errno = EFAULT;
            return Err(SocketError("invalid message"));
        }

        //  Once every INBOUND_POLL_RATE messages check for signals and process
        //  incoming commands. This happens only if we are not polling altogether
        //  because there are messages available all the time. If poll occurs,
        //  ticks is set to zero and thus we avoid this code.
        //
        //  Note that 'recv' uses different command throttling algorithm (the one
        //  described above) from the one used by 'send'. This is because counting
        //  ticks is more efficient than doing RDTSC all the time.
        self.ticks += 1;
        if self.ticks == INBOUND_POLL_RATE {
            self.process_commands(options, 0, false)?;
            self.ticks = 0;
        }

        //  Get the message.
        // let mut rc = self.xrecv(msg_);
        let mut rc = self.xrecv(msg);
        if ((rc != 0 && get_errno() != EAGAIN)) {
            return Err(SocketError("recieve failed"));
        }

        //  If we have the message, return immediately.
        if (rc == 0) {
            self.extract_flags(msg);
            return Ok(());
        }

        //  If the message cannot be fetched immediately, there are two scenarios.
        //  For non-blocking recv, commands are processed in case there's an
        //  activate_reader command already waiting in a command pipe.
        //  If it's not, return EAGAIN.
        if (flags & ZMQ_DONTWAIT != 0) || options.rcvtimeo == 0 {
            self.process_commands(options, 0, false)?;
            self.ticks = 0;

            // rc = self.xrecv(msg_);
            let rc = self.xrecv(msg);
            if rc < 0 {
                return Err(SocketError("receive failed"));
            }
            self.extract_flags(msg);

            return Ok(());
        }

        //  Compute the time when the timeout should occur.
        //  If the timeout is infinite, don't care.
        let mut timeout = options.rcvtimeo;
        let mut end = if timeout < 0 { 0 } else { (self.clock.now_ms() + timeout) };

        //  In blocking scenario, commands are processed over and over again until
        //  we are able to fetch a message.
        let mut block = (self.ticks != 0);
        loop {
            self.process_commands(options, if block { timeout } else { 0 }, false)?;
            rc = self.xrecv(msg);
            if rc == 0 {
                self.ticks = 0;
                break;
            }
            if get_errno() != EAGAIN {
                return Err(SocketError("EAGAIN"));
            }
            block = true;
            if timeout > 0 {
                timeout = (end - self.clock.now_ms());
                if timeout <= 0 {
                    // errno = EAGAIN;
                    return Err(SocketError("EAGAIN"));
                }
            }
        }

        self.extract_flags(msg);
        Ok(())
    }

    pub unsafe fn xrecv(&mut self, msg: &mut ZmqMsg) -> i32 {
        let mut rc = match self.socket_type {
            ZmqSocketType::Client => client_xrecv(self, msg),
            ZmqSocketType::Dealer => dealer_xrecv(self, msg),
            ZmqSocketType::Dgram => dgram_xrecv(self, msg),
            ZmqSocketType::Dish => dish_xrecv(self, msg),
            ZmqSocketType::Gather => gather_xrecv(self, msg),
            ZmqSocketType::Pair => pair_xrecv(self, msg),
            ZmqSocketType::Peer => peer_xrecv(self, msg),
            ZmqSocketType::Pub => pub_xrecv(self, msg),
            ZmqSocketType::Pull => pull_xrecv(self, msg),
            ZmqSocketType::Push => push_xrecv(self, msg),
            ZmqSocketType::Radio => radio_xrecv(self, msg),
            ZmqSocketType::Rep => rep_xrecv(self, msg),
            ZmqSocketType::Req => req_xrecv(self, msg),
            ZmqSocketType::Router => router_xrecv(self, msg),
            ZmqSocketType::Scatter => scatter_xrecv(self, msg),
            ZmqSocketType::Server => server_xrecv(self, msg),
            ZmqSocketType::Sub => sub_xrecv(self, msg),
            ZmqSocketType::Xpub => xpub_xrecv(self, msg),
            ZmqSocketType::Xsub => xsub_xrecv(self, msg),
        };
        rc
    }

    pub fn close(&mut self) -> Result<(), ZmqError> {
        // scoped_optional_lock_t sync_lock (_thread_safe ? &_sync : NULL);

        //  Remove all existing signalers for thread safe sockets
        if (self.thread_safe) {
            ((self.mailbox)).clear_signalers();
        }

        //  Mark the socket as dead
        self.tag = 0xdeadbeef;


        //  Transfer the ownership of the socket from this application thread
        //  to the reaper thread which will take care of the rest of shutdown
        //  process.
        self.send_reap(self);

        Ok(())
    }

    pub fn has_in(&mut self) -> bool {
        match self.socket_type {
            ZmqSocketType::Client => client_xhas_in(self),
            ZmqSocketType::Dealer => dealer_xhas_in(self),
            ZmqSocketType::Dgram => dgram_xhas_in(self),
            ZmqSocketType::Dish => dish_xhas_in(self),
            ZmqSocketType::Gather => gather_xhas_in(self),
            ZmqSocketType::Pair => pair_xhas_in(self),
            ZmqSocketType::Peer => peer_xhas_in(self),
            ZmqSocketType::Pub => pub_xhas_in(self),
            ZmqSocketType::Pull => pull_xhas_in(self),
            ZmqSocketType::Push => push_xhas_in(self),
            ZmqSocketType::Radio => radio_xhas_in(self),
            ZmqSocketType::Rep => rep_xhas_in(self),
            ZmqSocketType::Req => req_xhas_in(self),
            ZmqSocketType::Router => router_xhas_in(self),
            ZmqSocketType::Scatter => scatter_xhas_in(self),
            ZmqSocketType::Server => server_xhas_in(self),
            ZmqSocketType::Sub => sub_xhas_in(self),
            ZmqSocketType::Xpub => xpub_xhas_in(self),
            ZmqSocketType::Xsub => xsub_xhas_in(self),
        }
    }

    pub fn has_out(&mut self) -> bool {

        // self.xhas_out()
        match self.socket_type {
            ZmqSocketType::Client => client_xhas_out(self),
            ZmqSocketType::Dealer => dealer_xhas_out(self),
            ZmqSocketType::Dgram => dgram_xhas_out(self),
            ZmqSocketType::Dish => dish_xhas_out(self),
            ZmqSocketType::Gather => gather_xhas_out(self),
            ZmqSocketType::Pair => pair_xhas_out(self),
            ZmqSocketType::Peer => peer_xhas_out(self),
            ZmqSocketType::Pub => pub_xhas_out(self),
            ZmqSocketType::Pull => pull_xhas_out(self),
            ZmqSocketType::Push => push_xhas_out(self),
            ZmqSocketType::Radio => radio_xhas_out(self),
            ZmqSocketType::Rep => rep_xhas_out(self),
            ZmqSocketType::Req => req_xhas_out(self),
            ZmqSocketType::Router => router_xhas_out(self),
            ZmqSocketType::Scatter => scatter_xhas_out(self),
            ZmqSocketType::Server => server_xhas_out(self),
            ZmqSocketType::Sub => sub_xhas_out(self),
            ZmqSocketType::Xpub => xpub_xhas_out(self),
            ZmqSocketType::Xsub => xsub_has_out(self),
        }
    }

    pub fn start_reaping(&mut self, poller_: &mut ZmqPoller) {
        //  Plug the socket to the reaper thread.
        self.poller = Some(poller_);

        // fd_t fd;
        let mut fd: ZmqFd = -1;

        if (!self.thread_safe) {
            fd = ((self.mailbox)).get_fd();
        } else {
            // scoped_optional_lock_t sync_lock (_thread_safe ? &_sync : NULL);

            // _reaper_signaler = new (std::nothrow) signaler_t ();
            self.reaper_signaler = Some(ZmqSignaler::new());
            // zmq_assert (_reaper_signaler);

            //  Add signaler to the safe mailbox
            fd = self.reaper_signaler.get_fd();
            (self.mailbox).add_signaler(&mut self.reaper_signaler.unwrap());

            //  Send a signal to make sure reaper handle existing commands
            self.reaper_signaler.send();
        }

        self.handle = self.poller.add_fd(fd, self);
        self.poller.set_pollin(self.handle.unwrap());

        //  Initialise the termination and check whether it can be deallocated
        //  immediately.
        self.terminate();
        self.check_destroy();
    }

    pub fn process_commands(&mut self, options: &ZmqOptions, timeout_: i32, throttle_: bool) -> Result<(), ZmqError> {
        if timeout_ == 0 {
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
            if tsc && throttle_ {
                //  Check whether TSC haven't jumped backwards (in case of migration
                //  between CPU cores) and whether certain time have elapsed since
                //  last command processing. If it didn't do nothing.
                if tsc >= self.last_tsc && tsc - self.last_tsc <= MAX_COMMAND_DELAY {
                    Ok(())
                }
                self.last_tsc = tsc;
            }
        }

        //  Check whether there are any commands pending for this thread.
        let mut cmd = ZmqCommand::new();
        let mut rc = self.mailbox.recv(&mut cmd, timeout_);

        // if (rc != 0 && errno == EINTR)
        //     return -1;

        //  Process all available commands.
        while rc == 0 || get_errno() == EINTR {
            if rc == 0 {
                obj_process_command(options, &mut cmd, self.pipe.unwrap());
            }
            rc = self.mailbox.recv(&mut cmd, 0);
        }

        // zmq_assert (errno == EAGAIN);

        if self.ctx_terminated {
            // errno = ETERM;
            return Err(InvalidContext("context has been terminated"));
        }

        Ok(())
    }

    pub unsafe fn process_stop(&mut self, options: &ZmqOptions) {
        self.stop_monitor(options, false);
        self.ctx_terminated = true;
    }

    pub unsafe fn process_bind(&mut self, options: &mut ZmqOptions, pipe: &mut ZmqPipe) {
        self.attach_pipe(options, pipe, false, false);
    }

    pub unsafe fn process_term(&mut self, linger_: i32) {
        //  Unregister all inproc endpoints associated with this socket.
        //  Doing this we make sure that no new pipes from other sockets (inproc)
        //  will be initiated.
        self.unregister_endpoints(self);

        //  Ask all attached pipes to terminate.
        // for (pipes_t::size_type i = 0, size = _pipes.size (); i != size; ++i)
        for i in 0..self.pipes.len() {
            //  Only inprocs might have a disconnect message set
            self.pipes[i].send_disconnect_msg();
            self.pipes[i].terminate(false);
        }
        self.register_term_acks((self.pipes.size()));

        //  Continue the termination process immediately.
        own_process_term(&mut self.own.owned, &mut self.own.terminating, &mut self.own.term_acks, linger_);
    }

    pub fn process_term_endpoint(&mut self, options: &ZmqOptions, endpoint_: &String) -> Result<(), ZmqError> {
        self.term_endpoint(options, endpoint_)
    }

    pub unsafe fn process_pipe_stats_publish(&mut self, options: &ZmqOptions, outbound_queue_count_: u64, inbound_queue_count_: u64, endpoint_pair_: &mut ZmqEndpointUriPair) {
        let mut values: [u64; 2] = [outbound_queue_count_, inbound_queue_count_];
        self.event(options, endpoint_pair_, &values, 2, ZMQ_EVENT_PIPES_STATS);
    }

    pub unsafe fn query_pipes_stats(&mut self) -> i32 {
        {
            // scoped_lock_t lock (_monitor_sync);
            if !(self.monitor_events & ZMQ_EVENT_PIPES_STATS) {
                // errno = EINVAL;
                return -1;
            }
        }
        if (self.pipes.size() == 0) {
            // errno = EAGAIN;
            return -1;
        }
        // for (pipes_t::size_type i = 0, size = _pipes.size (); i != size; ++i)
        for i in 0..self.pipes.size() {
            self.pipes[i].send_stats_to_peer(self);
        }

        return 0;
    }

    pub fn update_pipe_options(&mut self, options: &mut ZmqOptions, option_: i32) {
        if option_ == ZMQ_SNDHWM || option_ == ZMQ_RCVHWM {
            // for (pipes_t::size_type i = 0, size = _pipes.size (); i != size; ++i)
            for i in 0..self.pipes.size() {
                self.pipes[i].set_hwms(options.rcvhwm, options.sndhwm);
                self.pipes[i].send_hwms_to_peer(options.sndhwm, options.rcvhwm);
            }
        }
    }

    pub fn process_destroy(&mut self) {
        self.destroyed = true;
    }

    // pub fn xsetsockopt(&mut self, a: i32, b: &[u8], c: usize) -> i32 {
    //     // self.setsockopt(a, b, c)
    //     return -1;
    // }

    // pub fn xgetsockopt(&mut self, a: i32, b: *mut c_void, c: *mut usize) -> Result<[u8], ZmqError> {
    //     // self.getsockopt(a, b, c)
    //     unimplemented!();
    //     return Err(SocketError("failed to get socket option"));
    // }

    // pub fn xhas_out(&mut self) -> bool {
    //     return false;
    // }

    // pub fn xsend(&mut self, msg_: *mut ZmqMsg) -> i32 {
    //     return -1;
    // }

    // pub fn xhas_in(&mut self) -> bool {
    //     return false;
    // }

    // pub fn xjoin(&mut self, Group: &str) -> i32 {
    //     return -1;
    // }

    // pub fn xleave(&mut self, group_: &str) -> i32 {
    //     return -1;
    // }

    // pub fn xrecv(&mut self, msg_: *mut ZmqMsg) -> i32 {
    //     return -1;
    // }

    // pub fn xread_activated(&mut self, pipe_: *mut ZmqPipe) {
    //     unimplemented!()
    // }

    // pub fn xwrite_activated(&mut self, pipe_: *mut ZmqPipe) {
    //     unimplemented!()
    // }

    // pub fn xhiccuped(&mut self, pipe_: *mut ZmqPipe) {
    //     unimplemented!()
    // }

    pub unsafe fn in_event(&mut self, options: &ZmqOptions) {
        {
            // scoped_optional_lock_t sync_lock (_thread_safe ? &_sync : NULL);

            //  If the socket is thread safe we need to unsignal the reaper signaler
            if self.thread_safe {
                self.reaper_signaler.recv();
            }

            self.process_commands(options, 0, false);
        }
        self.check_destroy();
    }

    pub unsafe fn out_event(&mut self) {
        unimplemented!()
    }

    pub unsafe fn timer_event(&mut self) {
        unimplemented!()
    }

    pub fn check_destroy(&mut self) {
        //  If the object was already marked as destroyed, finish the deallocation.
        if (self.destroyed) {
            //  Remove the socket from the reaper's poller.
            self.poller.rm_fd(&self.handle);

            //  Remove the socket from the context.
            self.destroy_socket(self);

            //  Notify the reaper about the fact.
            self.send_reaped();

            //  Deallocate.
            self.process_destroy();
        }
    }

    pub unsafe fn read_activated(&mut self, options: &mut ZmqOptions, pipe: &mut ZmqPipe) {
        self.xread_activated(options, pipe);
    }

    pub unsafe fn xread_activated(&mut self, options: &mut ZmqOptions, pipe: &mut ZmqPipe) {
        match self.socket_type {
            ZmqSocketType::Client => client_xread_activated(self, pipe),
            ZmqSocketType::Dealer => dealer_xread_activated(self, pipe),
            ZmqSocketType::Dgram => dgram_xread_activated(self, pipe),
            ZmqSocketType::Dish => dish_xread_activated(self, pipe),
            ZmqSocketType::Gather => gather_xread_activated(self, pipe),
            ZmqSocketType::Pair => pair_xread_activated(self, pipe),
            ZmqSocketType::Peer => peer_xread_activated(self, pipe),
            ZmqSocketType::Pub => pub_xread_activated(self, pipe),
            ZmqSocketType::Pull => pull_xread_activated(self, pipe),
            ZmqSocketType::Push => push_xread_activated(self, pipe),
            ZmqSocketType::Radio => radio_xread_activated(self, pipe),
            ZmqSocketType::Rep => rep_xread_activated(self, pipe),
            ZmqSocketType::Req => req_xread_activated(self, pipe),
            ZmqSocketType::Router => router_xread_activated(self, pipe),
            ZmqSocketType::Scatter => scatter_xread_activated(self, pipe),
            ZmqSocketType::Server => server_xread_activated(self, pipe),
            ZmqSocketType::Sub => sub_xread_activated(self, pipe),
            ZmqSocketType::Xpub => xpub_xread_activated(self, options, pipe),
            ZmqSocketType::Xsub => xsub_xread_activated(self, pipe),
        }
    }

    pub unsafe fn write_activated(&mut self, pipe: &mut ZmqPipe) {
        self.xwrite_activated(pipe);
    }

    pub unsafe fn xwrite_activated(&mut self, pipe: &mut ZmqPipe) {
        match self.socket_type {
            ZmqSocketType::Client => client_xwrite_activated(self, pipe),
            ZmqSocketType::Dealer => dealer_xwrite_activated(self, pipe),
            ZmqSocketType::Dgram => dgram_xwrite_activated(self, pipe),
            ZmqSocketType::Dish => dish_xwrite_activated(self, pipe),
            ZmqSocketType::Gather => gather_xwrite_activated(self, pipe),
            ZmqSocketType::Pair => pair_xwrite_activated(self, pipe),
            ZmqSocketType::Peer => peer_xwrite_activated(self, pipe),
            ZmqSocketType::Pub => pub_xwrite_activated(self, pipe),
            ZmqSocketType::Pull => pull_xwrite_activated(self, pipe),
            ZmqSocketType::Push => push_xwrite_activated(self, pipe),
            ZmqSocketType::Radio => radio_xwrite_activated(self, pipe),
            ZmqSocketType::Rep => rep_xwrite_activated(self, pipe),
            ZmqSocketType::Req => req_xwrite_activated(self, pipe),
            ZmqSocketType::Router => router_xwrite_activated(self, pipe),
            ZmqSocketType::Scatter => scatter_xwrite_activated(self, pipe),
            ZmqSocketType::Server => server_xwrite_activated(self, pipe),
            ZmqSocketType::Sub => sub_xwrite_activated(self, pipe),
            ZmqSocketType::Xpub => xpub_xwrite_activated(self, pipe),
            ZmqSocketType::Xsub => xsub_xwrite_activated(self, pipe),
        }
    }

    pub unsafe fn pipe_terminated(&mut self, pipe: &mut ZmqPipe) {
        //  Notify the specific socket type about the pipe termination.
        // self.xpipe_terminated(pipe_);

        self.xpipe_terminated(pipe);

        // Remove pipe from inproc pipes
        self.inprocs.erase_pipe(pipe);

        //  Remove the pipe from the list of attached pipes and confirm its
        //  termination if we are already shutting down.
        self.pipes.erase(pipe);

        // Remove the pipe from _endpoints (set it to NULL).
        let identifier = pipe.get_endpoint_pair().identifier();
        if !identifier.empty() {
            // std::pair<endpoints_t::iterator, endpoints_t::iterator> range;
            // range = _endpoints.equal_range (identifier);

            // for (endpoints_t::iterator it = range.first; it != range.second; ++it)
            for it in self.endpoints.iter_mut() {
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

    pub unsafe fn xpipe_terminated(&mut self, pipe: &mut ZmqPipe) {
        match self.socket_type {
            ZmqSocketType::Client => client_xpipe_terminated(self, pipe),
            ZmqSocketType::Dealer => dealer_xpipe_terminated(self, pipe),
            ZmqSocketType::Dgram => dgram_xpipe_terminated(self, pipe),
            ZmqSocketType::Dish => dish_xpipe_terminated(self, pipe),
            ZmqSocketType::Gather => gather_xpipe_terminated(self, pipe),
            ZmqSocketType::Pair => pair_xpipe_terminated(self, pipe),
            ZmqSocketType::Peer => peer_xpipe_terminated(self, pipe),
            ZmqSocketType::Pub => pub_xpipe_terminated(self, pipe),
            ZmqSocketType::Pull => pull_xpipe_terminated(self, pipe),
            ZmqSocketType::Push => push_xpipe_terminated(self, pipe),
            ZmqSocketType::Radio => radio_xpipe_terminated(self, pipe),
            ZmqSocketType::Rep => rep_xpipe_terminated(self, pipe),
            ZmqSocketType::Req => req_xpipe_terminated(self, pipe),
            ZmqSocketType::Router => router_xpipe_terminated(self, pipe),
            ZmqSocketType::Scatter => scatter_xpipe_terminated(self, pipe),
            ZmqSocketType::Server => server_xpipe_terminated(self, pipe),
            ZmqSocketType::Sub => sub_xpipe_terminated(self, pipe),
            ZmqSocketType::Xpub => xpub_xpipe_terminated(self, pipe),
            ZmqSocketType::Xsub => xsub_xpipe_terminated(self, pipe),
        }
    }

    pub unsafe fn extract_flags(&mut self, msg_: * ZmqMsg) {
        //  Test whether routing_id flag is valid for this socket type.
        if ((msg_.flags() & ZmqMsg::routing_id)) {
            // zmq_assert(options.recv_routing_id)
        };

        //  Remove MORE flag.
        self.rcvmore = (msg_.flags() & ZMQ_MSG_MORE) != 0;
    }

    pub unsafe fn monitor(&mut self, options: &mut ZmqOptions, endpoint: &str, events: u64, event_version: i32, type_: i32) -> Result<(), ZmqError> {
        if self.ctx_terminated {
            // errno = ETERM;
            return Err(SocketError("context was terminated"));
        }

        //  Event version 1 supports only first 16 events.
        if event_version == 1 && events >> 16 != 0 {
            // errno = EINVAL;
            return Err(SocketError("invalid version"));
        }

        //  Support deregistering monitoring endpoints as well
        if endpoint == null_mut() {
            self.stop_monitor(options, false);
            return Ok(());
        }
        //  Parse endpoint_uri_ string.
        // std::string protocol;
        let mut protocol = String::new();
        // std::string address;
        let mut address = String::new();
        if self.parse_uri(endpoint, &mut protocol, &mut address).is_err() || self.check_protocol(&protocol) {
            return Err(SocketError("invalid URI"));
        }

        //  Event notification only supported over inproc://
        // if (protocol != protocol_name::inproc) {
        //     // errno = EPROTONOSUPPORT;
        //     return -1;
        // }

        // already monitoring. Stop previous monitor before starting new one.
        if self.monitor_socket.is_some() {
            self.stop_monitor(options, true);
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
        self.monitor_events = events as i64;
        options.monitor_event_version = event_version;
        //  Create a monitor socket of the specified type.
        self.monitor_socket = Some(zmq_socket(self.get_ctx(), type_)?);

        //  Never block context termination on pending event messages
        let mut linger = 0;

        zmq_setsockopt(options, &mut self.monitor_socket.unwrap(), ZMQ_LINGER as i32, &linger.to_le_bytes(), 4)?;

        //  Spawn the monitor socket endpoint
        if zmq_bind(&mut self.monitor_socket.unwrap(), endpoint).is_err() {
            self.stop_monitor(options, false);
            return Err(SocketError("failed to bind"));
        }
        Ok(())
    }

    pub unsafe fn event_connected(&mut self, options: &ZmqOptions, endpoint_uri_pair_: &ZmqEndpointUriPair, err_: i32) {
        let mut values: [u64; 1] = [err_ as u64];
        self.event(options, endpoint_uri_pair_, &values, 1, ZMQ_EVENT_CONNECTED);
    }

    pub unsafe fn event_connect_delayed(&mut self, options: &ZmqOptions, endpoint_uri_pair_: &ZmqEndpointUriPair, err_: i32) {
        let mut values: [u64; 1] = [err_ as u64];
        self.event(options, endpoint_uri_pair_, &values, 1, ZMQ_EVENT_CONNECT_DELAYED);
    }

    pub unsafe fn event_connect_retried(&mut self, options: &ZmqOptions, endpoint_uri_pair_: &ZmqEndpointUriPair, err_: i32) {
        let mut values: [u64; 1] = [err_ as u64];
        self.event(options, endpoint_uri_pair_, &values, 1, ZMQ_EVENT_CONNECT_RETRIED);
    }

    pub unsafe fn event_listening(&mut self, options: &ZmqOptions, endpoint_uri_pair_: &ZmqEndpointUriPair, fd_: ZmqFd) {
        let mut values: [u64; 1] = [fd_ as u64];
        self.event(options, endpoint_uri_pair_, &values, 1, ZMQ_EVENT_LISTENING);
    }

    pub unsafe fn event_bind_failed(&mut self, options: &ZmqOptions, endpoint_uri_pair_: &ZmqEndpointUriPair, err_: i32) {
        let mut values: [u64; 1] = [err_ as u64];
        self.event(options, endpoint_uri_pair_, &values, 1, ZMQ_EVENT_BIND_FAILED);
    }

    pub unsafe fn event_accepted(&mut self, options: &ZmqOptions, endpoint_uri_pair_: &ZmqEndpointUriPair, fd_: ZmqFd) {
        let mut values: [u64; 1] = [fd_ as u64];
        self.event(options, endpoint_uri_pair_, &values, 1, ZMQ_EVENT_ACCEPTED);
    }

    pub unsafe fn event_accept_failed(&mut self, options: &ZmqOptions, endpoint_uri_pair_: &ZmqEndpointUriPair, err_: i32) {
        let mut values: [u64; 1] = [err_ as u64];
        self.event(options, endpoint_uri_pair_, &values, 1, ZMQ_EVENT_ACCEPT_FAILED);
    }

    pub unsafe fn event_closed(&mut self, options: &ZmqOptions, endpoint_uri_pair_: &ZmqEndpointUriPair, fd_: ZmqFd) {
        let mut values: [u64; 1] = [fd_ as u64];
        self.event(options, endpoint_uri_pair_, &values, 1, ZMQ_EVENT_CLOSED);
    }

    pub unsafe fn event_close_failed(&mut self, options: &ZmqOptions, endpoint_uri_pair_: &ZmqEndpointUriPair, err_: i32) {
        let mut values: [u64; 1] = [err_ as u64];
        self.event(options, endpoint_uri_pair_, &values, 1, ZMQ_EVENT_CLOSE_FAILED);
    }

    pub unsafe fn event_disconnected(&mut self, options: &ZmqOptions, endpoint_uri_pair_: &ZmqEndpointUriPair, fd_: ZmqFd) {
        let mut values: [u64; 1] = [fd_ as u64];
        self.event(options, endpoint_uri_pair_, &values, 1, ZMQ_EVENT_DISCONNECTED);
    }

    pub fn event_handshake_failed_no_detail(&mut self, options: &ZmqOptions, endpoint_pair_: &ZmqEndpointUriPair, err_: i32) {
        let mut values: [u64; 1] = [err_ as u64];
        self.event(options, endpoint_pair_, &values, 1, ZMQ_EVENT_HANDSHAKE_FAILED_NO_DETAIL);
    }

    pub fn event_handshake_failed_protocol(&mut self, options: &ZmqOptions, endpoint_pair_: &ZmqEndpointUriPair, err_: i32) {
        let mut values: [u64; 1] = [err_ as u64];
        self.event(options, endpoint_pair_, &values, 1, ZMQ_EVENT_HANDSHAKE_FAILED_PROTOCOL);
    }

    pub fn event_handshake_failed_auth(&mut self, options: &ZmqOptions, endpoint_pair_: &ZmqEndpointUriPair, err_: i32) {
        let mut values: [u64; 1] = [err_ as u64];
        self.event(options, endpoint_pair_, &values, 1, ZMQ_EVENT_HANDSHAKE_FAILED_AUTH);
    }

    pub unsafe fn event_handshake_succeeded(&mut self, options: &ZmqOptions, endpoint_pair_: &ZmqEndpointUriPair, err_: i32) {
        let mut values: [u64; 1] = [err_ as u64];
        self.event(options, endpoint_pair_, &values, 1, ZMQ_EVENT_HANDSHAKE_SUCCEEDED);
    }

    pub fn event(&mut self, options: &ZmqOptions, endpoint_uri_pair_: &ZmqEndpointUriPair, values_: &[u64], values_count_: u64, event_type: u32) {
        if self.monitor_events & event_type {
            self.monitor_event(options, event_type, values_, values_count_, endpoint_uri_pair_);
        }
    }

    pub fn monitor_event(&mut self, options: &ZmqOptions, event_: u32, values_: &[u64], values_count_: u64, endpoint_uri_pair_: &ZmqEndpointUriPair) {
        // this is a private method which is only called from
        // contexts where the _monitor_sync mutex has been locked before

        if self.monitor_socket {
            // zmq_msg_t msg;
            let mut msg = ZmqMsg::default();

            match options.monitor_event_version {
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
                    zmq_msg_init_size(&mut msg, 2 + 4);
                    let mut data = (zmq_msg_data(&mut msg));
                    //  Avoid dereferencing uint32_t on unaligned address
                    // libc::memcpy(data + 0, &event as *const c_void, 2);
                    let event_bytes = event.to_le_bytes();
                    data[0] = event_bytes[0];
                    data[1] = event_bytes[1];

                    // libc::memcpy(data + 2, &value as *const c_void, 4);
                    let value_bytes = value.to_le_bytes();
                    data[2] = value_bytes[0];
                    data[3] = value_bytes[1];
                    data[4] = value_bytes[2];
                    data[5] = value_bytes[3];

                    zmq_msg_send(&mut msg, &mut self.monitor_socket.unwrap(), ZMQ_SNDMORE)?;

                    let endpoint_uri = endpoint_uri_pair_.identifier();

                    //  Send address in second frame
                    zmq_msg_init_size(&mut msg, endpoint_uri.size());
                    // libc::memcpy(zmq_msg_data(&msg), endpoint_uri.as_ptr() as *const c_void,
                    //              endpoint_uri.size());
                    let data = zmq_msg_data(&mut msg);
                    let endpoint_uri_bytes = endpoint_uri.as_bytes();
                    for i in 0..endpoint_uri.size() {
                        data[i] = endpoint_uri_bytes[i];
                    }

                    zmq_msg_send(&mut msg, &mut self.monitor_socket.unwrap(), 0);
                }
                2 => {
                    //  Send event in first frame (64bit unsigned)
                    zmq_msg_init_size(&mut msg, size_of_val(&event_));
                    // libc::memcpy(zmq_msg_data(&mut msg), &event_ as *const c_void, 2);
                    let data = zmq_msg_data(&mut msg);
                    let event_bytes = event_.to_le_bytes();
                    for i in 0..size_of_val(&event_) {
                        data[i] = event_bytes[i];
                    }
                    zmq_msg_send(&mut msg, &mut self.monitor_socket.unwrap(), ZMQ_SNDMORE);

                    //  Send number of values that will follow in second frame
                    zmq_msg_init_size(&mut msg, size_of_val(&values_count_));
                    // libc::memcpy(zmq_msg_data(&mut msg), &values_count_ as *const c_void,
                    //              size_of_val(&values_count_));
                    let data = zmq_msg_data(&mut msg);
                    let values_count_bytes = values_count_.to_le_bytes();
                    for i in 0..size_of_val(&values_count_) {
                        data[i] = values_count_bytes[i];
                    }
                    zmq_msg_send(&mut msg, &mut self.monitor_socket.unwrap(), ZMQ_SNDMORE);

                    //  Send values in third-Nth frames (64bit unsigned)
                    // for (uint64_t i = 0; i < values_count_; ++i)
                    for i in 0..values_count_ {
                        zmq_msg_init_size(&mut msg, size_of_val(&values_[i]));
                        // libc::memcpy(zmq_msg_data(&mut msg), &values_[i],
                        //              size_of_val(&values_[i]));
                        let data = zmq_msg_data(&mut msg);
                        let values_bytes = values_[i].to_le_bytes();
                        for j in 0..size_of_val(&values_[i]) {
                            data[j] = values_bytes[j];
                        }
                        zmq_msg_send(&mut msg, &mut self.monitor_socket.unwrap(), ZMQ_SNDMORE);
                    }

                    //  Send local endpoint URI in second-to-last frame (string)
                    zmq_msg_init_size(&mut msg, endpoint_uri_pair_.local.size());
                    // libc::memcpy(zmq_msg_data(&mut msg), endpoint_uri_pair_.local.c_str(),
                    //              endpoint_uri_pair_.local.size());
                    let data = zmq_msg_data(&mut msg);
                    let local_bytes = endpoint_uri_pair_.local.as_bytes();
                    for i in 0..endpoint_uri_pair_.local.size() {
                        data[i] = local_bytes[i];
                    }
                    zmq_msg_send(&mut msg, &mut self.monitor_socket.unwrap(), ZMQ_SNDMORE);

                    //  Send remote endpoint URI in last frame (string)
                    zmq_msg_init_size(&mut msg, endpoint_uri_pair_.remote.size());
                    // libc::memcpy(zmq_msg_data(&mut msg), endpoint_uri_pair_.remote.c_str(),
                    //              endpoint_uri_pair_.remote.size());
                    let data = zmq_msg_data(&mut msg);
                    let remote_bytes = endpoint_uri_pair_.remote.as_bytes();
                    for i in 0..endpoint_uri_pair_.remote.size() {
                        data[i] = remote_bytes[i];
                    }
                    zmq_msg_send(&mut msg, &mut self.monitor_socket.unwrap(), 0);
                }
                _ => {}
            }
        }
    }

    pub unsafe fn stop_monitor(&mut self, options: &ZmqOptions, send_monitor_stopped_event_: bool) {
        // this is a private method which is only called from
        // contexts where the _monitor_sync mutex has been locked before

        if self.monitor_socket {
            if (self.monitor_events & ZMQ_EVENT_MONITOR_STOPPED) != 0 && send_monitor_stopped_event_ {
                // uint64_t values[1] = {0};
                let mut values: [u64; 1] = [0];
                self.monitor_event(options, ZMQ_EVENT_MONITOR_STOPPED, &values, 1,
                                   &ZmqEndpointUriPair::default());
            }
            zmq_close(&mut self.monitor_socket.unwrap());
            self.monitor_socket = None;
            self.monitor_events = 0;
        }
    }

    pub unsafe fn is_disconnected(&mut self) -> bool {
        self.disconnected
    }
}
