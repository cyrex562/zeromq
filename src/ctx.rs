/*
    Copyright (c) 2007-2017 Contributors as noted in the AUTHORS file

    This file is part of libzmq, the ZeroMQ core engine in C++.

    libzmq is free software; you can redistribute it and/or modify it under
    the terms of the GNU Lesser General Public License (LGPL) as published
    by the Free Software Foundation; either version 3 of the License, or
    (at your option) any later version.

    As a special exception, the Contributors give you permission to link
    this library with independent modules to produce an executable,
    regardless of the license terms of these independent modules, and to
    copy and distribute the resulting executable under terms of your choice,
    provided that you also meet, for each linked independent module, the
    terms and conditions of the license of that module. An independent
    module is a module which is not derived from or based on this library.
    If you modify this library, you must extend this exception to your
    version of the library.

    libzmq is distributed in the hope that it will be useful, but WITHOUT
    ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
    FITNESS FOR A PARTICULAR PURPOSE. See the GNU Lesser General Public
    License for more details.

    You should have received a copy of the GNU Lesser General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

// #include "precompiled.hpp"
// #include "macros.hpp"
// #ifndef ZMQ_HAVE_WINDOWS
// #include <unistd.h>
// #endif

// #include <limits>
// #include <climits>
// #include <new>
// #include <sstream>
// #include <string.h>

// #include "ctx.hpp"
// #include "socket_base.hpp"
// #include "io_thread.hpp"
// #include "reaper.hpp"
// #include "pipe.hpp"
// #include "err.hpp"
// #include "msg.hpp"
// #include "random.hpp"

// #ifdef ZMQ_HAVE_VMCI
// #include <vmci_sockets.h>
// #endif

// #ifdef ZMQ_USE_NSS
// #include <nss.h>
// #endif

// #ifdef ZMQ_USE_GNUTLS
// #include <gnutls/gnutls.h>
// #endif

// #define ZMQ_CTX_TAG_VALUE_GOOD 0xabadcafe
// #define ZMQ_CTX_TAG_VALUE_BAD 0xdeadbeef


use std::collections::HashSet;
use crate::endpoint::ZmqEndpoint;
use crate::socket_base::socket_base_t;

pub struct thread_ctx_t {
    // protected:
    //  Synchronisation of access to context options.
    // mutex_t _opt_sync;
    pub _opt_sync: mutex_t,

    // private:
    //  Thread parameters.
    pub _thread_priority: i32,
    pub _thread_sched_policy: i32,
    // std::set<int> _thread_affinity_cpus;
    pub _thread_affinity_cpus: HashSet<i32>,
    // std::string _thread_name_prefix;
    pub _thread_name_prefix: String,
}

impl thread_ctx_t {
    // thread_ctx_t ();

    //  Start a new thread with proper scheduling parameters.
    // void start_thread (thread_t &thread_,
    //                    thread_fn *tfn_,
    //                    arg_: *mut c_void,
    //                    const char *name_ = NULL) const;

    // int set (option_: i32, const optval_: *mut c_void, optvallen_: usize);
    // int get (option_: i32, optval_: *mut c_void, const optvallen_: *mut usize);

}

//  Context object encapsulates all the global state associated with
//  the library.

struct pending_connection_t
    {
        ZmqEndpoint endpoint;
        pipe_t *connect_pipe;
        pipe_t *bind_pipe;
    };

enum
    {
        term_tid = 0,
        reaper_tid = 1
    };

enum side
    {
        connect_side,
        bind_side
    };

// typedef array_t<socket_base_t> sockets_t;
pub type sockets_t = Vec<socket_base_t>;

// typedef std::vector<uint32_t> empty_slots_t;
pub type empty_slots_s = Vec<u32>;

// typedef std::vector<zmq::io_thread_t *> io_threads_t;
pub type io_threads_t = Vec<io_thread_t>;

// typedef std::map<std::string, endpoint_t> endpoints_t;
pub type endpoints_t = HashMap<String,ZmqEndpoint>;

// typedef std::multimap<std::string, pending_connection_t>
//       pending_connections_t;
pub type pending_connections_t = HashMap<String, pending_connection_t>

// class ctx_t ZMQ_FINAL : public thread_ctx_t
pub struct ZmqContext
{
    pub _thread_ctx_t: thread_ctx_t,
// public:
  // private:

    //  Used to check whether the object is a context.
    // uint32_t _tag;
pub _tag: u32,
    //  Sockets belonging to this context. We need the list so that
    //  we can notify the sockets when zmq_ctx_term() is called.
    //  The sockets will return ETERM then.
    pub _sockets: sockets_t,
    // sockets_t _sockets;
    //  List of unused thread slots.

    // empty_slots_t _empty_slots;
pub _empty_slots: empty_slots_t;

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
pub _slot_sync: mutex_t,

    //  The reaper thread.
    // zmq::reaper_t *_reaper;
pub _reaper: *mut reaper_t,

    //  I/O threads.

    // io_threads_t _io_threads;
    pub _io_threads: io_threads_t,

    //  Array of pointers to mailboxes for both application and I/O threads.
    // std::vector<i_mailbox *> _slots;
    pub _slots: Vec<*mut i_mailbox>,

    //  Mailbox for zmq_ctx_term thread.
    // mailbox_t _term_mailbox;
    pub _term_mailbox: mailbox_t,

    //  List of inproc endpoints within this context.
    // endpoints_t _endpoints;
    pub _endpoints: endpoints_t,

    // List of inproc connection endpoints pending a bind
    // pending_connections_t _pending_connections;
    pub _pending_connections: pending_connections_t,

    //  Synchronisation of access to the list of inproc endpoints.
    // mutex_t _endpoints_sync;
    pub _endpoints_sync: mutex_t,

    //  Maximum socket ID.
    // static atomic_counter_t max_socket_id;
    pub max_socket_id: atomic_counter_t,

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
pub _vmci_sync: mutex_t
// #endif
}

impl ZmqContext {
    //  Create the context object.
    // ctx_t ();

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
    // zmq::socket_base_t *create_socket (type_: i32);
    // void destroy_socket (zmq::socket_base_t *socket_);

    //  Send command to the destination thread.
    // void send_command (uint32_t tid_, const command_t &command_);

    //  Returns the I/O thread that is the least busy at the moment.
    //  Affinity specifies which I/O threads are eligible (0 = all).
    //  Returns NULL if no I/O thread is available.
    // zmq::io_thread_t *choose_io_thread (uint64_t affinity_);

    //  Returns reaper thread object.
    // zmq::object_t *get_reaper () const;

    //  Management of inproc endpoints.
    // int register_endpoint (addr_: *const c_char, const endpoint_t &endpoint_);
    // int unregister_endpoint (const std::string &addr_,
    //                          const socket_base_t *socket_);
    // void unregister_endpoints (const zmq::socket_base_t *socket_);
    // endpoint_t find_endpoint (addr_: *const c_char);
    // void pend_connection (const std::string &addr_,
    //                       const endpoint_t &endpoint_,
    //                       pipe_t **pipes_);
    // void connect_pending (addr_: *const c_char, zmq::socket_base_t *bind_socket_);

// #ifdef ZMQ_HAVE_VMCI
    // Return family for the VMCI socket or -1 if it's not available.
    // int get_vmci_socket_family ();
// #endif

    // ~ctx_t ();

    // bool valid () const;

    // bool start ();

    // static void
    //     connect_inproc_sockets (bind_socket_: *mut socket_base_t,
    //                             const options_t &bind_options_,
    //                             const pending_connection_t &pending_connection_,
    //                             side side_);
}

static int clipped_maxsocket (max_requested_: i32)
{
    if (max_requested_ >= zmq::poller_t::max_fds ()
        && zmq::poller_t::max_fds () != -1)
        // -1 because we need room for the reaper mailbox.
        max_requested_ = zmq::poller_t::max_fds () - 1;

    return max_requested_;
}

zmq::ZmqContext::ZmqContext () :
    _tag (ZMQ_CTX_TAG_VALUE_GOOD),
    _starting (true),
    _terminating (false),
    _reaper (NULL),
    _max_sockets (clipped_maxsocket (ZMQ_MAX_SOCKETS_DFLT)),
    _max_msgsz (INT_MAX),
    _io_thread_count (ZMQ_IO_THREADS_DFLT),
    _blocky (true),
    _ipv6 (false),
    _zero_copy (true)
{
// #ifdef HAVE_FORK
    _pid = getpid ();
// #endif
// #ifdef ZMQ_HAVE_VMCI
    _vmci_fd = -1;
    _vmci_family = -1;
// #endif

    //  Initialise crypto library, if needed.
    zmq::random_open ();

// #ifdef ZMQ_USE_NSS
    NSS_NoDB_Init (NULL);
// #endif

// #ifdef ZMQ_USE_GNUTLS
    gnutls_global_init ();
// #endif
}

bool zmq::ZmqContext::check_tag () const
{
    return _tag == ZMQ_CTX_TAG_VALUE_GOOD;
}

zmq::ZmqContext::~ZmqContext ()
{
    //  Check that there are no remaining _sockets.
    zmq_assert (_sockets.empty ());

    //  Ask I/O threads to terminate. If stop signal wasn't sent to I/O
    //  thread subsequent invocation of destructor would hang-up.
    const io_threads_t::size_type io_threads_size = _io_threads.size ();
    for (io_threads_t::size_type i = 0; i != io_threads_size; i++) {
        _io_threads[i]->stop ();
    }

    //  Wait till I/O threads actually terminate.
    for (io_threads_t::size_type i = 0; i != io_threads_size; i++) {
        LIBZMQ_DELETE (_io_threads[i]);
    }

    //  Deallocate the reaper thread object.
    LIBZMQ_DELETE (_reaper);

    //  The mailboxes in _slots themselves were deallocated with their
    //  corresponding io_thread/socket objects.

    //  De-initialise crypto library, if needed.
    zmq::random_close ();

// #ifdef ZMQ_USE_NSS
    NSS_Shutdown ();
// #endif

// #ifdef ZMQ_USE_GNUTLS
    gnutls_global_deinit ();
// #endif

    //  Remove the tag, so that the object is considered dead.
    _tag = ZMQ_CTX_TAG_VALUE_BAD;
}

bool zmq::ZmqContext::valid () const
{
    return _term_mailbox.valid ();
}

int zmq::ZmqContext::terminate ()
{
    _slot_sync.lock ();

    const bool save_terminating = _terminating;
    _terminating = false;

    // Connect up any pending inproc connections, otherwise we will hang
    pending_connections_t copy = _pending_connections;
    for (pending_connections_t::iterator p = copy.begin (), end = copy.end ();
         p != end; ++p) {
        let mut s: *mut socket_base_t =  create_socket (ZMQ_PAIR);
        // create_socket might fail eg: out of memory/sockets limit reached
        zmq_assert (s);
        s->bind (p->first.c_str ());
        s->close ();
    }
    _terminating = save_terminating;

    if (!_starting) {
// #ifdef HAVE_FORK
        if (_pid != getpid ()) {
            // we are a forked child process. Close all file descriptors
            // inherited from the parent.
            for (sockets_t::size_type i = 0, size = _sockets.size (); i != size;
                 i++) {
                _sockets[i]->get_mailbox ()->forked ();
            }
            _term_mailbox.forked ();
        }
// #endif

        //  Check whether termination was already underway, but interrupted and now
        //  restarted.
        const bool restarted = _terminating;
        _terminating = true;

        //  First attempt to terminate the context.
        if (!restarted) {
            //  First send stop command to sockets so that any blocking calls
            //  can be interrupted. If there are no sockets we can ask reaper
            //  thread to stop.
            for (sockets_t::size_type i = 0, size = _sockets.size (); i != size;
                 i++) {
                _sockets[i]->stop ();
            }
            if (_sockets.empty ())
                _reaper->stop ();
        }
        _slot_sync.unlock ();

        //  Wait till reaper thread closes all the sockets.
        ZmqCommand cmd;
        const int rc = _term_mailbox.recv (&cmd, -1);
        if (rc == -1 && errno == EINTR)
            return -1;
        errno_assert (rc == 0);
        zmq_assert (cmd.type == ZmqCommand::done);
        _slot_sync.lock ();
        zmq_assert (_sockets.empty ());
    }
    _slot_sync.unlock ();

// #ifdef ZMQ_HAVE_VMCI
    _vmci_sync.lock ();

    VMCISock_ReleaseAFValueFd (_vmci_fd);
    _vmci_family = -1;
    _vmci_fd = -1;

    _vmci_sync.unlock ();
// #endif

    //  Deallocate the resources.
    delete this;

    return 0;
}

int zmq::ZmqContext::shutdown ()
{
    scoped_lock_t locker (_slot_sync);

    if (!_terminating) {
        _terminating = true;

        if (!_starting) {
            //  Send stop command to sockets so that any blocking calls
            //  can be interrupted. If there are no sockets we can ask reaper
            //  thread to stop.
            for (sockets_t::size_type i = 0, size = _sockets.size (); i != size;
                 i++) {
                _sockets[i]->stop ();
            }
            if (_sockets.empty ())
                _reaper->stop ();
        }
    }

    return 0;
}

int zmq::ZmqContext::set (option_: i32, const optval_: *mut c_void, optvallen_: usize)
{
    const bool is_int = (optvallen_ == sizeof (int));
    int value = 0;
    if (is_int)
        memcpy (&value, optval_, sizeof (int));

    switch (option_) {
        case ZMQ_MAX_SOCKETS:
            if (is_int && value >= 1 && value == clipped_maxsocket (value)) {
                scoped_lock_t locker (_opt_sync);
                _max_sockets = value;
                return 0;
            }
            break;

        case ZMQ_IO_THREADS:
            if (is_int && value >= 0) {
                scoped_lock_t locker (_opt_sync);
                _io_thread_count = value;
                return 0;
            }
            break;

        case ZMQ_IPV6:
            if (is_int && value >= 0) {
                scoped_lock_t locker (_opt_sync);
                _ipv6 = (value != 0);
                return 0;
            }
            break;

        case ZMQ_BLOCKY:
            if (is_int && value >= 0) {
                scoped_lock_t locker (_opt_sync);
                _blocky = (value != 0);
                return 0;
            }
            break;

        case ZMQ_MAX_MSGSZ:
            if (is_int && value >= 0) {
                scoped_lock_t locker (_opt_sync);
                _max_msgsz = value < INT_MAX ? value : INT_MAX;
                return 0;
            }
            break;

        case ZMQ_ZERO_COPY_RECV:
            if (is_int && value >= 0) {
                scoped_lock_t locker (_opt_sync);
                _zero_copy = (value != 0);
                return 0;
            }
            break;

        default: {
            return thread_ctx_t::set (option_, optval_, optvallen_);
        }
    }

    errno = EINVAL;
    return -1;
}

int zmq::ZmqContext::get (option_: i32, optval_: *mut c_void, const optvallen_: *mut usize)
{
    const bool is_int = (*optvallen_ == sizeof (int));
    int *value = static_cast<int *> (optval_);

    switch (option_) {
        case ZMQ_MAX_SOCKETS:
            if (is_int) {
                scoped_lock_t locker (_opt_sync);
                *value = _max_sockets;
                return 0;
            }
            break;

        case ZMQ_SOCKET_LIMIT:
            if (is_int) {
                *value = clipped_maxsocket (65535);
                return 0;
            }
            break;

        case ZMQ_IO_THREADS:
            if (is_int) {
                scoped_lock_t locker (_opt_sync);
                *value = _io_thread_count;
                return 0;
            }
            break;

        case ZMQ_IPV6:
            if (is_int) {
                scoped_lock_t locker (_opt_sync);
                *value = _ipv6;
                return 0;
            }
            break;

        case ZMQ_BLOCKY:
            if (is_int) {
                scoped_lock_t locker (_opt_sync);
                *value = _blocky;
                return 0;
            }
            break;

        case ZMQ_MAX_MSGSZ:
            if (is_int) {
                scoped_lock_t locker (_opt_sync);
                *value = _max_msgsz;
                return 0;
            }
            break;

        case ZMQ_MSG_T_SIZE:
            if (is_int) {
                scoped_lock_t locker (_opt_sync);
                *value = sizeof (zmq_msg_t);
                return 0;
            }
            break;

        case ZMQ_ZERO_COPY_RECV:
            if (is_int) {
                scoped_lock_t locker (_opt_sync);
                *value = _zero_copy;
                return 0;
            }
            break;

        default: {
            return thread_ctx_t::get (option_, optval_, optvallen_);
        }
    }

    errno = EINVAL;
    return -1;
}

int zmq::ZmqContext::get (option_: i32)
{
    int optval = 0;
    size_t optvallen = sizeof (int);

    if (get (option_, &optval, &optvallen) == 0)
        return optval;

    errno = EINVAL;
    return -1;
}

bool zmq::ZmqContext::start ()
{
    //  Initialise the array of mailboxes. Additional two slots are for
    //  zmq_ctx_term thread and reaper thread.
    _opt_sync.lock ();
    const int term_and_reaper_threads_count = 2;
    const int mazmq = _max_sockets;
    const int ios = _io_thread_count;
    _opt_sync.unlock ();
    const int slot_count = mazmq + ios + term_and_reaper_threads_count;
    try {
        _slots.reserve (slot_count);
        _empty_slots.reserve (slot_count - term_and_reaper_threads_count);
    }
    catch (const std::bad_alloc &) {
        errno = ENOMEM;
        return false;
    }
    _slots.resize (term_and_reaper_threads_count);

    //  Initialise the infrastructure for zmq_ctx_term thread.
    _slots[term_tid] = &_term_mailbox;

    //  Create the reaper thread.
    _reaper = new (std::nothrow) reaper_t (this, reaper_tid);
    if (!_reaper) {
        errno = ENOMEM;
        goto fail_cleanup_slots;
    }
    if (!_reaper->get_mailbox ()->valid ())
        goto fail_cleanup_reaper;
    _slots[reaper_tid] = _reaper->get_mailbox ();
    _reaper->start ();

    //  Create I/O thread objects and launch them.
    _slots.resize (slot_count, NULL);

    for (int i = term_and_reaper_threads_count;
         i != ios + term_and_reaper_threads_count; i++) {
        io_thread_t *io_thread = new (std::nothrow) io_thread_t (this, i);
        if (!io_thread) {
            errno = ENOMEM;
            goto fail_cleanup_reaper;
        }
        if (!io_thread->get_mailbox ()->valid ()) {
            delete io_thread;
            goto fail_cleanup_reaper;
        }
        _io_threads.push_back (io_thread);
        _slots[i] = io_thread->get_mailbox ();
        io_thread->start ();
    }

    //  In the unused part of the slot array, create a list of empty slots.
    for (int32_t i = static_cast<int32_t> (_slots.size ()) - 1;
         i >= static_cast<int32_t> (ios) + term_and_reaper_threads_count; i--) {
        _empty_slots.push_back (i);
    }

    _starting = false;
    return true;

fail_cleanup_reaper:
    _reaper->stop ();
    delete _reaper;
    _reaper = NULL;

fail_cleanup_slots:
    _slots.clear ();
    return false;
}

zmq::socket_base_t *zmq::ZmqContext::create_socket (type_: i32)
{
    scoped_lock_t locker (_slot_sync);

    //  Once zmq_ctx_term() or zmq_ctx_shutdown() was called, we can't create
    //  new sockets.
    if (_terminating) {
        errno = ETERM;
        return NULL;
    }

    if (unlikely (_starting)) {
        if (!start ())
            return NULL;
    }

    //  If max_sockets limit was reached, return error.
    if (_empty_slots.empty ()) {
        errno = EMFILE;
        return NULL;
    }

    //  Choose a slot for the socket.
    const uint32_t slot = _empty_slots.back ();
    _empty_slots.pop_back ();

    //  Generate new unique socket ID.
    const int sid = (static_cast<int> (max_socket_id.add (1))) + 1;

    //  Create the socket and register its mailbox.
    socket_base_t *s = socket_base_t::create (type_, this, slot, sid);
    if (!s) {
        _empty_slots.push_back (slot);
        return NULL;
    }
    _sockets.push_back (s);
    _slots[slot] = s->get_mailbox ();

    return s;
}

void zmq::ZmqContext::destroy_socket (class socket_base_t *socket_)
{
    scoped_lock_t locker (_slot_sync);

    //  Free the associated thread slot.
    const uint32_t tid = socket_->get_tid ();
    _empty_slots.push_back (tid);
    _slots[tid] = NULL;

    //  Remove the socket from the list of sockets.
    _sockets.erase (socket_);

    //  If zmq_ctx_term() was already called and there are no more socket
    //  we can ask reaper thread to terminate.
    if (_terminating && _sockets.empty ())
        _reaper->stop ();
}

zmq::object_t *zmq::ZmqContext::get_reaper () const
{
    return _reaper;
}

zmq::thread_ctx_t::thread_ctx_t () :
    _thread_priority (ZMQ_THREAD_PRIORITY_DFLT),
    _thread_sched_policy (ZMQ_THREAD_SCHED_POLICY_DFLT)
{
}

void zmq::thread_ctx_t::start_thread (thread_t &thread_,
                                      thread_fn *tfn_,
                                      arg_: *mut c_void,
                                      name_: *const c_char) const
{
    thread_.setSchedulingParameters (_thread_priority, _thread_sched_policy,
                                     _thread_affinity_cpus);

    char namebuf[16] = "";
    snprintf (namebuf, sizeof (namebuf), "%s%sZMQbg%s%s",
              _thread_name_prefix.is_empty() ? "" : _thread_name_prefix,
              _thread_name_prefix.is_empty() ? "" : "/", name_ ? "/" : "",
              name_ ? name_ : "");
    thread_.start (tfn_, arg_, namebuf);
}

int zmq::thread_ctx_t::set (option_: i32, const optval_: *mut c_void, optvallen_: usize)
{
    const bool is_int = (optvallen_ == sizeof (int));
    int value = 0;
    if (is_int)
        memcpy (&value, optval_, sizeof (int));

    switch (option_) {
        case ZMQ_THREAD_SCHED_POLICY:
            if (is_int && value >= 0) {
                scoped_lock_t locker (_opt_sync);
                _thread_sched_policy = value;
                return 0;
            }
            break;

        case ZMQ_THREAD_AFFINITY_CPU_ADD:
            if (is_int && value >= 0) {
                scoped_lock_t locker (_opt_sync);
                _thread_affinity_cpus.insert (value);
                return 0;
            }
            break;

        case ZMQ_THREAD_AFFINITY_CPU_REMOVE:
            if (is_int && value >= 0) {
                scoped_lock_t locker (_opt_sync);
                if (0 == _thread_affinity_cpus.erase (value)) {
                    errno = EINVAL;
                    return -1;
                }
                return 0;
            }
            break;

        case ZMQ_THREAD_PRIORITY:
            if (is_int && value >= 0) {
                scoped_lock_t locker (_opt_sync);
                _thread_priority = value;
                return 0;
            }
            break;

        case ZMQ_THREAD_NAME_PREFIX:
            // start_thread() allows max 16 chars for thread name
            if (is_int) {
                std::ostringstream s;
                s << value;
                scoped_lock_t locker (_opt_sync);
                _thread_name_prefix = s.str ();
                return 0;
            } else if (optvallen_ > 0 && optvallen_ <= 16) {
                scoped_lock_t locker (_opt_sync);
                _thread_name_prefix.assign (static_cast<const char *> (optval_),
                                            optvallen_);
                return 0;
            }
            break;
    }

    errno = EINVAL;
    return -1;
}

int zmq::thread_ctx_t::get (option_: i32,
                            optval_: *mut c_void,
                            const optvallen_: *mut usize)
{
    const bool is_int = (*optvallen_ == sizeof (int));
    int *value = static_cast<int *> (optval_);

    switch (option_) {
        case ZMQ_THREAD_SCHED_POLICY:
            if (is_int) {
                scoped_lock_t locker (_opt_sync);
                *value = _thread_sched_policy;
                return 0;
            }
            break;

        case ZMQ_THREAD_NAME_PREFIX:
            if (is_int) {
                scoped_lock_t locker (_opt_sync);
                *value = atoi (_thread_name_prefix.c_str ());
                return 0;
            } else if (*optvallen_ >= _thread_name_prefix.size ()) {
                scoped_lock_t locker (_opt_sync);
                memcpy (optval_, _thread_name_prefix.data (),
                        _thread_name_prefix.size ());
                return 0;
            }
            break;
    }

    errno = EINVAL;
    return -1;
}

void zmq::ZmqContext::send_command (uint32_t tid_, const ZmqCommand &command_)
{
    _slots[tid_]->send (command_);
}

zmq::io_thread_t *zmq::ZmqContext::choose_io_thread (u64 affinity_)
{
    if (_io_threads.empty ())
        return NULL;

    //  Find the I/O thread with minimum load.
    int min_load = -1;
    io_thread_t *selected_io_thread = NULL;
    for (io_threads_t::size_type i = 0, size = _io_threads.size (); i != size;
         i++) {
        if (!affinity_ || (affinity_ & (u64 (1) << i))) {
            const int load = _io_threads[i]->get_load ();
            if (selected_io_thread == NULL || load < min_load) {
                min_load = load;
                selected_io_thread = _io_threads[i];
            }
        }
    }
    return selected_io_thread;
}

int zmq::ZmqContext::register_endpoint (addr_: *const c_char,
                                   const ZmqEndpoint &endpoint_)
{
    scoped_lock_t locker (_endpoints_sync);

    const bool inserted =
      _endpoints.ZMQ_MAP_INSERT_OR_EMPLACE (std::string (addr_), endpoint_)
        .second;
    if (!inserted) {
        errno = EADDRINUSE;
        return -1;
    }
    return 0;
}

int zmq::ZmqContext::unregister_endpoint (const std::string &addr_,
                                     const socket_base_t *const socket_)
{
    scoped_lock_t locker (_endpoints_sync);

    const endpoints_t::iterator it = _endpoints.find (addr_);
    if (it == _endpoints.end () || it->second.socket != socket_) {
        errno = ENOENT;
        return -1;
    }

    //  Remove endpoint.
    _endpoints.erase (it);

    return 0;
}

void zmq::ZmqContext::unregister_endpoints (const socket_base_t *const socket_)
{
    scoped_lock_t locker (_endpoints_sync);

    for (endpoints_t::iterator it = _endpoints.begin (),
                               end = _endpoints.end ();
         it != end;) {
        if (it->second.socket == socket_)
#if __cplusplus >= 201103L || (defined _MSC_VER && _MSC_VER >= 1700)
            it = _endpoints.erase (it);
// #else
            _endpoints.erase (it++);
// #endif
        else
            ++it;
    }
}

zmq::ZmqEndpoint zmq::ZmqContext::find_endpoint (addr_: *const c_char)
{
    scoped_lock_t locker (_endpoints_sync);

    endpoints_t::iterator it = _endpoints.find (addr_);
    if (it == _endpoints.end ()) {
        errno = ECONNREFUSED;
        ZmqEndpoint empty = {NULL, options_t ()};
        return empty;
    }
    ZmqEndpoint endpoint = it->second;

    //  Increment the command sequence number of the peer so that it won't
    //  get deallocated until "bind" command is issued by the caller.
    //  The subsequent 'bind' has to be called with inc_seqnum parameter
    //  set to false, so that the seqnum isn't incremented twice.
    endpoint.socket->inc_seqnum ();

    return endpoint;
}

void zmq::ZmqContext::pend_connection (const std::string &addr_,
                                  const ZmqEndpoint &endpoint_,
                                  pipe_t **pipes_)
{
    scoped_lock_t locker (_endpoints_sync);

    const pending_connection_t pending_connection = {endpoint_, pipes_[0],
                                                     pipes_[1]};

    const endpoints_t::iterator it = _endpoints.find (addr_);
    if (it == _endpoints.end ()) {
        //  Still no bind.
        endpoint_.socket->inc_seqnum ();
        _pending_connections.ZMQ_MAP_INSERT_OR_EMPLACE (addr_,
                                                        pending_connection);
    } else {
        //  Bind has happened in the mean time, connect directly
        connect_inproc_sockets (it->second.socket, it->second.options,
                                pending_connection, connect_side);
    }
}

void zmq::ZmqContext::connect_pending (addr_: *const c_char,
                                  zmq::socket_base_t *bind_socket_)
{
    scoped_lock_t locker (_endpoints_sync);

    const std::pair<pending_connections_t::iterator,
                    pending_connections_t::iterator>
      pending = _pending_connections.equal_range (addr_);
    for (pending_connections_t::iterator p = pending.first; p != pending.second;
         ++p)
        connect_inproc_sockets (bind_socket_, _endpoints[addr_].options,
                                p->second, bind_side);

    _pending_connections.erase (pending.first, pending.second);
}

void zmq::ZmqContext::connect_inproc_sockets (
  bind_socket_: *mut socket_base_t,
  const options_t &bind_options_,
  const pending_connection_t &pending_connection_,
  side side_)
{
    bind_socket_->inc_seqnum ();
    pending_connection_.bind_pipe->set_tid (bind_socket_->get_tid ());

    if (!bind_options_.recv_routing_id) {
        msg_t msg;
        const bool ok = pending_connection_.bind_pipe->read (&msg);
        zmq_assert (ok);
        const int rc = msg.close ();
        errno_assert (rc == 0);
    }

    if (!get_effective_conflate_option (pending_connection_.endpoint.options)) {
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

int zmq::ZmqContext::get_vmci_socket_family ()
{
    zmq::scoped_lock_t locker (_vmci_sync);

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
zmq::atomic_counter_t zmq::ZmqContext::max_socket_id;
