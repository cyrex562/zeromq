#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use std::{collections::{HashMap, HashSet}, sync::Mutex};
use crate::{socket_base::socket_base_t, command::command_t};
use crate::atomic_counter::atomic_counter_t;
use crate::i_mailbox::i_mailbox;
use crate::mailbox::mailbox_t;
use crate::mutex::mutex_t;
use crate::options::options_t;
use crate::pipe::pipe_t;
use crate::reaper::reaper_t;

pub const ZMQ_CTX_TAG_VALUE_GOOD: u32 = 0xCAFEBABE;
pub const ZMQ_CTX_TAG_VALUE_BAD: u32 = 0xDEADBEEF;

#[cfg(target_os="windows")]
pub type pid_t = i32;

pub struct endpoint_t
{
    pub socket: *mut socket_base_t,
    pub options: options_t,
}

pub struct thread_ctx_t
{
    pub _opt_sync: mutex_t,
    pub _thread_priority: i32,
    pub _thread_sched_policy: i32,
    pub _thread_affinity_cpus: HashSet<i32>,
    pub _thread_name_prefix: String,

}

pub const term_tid: i32 = 0;
pub const reaper_tid: i32 = 1;

pub struct pending_connection_t
{
    pub endpoint: endpoint_t,
    pub connect_pipe: *mut pipe_t,
    pub bind_pipe: *mut pipe_t,
}

pub enum side {
    connect_side,
    bind_side,
}

pub fn clipped_maxsocket(mut max_requested_: i32) {
    // TODO
    if max_requested_ > max_fds() && max_fds() != -1 {
        max_requested_ = max_fds() - 1;
    }
    return max_requested_
}

pub struct ctx_t
{
    pub _thread_ctx: thread_ctx_t,
    pub _tag: u32,
    pub _sockets: Vec<socket_base_t>,
    pub _empty_slots: Vec<u32>,
    pub _starting: bool,
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
    #[cfg(feature="fork")]
    pub _pid: pid_t,
    pub _vmci_fd: i32,
    pub _vmci_family: i32,
    pub _vmci_sync: Mutex<()>,
}

impl ctx_t {
    pub fn new() -> Self {
        Self {
            _thread_ctx: thread_ctx_t::new(),
            _tag: 0,
            _sockets: Vec::new(),
            _empty_slots: Vec::new(),
            _starting: false,
            _slot_sync: mutex_t::new(),
            _reaper: ptr::null_mut(),
            _io_threads: io_threads_t::new(),
            _slots: Vec::new(),
            _term_mailbox: mailbox_t::new(),
            _endpoints: HashMap::new(),
            _pending_connections: HashMap::new(),
            _endpoints_sync: mutex_t::new(),
            max_socket_id: atomic_counter_t::new(),
            _max_sockets: 0,
            _max_msgsz: 0,
            _io_thread_count: 0,
            _blocky: false,
            _ipv6: false,
            _zero_copy: false,
            #[cfg(feature="fork")]
            _pid: 0,
            _vmci_fd: 0,
            _vmci_family: 0,
            _vmci_sync: Mutex::new(()),
        }
    }

    pub fn check_tag(&mut self) -> bool {
        self._tag == ZMQ_CTX_TAG_VALUE_GOOD
    }

    pub fn valid(&mut self) -> bool {
        self._term_mailbox.valid()
    }

    pub fn terminate(&mut self) {
        self._slot_sync.lock();

        let save_terminating = self._terminating;
        self._terminating = false;
        for p in self._pending_connections {
            let s: *mut socket_base_t = create_socket(ZMQ_PAIR);
            s.bind(p.first);
            s.close();
        }
        self._terminating = save_terminating;

        if !self._starting {
            let restarted = self._terminating;
            self._terminating = true;
    
            if !restarted {
                for i in 0 .. self._sockets.len() {
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
        let mut locker = scoped_lock_t::new(&self._slot_sync);
        if !self._terminating {
            self._terminating = true;
            if !self._starting {
                for i in 0 .. self._sockets.len() {
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
}
