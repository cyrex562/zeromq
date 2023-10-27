#![allow(non_upper_case_globals)]

use std::ptr::null_mut;
use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
};

use crate::atomic_counter::ZmqAtomicCounter;
use crate::defines::{ZMQ_CTX_TAG_VALUE_GOOD, ZMQ_PAIR};
use crate::i_mailbox::IMailbox;
use crate::io_thread::ZmqIoThread;
use crate::mailbox::ZmqMailbox;
use crate::msg::ZmqMsg;
use crate::mutex::{scoped_lock_t, ZmqMutex};
use crate::options::{get_effective_conflate_option, ZmqOptions};
use crate::pipe::{send_routing_id, ZmqPipe};
use crate::poller::max_fds;
use crate::reaper::ZmqReaper;
use crate::thread::{ZmqThread, ZmqThreadFn};
use crate::{command::ZmqCommand, socket_base::ZmqSocketBase};

pub type io_threads_t<'a> = Vec<&'a mut ZmqIoThread<'a>>;

#[cfg(target_os = "windows")]
pub type ZmqPid = i32;

pub struct Endpoint<'a> {
    pub socket: &'a mut ZmqSocketBase<'a>,
    pub options: ZmqOptions,
}

pub struct ZmqThreadCtx {
    pub _opt_sync: ZmqMutex,
    pub _thread_priority: i32,
    pub _thread_sched_policy: i32,
    pub _thread_affinity_cpus: HashSet<i32>,
    pub _thread_name_prefix: String,
}

pub const term_tid: i32 = 0;
pub const reaper_tid: i32 = 1;

pub struct pending_connection_t<'a> {
    pub endpoint: Endpoint,
    pub connect_pipe: &'a mut ZmqPipe<'a>,
    pub bind_pipe: &'a mut ZmqPipe<'a>,
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

pub struct ZmqContext<'a> {
    pub _thread_ctx: ZmqThreadCtx,
    pub _tag: u32,
    pub _sockets: Vec<ZmqSocketBase>,
    pub _empty_slots: Vec<u32>,
    pub _starting: bool,
    pub _terminating: bool,
    pub _slot_sync: ZmqMutex,
    pub _reaper: &'a mut ZmqReaper,
    pub _io_threads: io_threads_t,
    pub _slots: Vec<&'a mut dyn IMailbox>,
    pub _term_mailbox: ZmqMailbox,
    pub _endpoints: HashMap<String, Endpoint>,
    pub _pending_connections: HashMap<String, pending_connection_t<'a>>,
    pub _endpoints_sync: ZmqMutex,
    pub max_socket_id: ZmqAtomicCounter,
    pub _max_sockets: i32,
    pub _max_msgsz: i32,
    pub _io_thread_count: i32,
    pub _blocky: bool,
    pub _ipv6: bool,
    pub _zero_copy: bool,
    #[cfg(feature = "fork")]
    pub _pid: ZmqPid,
    #[cfg(feature = "vmci")]
    pub _vmci_fd: i32,
    #[cfg(feature = "vmci")]
    pub _vmci_family: i32,
    #[cfg(feature = "vmci")]
    pub _vmci_sync: Mutex<()>,
    pub _opt_sync: ZmqMutex,
}

impl ZmqContext {
    pub fn new() -> Self {
        Self {
            _thread_ctx: ZmqThreadCtx::new(),
            _tag: 0,
            _sockets: Vec::new(),
            _empty_slots: Vec::new(),
            _starting: false,
            _terminating: false,
            _slot_sync: ZmqMutex::new(),
            _reaper: null_mut(),
            _io_threads: io_threads_t::new(),
            _slots: Vec::new(),
            _term_mailbox: ZmqMailbox::new(),
            _endpoints: HashMap::new(),
            _pending_connections: HashMap::new(),
            _endpoints_sync: ZmqMutex::new(),
            max_socket_id: ZmqAtomicCounter::new(0),
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
            _opt_sync: ZmqMutex::new(),
        }
    }

    pub fn check_tag(&mut self) -> bool {
        self._tag == ZMQ_CTX_TAG_VALUE_GOOD
    }

    pub fn valid(&mut self) -> bool {
        self._term_mailbox.valid()
    }

    pub fn terminate(&mut self) -> Result<(), ZmqError> {
        // self._slot_sync.lock();

        let save_terminating = self._terminating;
        self._terminating = false;
        for p in self._pending_connections {
            let mut s = self.create_socket(ZMQ_PAIR as i32).unwrap_or;
            s.bind(&p.0)?;
            s.close()?;
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

            let mut cmd = ZmqCommand::new();
            let rc = self._term_mailbox.recv(&cmd, -1);
            // TODO: && errno == EINTR
            if rc == -1 {
                return -1;
            }
        }
        // self._slot_sync.unlock();

        return 0;
    }

    pub fn shutdown(&mut self) -> Result<(),ZmqError> {
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

        Ok(())
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
        std::mem::size_of::<ZmqMsg>()
    }

    pub fn get_zero_copy_recv(&mut self) -> bool {
        self._zero_copy
    }

    pub fn start(&mut self) -> bool {
        // self._opt_sync.lock();
        let term_and_reaper_threads_count = 2;
        let mazmq = self._max_sockets;
        let ios = self._io_thread_count;
        self._opt_sync.unlock();
        let slot_count = mazmq + ios + term_and_reaper_threads_count;
        self._slots
            .reserve((slot_count - term_and_reaper_threads_count) as usize);
        self._slots[term_tid] = &self._term_mailbox;
        self._reaper = ZmqReaper::new(self, reaper_tid);
        self._slots[reaper_tid] = &self._reaper.get_mailbox();
        self._reaper.start();
        self._slots.reserve(slot_count);

        for i in term_and_reaper_threads_count..ios + term_and_reaper_threads_count {
            let io_thread = ZmqIoThread::new(self, i);
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

    pub fn create_socket(&mut self, type_: i32) -> Result<ZmqSocketBase, ZmqError> {
        if self._terminating {
            return Err(ZmqError::SocketError("Context is terminating"));
        }

        if self._starting {
            if !self.start() {
                return Err(ZmqError::SocketError("Context is starting"));
            }
        }

        if self._empty_slots.empty() {
            return Err(ZmqError::SocketError("No empty slots"));
        }

        let slot = self._empty_slots[-1];
        self._empty_slots.pop();
        let sid = self.max_socket_id.add(1) + 1;

        let mut s = ZmqSocketBase::new(self, sid, slot, type_);
        self._sockets.push(s);
        self._slots[slot] = s.get_mailbox();
        return Ok(t s);
    }

    pub unsafe fn destroy_socket(&mut self, socket_: &mut ZmqSocketBase) {
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

    pub fn get_reaper(&mut self) -> &mut ZmqReaper {
        self._reaper
    }

    pub unsafe fn start_thread(
        &mut self,
        thread_: &mut ZmqThread,
        tfn_: ZmqThreadFn,
        arg_: &[u8],
        name_: &str,
    ) -> bool {
        let thread_name = format!(
            "{}{}ZMQbg{}{}",
            if self._thread_ctx._thread_name_prefix.is_empty() {
                ""
            } else {
                &self._thread_ctx._thread_name_prefix
            },
            if self._thread_ctx._thread_name_prefix.is_empty() {
                ""
            } else {
                "/"
            },
            if name_.is_empty() { "/" } else { "" },
            if name_.is_empty() { name_ } else { "" }
        );
        let thread_name_cstr = std::ffi::CString::new(thread_name).unwrap();
        thread_.start(tfn_, arg_, thread_name_cstr.as_ptr());
        return true;
    }

    pub fn send_command(&mut self, tid_: u32, command_: &ZmqCommand) {
        self._slots[tid_ as usize].send(command_);
    }

    pub fn choose_io_thread(&mut self, affinity_: u64) -> *mut ZmqIoThread {
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

    pub fn register_endpoint(
        &mut self,
        addr_: &str,
        socket_: &mut ZmqSocketBase,
        options_: &mut ZmqOptions,
    ) -> i32 {
        let mut locker = scoped_lock_t::new(&mut self._endpoints_sync);
        let mut endpoint = Endpoint::new();
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

    pub fn unregister_endpoints(&mut self, socket_: &mut ZmqSocketBase) {
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

    pub fn find_endpoint(&mut self, addr_: &str) -> &mut ZmqSocketBase {
        let mut locker = scoped_lock_t::new(&mut self._endpoints_sync);
        let mut endpoint = self._endpoints.get(addr_);
        if endpoint.is_none() {
            return null_mut();
        }
        return endpoint.unwrap().socket;
    }

    pub unsafe fn pend_connection(
        &mut self,
        addr_: &str,
        endpoint_: &Endpoint,
        pipes_: &mut [&mut ZmqPipe],
    ) -> i32 {
        let mut locker = scoped_lock_t::new(&mut self._endpoints_sync);
        let mut pending_connection = pending_connection_t::new();
        pending_connection.endpoint = endpoint_.clone();
        pending_connection.connect_pipe = pipes_;
        pending_connection.bind_pipe = pipes_.offset(1);

        let mut found = false;
        for endpoint in self._endpoints {
            if endpoint.0 == addr_ {
                self.connect_inproc_sockets(
                    endpoint.1.socket,
                    &endpoint.1.options,
                    pending_connection,
                    side::connect_side,
                );
                found = true;
                break;
            }
        }
        if found == false {
            endpoint_.socket.inc_seqnum();
            self._pending_connections
                .insert(addr_.to_string(), pending_connection);
        }

        return 0;
    }

    pub unsafe fn connect_pending(&mut self, addr_: &str, bind_socket_: &mut ZmqSocketBase) {
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
        bind_socket_: &mut ZmqSocketBase,
        bind_options_: &ZmqOptions,
        pending_connection_: &mut pending_connection_t,
        side_: side,
    ) {
        bind_socket_.inc_seqnum();
        pending_connection_
            .bind_pipe
            .set_tid(bind_socket_.get_tid());
        if !bind_options_.recv_routing_id {
            let mut msg = ZmqMsg::new();
            let ok = pending_connection_.bind_pipe.read2(&msg);
            let rc = msg.close();
        }

        if !get_effective_conflate_option(&pending_connection_.endpoint.options) {
            pending_connection_
                .connect_pipe
                .set_hwms_boost(bind_options_.sndhwm, bind_options_.rcvhwm);
            pending_connection_.bind_pipe.set_hwms_boost(
                pending_connection_.endpoint.options.sndhwm,
                pending_connection_.endpoint.options.rcvhwm,
            );
            pending_connection_.connect_pipe.set_hwms(
                pending_connection_.endpoint.options.rcvhwm,
                pending_connection_.endpoint.options.sndhwm,
            );
            pending_connection_
                .bind_pipe
                .set_hwms(bind_options_.sndhwm, bind_options_.rcvhwm);
        } else {
            pending_connection_.connect_pipe.set_hwms(0, 0);
            pending_connection_.bind_pipe.set_hwms(0, 0);
        }

        if side_ == side::bind_side {
            let mut cmd = ZmqCommand::new();
            cmd.type_ = ZmqCommand::bind;
            cmd.args.bind.pipe = pending_connection_.bind_pipe;
            bind_socket_.process_command(cmd);
            bind_socket_.send_inproc_connected(pending_connection_.endpoint.socket);
        } else {
            pending_connection_.connect_pipe.send_bind(
                bind_socket_,
                pending_connection_.bind_pipe,
                false,
            );
        }

        if pending_connection_.endpoint.options.recv_routing_id
            && pending_connection_.endpoint.socket.check_tag()
        {
            send_routing_id(pending_connection_.bind_pipe, bind_options_);
        }
    }
}
