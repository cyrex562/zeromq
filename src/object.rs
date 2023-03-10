use crate::command::{CommandType, ZmqCommand};
use crate::context::ZmqContext;

#[derive(Default,Debug,Clone)]
pub struct object_t {
    //  Context provides access to the global state.
    // ZmqContext *const _ctx;
    ctx: *const ZmqContext,

    //  Thread ID of the thread the object belongs to.
    // uint32_t _tid;
    tid: u32,

    // ZMQ_NON_COPYABLE_NOR_MOVABLE (object_t)
}

impl object_t {
    // object_t (ZmqContext *ctx_, uint32_t tid_);
    // object_t::object_t (ZmqContext *ctx_, uint32_t tid_) : _ctx (ctx_), _tid (tid_)
    // {
    // }
    pub fn new(ctx: *mut ZmqContext, tid: u32) -> Self {
        Self {
            ctx,
            tid
        }
    }

    // object_t (object_t *parent_);
    pub fn from_parent(parent: *mut Self) -> Self {
        Self {
            ctx: parent.ctx,
            tid: parent.tid
        }
    }

    // virtual ~object_t ();

    // uint32_t get_tid () const;
    pub fn tid(&self) -> u32 {
        self.tid
    }

    // void set_tid (uint32_t id_);
    pub fn set_tid(&mut self, id_in: u32) {
        self.tid = id_in
    }

    // ZmqContext *get_ctx () const;
    pub fn ctx(&self) -> *const ZmqContext {
        self.ctx
    }

    // void process_command (const command_t &cmd_);
    pub fn process_command(&mut self, cmd: &ZmqCommand) {
        match cmd.type_ {
            CommandType::stop => {}
            CommandType::plug => {}
            CommandType::own => {}
            CommandType::attach => {}
            CommandType::bind => {}
            CommandType::activate_read => {}
            CommandType::activate_write => {}
            CommandType::hiccup => {}
            CommandType::pipe_term => {}
            CommandType::pipe_term_ack => {}
            CommandType::pipe_hwm => {}
            CommandType::term_req => {}
            CommandType::term => {}
            CommandType::term_ack => {}
            CommandType::term_endpoint => {}
            CommandType::reap => {}
            CommandType::reaped => {}
            CommandType::inproc_connected => {}
            CommandType::conn_failed => {}
            CommandType::pipe_peer_stats => {}
            CommandType::pipe_stats_publish => {}
            CommandType::done => {}
        }
    }



    //  Using following function, socket is able to access global
    //  repository of inproc endpoints.
    // int register_endpoint (addr_: *const c_char, const endpoint_t &endpoint_);
    pub fn register_endpoint(&mut self, addr: &str, )

    // int unregister_endpoint (const std::string &addr_, ZmqSocketBase *socket_);

    // void unregister_endpoints (ZmqSocketBase *socket_);

    // endpoint_t find_endpoint (addr_: *const c_char) const;

    // void pend_connection (const std::string &addr_,
    //                       const endpoint_t &endpoint_,
    //                       pipe_t **pipes_);

    // void connect_pending (addr_: *const c_char, ZmqSocketBase *bind_socket_);

    // void destroy_socket (ZmqSocketBase *socket_);

    //  Logs an message.
    // void log (format_: *const c_char, ...);

    // void send_inproc_connected (ZmqSocketBase *socket_);

    // void send_bind (own_t *destination_,
    //                 pipe_t *pipe_,
    //                 bool inc_seqnum_ = true);

    //  Chooses least loaded I/O thread.
    // io_thread_t *choose_io_thread (uint64_t affinity_) const;

    //  Derived object can use these functions to send commands
    //  to other objects.
    // void send_stop ();

    // void send_plug (own_t *destination_, bool inc_seqnum_ = true);

    // void send_own (own_t *destination_, own_t *object_);

    // void send_attach (session_base_t *destination_,
    //                   i_engine *engine_,
    //                   bool inc_seqnum_ = true);

    // void send_activate_read (pipe_t *destination_);

    // void send_activate_write (pipe_t *destination_, uint64_t msgs_read_);

    // void send_hiccup (pipe_t *destination_, pipe_: *mut c_void);

    // void send_pipe_peer_stats (pipe_t *destination_,
    //                            queue_count_: u64,
    //                            own_t *socket_base,
    //                            endpoint_uri_pair_t *endpoint_pair_);

    // void send_pipe_stats_publish (own_t *destination_,
    //                               outbound_queue_count_: u64,
    //                               inbound_queue_count_: u64,
    //                               endpoint_uri_pair_t *endpoint_pair_);

    // void send_pipe_term (pipe_t *destination_);

    // void send_pipe_term_ack (pipe_t *destination_);

    // void send_pipe_hwm (pipe_t *destination_, inhwm_: i32, outhwm_: i32);

    // void send_term_req (own_t *destination_, own_t *object_);

    // void send_term (own_t *destination_, linger_: i32);

    // void send_term_ack (own_t *destination_);

    // void send_term_endpoint (own_t *destination_, std::string *endpoint_);

    // void send_reap (ZmqSocketBase *socket_);

    // void send_reaped ();

    // void send_done ();

    // void send_conn_failed (session_base_t *destination_);


    //  These handlers can be overridden by the derived objects. They are
    //  called when command arrives from another thread.
    // virtual void process_stop ();

    // virtual void process_plug ();

    // virtual void process_own (own_t *object_);

    // virtual void process_attach (i_engine *engine_);

    // virtual void process_bind (pipe_t *pipe_);

    // virtual void process_activate_read ();

    // virtual void process_activate_write (uint64_t msgs_read_);

    // virtual void process_hiccup (pipe_: *mut c_void);

    // virtual void process_pipe_peer_stats (queue_count_: u64,
    //                                       own_t *socket_base_,
    //                                       endpoint_uri_pair_t *endpoint_pair_);

    // virtual void
    // process_pipe_stats_publish (outbound_queue_count_: u64,
    //                             inbound_queue_count_: u64,
    //                             endpoint_uri_pair_t *endpoint_pair_);

    // virtual void process_pipe_term ();

    // virtual void process_pipe_term_ack ();

    // virtual void process_pipe_hwm (inhwm_: i32, outhwm_: i32);

    // virtual void process_term_req (own_t *object_);

    // virtual void process_term (linger_: i32);

    // virtual void process_term_ack ();

    // virtual void process_term_endpoint (std::string *endpoint_);

    // virtual void process_reap (ZmqSocketBase *socket_);

    // virtual void process_reaped ();

    // virtual void process_conn_failed ();


    //  Special handler called after a command that requires a seqnum
    //  was processed. The implementation should catch up with its counter
    //  of processed commands here.
    // virtual void process_seqnum ();

    // void send_command (const command_t &cmd_);
}

void object_t::process_command (const ZmqCommand &cmd_)
{
    switch (cmd_.type) {
        case ZmqCommand::activate_read:
            process_activate_read ();
            break;

        case ZmqCommand::activate_write:
            process_activate_write (cmd_.args.activate_write.msgs_read);
            break;

        case ZmqCommand::stop:
            process_stop ();
            break;

        case ZmqCommand::plug:
            process_plug ();
            process_seqnum ();
            break;

        case ZmqCommand::own:
            process_own (cmd_.args.own.object);
            process_seqnum ();
            break;

        case ZmqCommand::attach:
            process_attach (cmd_.args.attach.engine);
            process_seqnum ();
            break;

        case ZmqCommand::bind:
            process_bind (cmd_.args.bind.pipe);
            process_seqnum ();
            break;

        case ZmqCommand::hiccup:
            process_hiccup (cmd_.args.hiccup.pipe);
            break;

        case ZmqCommand::pipe_peer_stats:
            process_pipe_peer_stats (cmd_.args.pipe_peer_stats.queue_count,
                                     cmd_.args.pipe_peer_stats.socket_base,
                                     cmd_.args.pipe_peer_stats.endpoint_pair);
            break;

        case ZmqCommand::pipe_stats_publish:
            process_pipe_stats_publish (
              cmd_.args.pipe_stats_publish.outbound_queue_count,
              cmd_.args.pipe_stats_publish.inbound_queue_count,
              cmd_.args.pipe_stats_publish.endpoint_pair);
            break;

        case ZmqCommand::pipe_term:
            process_pipe_term ();
            break;

        case ZmqCommand::pipe_term_ack:
            process_pipe_term_ack ();
            break;

        case ZmqCommand::pipe_hwm:
            process_pipe_hwm (cmd_.args.pipe_hwm.inhwm,
                              cmd_.args.pipe_hwm.outhwm);
            break;

        case ZmqCommand::term_req:
            process_term_req (cmd_.args.term_req.object);
            break;

        case ZmqCommand::term:
            process_term (cmd_.args.term.linger);
            break;

        case ZmqCommand::term_ack:
            process_term_ack ();
            break;

        case ZmqCommand::term_endpoint:
            process_term_endpoint (cmd_.args.term_endpoint.endpoint);
            break;

        case ZmqCommand::reap:
            process_reap (cmd_.args.reap.socket);
            break;

        case ZmqCommand::reaped:
            process_reaped ();
            break;

        case ZmqCommand::inproc_connected:
            process_seqnum ();
            break;

        case ZmqCommand::conn_failed:
            process_conn_failed ();
            break;

        case ZmqCommand::done:
        default:
            zmq_assert (false);
    }
}

int object_t::register_endpoint (addr_: *const c_char,
                                      const ZmqEndpoint &endpoint_)
{
    return _ctx.register_endpoint (addr_, endpoint_);
}

int object_t::unregister_endpoint (const std::string &addr_,
                                        ZmqSocketBase *socket_)
{
    return _ctx.unregister_endpoint (addr_, socket_);
}

void object_t::unregister_endpoints (ZmqSocketBase *socket_)
{
    return _ctx.unregister_endpoints (socket_);
}

ZmqEndpoint object_t::find_endpoint (addr_: &str) const
{
    return _ctx.find_endpoint (addr_);
}

void object_t::pend_connection (const std::string &addr_,
                                     const ZmqEndpoint &endpoint_,
                                     pipe_t **pipes_)
{
    _ctx.pend_connection (addr_, endpoint_, pipes_);
}

void object_t::connect_pending (addr_: *const c_char,
                                     ZmqSocketBase *bind_socket_)
{
    return _ctx.connect_pending (addr_, bind_socket_);
}

void object_t::destroy_socket (ZmqSocketBase *socket_)
{
    _ctx.destroy_socket (socket_);
}

io_thread_t *object_t::choose_io_thread (u64 affinity_) const
{
    return _ctx.choose_io_thread (affinity_);
}

void object_t::send_stop ()
{
    //  'stop' command goes always from administrative thread to
    //  the current object.
    ZmqCommand cmd;
    cmd.destination = this;
    cmd.type = ZmqCommand::stop;
    _ctx.send_command (_tid, cmd);
}

void object_t::send_plug (own_t *destination_, inc_seqnum_: bool)
{
    if (inc_seqnum_)
        destination_.inc_seqnum ();

    ZmqCommand cmd;
    cmd.destination = destination_;
    cmd.type = ZmqCommand::plug;
    send_command (cmd);
}

void object_t::send_own (own_t *destination_, own_t *object_)
{
    destination_.inc_seqnum ();
    ZmqCommand cmd;
    cmd.destination = destination_;
    cmd.type = ZmqCommand::own;
    cmd.args.own.object = object_;
    send_command (cmd);
}

void object_t::send_attach (session_base_t *destination_,
                                 i_engine *engine_,
                                 inc_seqnum_: bool)
{
    if (inc_seqnum_)
        destination_.inc_seqnum ();

    ZmqCommand cmd;
    cmd.destination = destination_;
    cmd.type = ZmqCommand::attach;
    cmd.args.attach.engine = engine_;
    send_command (cmd);
}

void object_t::send_conn_failed (session_base_t *destination_)
{
    ZmqCommand cmd;
    cmd.destination = destination_;
    cmd.type = ZmqCommand::conn_failed;
    send_command (cmd);
}

void object_t::send_bind (own_t *destination_,
                               pipe_t *pipe_,
                               inc_seqnum_: bool)
{
    if (inc_seqnum_)
        destination_.inc_seqnum ();

    ZmqCommand cmd;
    cmd.destination = destination_;
    cmd.type = ZmqCommand::bind;
    cmd.args.bind.pipe = pipe_;
    send_command (cmd);
}

void object_t::send_activate_read (pipe_t *destination_)
{
    ZmqCommand cmd;
    cmd.destination = destination_;
    cmd.type = ZmqCommand::activate_read;
    send_command (cmd);
}

void object_t::send_activate_write (pipe_t *destination_,
                                         u64 msgs_read_)
{
    ZmqCommand cmd;
    cmd.destination = destination_;
    cmd.type = ZmqCommand::activate_write;
    cmd.args.activate_write.msgs_read = msgs_read_;
    send_command (cmd);
}

void object_t::send_hiccup (pipe_t *destination_, pipe_: *mut c_void)
{
    ZmqCommand cmd;
    cmd.destination = destination_;
    cmd.type = ZmqCommand::hiccup;
    cmd.args.hiccup.pipe = pipe_;
    send_command (cmd);
}

void object_t::send_pipe_peer_stats (pipe_t *destination_,
                                          queue_count_: u64,
                                          own_t *socket_base_,
                                          EndpointUriPair *endpoint_pair_)
{
    ZmqCommand cmd;
    cmd.destination = destination_;
    cmd.type = ZmqCommand::pipe_peer_stats;
    cmd.args.pipe_peer_stats.queue_count = queue_count_;
    cmd.args.pipe_peer_stats.socket_base = socket_base_;
    cmd.args.pipe_peer_stats.endpoint_pair = endpoint_pair_;
    send_command (cmd);
}

void object_t::send_pipe_stats_publish (
  own_t *destination_,
  outbound_queue_count_: u64,
  inbound_queue_count_: u64,
  EndpointUriPair *endpoint_pair_)
{
    ZmqCommand cmd;
    cmd.destination = destination_;
    cmd.type = ZmqCommand::pipe_stats_publish;
    cmd.args.pipe_stats_publish.outbound_queue_count = outbound_queue_count_;
    cmd.args.pipe_stats_publish.inbound_queue_count = inbound_queue_count_;
    cmd.args.pipe_stats_publish.endpoint_pair = endpoint_pair_;
    send_command (cmd);
}

void object_t::send_pipe_term (pipe_t *destination_)
{
    ZmqCommand cmd;
    cmd.destination = destination_;
    cmd.type = ZmqCommand::pipe_term;
    send_command (cmd);
}

void object_t::send_pipe_term_ack (pipe_t *destination_)
{
    ZmqCommand cmd;
    cmd.destination = destination_;
    cmd.type = ZmqCommand::pipe_term_ack;
    send_command (cmd);
}

void object_t::send_pipe_hwm (pipe_t *destination_,
                                   inhwm_: i32,
                                   outhwm_: i32)
{
    ZmqCommand cmd;
    cmd.destination = destination_;
    cmd.type = ZmqCommand::pipe_hwm;
    cmd.args.pipe_hwm.inhwm = inhwm_;
    cmd.args.pipe_hwm.outhwm = outhwm_;
    send_command (cmd);
}

void object_t::send_term_req (own_t *destination_, own_t *object_)
{
    ZmqCommand cmd;
    cmd.destination = destination_;
    cmd.type = ZmqCommand::term_req;
    cmd.args.term_req.object = object_;
    send_command (cmd);
}

void object_t::send_term (own_t *destination_, linger_: i32)
{
    ZmqCommand cmd;
    cmd.destination = destination_;
    cmd.type = ZmqCommand::term;
    cmd.args.term.linger = linger_;
    send_command (cmd);
}

void object_t::send_term_ack (own_t *destination_)
{
    ZmqCommand cmd;
    cmd.destination = destination_;
    cmd.type = ZmqCommand::term_ack;
    send_command (cmd);
}

void object_t::send_term_endpoint (own_t *destination_,
                                        std::string *endpoint_)
{
    ZmqCommand cmd;
    cmd.destination = destination_;
    cmd.type = ZmqCommand::term_endpoint;
    cmd.args.term_endpoint.endpoint = endpoint_;
    send_command (cmd);
}

void object_t::send_reap (class ZmqSocketBase *socket_)
{
    ZmqCommand cmd;
    cmd.destination = _ctx.get_reaper ();
    cmd.type = ZmqCommand::reap;
    cmd.args.reap.socket = socket_;
    send_command (cmd);
}

void object_t::send_reaped ()
{
    ZmqCommand cmd;
    cmd.destination = _ctx.get_reaper ();
    cmd.type = ZmqCommand::reaped;
    send_command (cmd);
}

void object_t::send_inproc_connected (ZmqSocketBase *socket_)
{
    ZmqCommand cmd;
    cmd.destination = socket_;
    cmd.type = ZmqCommand::inproc_connected;
    send_command (cmd);
}

void object_t::send_done ()
{
    ZmqCommand cmd;
    cmd.destination = null_mut();
    cmd.type = ZmqCommand::done;
    _ctx.send_command (ZmqContext::TERM_TID, cmd);
}

void object_t::process_stop ()
{
    zmq_assert (false);
}

void object_t::process_plug ()
{
    zmq_assert (false);
}

void object_t::process_own (own_t *)
{
    zmq_assert (false);
}

void object_t::process_attach (i_engine *)
{
    zmq_assert (false);
}

void object_t::process_bind (pipe_t *)
{
    zmq_assert (false);
}

void object_t::process_activate_read ()
{
    zmq_assert (false);
}

void object_t::process_activate_write (u64)
{
    zmq_assert (false);
}

void object_t::process_hiccup (void *)
{
    zmq_assert (false);
}

void object_t::process_pipe_peer_stats (u64,
                                             own_t *,
                                             EndpointUriPair *)
{
    zmq_assert (false);
}

void object_t::process_pipe_stats_publish (u64,
                                                u64,
                                                EndpointUriPair *)
{
    zmq_assert (false);
}

void object_t::process_pipe_term ()
{
    zmq_assert (false);
}

void object_t::process_pipe_term_ack ()
{
    zmq_assert (false);
}

void object_t::process_pipe_hwm (int, int)
{
    zmq_assert (false);
}

void object_t::process_term_req (own_t *)
{
    zmq_assert (false);
}

void object_t::process_term (int)
{
    zmq_assert (false);
}

void object_t::process_term_ack ()
{
    zmq_assert (false);
}

void object_t::process_term_endpoint (std::string *)
{
    zmq_assert (false);
}

void object_t::process_reap (class ZmqSocketBase *)
{
    zmq_assert (false);
}

void object_t::process_reaped ()
{
    zmq_assert (false);
}

void object_t::process_seqnum ()
{
    zmq_assert (false);
}

void object_t::process_conn_failed ()
{
    zmq_assert (false);
}

void object_t::send_command (const ZmqCommand &cmd_)
{
    _ctx.send_command (cmd_.destination.get_tid (), cmd_);
}
