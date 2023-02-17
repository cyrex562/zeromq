pub struct ZmqSocketBase {
    own: own_t,

}

impl ZmqSocketBase {
    //  Returns false if object is not a socket.
    // bool check_tag () const;

    //  Returns whether the socket is thread-safe.
    // bool is_thread_safe () const;
}


pub struct socket_base_t_ : public own_t,
                      public array_item_t<>,
                      public i_poll_events,
                      public i_pipe_events
{
    friend class reaper_t;

// public:




    //  Create a socket of a specified type.
    static ZmqSocketBase *
    create (type_: i32, ZmqContext *parent_, uint32_t tid_, sid_: i32);

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
    int send (msg: &mut ZmqMessage flags: i32);
    int recv (msg: &mut ZmqMessage flags: i32);
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

    void event_connected (const EndpointUriPair &endpoint_uri_pair_,
                          fd_t fd_);
    void event_connect_delayed (const EndpointUriPair &endpoint_uri_pair_,
                                err_: i32);
    void event_connect_retried (const EndpointUriPair &endpoint_uri_pair_,
                                interval_: i32);
    void event_listening (const EndpointUriPair &endpoint_uri_pair_,
                          fd_t fd_);
    void event_bind_failed (const EndpointUriPair &endpoint_uri_pair_,
                            err_: i32);
    void event_accepted (const EndpointUriPair &endpoint_uri_pair_,
                         fd_t fd_);
    void event_accept_failed (const EndpointUriPair &endpoint_uri_pair_,
                              err_: i32);
    void event_closed (const EndpointUriPair &endpoint_uri_pair_,
                       fd_t fd_);
    void event_close_failed (const EndpointUriPair &endpoint_uri_pair_,
                             err_: i32);
    void event_disconnected (const EndpointUriPair &endpoint_uri_pair_,
                             fd_t fd_);
    void event_handshake_failed_no_detail (
      const EndpointUriPair &endpoint_uri_pair_, err_: i32);
    void event_handshake_failed_protocol (
      const EndpointUriPair &endpoint_uri_pair_, err_: i32);
    void
    event_handshake_failed_auth (const EndpointUriPair &endpoint_uri_pair_,
                                 err_: i32);
    void
    event_handshake_succeeded (const EndpointUriPair &endpoint_uri_pair_,
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
    ZmqSocketBase (ZmqContext *parent_,
                   uint32_t tid_,
                   sid_: i32,
                   bool thread_safe_ = false);
    ~ZmqSocketBase () ZMQ_OVERRIDE;

    //  Concrete algorithms for the x- methods are to be defined by
    //  individual socket types.
    virtual void xattach_pipe (pipe_t *pipe_,
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
    virtual int xsend (ZmqMessage *msg);

    //  The default implementation assumes that recv in not supported.
    virtual bool xhas_in ();
    virtual int xrecv (ZmqMessage *msg);

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
    void event (const EndpointUriPair &endpoint_uri_pair_,
                u64 values_[],
                values_count_: u64,
                u64 type_);

    // Socket event data dispatch
    void monitor_event (event_: u64,
                        const u64 values_[],
                        values_count_: u64,
                        const EndpointUriPair &endpoint_uri_pair_) const;

    // Monitor socket cleanup
    void stop_monitor (bool send_monitor_stopped_event_ = true);

    //  Creates new endpoint ID and adds the endpoint to the map.
    void add_endpoint (const EndpointUriPair &endpoint_pair_,
                       own_t *endpoint_,
                       pipe_t *pipe_);

    //  Map of open endpoints.
    typedef std::pair<own_t *, pipe_t *> endpoint_pipe_t;
    typedef std::multimap<std::string, endpoint_pipe_t> endpoints_t;
    endpoints_t _endpoints;

    //  Map of open inproc endpoints.
pub struct inprocs_t
    {
      // public:
        void emplace (endpoint_uri_: *const c_char, pipe_t *pipe_);
        int erase_pipes (endpoint_uri_str_: &str);
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
    void extract_flags (const ZmqMessage *msg);

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
    int check_protocol (protocol_: &str) const;

    //  Register the pipe with this socket.
    void attach_pipe (pipe_t *pipe_,
                      bool subscribe_to_all_ = false,
                      bool locally_initiated_ = false);

    //  Processes commands sent to this socket (if any). If timeout is -1,
    //  returns only after at least one command was processed.
    //  If throttle argument is true, commands are processed at most once
    //  in a predefined time period.
    int process_commands (timeout_: i32, bool throttle_);

    //  Handlers for incoming commands.
    void process_stop () ZMQ_FINAL;
    void process_bind (pipe_t *pipe_) ZMQ_FINAL;
    void
    process_pipe_stats_publish (outbound_queue_count_: u64,
                                inbound_queue_count_: u64,
                                EndpointUriPair *endpoint_pair_) ZMQ_FINAL;
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
    u64 _last_tsc;

    //  Number of messages received since last command processing.
    _ticks: i32;

    //  True if the last message received had MORE flag set.
    bool _rcvmore;

    //  Improves efficiency of time measurement.
    clock_t _clock;

    // Monitor socket;
    _monitor_socket: *mut c_void;

    // Bitmask of events being monitored
    i64 _monitor_events;

    // Last socket endpoint resolved URI
    std::string _last_endpoint;

    // Indicate if the socket is thread safe
    const bool _thread_safe;

    // Signaler to be used in the reaping stage
    signaler_t *_reaper_signaler;

    // Mutex to synchronize access to the monitor Pair socket
    mutex_t _monitor_sync;

    ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqSocketBase)

    // Add a flag for mark disconnect action
    bool _disconnected;
};
pub struct routing_socket_base_t : public ZmqSocketBase
{
  protected:
    routing_socket_base_t (class ZmqContext *parent_, uint32_t tid_, sid_: i32);
    ~routing_socket_base_t () ZMQ_OVERRIDE;

    // methods from ZmqSocketBase
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

    void add_out_pipe (Blob routing_id_, pipe_t *pipe_);
    bool has_out_pipe (const Blob &routing_id_) const;
    out_pipe_t *lookup_out_pipe (const Blob &routing_id_);
    const out_pipe_t *lookup_out_pipe (const Blob &routing_id_) const;
    void erase_out_pipe (const pipe_t *pipe_);
    out_pipe_t try_erase_out_pipe (const Blob &routing_id_);
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
    typedef std::map<Blob, out_pipe_t> out_pipes_t;
    out_pipes_t _out_pipes;

    // Next assigned name on a zmq_connect() call used by ROUTER and STREAM socket types
    std::string _connect_routing_id;
};

void ZmqSocketBase::inprocs_t::emplace (endpoint_uri_: *const c_char,
                                             pipe_t *pipe_)
{
    _inprocs.ZMQ_MAP_INSERT_OR_EMPLACE (std::string (endpoint_uri_), pipe_);
}

int ZmqSocketBase::inprocs_t::erase_pipes (
  endpoint_uri_str_: &str)
{
    const std::pair<map_t::iterator, map_t::iterator> range =
      _inprocs.equal_range (endpoint_uri_str_);
    if (range.first == range.second) {
        errno = ENOENT;
        return -1;
    }

    for (map_t::iterator it = range.first; it != range.second; ++it) {
        it->second->send_disconnect_msg ();
        it->second->terminate (true);
    }
    _inprocs.erase (range.first, range.second);
    return 0;
}

void ZmqSocketBase::inprocs_t::erase_pipe (const pipe_t *pipe_)
{
    for (map_t::iterator it = _inprocs.begin (), end = _inprocs.end ();
         it != end; ++it)
        if (it->second == pipe_) {
            _inprocs.erase (it);
            break;
        }
}

bool ZmqSocketBase::check_tag () const
{
    return _tag == 0xbaddecaf;
}

bool ZmqSocketBase::is_thread_safe () const
{
    return _thread_safe;
}

ZmqSocketBase *ZmqSocketBase::create (type_: i32,
pub struct ZmqContext *parent_,
                                                uint32_t tid_,
                                                sid_: i32)
{
    ZmqSocketBase *s = NULL;
    switch (type_) {
        case ZMQ_PAIR:
            s = new (std::nothrow) pair_t (parent_, tid_, sid_);
            break;
        case ZMQ_PUB:
            s = new (std::nothrow) pub_t (parent_, tid_, sid_);
            break;
        case ZMQ_SUB:
            s = new (std::nothrow) sub_t (parent_, tid_, sid_);
            break;
        case ZMQ_REQ:
            s = new (std::nothrow) req_t (parent_, tid_, sid_);
            break;
        case ZMQ_REP:
            s = new (std::nothrow) rep_t (parent_, tid_, sid_);
            break;
        case ZMQ_DEALER:
            s = new (std::nothrow) dealer_t (parent_, tid_, sid_);
            break;
        case ZMQ_ROUTER:
            s = new (std::nothrow) router_t (parent_, tid_, sid_);
            break;
        case ZMQ_PULL:
            s = new (std::nothrow) pull_t (parent_, tid_, sid_);
            break;
        case ZMQ_PUSH:
            s = new (std::nothrow) push_t (parent_, tid_, sid_);
            break;
        case ZMQ_XPUB:
            s = new (std::nothrow) xpub_t (parent_, tid_, sid_);
            break;
        case ZMQ_XSUB:
            s = new (std::nothrow) xsub_t (parent_, tid_, sid_);
            break;
        case ZMQ_STREAM:
            s = new (std::nothrow) stream_t (parent_, tid_, sid_);
            break;
        case ZMQ_SERVER:
            s = new (std::nothrow) server_t (parent_, tid_, sid_);
            break;
        case ZMQ_CLIENT:
            s = new (std::nothrow) client_t (parent_, tid_, sid_);
            break;
        case ZMQ_RADIO:
            s = new (std::nothrow) radio_t (parent_, tid_, sid_);
            break;
        case ZMQ_DISH:
            s = new (std::nothrow) dish_t (parent_, tid_, sid_);
            break;
        case ZMQ_GATHER:
            s = new (std::nothrow) gather_t (parent_, tid_, sid_);
            break;
        case ZMQ_SCATTER:
            s = new (std::nothrow) scatter_t (parent_, tid_, sid_);
            break;
        case ZMQ_DGRAM:
            s = new (std::nothrow) dgram_t (parent_, tid_, sid_);
            break;
        case ZMQ_PEER:
            s = new (std::nothrow) peer_t (parent_, tid_, sid_);
            break;
        case ZMQ_CHANNEL:
            s = new (std::nothrow) channel_t (parent_, tid_, sid_);
            break;
        default:
            errno = EINVAL;
            return NULL;
    }

    alloc_assert (s);

    if (s->_mailbox == NULL) {
        s->_destroyed = true;
        LIBZMQ_DELETE (s);
        return NULL;
    }

    return s;
}

ZmqSocketBase::ZmqSocketBase (ZmqContext *parent_,
                                   uint32_t tid_,
                                   sid_: i32,
                                   bool thread_safe_) :
    own_t (parent_, tid_),
    _sync (),
    _tag (0xbaddecaf),
    _ctx_terminated (false),
    _destroyed (false),
    _poller (NULL),
    _handle (static_cast<poller_t::handle_t> (NULL)),
    _last_tsc (0),
    _ticks (0),
    _rcvmore (false),
    _monitor_socket (NULL),
    _monitor_events (0),
    _thread_safe (thread_safe_),
    _reaper_signaler (NULL),
    _monitor_sync ()
{
    options.socket_id = sid_;
    options.ipv6 = (parent_->get (ZMQ_IPV6) != 0);
    options.linger.store (parent_->get (ZMQ_BLOCKY) ? -1 : 0);
    options.zero_copy = parent_->get (ZMQ_ZERO_COPY_RECV) != 0;

    if (_thread_safe) {
        _mailbox = new (std::nothrow) mailbox_safe_t (&_sync);
        zmq_assert (_mailbox);
    } else {
        mailbox_t *m = new (std::nothrow) mailbox_t ();
        zmq_assert (m);

        if (m->get_fd () != retired_fd)
            _mailbox = m;
        else {
            LIBZMQ_DELETE (m);
            _mailbox = NULL;
        }
    }
}

int ZmqSocketBase::get_peer_state (const routing_id_: *mut c_void,
                                        routing_id_size_: usize) const
{
    LIBZMQ_UNUSED (routing_id_);
    LIBZMQ_UNUSED (routing_id_size_);

    //  Only ROUTER sockets support this
    errno = ENOTSUP;
    return -1;
}

ZmqSocketBase::~ZmqSocketBase ()
{
    if (_mailbox)
        LIBZMQ_DELETE (_mailbox);

    if (_reaper_signaler)
        LIBZMQ_DELETE (_reaper_signaler);

    scoped_lock_t lock (_monitor_sync);
    stop_monitor ();

    zmq_assert (_destroyed);
}

i_mailbox *ZmqSocketBase::get_mailbox () const
{
    return _mailbox;
}

void ZmqSocketBase::stop ()
{
    //  Called by ctx when it is terminated (zmq_ctx_term).
    //  'stop' command is sent from the threads that called zmq_ctx_term to
    //  the thread owning the socket. This way, blocking call in the
    //  owner thread can be interrupted.
    send_stop ();
}

// TODO consider renaming protocol_ to scheme_ in conformance with RFC 3986
// terminology, but this requires extensive changes to be consistent
int ZmqSocketBase::parse_uri (uri_: *const c_char,
                                   std::string &protocol_,
                                   std::string &path_)
{
    zmq_assert (uri_ != NULL);

    const std::string uri (uri_);
    const std::string::size_type pos = uri.find ("://");
    if (pos == std::string::npos) {
        errno = EINVAL;
        return -1;
    }
    protocol_ = uri.substr (0, pos);
    path_ = uri.substr (pos + 3);

    if (protocol_.is_empty() || path_.empty ()) {
        errno = EINVAL;
        return -1;
    }
    return 0;
}

int ZmqSocketBase::check_protocol (protocol_: &str) const
{
    //  First check out whether the protocol is something we are aware of.
    if (protocol_ != protocol_name::inproc
// #if defined ZMQ_HAVE_IPC
        && protocol_ != protocol_name::ipc
// #endif
        && protocol_ != protocol_name::tcp
// #ifdef ZMQ_HAVE_WS
        && protocol_ != protocol_name::ws
// #endif
// #ifdef ZMQ_HAVE_WSS
        && protocol_ != protocol_name::wss
// #endif
// #if defined ZMQ_HAVE_OPENPGM
        //  pgm/epgm transports only available if 0MQ is compiled with OpenPGM.
        && protocol_ != protocol_name::pgm
        && protocol_ != protocol_name::epgm
// #endif
// #if defined ZMQ_HAVE_TIPC
        // TIPC transport is only available on Linux.
        && protocol_ != protocol_name::tipc
// #endif
// #if defined ZMQ_HAVE_NORM
        && protocol_ != protocol_name::norm
// #endif
// #if defined ZMQ_HAVE_VMCI
        && protocol_ != protocol_name::vmci
// #endif
        && protocol_ != protocol_name::udp) {
        errno = EPROTONOSUPPORT;
        return -1;
    }

    //  Check whether socket type and transport protocol match.
    //  Specifically, multicast protocols can't be combined with
    //  bi-directional messaging patterns (socket types).
// #if defined ZMQ_HAVE_OPENPGM || defined ZMQ_HAVE_NORM
// #if defined ZMQ_HAVE_OPENPGM && defined ZMQ_HAVE_NORM
    if ((protocol_ == protocol_name::pgm || protocol_ == protocol_name::epgm
         || protocol_ == protocol_name::norm)
#elif defined ZMQ_HAVE_OPENPGM
    if ((protocol_ == protocol_name::pgm || protocol_ == protocol_name::epgm)
// #else // defined ZMQ_HAVE_NORM
    if (protocol_ == protocol_name::norm
// #endif
        && options.type != ZMQ_PUB && options.type != ZMQ_SUB
        && options.type != ZMQ_XPUB && options.type != ZMQ_XSUB) {
        errno = ENOCOMPATPROTO;
        return -1;
    }
// #endif

    if (protocol_ == protocol_name::udp
        && (options.type != ZMQ_DISH && options.type != ZMQ_RADIO
            && options.type != ZMQ_DGRAM)) {
        errno = ENOCOMPATPROTO;
        return -1;
    }

    //  Protocol is available.
    return 0;
}

void ZmqSocketBase::attach_pipe (pipe_t *pipe_,
                                      bool subscribe_to_all_,
                                      bool locally_initiated_)
{
    //  First, register the pipe so that we can terminate it later on.
    pipe_->set_event_sink (this);
    _pipes.push_back (pipe_);

    //  Let the derived socket type know about new pipe.
    xattach_pipe (pipe_, subscribe_to_all_, locally_initiated_);

    //  If the socket is already being closed, ask any new pipes to terminate
    //  straight away.
    if (is_terminating ()) {
        register_term_acks (1);
        pipe_->terminate (false);
    }
}

int ZmqSocketBase::setsockopt (option_: i32,
                                    const optval_: *mut c_void,
                                    optvallen_: usize)
{
    scoped_optional_lock_t sync_lock (_thread_safe ? &_sync : NULL);

    if (unlikely (_ctx_terminated)) {
        errno = ETERM;
        return -1;
    }

    //  First, check whether specific socket type overloads the option.
    int rc = xsetsockopt (option_, optval_, optvallen_);
    if (rc == 0 || errno != EINVAL) {
        return rc;
    }

    //  If the socket type doesn't support the option, pass it to
    //  the generic option parser.
    rc = options.setsockopt (option_, optval_, optvallen_);
    update_pipe_options (option_);

    return rc;
}

int ZmqSocketBase::getsockopt (option_: i32,
                                    optval_: *mut c_void,
                                    optvallen_: *mut usize)
{
    scoped_optional_lock_t sync_lock (_thread_safe ? &_sync : NULL);

    if (unlikely (_ctx_terminated)) {
        errno = ETERM;
        return -1;
    }

    //  First, check whether specific socket type overloads the option.
    int rc = xgetsockopt (option_, optval_, optvallen_);
    if (rc == 0 || errno != EINVAL) {
        return rc;
    }

    if (option_ == ZMQ_RCVMORE) {
        return do_getsockopt<int> (optval_, optvallen_, _rcvmore ? 1 : 0);
    }

    if (option_ == ZMQ_FD) {
        if (_thread_safe) {
            // thread safe socket doesn't provide file descriptor
            errno = EINVAL;
            return -1;
        }

        return do_getsockopt<fd_t> (
          optval_, optvallen_,
          (static_cast<mailbox_t *> (_mailbox))->get_fd ());
    }

    if (option_ == ZMQ_EVENTS) {
        let rc: i32 = process_commands (0, false);
        if (rc != 0 && (errno == EINTR || errno == ETERM)) {
            return -1;
        }
        errno_assert (rc == 0);

        return do_getsockopt<int> (optval_, optvallen_,
                                   (has_out () ? ZMQ_POLLOUT : 0)
                                     | (has_in () ? ZMQ_POLLIN : 0));
    }

    if (option_ == ZMQ_LAST_ENDPOINT) {
        return do_getsockopt (optval_, optvallen_, _last_endpoint);
    }

    if (option_ == ZMQ_THREAD_SAFE) {
        return do_getsockopt<int> (optval_, optvallen_, _thread_safe ? 1 : 0);
    }

    return options.getsockopt (option_, optval_, optvallen_);
}

int ZmqSocketBase::join (group_: *const c_char)
{
    scoped_optional_lock_t sync_lock (_thread_safe ? &_sync : NULL);

    return xjoin (group_);
}

int ZmqSocketBase::leave (group_: *const c_char)
{
    scoped_optional_lock_t sync_lock (_thread_safe ? &_sync : NULL);

    return xleave (group_);
}

void ZmqSocketBase::add_signaler (signaler_t *s_)
{
    zmq_assert (_thread_safe);

    scoped_lock_t sync_lock (_sync);
    (static_cast<mailbox_safe_t *> (_mailbox))->add_signaler (s_);
}

void ZmqSocketBase::remove_signaler (signaler_t *s_)
{
    zmq_assert (_thread_safe);

    scoped_lock_t sync_lock (_sync);
    (static_cast<mailbox_safe_t *> (_mailbox))->remove_signaler (s_);
}

int ZmqSocketBase::bind (endpoint_uri_: *const c_char)
{
    scoped_optional_lock_t sync_lock (_thread_safe ? &_sync : NULL);

    if (unlikely (_ctx_terminated)) {
        errno = ETERM;
        return -1;
    }

    //  Process pending commands, if any.
    int rc = process_commands (0, false);
    if (unlikely (rc != 0)) {
        return -1;
    }

    //  Parse endpoint_uri_ string.
    std::string protocol;
    std::string address;
    if (parse_uri (endpoint_uri_, protocol, address)
        || check_protocol (protocol)) {
        return -1;
    }

    if (protocol == protocol_name::inproc) {
        const ZmqEndpoint endpoint = {this, options};
        rc = register_endpoint (endpoint_uri_, endpoint);
        if (rc == 0) {
            connect_pending (endpoint_uri_, this);
            _last_endpoint.assign (endpoint_uri_);
            options.connected = true;
        }
        return rc;
    }

// #if defined ZMQ_HAVE_OPENPGM || defined ZMQ_HAVE_NORM
// #if defined ZMQ_HAVE_OPENPGM && defined ZMQ_HAVE_NORM
    if (protocol == protocol_name::pgm || protocol == protocol_name::epgm
        || protocol == protocol_name::norm) {
#elif defined ZMQ_HAVE_OPENPGM
    if (protocol == protocol_name::pgm || protocol == protocol_name::epgm) {
// #else // defined ZMQ_HAVE_NORM
    if (protocol == protocol_name::norm) {
// #endif
        //  For convenience's sake, bind can be used interchangeable with
        //  connect for PGM, EPGM, NORM transports.
        rc = connect (endpoint_uri_);
        if (rc != -1)
            options.connected = true;
        return rc;
    }
// #endif

    if (protocol == protocol_name::udp) {
        if (!(options.type == ZMQ_DGRAM || options.type == ZMQ_DISH)) {
            errno = ENOCOMPATPROTO;
            return -1;
        }

        //  Choose the I/O thread to run the session in.
        io_thread_t *io_thread = choose_io_thread (options.affinity);
        if (!io_thread) {
            errno = EMTHREAD;
            return -1;
        }

        Address *paddr =
          new (std::nothrow) Address (protocol, address, this->get_ctx ());
        alloc_assert (paddr);

        paddr->resolved.udp_addr = new (std::nothrow) UdpAddress ();
        alloc_assert (paddr->resolved.udp_addr);
        rc = paddr->resolved.udp_addr->resolve (address, true,
                                                options.ipv6);
        if (rc != 0) {
            LIBZMQ_DELETE (paddr);
            return -1;
        }

        session_base_t *session =
          session_base_t::create (io_thread, true, this, options, paddr);
        errno_assert (session);

        //  Create a bi-directional pipe.
        object_t *parents[2] = {this, session};
        pipe_t *new_pipes[2] = {NULL, NULL};

        int hwms[2] = {options.sndhwm, options.rcvhwm};
        bool conflates[2] = {false, false};
        rc = pipepair (parents, new_pipes, hwms, conflates);
        errno_assert (rc == 0);

        //  Attach local end of the pipe to the socket object.
        attach_pipe (new_pipes[0], true, true);
        pipe_t *const newpipe = new_pipes[0];

        //  Attach remote end of the pipe to the session object later on.
        session->attach_pipe (new_pipes[1]);

        //  Save last endpoint URI
        paddr->to_string (_last_endpoint);

        //  TODO shouldn't this use _last_endpoint instead of endpoint_uri_? as in the other cases
        add_endpoint (EndpointUriPair (endpoint_uri_, std::string (),
                                           endpoint_type_none),
                      static_cast<own_t *> (session), newpipe);

        return 0;
    }

    //  Remaining transports require to be run in an I/O thread, so at this
    //  point we'll choose one.
    io_thread_t *io_thread = choose_io_thread (options.affinity);
    if (!io_thread) {
        errno = EMTHREAD;
        return -1;
    }

    if (protocol == protocol_name::tcp) {
        tcp_listener_t *listener =
          new (std::nothrow) tcp_listener_t (io_thread, this, options);
        alloc_assert (listener);
        rc = listener->set_local_address (address.c_str ());
        if (rc != 0) {
            LIBZMQ_DELETE (listener);
            event_bind_failed (make_unconnected_bind_endpoint_pair (address),
                               zmq_errno ());
            return -1;
        }

        // Save last endpoint URI
        listener->get_local_address (_last_endpoint);

        add_endpoint (make_unconnected_bind_endpoint_pair (_last_endpoint),
                      static_cast<own_t *> (listener), NULL);
        options.connected = true;
        return 0;
    }

// #ifdef ZMQ_HAVE_WS
// #ifdef ZMQ_HAVE_WSS
    if (protocol == protocol_name::ws || protocol == protocol_name::wss) {
        ws_listener_t *listener = new (std::nothrow) ws_listener_t (
          io_thread, this, options, protocol == protocol_name::wss);
// #else
    if (protocol == protocol_name::ws) {
        ws_listener_t *listener =
          new (std::nothrow) ws_listener_t (io_thread, this, options, false);
// #endif
        alloc_assert (listener);
        rc = listener->set_local_address (address.c_str ());
        if (rc != 0) {
            LIBZMQ_DELETE (listener);
            event_bind_failed (make_unconnected_bind_endpoint_pair (address),
                               zmq_errno ());
            return -1;
        }

        // Save last endpoint URI
        listener->get_local_address (_last_endpoint);

        add_endpoint (make_unconnected_bind_endpoint_pair (_last_endpoint),
                      static_cast<own_t *> (listener), NULL);
        options.connected = true;
        return 0;
    }
// #endif

// #if defined ZMQ_HAVE_IPC
    if (protocol == protocol_name::ipc) {
        ipc_listener_t *listener =
          new (std::nothrow) ipc_listener_t (io_thread, this, options);
        alloc_assert (listener);
        int rc = listener->set_local_address (address.c_str ());
        if (rc != 0) {
            LIBZMQ_DELETE (listener);
            event_bind_failed (make_unconnected_bind_endpoint_pair (address),
                               zmq_errno ());
            return -1;
        }

        // Save last endpoint URI
        listener->get_local_address (_last_endpoint);

        add_endpoint (make_unconnected_bind_endpoint_pair (_last_endpoint),
                      static_cast<own_t *> (listener), NULL);
        options.connected = true;
        return 0;
    }
// #endif
// #if defined ZMQ_HAVE_TIPC
    if (protocol == protocol_name::tipc) {
        tipc_listener_t *listener =
          new (std::nothrow) tipc_listener_t (io_thread, this, options);
        alloc_assert (listener);
        int rc = listener->set_local_address (address.c_str ());
        if (rc != 0) {
            LIBZMQ_DELETE (listener);
            event_bind_failed (make_unconnected_bind_endpoint_pair (address),
                               zmq_errno ());
            return -1;
        }

        // Save last endpoint URI
        listener->get_local_address (_last_endpoint);

        // TODO shouldn't this use _last_endpoint as in the other cases?
        add_endpoint (make_unconnected_bind_endpoint_pair (endpoint_uri_),
                      static_cast<own_t *> (listener), NULL);
        options.connected = true;
        return 0;
    }
// #endif
// #if defined ZMQ_HAVE_VMCI
    if (protocol == protocol_name::vmci) {
        vmci_listener_t *listener =
          new (std::nothrow) vmci_listener_t (io_thread, this, options);
        alloc_assert (listener);
        int rc = listener->set_local_address (address.c_str ());
        if (rc != 0) {
            LIBZMQ_DELETE (listener);
            event_bind_failed (make_unconnected_bind_endpoint_pair (address),
                               zmq_errno ());
            return -1;
        }

        listener->get_local_address (_last_endpoint);

        add_endpoint (make_unconnected_bind_endpoint_pair (_last_endpoint),
                      static_cast<own_t *> (listener), NULL);
        options.connected = true;
        return 0;
    }
// #endif

    zmq_assert (false);
    return -1;
}

int ZmqSocketBase::connect (endpoint_uri_: *const c_char)
{
    scoped_optional_lock_t sync_lock (_thread_safe ? &_sync : NULL);
    return connect_internal (endpoint_uri_);
}

int ZmqSocketBase::connect_internal (endpoint_uri_: *const c_char)
{
    if (unlikely (_ctx_terminated)) {
        errno = ETERM;
        return -1;
    }

    //  Process pending commands, if any.
    int rc = process_commands (0, false);
    if (unlikely (rc != 0)) {
        return -1;
    }

    //  Parse endpoint_uri_ string.
    std::string protocol;
    std::string address;
    if (parse_uri (endpoint_uri_, protocol, address)
        || check_protocol (protocol)) {
        return -1;
    }

    if (protocol == protocol_name::inproc) {
        //  TODO: inproc connect is specific with respect to creating pipes
        //  as there's no 'reconnect' functionality implemented. Once that
        //  is in place we should follow generic pipe creation algorithm.

        //  Find the peer endpoint.
        const ZmqEndpoint peer = find_endpoint (endpoint_uri_);

        // The total HWM for an inproc connection should be the sum of
        // the binder's HWM and the connector's HWM.
        let sndhwm: i32 = peer.socket == NULL ? options.sndhwm
                           : options.sndhwm != 0 && peer.options.rcvhwm != 0
                             ? options.sndhwm + peer.options.rcvhwm
                             : 0;
        let rcvhwm: i32 = peer.socket == NULL ? options.rcvhwm
                           : options.rcvhwm != 0 && peer.options.sndhwm != 0
                             ? options.rcvhwm + peer.options.sndhwm
                             : 0;

        //  Create a bi-directional pipe to connect the peers.
        object_t *parents[2] = {this, peer.socket == NULL ? this : peer.socket};
        pipe_t *new_pipes[2] = {NULL, NULL};

        const bool conflate = get_effective_conflate_option (options);

        int hwms[2] = {conflate ? -1 : sndhwm, conflate ? -1 : rcvhwm};
        bool conflates[2] = {conflate, conflate};
        rc = pipepair (parents, new_pipes, hwms, conflates);
        if (!conflate) {
            new_pipes[0]->set_hwms_boost (peer.options.sndhwm,
                                          peer.options.rcvhwm);
            new_pipes[1]->set_hwms_boost (options.sndhwm, options.rcvhwm);
        }

        errno_assert (rc == 0);

        if (!peer.socket) {
            //  The peer doesn't exist yet so we don't know whether
            //  to send the routing id message or not. To resolve this,
            //  we always send our routing id and drop it later if
            //  the peer doesn't expect it.
            send_routing_id (new_pipes[0], options);

// #ifdef ZMQ_BUILD_DRAFT_API
            //  If set, send the hello msg of the local socket to the peer.
            if (options.can_send_hello_msg && options.hello_msg.size () > 0) {
                send_hello_msg (new_pipes[0], options);
            }
// #endif

            const ZmqEndpoint endpoint = {this, options};
            pend_connection (std::string (endpoint_uri_), endpoint, new_pipes);
        } else {
            //  If required, send the routing id of the local socket to the peer.
            if (peer.options.recv_routing_id) {
                send_routing_id (new_pipes[0], options);
            }

            //  If required, send the routing id of the peer to the local socket.
            if (options.recv_routing_id) {
                send_routing_id (new_pipes[1], peer.options);
            }

// #ifdef ZMQ_BUILD_DRAFT_API
            //  If set, send the hello msg of the local socket to the peer.
            if (options.can_send_hello_msg && options.hello_msg.size () > 0) {
                send_hello_msg (new_pipes[0], options);
            }

            //  If set, send the hello msg of the peer to the local socket.
            if (peer.options.can_send_hello_msg
                && peer.options.hello_msg.size () > 0) {
                send_hello_msg (new_pipes[1], peer.options);
            }

            if (peer.options.can_recv_disconnect_msg
                && peer.options.disconnect_msg.size () > 0)
                new_pipes[0]->set_disconnect_msg (peer.options.disconnect_msg);
// #endif

            //  Attach remote end of the pipe to the peer socket. Note that peer's
            //  seqnum was incremented in find_endpoint function. We don't need it
            //  increased here.
            send_bind (peer.socket, new_pipes[1], false);
        }

        //  Attach local end of the pipe to this socket object.
        attach_pipe (new_pipes[0], false, true);

        // Save last endpoint URI
        _last_endpoint.assign (endpoint_uri_);

        // remember inproc connections for disconnect
        _inprocs.emplace (endpoint_uri_, new_pipes[0]);

        options.connected = true;
        return 0;
    }
    const bool is_single_connect =
      (options.type == ZMQ_DEALER || options.type == ZMQ_SUB
       || options.type == ZMQ_PUB || options.type == ZMQ_REQ);
    if (unlikely (is_single_connect)) {
        if (0 != _endpoints.count (endpoint_uri_)) {
            // There is no valid use for multiple connects for SUB-PUB nor
            // DEALER-ROUTER nor REQ-REP. Multiple connects produces
            // nonsensical results.
            return 0;
        }
    }

    //  Choose the I/O thread to run the session in.
    io_thread_t *io_thread = choose_io_thread (options.affinity);
    if (!io_thread) {
        errno = EMTHREAD;
        return -1;
    }

    Address *paddr =
      new (std::nothrow) Address (protocol, address, this->get_ctx ());
    alloc_assert (paddr);

    //  Resolve address (if needed by the protocol)
    if (protocol == protocol_name::tcp) {
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
        const char *check = address;
        if (isalnum (*check) || isxdigit (*check) || *check == '['
            || *check == ':') {
            check++;
            while (isalnum (*check) || isxdigit (*check) || *check == '.'
                   || *check == '-' || *check == ':' || *check == '%'
                   || *check == ';' || *check == '[' || *check == ']'
                   || *check == '_' || *check == '*') {
                check++;
            }
        }
        //  Assume the worst, now look for success
        rc = -1;
        //  Did we reach the end of the address safely?
        if (*check == 0) {
            //  Do we have a valid port string? (cannot be '*' in connect
            check = strrchr (address, ':');
            if (check) {
                check++;
                if (*check && (isdigit (*check)))
                    rc = 0; //  Valid
            }
        }
        if (rc == -1) {
            errno = EINVAL;
            LIBZMQ_DELETE (paddr);
            return -1;
        }
        //  Defer resolution until a socket is opened
        paddr->resolved.tcp_addr = NULL;
    }
// #ifdef ZMQ_HAVE_WS
// #ifdef ZMQ_HAVE_WSS
    else if (protocol == protocol_name::ws || protocol == protocol_name::wss) {
        if (protocol == protocol_name::wss) {
            paddr->resolved.wss_addr = new (std::nothrow) WssAddress ();
            alloc_assert (paddr->resolved.wss_addr);
            rc = paddr->resolved.wss_addr->resolve (address, false,
                                                    options.ipv6);
        } else
// #else
    else if (protocol == protocol_name::ws) {
// #endif
        {
            paddr->resolved.ws_addr = new (std::nothrow) WsAddress ();
            alloc_assert (paddr->resolved.ws_addr);
            rc = paddr->resolved.ws_addr->resolve (address, false,
                                                   options.ipv6);
        }

        if (rc != 0) {
            LIBZMQ_DELETE (paddr);
            return -1;
        }
    }
// #endif

// #if defined ZMQ_HAVE_IPC
    else if (protocol == protocol_name::ipc) {
        paddr->resolved.ipc_addr = new (std::nothrow) IpcAddress ();
        alloc_assert (paddr->resolved.ipc_addr);
        int rc = paddr->resolved.ipc_addr->resolve (address.c_str ());
        if (rc != 0) {
            LIBZMQ_DELETE (paddr);
            return -1;
        }
    }
// #endif

    if (protocol == protocol_name::udp) {
        if (options.type != ZMQ_RADIO) {
            errno = ENOCOMPATPROTO;
            LIBZMQ_DELETE (paddr);
            return -1;
        }

        paddr->resolved.udp_addr = new (std::nothrow) UdpAddress ();
        alloc_assert (paddr->resolved.udp_addr);
        rc = paddr->resolved.udp_addr->resolve (address, false,
                                                options.ipv6);
        if (rc != 0) {
            LIBZMQ_DELETE (paddr);
            return -1;
        }
    }

    // TBD - Should we check address for ZMQ_HAVE_NORM???

// #ifdef ZMQ_HAVE_OPENPGM
    if (protocol == protocol_name::pgm || protocol == protocol_name::epgm) {
        struct pgm_addrinfo_t *res = NULL;
        uint16_t port_number = 0;
        int rc =
          pgm_socket_t::init_address (address, &res, &port_number);
        if (res != NULL)
            pgm_freeaddrinfo (res);
        if (rc != 0 || port_number == 0) {
            return -1;
        }
    }
// #endif
// #if defined ZMQ_HAVE_TIPC
    else if (protocol == protocol_name::tipc) {
        paddr->resolved.tipc_addr = new (std::nothrow) TipcAddress ();
        alloc_assert (paddr->resolved.tipc_addr);
        int rc = paddr->resolved.tipc_addr->resolve (address.c_str ());
        if (rc != 0) {
            LIBZMQ_DELETE (paddr);
            return -1;
        }
        const sockaddr_tipc *const saddr =
          reinterpret_cast<const sockaddr_tipc *> (
            paddr->resolved.tipc_addr->addr ());
        // Cannot connect to random Port Identity
        if (saddr->addrtype == TIPC_ADDR_ID
            && paddr->resolved.tipc_addr->is_random ()) {
            LIBZMQ_DELETE (paddr);
            errno = EINVAL;
            return -1;
        }
    }
// #endif
// #if defined ZMQ_HAVE_VMCI
    else if (protocol == protocol_name::vmci) {
        paddr->resolved.vmci_addr =
          new (std::nothrow) VmciAddress (this->get_ctx ());
        alloc_assert (paddr->resolved.vmci_addr);
        int rc = paddr->resolved.vmci_addr->resolve (address.c_str ());
        if (rc != 0) {
            LIBZMQ_DELETE (paddr);
            return -1;
        }
    }
// #endif

    //  Create session.
    session_base_t *session =
      session_base_t::create (io_thread, true, this, options, paddr);
    errno_assert (session);

    //  PGM does not support subscription forwarding; ask for all data to be
    //  sent to this pipe. (same for NORM, currently?)
// #if defined ZMQ_HAVE_OPENPGM && defined ZMQ_HAVE_NORM
    const bool subscribe_to_all =
      protocol == protocol_name::pgm || protocol == protocol_name::epgm
      || protocol == protocol_name::norm || protocol == protocol_name::udp;
#elif defined ZMQ_HAVE_OPENPGM
    const bool subscribe_to_all = protocol == protocol_name::pgm
                                  || protocol == protocol_name::epgm
                                  || protocol == protocol_name::udp;
#elif defined ZMQ_HAVE_NORM
    const bool subscribe_to_all =
      protocol == protocol_name::norm || protocol == protocol_name::udp;
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

std::string
ZmqSocketBase::resolve_tcp_addr (std::string endpoint_uri_pair_,
                                      tcp_address_: *const c_char)
{
    // The resolved last_endpoint is used as a key in the endpoints map.
    // The address passed by the user might not match in the TCP case due to
    // IPv4-in-IPv6 mapping (EG: tcp://[::ffff:127.0.0.1]:9999), so try to
    // resolve before giving up. Given at this stage we don't know whether a
    // socket is connected or bound, try with both.
    if (_endpoints.find (endpoint_uri_pair_) == _endpoints.end ()) {
        TcpAddress *tcp_addr = new (std::nothrow) TcpAddress ();
        alloc_assert (tcp_addr);
        int rc = tcp_addr->resolve (tcp_address_, false, options.ipv6);

        if (rc == 0) {
            tcp_addr->to_string (endpoint_uri_pair_);
            if (_endpoints.find (endpoint_uri_pair_) == _endpoints.end ()) {
                rc = tcp_addr->resolve (tcp_address_, true, options.ipv6);
                if (rc == 0) {
                    tcp_addr->to_string (endpoint_uri_pair_);
                }
            }
        }
        LIBZMQ_DELETE (tcp_addr);
    }
    return endpoint_uri_pair_;
}

void ZmqSocketBase::add_endpoint (
  const EndpointUriPair &endpoint_pair_, own_t *endpoint_, pipe_t *pipe_)
{
    //  Activate the session. Make it a child of this socket.
    launch_child (endpoint_);
    _endpoints.ZMQ_MAP_INSERT_OR_EMPLACE (endpoint_pair_.identifier (),
                                          endpoint_pipe_t (endpoint_, pipe_));

    if (pipe_ != NULL)
        pipe_->set_endpoint_pair (endpoint_pair_);
}

int ZmqSocketBase::term_endpoint (endpoint_uri_: *const c_char)
{
    scoped_optional_lock_t sync_lock (_thread_safe ? &_sync : NULL);

    //  Check whether the context hasn't been shut down yet.
    if (unlikely (_ctx_terminated)) {
        errno = ETERM;
        return -1;
    }

    //  Check whether endpoint address passed to the function is valid.
    if (unlikely (!endpoint_uri_)) {
        errno = EINVAL;
        return -1;
    }

    //  Process pending commands, if any, since there could be pending unprocessed process_own()'s
    //  (from launch_child() for example) we're asked to terminate now.
    let rc: i32 = process_commands (0, false);
    if (unlikely (rc != 0)) {
        return -1;
    }

    //  Parse endpoint_uri_ string.
    std::string uri_protocol;
    std::string uri_path;
    if (parse_uri (endpoint_uri_, uri_protocol, uri_path)
        || check_protocol (uri_protocol)) {
        return -1;
    }

    const std::string endpoint_uri_str = std::string (endpoint_uri_);

    // Disconnect an inproc socket
    if (uri_protocol == protocol_name::inproc) {
        return unregister_endpoint (endpoint_uri_str, this) == 0
                 ? 0
                 : _inprocs.erase_pipes (endpoint_uri_str);
    }

    const std::string resolved_endpoint_uri =
      uri_protocol == protocol_name::tcp
        ? resolve_tcp_addr (endpoint_uri_str, uri_path.c_str ())
        : endpoint_uri_str;

    //  Find the endpoints range (if any) corresponding to the endpoint_uri_pair_ string.
    const std::pair<endpoints_t::iterator, endpoints_t::iterator> range =
      _endpoints.equal_range (resolved_endpoint_uri);
    if (range.first == range.second) {
        errno = ENOENT;
        return -1;
    }

    for (endpoints_t::iterator it = range.first; it != range.second; ++it) {
        //  If we have an associated pipe, terminate it.
        if (it->second.second != NULL)
            it->second.second->terminate (false);
        term_child (it->second.first);
    }
    _endpoints.erase (range.first, range.second);

    if (options.reconnect_stop & ZMQ_RECONNECT_STOP_AFTER_DISCONNECT) {
        _disconnected = true;
    }

    return 0;
}

int ZmqSocketBase::send (msg: &mut ZmqMessage flags: i32)
{
    scoped_optional_lock_t sync_lock (_thread_safe ? &_sync : NULL);

    //  Check whether the context hasn't been shut down yet.
    if (unlikely (_ctx_terminated)) {
        errno = ETERM;
        return -1;
    }

    //  Check whether message passed to the function is valid.
    if (unlikely (!msg || !msg->check ())) {
        errno = EFAULT;
        return -1;
    }

    //  Process pending commands, if any.
    int rc = process_commands (0, true);
    if (unlikely (rc != 0)) {
        return -1;
    }

    //  Clear any user-visible flags that are set on the message.
    msg->reset_flags (ZmqMessage::more);

    //  At this point we impose the flags on the message.
    if (flags & ZMQ_SNDMORE)
        msg->set_flags (ZmqMessage::more);

    msg->reset_metadata ();

    //  Try to send the message using method in each socket class
    rc = xsend (msg);
    if (rc == 0) {
        return 0;
    }
    //  Special case for ZMQ_PUSH: -2 means pipe is dead while a
    //  multi-part send is in progress and can't be recovered, so drop
    //  silently when in blocking mode to keep backward compatibility.
    if (unlikely (rc == -2)) {
        if (!((flags & ZMQ_DONTWAIT) || options.sndtimeo == 0)) {
            rc = msg->close ();
            errno_assert (rc == 0);
            rc = msg->init ();
            errno_assert (rc == 0);
            return 0;
        }
    }
    if (unlikely (errno != EAGAIN)) {
        return -1;
    }

    //  In case of non-blocking send we'll simply propagate
    //  the error - including EAGAIN - up the stack.
    if ((flags & ZMQ_DONTWAIT) || options.sndtimeo == 0) {
        return -1;
    }

    //  Compute the time when the timeout should occur.
    //  If the timeout is infinite, don't care.
    int timeout = options.sndtimeo;
    const u64 end = timeout < 0 ? 0 : (_clock.now_ms () + timeout);

    //  Oops, we couldn't send the message. Wait for the next
    //  command, process it and try to send the message again.
    //  If timeout is reached in the meantime, return EAGAIN.
    while (true) {
        if (unlikely (process_commands (timeout, false) != 0)) {
            return -1;
        }
        rc = xsend (msg);
        if (rc == 0)
            break;
        if (unlikely (errno != EAGAIN)) {
            return -1;
        }
        if (timeout > 0) {
            timeout = static_cast<int> (end - _clock.now_ms ());
            if (timeout <= 0) {
                errno = EAGAIN;
                return -1;
            }
        }
    }

    return 0;
}

int ZmqSocketBase::recv (msg: &mut ZmqMessage flags: i32)
{
    scoped_optional_lock_t sync_lock (_thread_safe ? &_sync : NULL);

    //  Check whether the context hasn't been shut down yet.
    if (unlikely (_ctx_terminated)) {
        errno = ETERM;
        return -1;
    }

    //  Check whether message passed to the function is valid.
    if (unlikely (!msg || !msg->check ())) {
        errno = EFAULT;
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
    if (++_ticks == inbound_poll_rate) {
        if (unlikely (process_commands (0, false) != 0)) {
            return -1;
        }
        _ticks = 0;
    }

    //  Get the message.
    int rc = xrecv (msg);
    if (unlikely (rc != 0 && errno != EAGAIN)) {
        return -1;
    }

    //  If we have the message, return immediately.
    if (rc == 0) {
        extract_flags (msg);
        return 0;
    }

    //  If the message cannot be fetched immediately, there are two scenarios.
    //  For non-blocking recv, commands are processed in case there's an
    //  activate_reader command already waiting in a command pipe.
    //  If it's not, return EAGAIN.
    if ((flags & ZMQ_DONTWAIT) || options.rcvtimeo == 0) {
        if (unlikely (process_commands (0, false) != 0)) {
            return -1;
        }
        _ticks = 0;

        rc = xrecv (msg);
        if (rc < 0) {
            return rc;
        }
        extract_flags (msg);

        return 0;
    }

    //  Compute the time when the timeout should occur.
    //  If the timeout is infinite, don't care.
    int timeout = options.rcvtimeo;
    const u64 end = timeout < 0 ? 0 : (_clock.now_ms () + timeout);

    //  In blocking scenario, commands are processed over and over again until
    //  we are able to fetch a message.
    bool block = (_ticks != 0);
    while (true) {
        if (unlikely (process_commands (block ? timeout : 0, false) != 0)) {
            return -1;
        }
        rc = xrecv (msg);
        if (rc == 0) {
            _ticks = 0;
            break;
        }
        if (unlikely (errno != EAGAIN)) {
            return -1;
        }
        block = true;
        if (timeout > 0) {
            timeout = static_cast<int> (end - _clock.now_ms ());
            if (timeout <= 0) {
                errno = EAGAIN;
                return -1;
            }
        }
    }

    extract_flags (msg);
    return 0;
}

int ZmqSocketBase::close ()
{
    scoped_optional_lock_t sync_lock (_thread_safe ? &_sync : NULL);

    //  Remove all existing signalers for thread safe sockets
    if (_thread_safe)
        (static_cast<mailbox_safe_t *> (_mailbox))->clear_signalers ();

    //  Mark the socket as dead
    _tag = 0xdeadbeef;


    //  Transfer the ownership of the socket from this application thread
    //  to the reaper thread which will take care of the rest of shutdown
    //  process.
    send_reap (this);

    return 0;
}

bool ZmqSocketBase::has_in ()
{
    return xhas_in ();
}

bool ZmqSocketBase::has_out ()
{
    return xhas_out ();
}

void ZmqSocketBase::start_reaping (poller_t *poller_)
{
    //  Plug the socket to the reaper thread.
    _poller = poller_;

    fd_t fd;

    if (!_thread_safe)
        fd = (static_cast<mailbox_t *> (_mailbox))->get_fd ();
    else {
        scoped_optional_lock_t sync_lock (_thread_safe ? &_sync : NULL);

        _reaper_signaler = new (std::nothrow) signaler_t ();
        zmq_assert (_reaper_signaler);

        //  Add signaler to the safe mailbox
        fd = _reaper_signaler->get_fd ();
        (static_cast<mailbox_safe_t *> (_mailbox))
          ->add_signaler (_reaper_signaler);

        //  Send a signal to make sure reaper handle existing commands
        _reaper_signaler->send ();
    }

    _handle = _poller->add_fd (fd, this);
    _poller->set_pollin (_handle);

    //  Initialise the termination and check whether it can be deallocated
    //  immediately.
    terminate ();
    check_destroy ();
}

int ZmqSocketBase::process_commands (timeout_: i32, bool throttle_)
{
    if (timeout_ == 0) {
        //  If we are asked not to wait, check whether we haven't processed
        //  commands recently, so that we can throttle the new commands.

        //  Get the CPU's tick counter. If 0, the counter is not available.
        const u64 tsc = clock_t::rdtsc ();

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
            if (tsc >= _last_tsc && tsc - _last_tsc <= max_command_delay)
                return 0;
            _last_tsc = tsc;
        }
    }

    //  Check whether there are any commands pending for this thread.
    ZmqCommand cmd;
    int rc = _mailbox->recv (&cmd, timeout_);

    if (rc != 0 && errno == EINTR)
        return -1;

    //  Process all available commands.
    while (rc == 0 || errno == EINTR) {
        if (rc == 0) {
            cmd.destination->process_command (cmd);
        }
        rc = _mailbox->recv (&cmd, 0);
    }

    zmq_assert (errno == EAGAIN);

    if (_ctx_terminated) {
        errno = ETERM;
        return -1;
    }

    return 0;
}

void ZmqSocketBase::process_stop ()
{
    //  Here, someone have called zmq_ctx_term while the socket was still alive.
    //  We'll remember the fact so that any blocking call is interrupted and any
    //  further attempt to use the socket will return ETERM. The user is still
    //  responsible for calling zmq_close on the socket though!
    scoped_lock_t lock (_monitor_sync);
    stop_monitor ();

    _ctx_terminated = true;
}

void ZmqSocketBase::process_bind (pipe_t *pipe_)
{
    attach_pipe (pipe_);
}

void ZmqSocketBase::process_term (linger_: i32)
{
    //  Unregister all inproc endpoints associated with this socket.
    //  Doing this we make sure that no new pipes from other sockets (inproc)
    //  will be initiated.
    unregister_endpoints (this);

    //  Ask all attached pipes to terminate.
    for (pipes_t::size_type i = 0, size = _pipes.size (); i != size; ++i) {
        //  Only inprocs might have a disconnect message set
        _pipes[i]->send_disconnect_msg ();
        _pipes[i]->terminate (false);
    }
    register_term_acks (static_cast<int> (_pipes.size ()));

    //  Continue the termination process immediately.
    own_t::process_term (linger_);
}

void ZmqSocketBase::process_term_endpoint (std::string *endpoint_)
{
    term_endpoint (endpoint_->c_str ());
    delete endpoint_;
}

void ZmqSocketBase::process_pipe_stats_publish (
  outbound_queue_count_: u64,
  inbound_queue_count_: u64,
  EndpointUriPair *endpoint_pair_)
{
    u64 values[2] = {outbound_queue_count_, inbound_queue_count_};
    event (*endpoint_pair_, values, 2, ZMQ_EVENT_PIPES_STATS);
    delete endpoint_pair_;
}

/*
 * There are 2 pipes per connection, and the inbound one _must_ be queried from
 * the I/O thread. So ask the outbound pipe, in the application thread, to send
 * a message (pipe_peer_stats) to its peer. The message will carry the outbound
 * pipe stats and endpoint, and the reference to the socket object.
 * The inbound pipe on the I/O thread will then add its own stats and endpoint,
 * and write back a message to the socket object (pipe_stats_publish) which
 * will raise an event with the data.
 */
int ZmqSocketBase::query_pipes_stats ()
{
    {
        scoped_lock_t lock (_monitor_sync);
        if (!(_monitor_events & ZMQ_EVENT_PIPES_STATS)) {
            errno = EINVAL;
            return -1;
        }
    }
    if (_pipes.size () == 0) {
        errno = EAGAIN;
        return -1;
    }
    for (pipes_t::size_type i = 0, size = _pipes.size (); i != size; ++i) {
        _pipes[i]->send_stats_to_peer (this);
    }

    return 0;
}

void ZmqSocketBase::update_pipe_options (option_: i32)
{
    if (option_ == ZMQ_SNDHWM || option_ == ZMQ_RCVHWM) {
        for (pipes_t::size_type i = 0, size = _pipes.size (); i != size; ++i) {
            _pipes[i]->set_hwms (options.rcvhwm, options.sndhwm);
            _pipes[i]->send_hwms_to_peer (options.sndhwm, options.rcvhwm);
        }
    }
}

void ZmqSocketBase::process_destroy ()
{
    _destroyed = true;
}

int ZmqSocketBase::xsetsockopt (int, const void *, size_t)
{
    errno = EINVAL;
    return -1;
}

int ZmqSocketBase::xgetsockopt (int, void *, size_t *)
{
    errno = EINVAL;
    return -1;
}

bool ZmqSocketBase::xhas_out ()
{
    return false;
}

int ZmqSocketBase::xsend (ZmqMessage *)
{
    errno = ENOTSUP;
    return -1;
}

bool ZmqSocketBase::xhas_in ()
{
    return false;
}

int ZmqSocketBase::xjoin (group_: *const c_char)
{
    LIBZMQ_UNUSED (group_);
    errno = ENOTSUP;
    return -1;
}

int ZmqSocketBase::xleave (group_: *const c_char)
{
    LIBZMQ_UNUSED (group_);
    errno = ENOTSUP;
    return -1;
}

int ZmqSocketBase::xrecv (ZmqMessage *)
{
    errno = ENOTSUP;
    return -1;
}

void ZmqSocketBase::xread_activated (pipe_t *)
{
    zmq_assert (false);
}
void ZmqSocketBase::xwrite_activated (pipe_t *)
{
    zmq_assert (false);
}

void ZmqSocketBase::xhiccuped (pipe_t *)
{
    zmq_assert (false);
}

void ZmqSocketBase::in_event ()
{
    //  This function is invoked only once the socket is running in the context
    //  of the reaper thread. Process any commands from other threads/sockets
    //  that may be available at the moment. Ultimately, the socket will
    //  be destroyed.
    {
        scoped_optional_lock_t sync_lock (_thread_safe ? &_sync : NULL);

        //  If the socket is thread safe we need to unsignal the reaper signaler
        if (_thread_safe)
            _reaper_signaler->recv ();

        process_commands (0, false);
    }
    check_destroy ();
}

void ZmqSocketBase::out_event ()
{
    zmq_assert (false);
}

void ZmqSocketBase::timer_event (int)
{
    zmq_assert (false);
}

void ZmqSocketBase::check_destroy ()
{
    //  If the object was already marked as destroyed, finish the deallocation.
    if (_destroyed) {
        //  Remove the socket from the reaper's poller.
        _poller->rm_fd (_handle);

        //  Remove the socket from the context.
        destroy_socket (this);

        //  Notify the reaper about the fact.
        send_reaped ();

        //  Deallocate.
        own_t::process_destroy ();
    }
}

void ZmqSocketBase::read_activated (pipe_t *pipe_)
{
    xread_activated (pipe_);
}

void ZmqSocketBase::write_activated (pipe_t *pipe_)
{
    xwrite_activated (pipe_);
}

void ZmqSocketBase::hiccuped (pipe_t *pipe_)
{
    if (options.immediate == 1)
        pipe_->terminate (false);
    else
        // Notify derived sockets of the hiccup
        xhiccuped (pipe_);
}

void ZmqSocketBase::pipe_terminated (pipe_t *pipe_)
{
    //  Notify the specific socket type about the pipe termination.
    xpipe_terminated (pipe_);

    // Remove pipe from inproc pipes
    _inprocs.erase_pipe (pipe_);

    //  Remove the pipe from the list of attached pipes and confirm its
    //  termination if we are already shutting down.
    _pipes.erase (pipe_);

    // Remove the pipe from _endpoints (set it to NULL).
    const std::string &identifier = pipe_->get_endpoint_pair ().identifier ();
    if (!identifier.empty ()) {
        std::pair<endpoints_t::iterator, endpoints_t::iterator> range;
        range = _endpoints.equal_range (identifier);

        for (endpoints_t::iterator it = range.first; it != range.second; ++it) {
            if (it->second.second == pipe_) {
                it->second.second = NULL;
                break;
            }
        }
    }

    if (is_terminating ())
        unregister_term_ack ();
}

void ZmqSocketBase::extract_flags (const ZmqMessage *msg)
{
    //  Test whether routing_id flag is valid for this socket type.
    if (unlikely (msg->flags () & ZmqMessage::routing_id))
        zmq_assert (options.recv_routing_id);

    //  Remove MORE flag.
    _rcvmore = (msg->flags () & ZmqMessage::more) != 0;
}

int ZmqSocketBase::monitor (endpoint_: *const c_char,
                                 events_: u64,
                                 event_version_: i32,
                                 type_: i32)
{
    scoped_lock_t lock (_monitor_sync);

    if (unlikely (_ctx_terminated)) {
        errno = ETERM;
        return -1;
    }

    //  Event version 1 supports only first 16 events.
    if (unlikely (event_version_ == 1 && events_ >> 16 != 0)) {
        errno = EINVAL;
        return -1;
    }

    //  Support deregistering monitoring endpoints as well
    if (endpoint_ == NULL) {
        stop_monitor ();
        return 0;
    }
    //  Parse endpoint_uri_ string.
    std::string protocol;
    std::string address;
    if (parse_uri (endpoint_, protocol, address) || check_protocol (protocol))
        return -1;

    //  Event notification only supported over inproc://
    if (protocol != protocol_name::inproc) {
        errno = EPROTONOSUPPORT;
        return -1;
    }

    // already monitoring. Stop previous monitor before starting new one.
    if (_monitor_socket != NULL) {
        stop_monitor (true);
    }

    // Check if the specified socket type is supported. It must be a
    // one-way socket types that support the SNDMORE flag.
    switch (type_) {
        case ZMQ_PAIR:
            break;
        case ZMQ_PUB:
            break;
        case ZMQ_PUSH:
            break;
        default:
            errno = EINVAL;
            return -1;
    }

    //  Register events to monitor
    _monitor_events = events_;
    options.monitor_event_version = event_version_;
    //  Create a monitor socket of the specified type.
    _monitor_socket = zmq_socket (get_ctx (), type_);
    if (_monitor_socket == NULL)
        return -1;

    //  Never block context termination on pending event messages
    int linger = 0;
    int rc =
      zmq_setsockopt (_monitor_socket, ZMQ_LINGER, &linger, mem::size_of::<linger>());
    if (rc == -1)
        stop_monitor (false);

    //  Spawn the monitor socket endpoint
    rc = zmq_bind (_monitor_socket, endpoint_);
    if (rc == -1)
        stop_monitor (false);
    return rc;
}

void ZmqSocketBase::event_connected (
  const EndpointUriPair &endpoint_uri_pair_, fd_t fd_)
{
    u64 values[1] = {static_cast<u64> (fd_)};
    event (endpoint_uri_pair_, values, 1, ZMQ_EVENT_CONNECTED);
}

void ZmqSocketBase::event_connect_delayed (
  const EndpointUriPair &endpoint_uri_pair_, err_: i32)
{
    u64 values[1] = {static_cast<u64> (err_)};
    event (endpoint_uri_pair_, values, 1, ZMQ_EVENT_CONNECT_DELAYED);
}

void ZmqSocketBase::event_connect_retried (
  const EndpointUriPair &endpoint_uri_pair_, interval_: i32)
{
    u64 values[1] = {static_cast<u64> (interval_)};
    event (endpoint_uri_pair_, values, 1, ZMQ_EVENT_CONNECT_RETRIED);
}

void ZmqSocketBase::event_listening (
  const EndpointUriPair &endpoint_uri_pair_, fd_t fd_)
{
    u64 values[1] = {static_cast<u64> (fd_)};
    event (endpoint_uri_pair_, values, 1, ZMQ_EVENT_LISTENING);
}

void ZmqSocketBase::event_bind_failed (
  const EndpointUriPair &endpoint_uri_pair_, err_: i32)
{
    u64 values[1] = {static_cast<u64> (err_)};
    event (endpoint_uri_pair_, values, 1, ZMQ_EVENT_BIND_FAILED);
}

void ZmqSocketBase::event_accepted (
  const EndpointUriPair &endpoint_uri_pair_, fd_t fd_)
{
    u64 values[1] = {static_cast<u64> (fd_)};
    event (endpoint_uri_pair_, values, 1, ZMQ_EVENT_ACCEPTED);
}

void ZmqSocketBase::event_accept_failed (
  const EndpointUriPair &endpoint_uri_pair_, err_: i32)
{
    u64 values[1] = {static_cast<u64> (err_)};
    event (endpoint_uri_pair_, values, 1, ZMQ_EVENT_ACCEPT_FAILED);
}

void ZmqSocketBase::event_closed (
  const EndpointUriPair &endpoint_uri_pair_, fd_t fd_)
{
    u64 values[1] = {static_cast<u64> (fd_)};
    event (endpoint_uri_pair_, values, 1, ZMQ_EVENT_CLOSED);
}

void ZmqSocketBase::event_close_failed (
  const EndpointUriPair &endpoint_uri_pair_, err_: i32)
{
    u64 values[1] = {static_cast<u64> (err_)};
    event (endpoint_uri_pair_, values, 1, ZMQ_EVENT_CLOSE_FAILED);
}

void ZmqSocketBase::event_disconnected (
  const EndpointUriPair &endpoint_uri_pair_, fd_t fd_)
{
    u64 values[1] = {static_cast<u64> (fd_)};
    event (endpoint_uri_pair_, values, 1, ZMQ_EVENT_DISCONNECTED);
}

void ZmqSocketBase::event_handshake_failed_no_detail (
  const endpoint_uri_pair_t &endpoint_uri_pair_, err_: i32)
{
    u64 values[1] = {static_cast<u64> (err_)};
    event (endpoint_uri_pair_, values, 1, ZMQ_EVENT_HANDSHAKE_FAILED_NO_DETAIL);
}

void ZmqSocketBase::event_handshake_failed_protocol (
  const endpoint_uri_pair_t &endpoint_uri_pair_, err_: i32)
{
    u64 values[1] = {static_cast<u64> (err_)};
    event (endpoint_uri_pair_, values, 1, ZMQ_EVENT_HANDSHAKE_FAILED_PROTOCOL);
}

void ZmqSocketBase::event_handshake_failed_auth (
  const endpoint_uri_pair_t &endpoint_uri_pair_, err_: i32)
{
    u64 values[1] = {static_cast<u64> (err_)};
    event (endpoint_uri_pair_, values, 1, ZMQ_EVENT_HANDSHAKE_FAILED_AUTH);
}

void ZmqSocketBase::event_handshake_succeeded (
  const endpoint_uri_pair_t &endpoint_uri_pair_, err_: i32)
{
    u64 values[1] = {static_cast<u64> (err_)};
    event (endpoint_uri_pair_, values, 1, ZMQ_EVENT_HANDSHAKE_SUCCEEDED);
}

void ZmqSocketBase::event (const endpoint_uri_pair_t &endpoint_uri_pair_,
                                u64 values_[],
                                values_count_: u64,
                                u64 type_)
{
    scoped_lock_t lock (_monitor_sync);
    if (_monitor_events & type_) {
        monitor_event (type_, values_, values_count_, endpoint_uri_pair_);
    }
}

//  Send a monitor event
void ZmqSocketBase::monitor_event (
  event_: u64,
  const u64 values_[],
  values_count_: u64,
  const endpoint_uri_pair_t &endpoint_uri_pair_) const
{
    // this is a private method which is only called from
    // contexts where the _monitor_sync mutex has been locked before

    if (_monitor_socket) {
        zmq_ZmqMessage msg;

        switch (options.monitor_event_version) {
            case 1: {
                //  The API should not allow to activate unsupported events
                zmq_assert (event_ <= std::numeric_limits<uint16_t>::max ());
                //  v1 only allows one value
                zmq_assert (values_count_ == 1);
                zmq_assert (values_[0]
                            <= std::numeric_limits<uint32_t>::max ());

                //  Send event and value in first frame
                const uint16_t event = static_cast<uint16_t> (event_);
                const uint32_t value = static_cast<uint32_t> (values_[0]);
                zmq_msg_init_size (&msg, mem::size_of::<event>() + mem::size_of::<value>());
                uint8_t *data = static_cast<uint8_t *> (zmq_msg_data (&msg));
                //  Avoid dereferencing uint32_t on unaligned address
                memcpy (data + 0, &event, mem::size_of::<event>());
                memcpy (data + mem::size_of::<event>(), &value, mem::size_of::<value>());
                zmq_msg_send (&msg, _monitor_socket, ZMQ_SNDMORE);

                const std::string &endpoint_uri =
                  endpoint_uri_pair_.identifier ();

                //  Send address in second frame
                zmq_msg_init_size (&msg, endpoint_uri.size ());
                memcpy (zmq_msg_data (&msg), endpoint_uri,
                        endpoint_uri.size ());
                zmq_msg_send (&msg, _monitor_socket, 0);
            } break;
            case 2: {
                //  Send event in first frame (64bit unsigned)
                zmq_msg_init_size (&msg, mem::size_of::<event_>());
                memcpy (zmq_msg_data (&msg), &event_, mem::size_of::<event_>());
                zmq_msg_send (&msg, _monitor_socket, ZMQ_SNDMORE);

                //  Send number of values that will follow in second frame
                zmq_msg_init_size (&msg, mem::size_of::<values_count_>());
                memcpy (zmq_msg_data (&msg), &values_count_,
                        mem::size_of::<values_count_>());
                zmq_msg_send (&msg, _monitor_socket, ZMQ_SNDMORE);

                //  Send values in third-Nth frames (64bit unsigned)
                for (u64 i = 0; i < values_count_; ++i) {
                    zmq_msg_init_size (&msg, sizeof (values_[i]));
                    memcpy (zmq_msg_data (&msg), &values_[i],
                            sizeof (values_[i]));
                    zmq_msg_send (&msg, _monitor_socket, ZMQ_SNDMORE);
                }

                //  Send local endpoint URI in second-to-last frame (string)
                zmq_msg_init_size (&msg, endpoint_uri_pair_.local.size ());
                memcpy (zmq_msg_data (&msg), endpoint_uri_pair_.local,
                        endpoint_uri_pair_.local.size ());
                zmq_msg_send (&msg, _monitor_socket, ZMQ_SNDMORE);

                //  Send remote endpoint URI in last frame (string)
                zmq_msg_init_size (&msg, endpoint_uri_pair_.remote.size ());
                memcpy (zmq_msg_data (&msg), endpoint_uri_pair_.remote,
                        endpoint_uri_pair_.remote.size ());
                zmq_msg_send (&msg, _monitor_socket, 0);
            } break;
        }
    }
}

void ZmqSocketBase::stop_monitor (bool send_monitor_stopped_event_)
{
    // this is a private method which is only called from
    // contexts where the _monitor_sync mutex has been locked before

    if (_monitor_socket) {
        if ((_monitor_events & ZMQ_EVENT_MONITOR_STOPPED)
            && send_monitor_stopped_event_) {
            u64 values[1] = {0};
            monitor_event (ZMQ_EVENT_MONITOR_STOPPED, values, 1,
                           endpoint_uri_pair_t ());
        }
        zmq_close (_monitor_socket);
        _monitor_socket = NULL;
        _monitor_events = 0;
    }
}

bool ZmqSocketBase::is_disconnected () const
{
    return _disconnected;
}

routing_socket_base_t::routing_socket_base_t (class ZmqContext *parent_,
                                                   uint32_t tid_,
                                                   sid_: i32) :
    ZmqSocketBase (parent_, tid_, sid_)
{
}

routing_socket_base_t::~routing_socket_base_t ()
{
    zmq_assert (_out_pipes.empty ());
}

int routing_socket_base_t::xsetsockopt (option_: i32,
                                             const optval_: *mut c_void,
                                             optvallen_: usize)
{
    switch (option_) {
        case ZMQ_CONNECT_ROUTING_ID:
            // TODO why isn't it possible to set an empty connect_routing_id
            //   (which is the default value)
            if (optval_ && optvallen_) {
                _connect_routing_id.assign (static_cast<const char *> (optval_),
                                            optvallen_);
                return 0;
            }
            break;
    }
    errno = EINVAL;
    return -1;
}

void routing_socket_base_t::xwrite_activated (pipe_t *pipe_)
{
    const out_pipes_t::iterator end = _out_pipes.end ();
    out_pipes_t::iterator it;
    for (it = _out_pipes.begin (); it != end; ++it)
        if (it->second.pipe == pipe_)
            break;

    zmq_assert (it != end);
    zmq_assert (!it->second.active);
    it->second.active = true;
}

std::string routing_socket_base_t::extract_connect_routing_id ()
{
    std::string res = ZMQ_MOVE (_connect_routing_id);
    _connect_routing_id.clear ();
    return res;
}

bool routing_socket_base_t::connect_routing_id_is_set () const
{
    return !_connect_routing_id.is_empty();
}

void routing_socket_base_t::add_out_pipe (Blob routing_id_,
                                               pipe_t *pipe_)
{
    //  Add the record into output pipes lookup table
    const out_pipe_t outpipe = {pipe_, true};
    const bool ok =
      _out_pipes.ZMQ_MAP_INSERT_OR_EMPLACE (ZMQ_MOVE (routing_id_), outpipe)
        .second;
    zmq_assert (ok);
}

bool routing_socket_base_t::has_out_pipe (const Blob &routing_id_) const
{
    return 0 != _out_pipes.count (routing_id_);
}

routing_socket_base_t::out_pipe_t *
routing_socket_base_t::lookup_out_pipe (const Blob &routing_id_)
{
    // TODO we could probably avoid constructor a temporary Blob to call this function
    out_pipes_t::iterator it = _out_pipes.find (routing_id_);
    return it == _out_pipes.end () ? NULL : &it->second;
}

const routing_socket_base_t::out_pipe_t *
routing_socket_base_t::lookup_out_pipe (const Blob &routing_id_) const
{
    // TODO we could probably avoid constructor a temporary Blob to call this function
    const out_pipes_t::const_iterator it = _out_pipes.find (routing_id_);
    return it == _out_pipes.end () ? NULL : &it->second;
}

void routing_socket_base_t::erase_out_pipe (const pipe_t *pipe_)
{
    const size_t erased = _out_pipes.erase (pipe_->get_routing_id ());
    zmq_assert (erased);
}

routing_socket_base_t::out_pipe_t
routing_socket_base_t::try_erase_out_pipe (const Blob &routing_id_)
{
    const out_pipes_t::iterator it = _out_pipes.find (routing_id_);
    out_pipe_t res = {NULL, false};
    if (it != _out_pipes.end ()) {
        res = it->second;
        _out_pipes.erase (it);
    }
    return res;
}
