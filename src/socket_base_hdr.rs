/*
    Copyright (c) 2007-2016 Contributors as noted in the AUTHORS file

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

// #ifndef __ZMQ_SOCKET_BASE_HPP_INCLUDED__
// #define __ZMQ_SOCKET_BASE_HPP_INCLUDED__

// #include <string>
// #include <map>
// #include <stdarg.h>

// #include "own.hpp"
// #include "array.hpp"
// #include "blob.hpp"
// #include "stdint.hpp"
// #include "poller.hpp"
// #include "i_poll_events.hpp"
// #include "i_mailbox.hpp"
// #include "clock.hpp"
// #include "pipe.hpp"
// #include "endpoint.hpp"

extern "C" {
void zmq_free_event (data_: *mut c_void, hint_: *mut c_void);
}

namespace zmq
{
class ZmqContext;
class msg_t;
class pipe_t;

class socket_base_t : public own_t,
                      public array_item_t<>,
                      public i_poll_events,
                      public i_pipe_events
{
    friend class reaper_t;

// public:
    //  Returns false if object is not a socket.
    bool check_tag () const;

    //  Returns whether the socket is thread-safe.
    bool is_thread_safe () const;

    //  Create a socket of a specified type.
    static socket_base_t *
    create (type_: i32, zmq::ZmqContext *parent_, uint32_t tid_, sid_: i32);

    //  Returns the mailbox associated with this socket.
    i_mailbox *get_mailbox () const;

    //  Interrupt blocking call if the socket is stuck in one.
    //  This function can be called from a different thread!
    void stop ();

    //  Interface for communication with the API layer.
    int setsockopt (option_: i32, const optval_: *mut c_void, optvallen_: usize);
    int getsockopt (option_: i32, optval_: *mut c_void, optvallen_: *mut usize);
    int bind (endpoint_uri_: *const c_char);
    int connect (endpoint_uri_: *const c_char);
    int term_endpoint (endpoint_uri_: *const c_char);
    int send (zmq::msg_t *msg_, flags_: i32);
    int recv (zmq::msg_t *msg_, flags_: i32);
    void add_signaler (signaler_t *s_);
    void remove_signaler (signaler_t *s_);
    int close ();

    //  These functions are used by the polling mechanism to determine
    //  which events are to be reported from this socket.
    bool has_in ();
    bool has_out ();

    //  Joining and leaving groups
    int join (group_: *const c_char);
    int leave (group_: *const c_char);

    //  Using this function reaper thread ask the socket to register with
    //  its poller.
    void start_reaping (poller_t *poller_);

    //  i_poll_events implementation. This interface is used when socket
    //  is handled by the poller in the reaper thread.
    void in_event () ZMQ_FINAL;
    void out_event () ZMQ_FINAL;
    void timer_event (id_: i32) ZMQ_FINAL;

    //  i_pipe_events interface implementation.
    void read_activated (pipe_t *pipe_) ZMQ_FINAL;
    void write_activated (pipe_t *pipe_) ZMQ_FINAL;
    void hiccuped (pipe_t *pipe_) ZMQ_FINAL;
    void pipe_terminated (pipe_t *pipe_) ZMQ_FINAL;
    void lock ();
    void unlock ();

    int monitor (endpoint_: *const c_char,
                 events_: u64,
                 event_version_: i32,
                 type_: i32);

    void event_connected (const endpoint_uri_pair_t &endpoint_uri_pair_,
                          zmq::fd_t fd_);
    void event_connect_delayed (const endpoint_uri_pair_t &endpoint_uri_pair_,
                                err_: i32);
    void event_connect_retried (const endpoint_uri_pair_t &endpoint_uri_pair_,
                                interval_: i32);
    void event_listening (const endpoint_uri_pair_t &endpoint_uri_pair_,
                          zmq::fd_t fd_);
    void event_bind_failed (const endpoint_uri_pair_t &endpoint_uri_pair_,
                            err_: i32);
    void event_accepted (const endpoint_uri_pair_t &endpoint_uri_pair_,
                         zmq::fd_t fd_);
    void event_accept_failed (const endpoint_uri_pair_t &endpoint_uri_pair_,
                              err_: i32);
    void event_closed (const endpoint_uri_pair_t &endpoint_uri_pair_,
                       zmq::fd_t fd_);
    void event_close_failed (const endpoint_uri_pair_t &endpoint_uri_pair_,
                             err_: i32);
    void event_disconnected (const endpoint_uri_pair_t &endpoint_uri_pair_,
                             zmq::fd_t fd_);
    void event_handshake_failed_no_detail (
      const endpoint_uri_pair_t &endpoint_uri_pair_, err_: i32);
    void event_handshake_failed_protocol (
      const endpoint_uri_pair_t &endpoint_uri_pair_, err_: i32);
    void
    event_handshake_failed_auth (const endpoint_uri_pair_t &endpoint_uri_pair_,
                                 err_: i32);
    void
    event_handshake_succeeded (const endpoint_uri_pair_t &endpoint_uri_pair_,
                               err_: i32);

    //  Query the state of a specific peer. The default implementation
    //  always returns an ENOTSUP error.
    virtual int get_peer_state (const routing_id_: *mut c_void,
                                routing_id_size_: usize) const;

    //  Request for pipes statistics - will generate a ZMQ_EVENT_PIPES_STATS
    //  after gathering the data asynchronously. Requires event monitoring to
    //  be enabled.
    int query_pipes_stats ();

    bool is_disconnected () const;

  protected:
    socket_base_t (zmq::ZmqContext *parent_,
                   uint32_t tid_,
                   sid_: i32,
                   bool thread_safe_ = false);
    ~socket_base_t () ZMQ_OVERRIDE;

    //  Concrete algorithms for the x- methods are to be defined by
    //  individual socket types.
    virtual void xattach_pipe (zmq::pipe_t *pipe_,
                               bool subscribe_to_all_ = false,
                               bool locally_initiated_ = false) = 0;

    //  The default implementation assumes there are no specific socket
    //  options for the particular socket type. If not so, ZMQ_FINAL this
    //  method.
    virtual int
    xsetsockopt (option_: i32, const optval_: *mut c_void, optvallen_: usize);

    //  The default implementation assumes there are no specific socket
    //  options for the particular socket type. If not so, ZMQ_FINAL this
    //  method.
    virtual int xgetsockopt (option_: i32, optval_: *mut c_void, optvallen_: *mut usize);

    //  The default implementation assumes that send is not supported.
    virtual bool xhas_out ();
    virtual int xsend (zmq::msg_t *msg_);

    //  The default implementation assumes that recv in not supported.
    virtual bool xhas_in ();
    virtual int xrecv (zmq::msg_t *msg_);

    //  i_pipe_events will be forwarded to these functions.
    virtual void xread_activated (pipe_t *pipe_);
    virtual void xwrite_activated (pipe_t *pipe_);
    virtual void xhiccuped (pipe_t *pipe_);
    virtual void xpipe_terminated (pipe_t *pipe_) = 0;

    //  the default implementation assumes that joub and leave are not supported.
    virtual int xjoin (group_: *const c_char);
    virtual int xleave (group_: *const c_char);

    //  Delay actual destruction of the socket.
    void process_destroy () ZMQ_FINAL;

    int connect_internal (endpoint_uri_: *const c_char);

    // Mutex for synchronize access to the socket in thread safe mode
    mutex_t _sync;

  // private:
    // test if event should be sent and then dispatch it
    void event (const endpoint_uri_pair_t &endpoint_uri_pair_,
                uint64_t values_[],
                values_count_: u64,
                uint64_t type_);

    // Socket event data dispatch
    void monitor_event (event_: u64,
                        const uint64_t values_[],
                        values_count_: u64,
                        const endpoint_uri_pair_t &endpoint_uri_pair_) const;

    // Monitor socket cleanup
    void stop_monitor (bool send_monitor_stopped_event_ = true);

    //  Creates new endpoint ID and adds the endpoint to the map.
    void add_endpoint (const endpoint_uri_pair_t &endpoint_pair_,
                       own_t *endpoint_,
                       pipe_t *pipe_);

    //  Map of open endpoints.
    typedef std::pair<own_t *, pipe_t *> endpoint_pipe_t;
    typedef std::multimap<std::string, endpoint_pipe_t> endpoints_t;
    endpoints_t _endpoints;

    //  Map of open inproc endpoints.
    class inprocs_t
    {
^      // public:
        void emplace (endpoint_uri_: *const c_char, pipe_t *pipe_);
        int erase_pipes (const std::string &endpoint_uri_str_);
        void erase_pipe (const pipe_t *pipe_);

      // private:
        typedef std::multimap<std::string, pipe_t *> map_t;
        map_t _inprocs;
    };
    inprocs_t _inprocs;

    //  To be called after processing commands or invoking any command
    //  handlers explicitly. If required, it will deallocate the socket.
    void check_destroy ();

    //  Moves the flags from the message to local variables,
    //  to be later retrieved by getsockopt.
    void extract_flags (const msg_t *msg_);

    //  Used to check whether the object is a socket.
    uint32_t _tag;

    //  If true, associated context was already terminated.
    bool _ctx_terminated;

    //  If true, object should have been already destroyed. However,
    //  destruction is delayed while we unwind the stack to the point
    //  where it doesn't intersect the object being destroyed.
    bool _destroyed;

    //  Parse URI string.
    static int
    parse_uri (uri_: *const c_char, std::string &protocol_, std::string &path_);

    //  Check whether transport protocol, as specified in connect or
    //  bind, is available and compatible with the socket type.
    int check_protocol (const std::string &protocol_) const;

    //  Register the pipe with this socket.
    void attach_pipe (zmq::pipe_t *pipe_,
                      bool subscribe_to_all_ = false,
                      bool locally_initiated_ = false);

    //  Processes commands sent to this socket (if any). If timeout is -1,
    //  returns only after at least one command was processed.
    //  If throttle argument is true, commands are processed at most once
    //  in a predefined time period.
    int process_commands (timeout_: i32, bool throttle_);

    //  Handlers for incoming commands.
    void process_stop () ZMQ_FINAL;
    void process_bind (zmq::pipe_t *pipe_) ZMQ_FINAL;
    void
    process_pipe_stats_publish (outbound_queue_count_: u64,
                                inbound_queue_count_: u64,
                                endpoint_uri_pair_t *endpoint_pair_) ZMQ_FINAL;
    void process_term (linger_: i32) ZMQ_FINAL;
    void process_term_endpoint (std::string *endpoint_) ZMQ_FINAL;

    void update_pipe_options (option_: i32);

    std::string resolve_tcp_addr (std::string endpoint_uri_,
                                  tcp_address_: *const c_char);

    //  Socket's mailbox object.
    i_mailbox *_mailbox;

    //  List of attached pipes.
    typedef array_t<pipe_t, 3> pipes_t;
    pipes_t _pipes;

    //  Reaper's poller and handle of this socket within it.
    poller_t *_poller;
    poller_t::handle_t _handle;

    //  Timestamp of when commands were processed the last time.
    uint64_t _last_tsc;

    //  Number of messages received since last command processing.
    _ticks: i32;

    //  True if the last message received had MORE flag set.
    bool _rcvmore;

    //  Improves efficiency of time measurement.
    clock_t _clock;

    // Monitor socket;
    _monitor_socket: *mut c_void;

    // Bitmask of events being monitored
    int64_t _monitor_events;

    // Last socket endpoint resolved URI
    std::string _last_endpoint;

    // Indicate if the socket is thread safe
    const bool _thread_safe;

    // Signaler to be used in the reaping stage
    signaler_t *_reaper_signaler;

    // Mutex to synchronize access to the monitor Pair socket
    mutex_t _monitor_sync;

    ZMQ_NON_COPYABLE_NOR_MOVABLE (socket_base_t)

    // Add a flag for mark disconnect action
    bool _disconnected;
};

class routing_socket_base_t : public socket_base_t
{
  protected:
    routing_socket_base_t (class ZmqContext *parent_, uint32_t tid_, sid_: i32);
    ~routing_socket_base_t () ZMQ_OVERRIDE;

    // methods from socket_base_t
    int xsetsockopt (option_: i32,
                     const optval_: *mut c_void,
                     optvallen_: usize) ZMQ_OVERRIDE;
    void xwrite_activated (pipe_t *pipe_) ZMQ_FINAL;

    // own methods
    std::string extract_connect_routing_id ();
    bool connect_routing_id_is_set () const;

    struct out_pipe_t
    {
        pipe_t *pipe;
        bool active;
    };

    void add_out_pipe (blob_t routing_id_, pipe_t *pipe_);
    bool has_out_pipe (const blob_t &routing_id_) const;
    out_pipe_t *lookup_out_pipe (const blob_t &routing_id_);
    const out_pipe_t *lookup_out_pipe (const blob_t &routing_id_) const;
    void erase_out_pipe (const pipe_t *pipe_);
    out_pipe_t try_erase_out_pipe (const blob_t &routing_id_);
    template <typename Func> bool any_of_out_pipes (Func func_)
    {
        bool res = false;
        for (out_pipes_t::iterator it = _out_pipes.begin (),
                                   end = _out_pipes.end ();
             it != end && !res; ++it) {
            res |= func_ (*it->second.pipe);
        }

        return res;
    }

  // private:
    //  Outbound pipes indexed by the peer IDs.
    typedef std::map<blob_t, out_pipe_t> out_pipes_t;
    out_pipes_t _out_pipes;

    // Next assigned name on a zmq_connect() call used by ROUTER and STREAM socket types
    std::string _connect_routing_id;
};
}

// #endif
