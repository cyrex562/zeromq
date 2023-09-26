#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use std::{collections::{HashMap, HashSet}, ptr, sync::Mutex};
use std::ffi::c_void;
use std::ptr::null_mut;
use crate::{socket_base::socket_base_t, command::command_t};
use crate::atomic_counter::atomic_counter_t;
use crate::defines::ZMQ_PAIR;
use crate::i_mailbox::i_mailbox;
use crate::io_thread::io_thread_t;
use crate::mailbox::mailbox_t;
use crate::msg::msg_t;
use crate::mutex::{mutex_t, scoped_lock_t};
use crate::options::{get_effective_conflate_option, options_t};
use crate::pipe::{pipe_t, send_routing_id};
use crate::poller::max_fds;
use crate::reaper::reaper_t;
use crate::thread::{thread_fn, thread_t};

pub type io_threads_t = Vec<*mut io_thread_t>;

pub const ZMQ_CTX_TAG_VALUE_GOOD: u32 = 0xCAFEBABE;
pub const ZMQ_CTX_TAG_VALUE_BAD: u32 = 0xDEADBEEF;

#[cfg(target_os = "windows")]
pub type pid_t = i32;

pub struct endpoint_t {
    pub socket: *mut socket_base_t,
    pub options: options_t,
}

pub struct thread_ctx_t {
    pub _opt_sync: mutex_t,
    pub _thread_priority: i32,
    pub _thread_sched_policy: i32,
    pub _thread_affinity_cpus: HashSet<i32>,
    pub _thread_name_prefix: String,

}

pub const term_tid: i32 = 0;
pub const reaper_tid: i32 = 1;

pub struct pending_connection_t {
    pub endpoint: endpoint_t,
    pub connect_pipe: *mut pipe_t,
    pub bind_pipe: *mut pipe_t,
}

pub enum side {
    connect_side,
    bind_side,
}

pub fn clipped_maxsocket(mut max_requested_: i32) -> i32 {
    if max_requested_ > max_fds() && max_fds() != -1 {
        max_requested_ = max_fds() - 1;
    }
    max_requested_
}

pub struct ctx_t {
    pub _thread_ctx: thread_ctx_t,
    pub _tag: u32,
    pub _sockets: Vec<socket_base_t>,
    pub _empty_slots: Vec<u32>,
    pub _starting: bool,
    pub _terminating: bool,
    pub _slot_sync: mutex_t,
    pub _reaper: *mut reaper_t,
    pub _io_threads: io_threads_t,
    pub _slots: Vec<*mut i_mailbox>,
    pub _term_mailbox: mailbox_t,
    pub _endpoints: HashMap<String, endpoint_t>,
    pub _pending_connections: HashMap<String, pending_connection_t>,
    pub _endpoints_sync: mutex_t,
    pub max_socket_id: atomic_counter_t,
    pub _max_sockets: i32,
    pub _max_msgsz: i32,
    pub _io_thread_count: i32,
    pub _blocky: bool,
    pub _ipv6: bool,
    pub _zero_copy: bool,
    #[cfg(feature = "fork")]
    pub _pid: pid_t,
    #[cfg(feature = "vmci")]
    pub _vmci_fd: i32,
    #[cfg(feature = "vmci")]
    pub _vmci_family: i32,
    #[cfg(feature = "vmci")]
    pub _vmci_sync: Mutex<()>,
    pub _opt_sync: mutex_t,
}

impl ctx_t {
    pub fn new() -> Self {
        Self {
            _thread_ctx: thread_ctx_t::new(),
            _tag: 0,
            _sockets: Vec::new(),
            _empty_slots: Vec::new(),
            _starting: false,
            _terminating: false,
            _slot_sync: mutex_t::new(),
            _reaper: null_mut(),
            _io_threads: io_threads_t::new(),
            _slots: Vec::new(),
            _term_mailbox: mailbox_t::new(),
            _endpoints: HashMap::new(),
            _pending_connections: HashMap::new(),
            _endpoints_sync: mutex_t::new(),
            max_socket_id: atomic_counter_t::new(0),
            _max_sockets: 0,
            _max_msgsz: 0,
            _io_thread_count: 0,
            _blocky: false,
            _ipv6: false,
            _zero_copy: false,
            #[cfg(feature = "fork")]
            _pid: 0, // TODO getpid()
            _vmci_fd: 0,
            _vmci_family: 0,
            _vmci_sync: Mutex::new(()),
            _opt_sync: mutex_t::new(),
        }
    }

    pub fn check_tag(&mut self) -> bool {
        self._tag == ZMQ_CTX_TAG_VALUE_GOOD
    }

    pub fn valid(&mut self) -> bool {
        self._term_mailbox.valid()
    }

    pub fn terminate(&mut self) -> i32 {
        self._slot_sync.lock();

        let save_terminating = self._terminating;
        self._terminating = false;
        for p in self._pending_connections {
            let s: *mut socket_base_t = self.create_socket(ZMQ_PAIR as i32);
            s.bind(p.0);
            s.close();
        }
        self._terminating = save_terminating;

        if !self._starting {
            let restarted = self._terminating;
            self._terminating = true;

            if !restarted {
                for i in 0..self._sockets.len() {
                    self._sockets[i].stop();
                }
                if self._sockets.is_empty() {
                    self._reaper.stop();
                }
            }
            self._slot_sync.unlock();

            let mut cmd = command_t::new();
            let rc = self._term_mailbox.recv(&cmd, -1);
            // TODO: && errno == EINTR
            if rc == -1 {
                return -1;
            }
        }
        self._slot_sync.unlock();

        return 0;
    }

    pub fn shutdown(&mut self) -> i32 {
        let mut locker = scoped_lock_t::new(&mut self._slot_sync);
        if !self._terminating {
            self._terminating = true;
            if !self._starting {
                for i in 0..self._sockets.len() {
                    self._sockets[i].stop();
                }
                if self._sockets.is_empty() {
                    self._reaper.stop();
                }
            }
        }

        return 0;
    }

    pub fn set_max_sockets(&mut self, max_sockets_: i32) {
        self._max_sockets = max_sockets_;
    }

    pub fn set_io_threads(&mut self, io_thread_count: i32) {
        self._io_thread_count = io_thread_count;
    }

    pub fn set_ipv6(&mut self, ipv6: bool) {
        self._ipv6 = ipv6;
    }

    pub fn set_blocky(&mut self, blocky: bool) {
        self._blocky = blocky;
    }

    pub fn set_max_msgsz(&mut self, max_msgsz: i32) {
        self._max_msgsz = max_msgsz;
    }

    pub fn set_zero_copy_recv(&mut self, zero_copy: bool) {
        self._zero_copy = zero_copy;
    }

    pub fn get_max_sockets(&mut self) -> i32 {
        self._max_sockets
    }

    pub fn get_socket_limit(&mut self) -> i32 {
        self._max_sockets
    }

    pub fn get_io_thread_count(&mut self) -> i32 {
        self._io_thread_count
    }

    pub fn get_ipv6(&mut self) -> bool {
        self._ipv6
    }

    pub fn get_blocky(&mut self) -> bool {
        self._blocky
    }

    pub fn get_max_msgsz(&mut self) -> i32 {
        self._max_msgsz
    }

    pub fn get_msg_t_sz(&mut self) -> usize {
        std::mem::size_of::<msg_t>()
    }

    pub fn get_zero_copy_recv(&mut self) -> bool {
        self._zero_copy
    }

    pub fn start(&mut self) -> bool {
        self._opt_sync.lock();
        let term_and_reaper_threads_count = 2;
        let mazmq = self._max_sockets;
        let ios = self._io_thread_count;
        self._opt_sync.unlock();
        let slot_count = mazmq + ios + term_and_reaper_threads_count;
        self._slots.reserve((slot_count - term_and_reaper_threads_count) as usize);
        self._slots[term_tid] = &self._term_mailbox;
        self._reaper = reaper_t::new(self, reaper_tid);
        self._slots[reaper_tid] = &self._reaper.get_mailbox();
        self._reaper.start();
        self._slots.resize(slot_count as usize, null_mut());

        for i in term_and_reaper_threads_count..ios + term_and_reaper_threads_count {
            let io_thread = io_thread_t::new(self, i);
            // if io_thread.get_mailbox().valid() == false{}
            self._io_threads.push(&mut io_thread);
            self._slots[i] = &io_thread.get_mailbox();
            io_thread.start();
        }

        for i in self._slots.len() - 1..ios + term_and_reaper_threads_count {
            self._empty_slots.push(i as u32);
        }

        self._starting = false;
        return true;

        // self._reaper.stop();
        // self._reaper = null_mut();

        // selff._slots.clear()
    }

    pub fn create_socket(&mut self, type_: i32) -> *mut socket_base_t {
        if self._terminating {
            return null_mut();
        }

        if self._starting {
            if !self.start() {
                return null_mut();
            }
        }

        if self._empty_slots.empty() {
            return null_mut();
        }

        let slot = self._empty_slots[-1];
        self._empty_slots.pop();
        let sid = self.max_socket_id.add(1) + 1;

        let s = socket_base_t::new(self, sid, slot, type_);
        self._sockets.push(s);
        self._slots[slot] = s.get_mailbox();
        return &mut s;
    }

    pub unsafe fn destroy_socket(&mut self, socket_: *mut socket_base_t) {
        let mut locker = scoped_lock_t::new(&mut self._slot_sync);
        let slot = (*socket_).get_slot();
        self._slots[slot] = null_mut();
        self._empty_slots.push(slot);
        for i in 0..self._sockets.len() {
            if self._sockets[i] == *socket_ {
                self._sockets.remove(i);
                break;
            }
        }
        // self._sockets.remove(socket_);

        if self._terminating && self._sockets.is_empty() {
            self._reaper.stop();
        }
    }

    pub fn get_reaper(&mut self) -> *mut reaper_t {
        self._reaper
    }

    pub unsafe fn start_thread(&mut self, thread_: &mut thread_t, tfn_: thread_fn, arg_: *mut c_void, name_: &str) -> bool {
        let thread_name = format!("{}{}ZMQbg{}{}",
                                  if self._thread_ctx._thread_name_prefix.is_empty() { "" } else { &self._thread_ctx._thread_name_prefix },
                                  if self._thread_ctx._thread_name_prefix.is_empty() { "" } else { "/" },
                                  if name_.is_empty() { "/" } else { "" },
                                  if name_.is_empty() { name_ } else { "" }
        );
        let thread_name_cstr = std::ffi::CString::new(thread_name).unwrap();
        thread_.start(tfn_, arg_, thread_name_cstr.as_ptr());
        return true;
    }

    pub fn send_command(&mut self, tid_: u32, command_: &mut command_t) {
        self._slots[tid_ as usize].send(command_);
    }

    pub fn choose_io_thread(&mut self, affinity_: u64) -> *mut io_thread_t {
        let mut min_load = 0x7fffffff;
        let mut result = null_mut();
        for i in 0..self._io_threads.len() {
            let load = self._io_threads[i].get_load();
            if load < min_load {
                min_load = load;
                result = self._io_threads[i];
            }
        }
        return result;
    }

    pub fn register_endpoint(&mut self, addr_: &str, socket_: *mut socket_base_t, options_: &mut options_t) -> i32 {
        let mut locker = scoped_lock_t::new(&mut self._endpoints_sync);
        let mut endpoint = endpoint_t::new();
        endpoint.socket = socket_;
        endpoint.options = options_.clone();
        self._endpoints.insert(addr_.to_string(), endpoint);
        return 0;
    }

    pub fn unregister_endpoint(&mut self, addr_: &str) -> i32 {
        let mut locker = scoped_lock_t::new(&mut self._endpoints_sync);
        self._endpoints.remove(addr_);
        return 0;
    }

    pub fn unregister_endpoints(&mut self, socket_: *mut socket_base_t) {
        let mut locker = scoped_lock_t::new(&mut self._endpoints_sync);
        let mut to_remove = Vec::new();
        for i in self._endpoints {
            if i.1.socket == socket_ {
                to_remove.push(i.0);
            }
        }
        for i in to_remove {
            self._endpoints.remove(&i);
        }
    }

    pub fn find_endpoint(&mut self, addr_: &str) -> *mut socket_base_t {
        let mut locker = scoped_lock_t::new(&mut self._endpoints_sync);
        let mut endpoint = self._endpoints.get(addr_);
        if endpoint.is_none() {
            return null_mut();
        }
        return endpoint.unwrap().socket;
    }

    pub unsafe fn pend_connection(&mut self, addr_: &str, endpoint_: &endpoint_t, pipes_: *mut *mut pipe_t) -> i32 {
        let mut locker = scoped_lock_t::new(&mut self._endpoints_sync);
        let mut pending_connection = pending_connection_t::new();
        pending_connection.endpoint = endpoint_.clone();
        pending_connection.connect_pipe = pipes_;
        pending_connection.bind_pipe = pipes_.offset(1);

        let mut found = false;
        for endpoint in self._endpoints {
            if endpoint.0 == addr_ {
                self.connect_inproc_sockets(endpoint.1.socket, &endpoint.1.options, pending_connection, side::connect_side);
                found = true;
                break;
            }
        }
        if found == false {
            endpoint_.socket.inc_seqnum();
            self._pending_connections.insert(addr_.to_string(), pending_connection);
        }

        return 0;
    }

    pub unsafe fn connect_pending(&mut self, addr_: &str, bind_socket_: *mut socket_base_t) {
        let mut locker = scoped_lock_t::new(&mut self._endpoints_sync);
        for k in self._pending_connections.keys() {
            if k == addr_ {
                let ep = self._endpoints.get(addr_).unwrap();
                let pc = self._pending_connections.get_mut(addr_).unwrap();
                self.connect_inproc_sockets(bind_socket_, &ep.options.clone(), pc, side::bind_side);
                self._pending_connections.remove(k);
            }
        }
    }

    pub unsafe fn connect_inproc_sockets(
        &mut self,
        bind_socket_: *mut socket_base_t,
        bind_options_: &options_t,
        pending_connection_: &mut pending_connection_t,
        side_: side,
    ) {
        bind_socket_.inc_seqnum();
        pending_connection_.bind_pipe.set_tid(bind_socket_.get_tid());
        if !bind_options_.recv_routing_id {
            let mut msg = msg_t::new();
            let ok = pending_connection_.bind_pipe.read2(&msg);
            let rc = msg.close();
        }

        if !get_effective_conflate_option(&pending_connection_.endpoint.options) {
            pending_connection_.connect_pipe.set_hwms_boost(bind_options_.sndhwm, bind_options_.rcvhwm);
            pending_connection_.bind_pipe.set_hwms_boost(
                pending_connection_.endpoint.options.sndhwm,
                pending_connection_.endpoint.options.rcvhwm,
            );
            pending_connection_.connect_pipe.set_hwms(
                pending_connection_.endpoint.options.rcvhwm,
                pending_connection_.endpoint.options.sndhwm,
            );
            pending_connection_.bind_pipe.set_hwms(bind_options_.sndhwm, bind_options_.rcvhwm);
        } else {
            pending_connection_.connect_pipe.set_hwms(0, 0);
            pending_connection_.bind_pipe.set_hwms(0, 0);
        }

        if side_ == side::bind_side {
            let mut cmd = command_t::new();
            cmd.type_ = command_t::bind;
            cmd.args.bind.pipe = pending_connection_.bind_pipe;
            bind_socket_.process_command(cmd);
            bind_socket_.send_inproc_connected(
                pending_connection_.endpoint.socket
            );
        } else {
            pending_connection_.connect_pipe.send_bind(bind_socket_, pending_connection_.bind_pipe, false);
        }

        if pending_connection_.endpoint.options.recv_routing_id && pending_connection_.endpoint.socket.check_tag() {
            send_routing_id(pending_connection_.bind_pipe, bind_options_);
        }
    }
}
