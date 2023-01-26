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

// #ifndef __ZMQ_CTX_HPP_INCLUDED__
// #define __ZMQ_CTX_HPP_INCLUDED__

// #include <map>
// #include <vector>
// #include <string>
// #include <stdarg.h>

// #include "mailbox.hpp"
// #include "array.hpp"
// #include "config.hpp"
// #include "mutex.hpp"
// #include "stdint.hpp"
// #include "options.hpp"
// #include "atomic_counter.hpp"
// #include "thread.hpp"

// namespace zmq
// {
// class object_t;
// class io_thread_t;
// class socket_base_t;
// class reaper_t;
// class pipe_t;

use std::collections::HashSet;

//  Information associated with inproc endpoint. Note that endpoint options
//  are registered as well so that the peer can access them without a need
//  for synchronisation, handshaking or similar.
#[derive(Default,Debug,Clone)]
pub struct endpoint_t
{
    // socket_base_t *socket;
    pub socket: *mut socket_base_t,
    // options_t options;
    pub options: options_t,
}

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
        endpoint_t endpoint;
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
pub type endpoints_t = HashMap<String,endpoint_t>;

// typedef std::multimap<std::string, pending_connection_t>
//       pending_connections_t;
pub type pending_connections_t = HashMap<String, pending_connection_t>

// class ctx_t ZMQ_FINAL : public thread_ctx_t
pub struct ctx_t
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

impl ctx_t {
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

// }

// #endif
