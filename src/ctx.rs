use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use libc::{EADDRINUSE, ECONNREFUSED, EINTR, EINVAL, EMFILE, ENOENT, ENOMEM, getpid, pid_t};
use crate::atomic_counter::AtomicCounter;
use crate::endpoint::ZmqEndpoint;
use crate::i_mailbox::i_mailbox;
use crate::io_thread::io_thread_t;
use crate::mailbox::mailbox_t;
use crate::pending_connection::{PendingConnection};
use crate::pipe::pipe_t;
use crate::reaper::reaper_t;
use crate::socket_base::ZmqSocketBase;
use crate::thread_ctx::ThreadCtx;
use std::{mem, process};
use std::intrinsics::unlikely;
use std::mem::size_of;
use std::ptr::null_mut;
use anyhow;
use crate::command::ZmqCommand;
use crate::ctx::side::{bind_side, connect_side};
use crate::ctx::tid_type::{reaper_tid, term_tid};
use crate::msg::ZmqMessage;
use crate::object::object_t;
use crate::options::{get_effective_conflate_option, ZmqOptions};
use crate::zmq_hdr::{ZMQ_PAIR, ZMQ_MAX_SOCKETS, ZMQ_IO_THREADS, ZMQ_IPV6, ZMQ_BLOCKY, ZMQ_MAX_MSGSZ, ZMQ_ZERO_COPY_RECV, ZMQ_SOCKET_LIMIT, zmq_free_fn, zmq_ZmqMessage, ZMQ_ZmqMessage_SIZE};

//  Context object encapsulates all the global state associated with
//  the library.

enum tid_type
    {
        term_tid = 0,
        reaper_tid = 1
    }

enum side
    {
        connect_side,
        bind_side
    }

// #define ZMQ_CTX_TAG_VALUE_GOOD 0xabadcafe
pub const ZMQ_CTX_TAG_VALUE_GOOD: u32 = 0xabadcafe;
// #define ZMQ_CTX_TAG_VALUE_BAD 0xdeadbeef
pub const ZMQ_CTX_TAG_VALUE_BAD: u32 = 0xdeadbeef;

// // typedef array_t<ZmqSocketBase> sockets_t;
// pub type sockets_t = Vec<ZmqSocketBase>;
//
// // typedef std::vector<uint32_t> empty_slots_t;
// pub type empty_slots_s = Vec<u32>;
//
// // typedef std::vector<io_thread_t *> io_threads_t;
// pub type io_threads_t = Vec<io_thread_t>;
//
// // typedef std::map<std::string, endpoint_t> endpoints_t;
// pub type endpoints_t = HashMap<String,ZmqEndpoint>;
//
// // typedef std::multimap<std::string, PendingConnection>
// //       pending_connections_t;
// pub type pending_connections_t = HashMap<String, PendingConnection>

// class ctx_t ZMQ_FINAL : public ThreadCtx
#[derive(Default,Debug,Clone)]
pub struct ZmqContext
{
    pub _thread_ctx_t: ThreadCtx,
// public:
  // private:

    //  Used to check whether the object is a context.
    // uint32_t _tag;
pub _tag: u32,
    //  Sockets belonging to this context. We need the list so that
    //  we can notify the sockets when zmq_ctx_term() is called.
    //  The sockets will return ETERM then.
    pub _sockets: Vec<ZmqSocketBase>,
    // sockets_t _sockets;
    //  List of unused thread slots.

    // empty_slots_t _empty_slots;
pub _empty_slots: Vec<u32>,

    //  If true, zmq_init has been called but no socket has been created
    //  yet. Launching of I/O threads is delayed.
    // bool _starting;
pub _starting: bool,

    //  If true, zmq_ctx_term was already called.
    // bool _terminating;
    pub _terminating: bool,

    //  Synchronisation of accesses to global slot-related data:
    //  sockets, empty_slots, terminating. It also synchronises
    //  access to zombie sockets as such (as opposed to slots) and provides
    //  a memory barrier to ensure that all CPU cores see the same data.
    // mutex_t _slot_sync;
pub _slot_sync: Mutex<u8>,

    //  The reaper thread.
    // reaper_t *_reaper;
pub _reaper: Option<reaper_t>,

    //  I/O threads.

    // io_threads_t _io_threads;
    pub _io_threads: Vec<io_thread_t>,

    //  Array of pointers to mailboxes for both application and I/O threads.
    // std::vector<i_mailbox *> _slots;
    pub _slots: Vec<*mut i_mailbox>,

    //  Mailbox for zmq_ctx_term thread.
    // mailbox_t _term_mailbox;
    pub _term_mailbox: mailbox_t,

    //  List of inproc endpoints within this context.
    // endpoints_t _endpoints;
    pub _endpoints: HashMap<String,ZmqEndpoint>,

    // List of inproc connection endpoints pending a bind
    // pending_connections_t _pending_connections;
    pub _pending_connections: HashMap<String, PendingConnection>,

    //  Synchronisation of access to the list of inproc endpoints.
    // mutex_t _endpoints_sync;
    pub _endpoints_sync: Mutex<u8>,

    //  Maximum socket ID.
    // static AtomicCounter max_socket_id;
    pub max_socket_id: AtomicCounter,

    //  Maximum number of sockets that can be opened at the same time.
    pub _max_sockets: i32,

    //  Maximum allowed message size
    pub _max_msgsz: i32,

    //  Number of I/O threads to launch.
    pub _io_thread_count: i32,

    //  Does context wait (possibly forever) on termination?
    // bool _blocky;
    pub _blocky: bool,

    //  Is IPv6 enabled on this context?
    // bool _ipv6;
    pub _ipv6: bool,

    // Should we use zero copy message decoding in this context?
    // bool _zero_copy;
    pub _zero_copy: bool,

    // ZMQ_NON_COPYABLE_NOR_MOVABLE (ctx_t)

// #ifdef HAVE_FORK
    // the process that created this context. Used to detect forking.
    // pid_t _pid;
    pub _pid: pid_t,
// #endif

// #ifdef ZMQ_HAVE_VMCI
//     _vmci_fd: i32;
pub _vmci_fd: i32,
    // _vmci_family: i32;
    pub _vmci_family: i32,
    // mutex_t _vmci_sync;
pub _vmci_sync: Mutex<u8>
// #endif
}

impl ZmqContext {
    //  Create the context object.
    // ctx_t ();

// ZmqContext::ZmqContext () :
//     _tag (ZMQ_CTX_TAG_VALUE_GOOD),
//     _starting (true),
//     _terminating (false),
//     _reaper (NULL),
//     _max_sockets (clipped_maxsocket (ZMQ_MAX_SOCKETS_DFLT)),
//     _max_msgsz (INT_MAX),
//     _io_thread_count (ZMQ_IO_THREADS_DFLT),
//     _blocky (true),
//     _ipv6 (false),
//     _zero_copy (true)
// {
// // #ifdef HAVE_FORK
//     _pid = getpid ();
// // #endif
// // #ifdef ZMQ_HAVE_VMCI
//     _vmci_fd = -1;
//     _vmci_family = -1;
// // #endif
//
//     //  Initialise crypto library, if needed.
//     random_open ();
//
// // #ifdef ZMQ_USE_NSS
//     NSS_NoDB_Init (NULL);
// // #endif
//
// // #ifdef ZMQ_USE_GNUTLS
//     gnutls_global_init ();
// // #endif
// }
pub fn new() -> Self {
        Self {
            _thread_ctx_t: Default::default(),
            _tag: ZMQ_CTX_TAG_VALUE_GOOD,
            _sockets: vec![],
            _empty_slots: vec![],
            _starting: true,
            _terminating: false,
            _slot_sync: Mutex::new(0),
            _reaper: None,
            _io_threads: vec![],
            _slots: vec![],
            _term_mailbox: mailbox_t,
            _endpoints: Default::default(),
            _pending_connections: Default::default(),
            _endpoints_sync: Mutex::new(0),
            max_socket_id: AtomicCounter::new(),
            _max_sockets: ZMQ_MAX_SOCKETS_DFLT,
            _max_msgsz: i32::MAX,
            _io_thread_count: ZMQ_IO_THREADS_DFLT,
            _blocky: true,
            _ipv6: false,
            _zero_copy: false,
            _pid: process:id(),
            _vmci_fd: -1,
            _vmci_family: -1,
            _vmci_sync: Mutex::new(0),
        }
    }

    // bool ZmqContext::check_tag () const
    pub fn check_tag(&self) -> bool {
        self._tag == ZMQ_CTX_TAG_VALUE_GOOD
    }

    // bool ZmqContext::valid () const
    pub fn valid(&self) -> bool {
        self._term_mailbox.valid()
    }


    // int ZmqContext::terminate ()
    pub fn terminate(&mut self) -> anyhow::Result<i32> {
        let _guard = self._slot_sync.lock()?;

        let save_terminating = self._terminating;
        self._terminating = false;

        // Connect up any pending inproc connections, otherwise we will hang
        let copy = self._pending_connections.clone();
        // for (pending_connections_t::iterator p = copy.begin (), end = copy.end ();
        //      p != end; ++p)
        for (key, val) in copy.iter() {
            let mut s = self.create_socket(ZMQ_PAIR);
            // create_socket might fail eg: out of memory/sockets limit reached
            zmq_assert(s);
            s.bind(val);
            s.close();
        }
        self._terminating = save_terminating;

        if !self._starting {
// #ifdef HAVE_FORK
            if self._pid != process::id() {
                // we are a forked child process. Close all file descriptors
                // inherited from the parent.
                // for (sockets_t::size_type i = 0, size = _sockets.size (); i != size;
                //      i++)
                for i in 0..self._sockets.len() {
                    self._sockets[i].get_mailbox().forked();
                }
                self._term_mailbox.forked();
            }
// #endif

            //  Check whether termination was already underway, but interrupted and now
            //  restarted.
            let restarted = self._terminating;
            self._terminating = true;

            //  First attempt to terminate the context.
            if !restarted {
                //  First send stop command to sockets so that any blocking calls
                //  can be interrupted. If there are no sockets we can ask reaper
                //  thread to stop.
                // for (sockets_t::size_type i = 0, size = _sockets.size (); i != size;
                //      i++)
                for i in 0..self._sockets.len() {
                    self._sockets[i].stop();
                }
                if self._sockets.empty() {
                    self._reaper.stop();
                }
            }
            self._slot_sync.unlock();

            //  Wait till reaper thread closes all the sockets.
            let cmd = ZmqCommand::default();
            let rc = self._term_mailbox.recv(&cmd, -1);
            if rc == -1 && errno == EINTR {
                return -1;
            }
            errno_assert(rc == 0);
            zmq_assert(cmd.type_ == ZmqCommand::done);
            let _lock = self._slot_sync.lock()?;
            zmq_assert(self._sockets.empty());
        }
        self._slot_sync.unlock();

// #ifdef ZMQ_HAVE_VMCI
        let _ = self._vmci_sync.lock().expect("TODO: panic message");

        VMCISock_ReleaseAFValueFd(self._vmci_fd);
        self._vmci_family = -1;
        self._vmci_fd = -1;

        self._vmci_sync.unlock();
// #endif

        //  Deallocate the resources.
        // delete this;

        return 0;
    }

    // int ZmqContext::shutdown ()
    pub fn shutdown(&mut self) -> i32 {
        // scoped_lock_t locker (_slot_sync);
        let locker = &self._slot_sync;

        if !self._terminating {
            self._terminating = true;

            if !self._starting {
                //  Send stop command to sockets so that any blocking calls
                //  can be interrupted. If there are no sockets we can ask reaper
                //  thread to stop.
                // for (sockets_t::size_type i = 0, size = _sockets.size (); i != size;
                //      i++)
                for i in 0..self._sockets.len() {
                    self._sockets[i].stop();
                }
                if self._sockets.empty() {
                    self._reaper.stop();
                }
            }
        }

        return 0;
    }

    pub fn set(&mut self, option: i32, opt_val: &mut [u8], opt_val_len: usize) -> i32 {
        let is_int = (opt_val_len == mem::size_of::<i32>());
        let mut value = 0i32;
        if is_int {
            let bytes: [u8; 4] = [0; 4];
            bytes.clone_from_slice(&opt_val);
            value = i32::from_le_bytes(bytes);
            // memcpy(&value, optval_, sizeof(int));
        }

        match option {
            ZMQ_MAX_SOCKETS => {
                if is_int && value >= 1 && value == clipped_maxsocket(value) {
                    // let locker = scoped_lock_t::new(self._opt_sync);
                    self._max_sockets = value;
                    return 0;
                }
            }

            ZMQ_IO_THREADS => {
                if is_int && value >= 0 {
                    // let locker = scoped_lock_t::new(self._opt_sync);
                    self._io_thread_count = value;
                    return 0;
                }
            }

            ZMQ_IPV6 => if is_int && value >= 0 {
                // let locker = scoped_lock_t::new(self._opt_sync);
                self._ipv6 = (value != 0);
                return 0;
            }

            ZMQ_BLOCKY => if is_int && value >= 0 {
                // scoped_lock_t locker (_opt_sync);
                self._blocky = (value != 0);
                return 0;
            }

            ZMQ_MAX_MSGSZ => if is_int && value >= 0 {
                // scoped_lock_t locker (_opt_sync);
                self._max_msgsz = if value < i32::MAX { value } else { i32::MAX };
                return 0;
            }

            ZMQ_ZERO_COPY_RECV => if is_int && value >= 0 {
                // scoped_lock_t locker (_opt_sync);
                self._zero_copy = (value != 0);
                return 0;
            }

            _ => {
                return ThreadCtx::set(option, opt_val, opt_val_len);
            }
        }

        errno = EINVAL;
        return -1;
    }

    // int ZmqContext::get (option_: i32, optval_: *mut c_void, const optvallen_: *mut usize)
    pub fn get(&mut self, option: i32, opt_val: &mut [u8], opt_val_len: &mut usize) -> i32 {
        // const bool is_int = (*optvallen_ == sizeof (int));
        let is_int = *opt_val_len == size_of::<i32>();
        // int *value = static_cast<int *> (optval_);
        // let mut value = 0i32;
        //     let mut bytes: [u8;4] = [0;4];
        //     bytes.clone_from_slice(optval_);
        //     value = i32::from_le_bytes(bytes);

        match option {
            ZMQ_MAX_SOCKETS => {
                if is_int {
                    // scoped_lock_t
                    // locker(_opt_sync);
                    // value = self._max_sockets;
                    opt_val.clone_from_slice(self._max_sockets.to_le_bytes().as_slice());
                    return 0;
                }
            }

            ZMQ_SOCKET_LIMIT => {
                if is_int {
                    // *value = clipped_maxsocket(65535);
                    let x = clipped_maxsocket(65535);
                    opt_val.clone_from_slice(x.to_le_bytes().as_slice());
                    return 0;
                }
            }

            ZMQ_IO_THREADS => if (is_int) {
                // scoped_lock_t locker (_opt_sync);
                // *value = _io_thread_count;
                opt_val.clone_from_slice(self._io_thread_count.to_le_bytes().as_slice());
                return 0;
            }

            ZMQ_IPV6 => if (is_int) {
                // scoped_lock_t locker (_opt_sync);
                // *value = _ipv6;
                opt_val.clone_from_slice(self._ipv6.to_le_bytes().as_slice());
                return 0;
            }

            ZMQ_BLOCKY => if (is_int) {
                // scoped_lock_t locker (_opt_sync);
                // *value = _blocky;
                opt_val.clone_from_slice(self._blocky.to_le_bytes().as_slice());
                return 0;
            }

            ZMQ_MAX_MSGSZ => if (is_int) {
                // scoped_lock_t locker (_opt_sync);
                // *value = _max_msgsz;
                opt_val.clone_from_slice(self._max_msgsz.to_le_bytes().as_slice());
                return 0;
            }

            ZMQ_ZmqMessage_SIZE => if (is_int) {
                // scoped_lock_t locker (_opt_sync);
                // *value = sizeof (zmq_ZmqMessage);
                let x = size_of::<zmq_ZmqMessage>();
                opt_val.clone_from_slice(x.to_le_bytes().as_slice());
                return 0;
            }

            ZMQ_ZERO_COPY_RECV => if (is_int) {
                // scoped_lock_t locker (_opt_sync);
                // *value = self._zero_copy;
                opt_val[0] = self._zero_copy.into();
                return 0;
            }

            _ => {
                return ThreadCtx::get(option, opt_val, optvallen_);
            }
        }

        errno = EINVAL;
        return -1;
    }

    // int ZmqContext::get (option_: i32)
    pub fn get2(&mut self, option_: i32) -> i32 {
        // int optval = 0;
        let mut optval: i32 = 0;
        // size_t optvallen = sizeof (int);
        let mut optvallen = size_of::<i32>();

        let mut opt_bytes: [u8; 4] = [0; 4];
        if self.get(option_, &mut opt_bytes, &mut optvallen) == 0 {
            optval = i32::from_le_bytes(opt_bytes);
            return optval;
        }

        errno = EINVAL;
        return -1;
    }


    pub fn start(&mut self) -> bool {
        //  Initialise the array of mailboxes. Additional two slots are for
        //  zmq_ctx_term thread and reaper thread.
        self._opt_sync.lock();
        let term_and_reaper_threads_count = 2usize;
        let mazmq = self._max_sockets;
        let ios = self._io_thread_count;
        self._opt_sync.unlock();
        let slot_count: usize = (mazmq + ios + term_and_reaper_threads_count) as usize;
        // try {
        //     _slots.reserve (slot_count);
        //     _empty_slots.reserve (slot_count - term_and_reaper_threads_count);
        // }
        // catch (const std::bad_alloc &) {
        //     errno = ENOMEM;
        //     return false;
        // }
        self._slots.reserve(slot_count);
        self._empty_slots.reserve(slot_count - term_and_reaper_threads_count);
        self._slots.resize(term_and_reaper_threads_count, null_mut());

        //  Initialise the infrastructure for zmq_ctx_term thread.
        self._slots[term_tid] = &self._term_mailbox;

        //  Create the reaper thread.
        self._reaper = reaper_t::new(self, reaper_tid);
        if self._reaper.is_none() {
            errno = ENOMEM;
            // goto fail_cleanup_slots;
        }
        if !self._reaper.get_mailbox().valid() {
//     goto
//     fail_cleanup_reaper;
        }
        self._slots[reaper_tid] = self._reaper.get_mailbox();
        self._reaper.start();

        //  Create I/O thread objects and launch them.
        self._slots.resize(slot_count, null_mut());

        // for (int i = term_and_reaper_threads_count;
        //      i != ios + term_and_reaper_threads_count; i++)
        for i in term_and_reaper_threads_count..ios + term_and_reaper_threads_count {
            let io_thread = io_thread_t::new(self, i);
            if !io_thread {
                errno = ENOMEM;
                // goto fail_cleanup_reaper;
            }
            if !io_thread.get_mailbox().valid() {
                // delete io_thread;
                // goto fail_cleanup_reaper;
            }
            self._io_threads.push_back(io_thread);
            self._slots[i] = io_thread.get_mailbox();
            io_thread.start();
        }

        //  In the unused part of the slot array, create a list of empty slots.
        // for (int32_t i = static_cast<int32_t> (_slots.size ()) - 1;
        //      i >= static_cast<int32_t> (ios) + term_and_reaper_threads_count; i--)
        for i in self._slots.len() - 1..ios + term_and_reaper_team_threads_count {
            self._empty_slots.push_back(i);
        }

        self._starting = false;
        return true;

        // TODO:
// fail_cleanup_reaper:
//     _reaper->stop ();
//     delete _reaper;
//     _reaper = NULL;

        // TODO:
// fail_cleanup_slots:
//     _slots.clear ();
//     return false;
        return false;
    }

    // ZmqSocketBase *ZmqContext::create_socket (type_: i32)
    pub fn create_socket(&mut self, type_: i32) -> Option<ZmqSocketBase> {
        // scoped_lock_t locker (_slot_sync);

        //  Once zmq_ctx_term() or zmq_ctx_shutdown() was called, we can't create
        //  new sockets.
        if self._terminating {
            errno = ETERM;
            return None;
        }

        if unlikely(self._starting) {
            if !self.start() {
                return None;
            }
        }

        //  If max_sockets limit was reached, return error.
        if self._empty_slots.empty() {
            errno = EMFILE;
            return None;
        }

        //  Choose a slot for the socket.
        // const uint32_t slot = _empty_slots.back ();
        let slot = self._empty_slots.last_mut().unwrap();
        self._empty_slots.pop_back();

        //  Generate new unique socket ID.
        // const int sid = (static_cast<int> (max_socket_id.add (1))) + 1;
        let sid = max_socket_id.add(1) + 1;

        //  Create the socket and register its mailbox.
        let s = ZmqSocketBase::create(type_, self, slot, sid);
        if (!s) {
            self._empty_slots.push_back(slot);
            return None;
        }
        self._sockets.push_back(s);
        self._slots[slot] = s.get_mailbox();

        return Some(s);
    }

    // void ZmqContext::destroy_socket (class ZmqSocketBase *socket_)
    pub fn destroy_socket(&mut self, socket_: &mut ZmqSocketBase) {
        // scoped_lock_t locker (_slot_sync);

        //  Free the associated thread slot.
        let tid = socket_.get_tid();
        self._empty_slots.push_back(tid);
        self._slots[tid] = null_mut();

        //  Remove the socket from the list of sockets.
        self._sockets.erase(socket_);

        //  If zmq_ctx_term() was already called and there are no more socket
        //  we can ask reaper thread to terminate.
        if self._terminating && self._sockets.empty() {
            self._reaper.stop();
        }
    }

    // object_t *ZmqContext::get_reaper () const
    pub fn get_reaper(&mut self) -> Option<reaper_t> {
        return self._reaper.clone();
    }

    // void ZmqContext::send_command (uint32_t tid_, const ZmqCommand &command_)
    pub fn send_command(&mut self, tid_: u32, command_: &mut ZmqCommand) {
        self._slots[tid_].send(command_);
    }

    // io_thread_t *ZmqContext::choose_io_thread (u64 affinity_)
    pub fn choose_io_thread(&mut self, affinity_: u64) -> Option<io_thread_t> {
        if self._io_threads.empty() {
            return None;
        }

        //  Find the I/O thread with minimum load.
        let mut min_load = -1;
        // io_thread_t *selected_io_thread = NULL;
        let mut selected_io_thread: Option<io_thread_t> = None;
        // for (io_threads_t::size_type i = 0, size = _io_threads.size (); i != size;
        //      i++)
        for i in 0..self._io_threads.len() {
            if !affinity_ || (affinity_ & (1 << i)) {
                let load = self._io_threads[i].get_load();
                if selected_io_thread.is_none() || load < min_load {
                    min_load = load;
                    selected_io_thread = Some(self._io_threads[i].clone());
                }
            }
        }
        return selected_io_thread;
    }

    // int ZmqContext::register_endpoint (addr_: *const c_char,
    //                                    const ZmqEndpoint &endpoint_)
    pub fn register_endpoint(&mut self, addr_: &str, endpoint: &mut ZmqEndpoint) -> i32 {
        // scoped_lock_t locker (_endpoints_sync);

        let inserted = self._endpoints.ZMQ_MAP_INSERT_OR_EMPLACE(addr_, endpoint_).second;
        if !inserted {
            errno = EADDRINUSE;
            return -1;
        }
        return 0;
    }

// int ZmqContext::unregister_endpoint (const std::string &addr_,
//                                      const ZmqSocketBase *const socket_)
    pub fn unregister_endpoint(&mut self, addr_: &mut str, socket_: &mut ZmqSocketBase) -> i32 {
        // scoped_lock_t locker (_endpoints_sync);

        // const endpoints_t::iterator it = _endpoints.find (addr_);
        // if (it == _endpoints.end () || it->second.socket != socket_) {
        //     errno = ENOENT;
        //     return -1;
        // }

        let item = self._endpoints.get(addr_);
        if item.is_none() {
            errno = ENOENT;
            return -1;
        }

        if item.unwrap().socket != socket_ {
            errno = ENOENT;
            return -1;
        }

        match self._endpoints.remove(addr_) {
            Some(_) => 0,
            None => {
                errno = ENOENT;
                -1
            }
        }

        //  Remove endpoint.
        // _endpoints.erase (it);

        // return 0;
    }

    // void ZmqContext::unregister_endpoints (const ZmqSocketBase *const socket_)
    pub fn unregister_endpoints(&mut self, socket_: &mut ZmqSocketBase)
    {
        // scoped_lock_t locker (_endpoints_sync);

        // for (endpoints_t::iterator it = _endpoints.begin (),
        //                            end = _endpoints.end ();
        //      it != end;)

        let mut erase_list: Vec<String> = vec![];

        for (k, v) in self._endpoints.iter_mut()
        {
    //         if (it->second.socket == socket_)
    // #if __cplusplus >= 201103L || (defined _MSC_VER && _MSC_VER >= 1700)
    //             it = _endpoints.erase (it);
    // // #else
    //             _endpoints.erase (it++);
    // // #endif
    //         else
    //             ++it;
            if v.socket == socket_ {
                erase_list.push(k.clone())
            }
        }

        for element in erase_list.iter() {
            self._endpoints.remove(element);
        }
    }

// ZmqEndpoint ZmqContext::find_endpoint (addr_: *const c_char)
    pub fn find_endpoint(&mut self, addr_: &str) -> Option<ZmqEndpoint> {
        // scoped_lock_t locker (_endpoints_sync);

        // endpoints_t::iterator it = _endpoints.find (addr_);
        let endpoint = self._endpoints.get_mut(addr_);

        if endpoint.is_none() {
            errno = ECONNREFUSED;
            // ZmqEndpoint empty = {NULL, ZmqOptions ()};
            return None;
        }
        // ZmqEndpoint endpoint = it->second;

        //  Increment the command sequence number of the peer so that it won't
        //  get deallocated until "bind" command is issued by the caller.
        //  The subsequent 'bind' has to be called with inc_seqnum parameter
        //  set to false, so that the seqnum isn't incremented twice.
        endpoint.unwrap().socket.inc_seqnum();
        let x = endpoint.unwrap().clone();
        return Some(x);
    }

    // void ZmqContext::pend_connection (const std::string &addr_,
    //                                   const ZmqEndpoint &endpoint_,
    //                                   pipe_t **pipes_)
    pub fn pend_connection(&mut self, addr_: &mut str, endpoint_: &mut ZmqEndpoint, pipes_: &mut [pipe_t])
    {
        // scoped_lock_t locker (_endpoints_sync);

        // const PendingConnection pending_connection = {endpoint_, pipes_[0],
        //                                                  pipes_[1]};
        let pending_connection = PendingConnection {
            endpoint: endpoint_.clone(),
            connect_pipe: pipes_[0].clone(),
            bind_pipe: pipes_[1].clone(),
        };

        // const endpoints_t::iterator it = _endpoints.find (addr_);
        let it = self._endpoints.get(addr_);
        // if (it == _endpoints.end ())
        if it.is_none()
        {
            //  Still no bind.
            endpoint_.socket.inc_seqnum ();
            self._pending_connections.ZMQ_MAP_INSERT_OR_EMPLACE (addr_,
                                                            pending_connection);
        } else {
            //  Bind has happened in the mean time, connect directly
            self.connect_inproc_sockets(it.unwrap().socket, it.unwrap().options.clone(),
                                    pending_connection, connect_side);
        }
    }

    // void ZmqContext::connect_pending (addr_: *const c_char,
    //                                   ZmqSocketBase *bind_socket_)
    pub fn connect_pending(&mut self, addr_: &str, bind_socket_: &mut ZmqSocketBase)
    {
        // scoped_lock_t locker (_endpoints_sync);

        // const std::pair<pending_connections_t::iterator,
        //                 pending_connections_t::iterator>
        //
        //   let pending = self._pending_connections.equal_range (addr_);
        // for (pending_connections_t::iterator p = pending.first; p != pending.second;
        //      ++p)
        //     connect_inproc_sockets (bind_socket_, _endpoints[addr_].options,
        //                             p->second, bind_side);
        let pending = self._pending_connections.get(addr_);
        if pending.is_some() {
            self.connect_inproc_sockets(bind_socket_,
                                        &mut self._endpoints[addr_].options.clone(),
                                        &mut pending.unwrap(),
                                        bind_side)
        }

        self._pending_connections.remove(addr_);
    }

    // void ZmqContext::connect_inproc_sockets (
    //   bind_socket_: *mut ZmqSocketBase,
    //   const ZmqOptions &bind_options_,
    //   const PendingConnection &pending_connection_,
    //   side side_)
    pub fn connect_inproc_sockets(
        &mut self,
        bind_socket_: &mut ZmqSocketBase,
        bind_options_: &mut ZmqOptions,
        pending_connection_: &mut PendingConnection,
        side_: side)
    {
        bind_socket.inc_seqnum ();
        pending_connection_.bind_pipe.set_tid (bind_socket_.get_tid ());

        if (!bind_options_.recv_routing_id) {
            // ZmqMessage msg;
            let msg = ZmqMessage{
                data: (),
                size: 0,
                hint: ()
            };
            let ok = pending_connection_.bind_pipe.read (&msg);
            zmq_assert (ok);
            let rc = msg.close ();
            errno_assert (rc == 0);
        }

        if !get_effective_conflate_option (pending_connection_.endpoint.options) {
            pending_connection_.connect_pipe->set_hwms_boost (bind_options_.sndhwm,
                                                              bind_options_.rcvhwm);
            pending_connection_.bind_pipe->set_hwms_boost (
              pending_connection_.endpoint.options.sndhwm,
              pending_connection_.endpoint.options.rcvhwm);

            pending_connection_.connect_pipe->set_hwms (
              pending_connection_.endpoint.options.rcvhwm,
              pending_connection_.endpoint.options.sndhwm);
            pending_connection_.bind_pipe->set_hwms (bind_options_.rcvhwm,
                                                     bind_options_.sndhwm);
        } else {
            pending_connection_.connect_pipe->set_hwms (-1, -1);
            pending_connection_.bind_pipe->set_hwms (-1, -1);
        }

    // #ifdef ZMQ_BUILD_DRAFT_API
        if (bind_options_.can_recv_disconnect_msg
            && !bind_options_.disconnect_msg.empty ())
            pending_connection_.connect_pipe->set_disconnect_msg (
              bind_options_.disconnect_msg);
    // #endif

        if (side_ == bind_side) {
            ZmqCommand cmd;
            cmd.type = ZmqCommand::bind;
            cmd.args.bind.pipe = pending_connection_.bind_pipe;
            bind_socket_->process_command (cmd);
            bind_socket_->send_inproc_connected (
              pending_connection_.endpoint.socket);
        } else
            pending_connection_.connect_pipe->send_bind (
              bind_socket_, pending_connection_.bind_pipe, false);

        // When a ctx is terminated all pending inproc connection will be
        // connected, but the socket will already be closed and the pipe will be
        // in waiting_for_delimiter state, which means no more writes can be done
        // and the routing id write fails and causes an assert. Check if the socket
        // is open before sending.
        if (pending_connection_.endpoint.options.recv_routing_id
            && pending_connection_.endpoint.socket->check_tag ()) {
            send_routing_id (pending_connection_.bind_pipe, bind_options_);
        }

    // #ifdef ZMQ_BUILD_DRAFT_API
        //  If set, send the hello msg of the bind socket to the pending connection.
        if (bind_options_.can_send_hello_msg
            && bind_options_.hello_msg.size () > 0) {
            send_hello_msg (pending_connection_.bind_pipe, bind_options_);
        }
    // #endif
    }

// #ifdef ZMQ_HAVE_VMCI

    int ZmqContext::get_vmci_socket_family ()
    {
        scoped_lock_t locker (_vmci_sync);

        if (_vmci_fd == -1) {
            _vmci_family = VMCISock_GetAFValueFd (&_vmci_fd);

            if (_vmci_fd != -1) {
    // #ifdef FD_CLOEXEC
                int rc = fcntl (_vmci_fd, F_SETFD, FD_CLOEXEC);
                errno_assert (rc != -1);
    // #endif
            }
        }

        return _vmci_family;
    }

    // #endif

    //  The last used socket ID, or 0 if no socket was used so far. Note that this
    //  is a global variable. Thus, even sockets created in different contexts have
    //  unique IDs.
    AtomicCounter ZmqContext::max_socket_id;


//     ZmqContext::~ZmqContext ()
// {
//     //  Check that there are no remaining _sockets.
//     zmq_assert (_sockets.empty ());
//
//     //  Ask I/O threads to terminate. If stop signal wasn't sent to I/O
//     //  thread subsequent invocation of destructor would hang-up.
//     const io_threads_t::size_type io_threads_size = _io_threads.size ();
//     for (io_threads_t::size_type i = 0; i != io_threads_size; i++) {
//         _io_threads[i]->stop ();
//     }
//
//     //  Wait till I/O threads actually terminate.
//     for (io_threads_t::size_type i = 0; i != io_threads_size; i++) {
//         LIBZMQ_DELETE (_io_threads[i]);
//     }
//
//     //  Deallocate the reaper thread object.
//     LIBZMQ_DELETE (_reaper);
//
//     //  The mailboxes in _slots themselves were deallocated with their
//     //  corresponding io_thread/socket objects.
//
//     //  De-initialise crypto library, if needed.
//     random_close ();
//
// // #ifdef ZMQ_USE_NSS
//     NSS_Shutdown ();
// // #endif
//
// // #ifdef ZMQ_USE_GNUTLS
//     gnutls_global_deinit ();
// // #endif
//
//     //  Remove the tag, so that the object is considered dead.
//     _tag = ZMQ_CTX_TAG_VALUE_BAD;
// }


    //  Returns false if object is not a context.
    // bool check_tag () const;

    //  This function is called when user invokes zmq_ctx_term. If there are
    //  no more sockets open it'll cause all the infrastructure to be shut
    //  down. If there are open sockets still, the deallocation happens
    //  after the last one is closed.
    // int terminate ();

    // This function starts the terminate process by unblocking any blocking
    // operations currently in progress and stopping any more socket activity
    // (except zmq_close).
    // This function is non-blocking.
    // terminate must still be called afterwards.
    // This function is optional, terminate will unblock any current
    // operations as well.
    // int shutdown ();

    //  Set and get context properties.
    // int set (option_: i32, const optval_: *mut c_void, optvallen_: usize);
    // int get (option_: i32, optval_: *mut c_void, const optvallen_: *mut usize);
    // int get (option_: i32);

    //  Create and destroy a socket.
    // ZmqSocketBase *create_socket (type_: i32);
    // void destroy_socket (ZmqSocketBase *socket_);

    //  Send command to the destination thread.
    // void send_command (uint32_t tid_, const command_t &command_);

    //  Returns the I/O thread that is the least busy at the moment.
    //  Affinity specifies which I/O threads are eligible (0 = all).
    //  Returns NULL if no I/O thread is available.
    // io_thread_t *choose_io_thread (uint64_t affinity_);

    //  Returns reaper thread object.
    // object_t *get_reaper () const;

    //  Management of inproc endpoints.
    // int register_endpoint (addr_: *const c_char, const endpoint_t &endpoint_);
    // int unregister_endpoint (const std::string &addr_,
    //                          const ZmqSocketBase *socket_);
    // void unregister_endpoints (const ZmqSocketBase *socket_);
    // endpoint_t find_endpoint (addr_: *const c_char);
    // void pend_connection (const std::string &addr_,
    //                       const endpoint_t &endpoint_,
    //                       pipe_t **pipes_);
    // void connect_pending (addr_: *const c_char, ZmqSocketBase *bind_socket_);

// #ifdef ZMQ_HAVE_VMCI
    // Return family for the VMCI socket or -1 if it's not available.
    // int get_vmci_socket_family ();
// #endif

    // ~ctx_t ();

    // bool valid () const;

    // bool start ();

    // static void
    //     connect_inproc_sockets (bind_socket_: *mut ZmqSocketBase,
    //                             const ZmqOptions &bind_options_,
    //                             const PendingConnection &pending_connection_,
    //                             side side_);
} // impl ZmqContext

pub fn clipped_maxsocket (mut max_requested: i32) -> i32
{
    if max_requested >= max_fds()
        && max_fds() != -1 {
        // -1 because we need room for the reaper mailbox.
        max_requested = max_fds() - 1;
    }

    return max_requested;
}
