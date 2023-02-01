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

// #include "precompiled.hpp"
// #include <string.h>
// #include <stdarg.h>

// #include "object.hpp"
// #include "ctx.hpp"
// #include "err.hpp"
// #include "pipe.hpp"
// #include "io_thread.hpp"
// #include "session_base.hpp"
// #include "socket_base.hpp"
pub struct object_t
{
// public:
    object_t (zmq::ZmqContext *ctx_, uint32_t tid_);
    object_t (object_t *parent_);
    virtual ~object_t ();

    uint32_t get_tid () const;
    void set_tid (uint32_t id_);
    ZmqContext *get_ctx () const;
    void process_command (const zmq::command_t &cmd_);
    void send_inproc_connected (zmq::socket_base_t *socket_);
    void send_bind (zmq::own_t *destination_,
                    zmq::pipe_t *pipe_,
                    bool inc_seqnum_ = true);

  protected:
    //  Using following function, socket is able to access global
    //  repository of inproc endpoints.
    int register_endpoint (addr_: *const c_char, const zmq::endpoint_t &endpoint_);
    int unregister_endpoint (const std::string &addr_, socket_base_t *socket_);
    void unregister_endpoints (zmq::socket_base_t *socket_);
    zmq::endpoint_t find_endpoint (addr_: *const c_char) const;
    void pend_connection (const std::string &addr_,
                          const endpoint_t &endpoint_,
                          pipe_t **pipes_);
    void connect_pending (addr_: *const c_char, zmq::socket_base_t *bind_socket_);

    void destroy_socket (zmq::socket_base_t *socket_);

    //  Logs an message.
    void log (format_: *const c_char, ...);

    //  Chooses least loaded I/O thread.
    zmq::io_thread_t *choose_io_thread (uint64_t affinity_) const;

    //  Derived object can use these functions to send commands
    //  to other objects.
    void send_stop ();
    void send_plug (zmq::own_t *destination_, bool inc_seqnum_ = true);
    void send_own (zmq::own_t *destination_, zmq::own_t *object_);
    void send_attach (zmq::session_base_t *destination_,
                      zmq::i_engine *engine_,
                      bool inc_seqnum_ = true);
    void send_activate_read (zmq::pipe_t *destination_);
    void send_activate_write (zmq::pipe_t *destination_, uint64_t msgs_read_);
    void send_hiccup (zmq::pipe_t *destination_, pipe_: *mut c_void);
    void send_pipe_peer_stats (zmq::pipe_t *destination_,
                               queue_count_: u64,
                               zmq::own_t *socket_base,
                               endpoint_uri_pair_t *endpoint_pair_);
    void send_pipe_stats_publish (zmq::own_t *destination_,
                                  outbound_queue_count_: u64,
                                  inbound_queue_count_: u64,
                                  endpoint_uri_pair_t *endpoint_pair_);
    void send_pipe_term (zmq::pipe_t *destination_);
    void send_pipe_term_ack (zmq::pipe_t *destination_);
    void send_pipe_hwm (zmq::pipe_t *destination_, inhwm_: i32, outhwm_: i32);
    void send_term_req (zmq::own_t *destination_, zmq::own_t *object_);
    void send_term (zmq::own_t *destination_, linger_: i32);
    void send_term_ack (zmq::own_t *destination_);
    void send_term_endpoint (own_t *destination_, std::string *endpoint_);
    void send_reap (zmq::socket_base_t *socket_);
    void send_reaped ();
    void send_done ();
    void send_conn_failed (zmq::session_base_t *destination_);


    //  These handlers can be overridden by the derived objects. They are
    //  called when command arrives from another thread.
    virtual void process_stop ();
    virtual void process_plug ();
    virtual void process_own (zmq::own_t *object_);
    virtual void process_attach (zmq::i_engine *engine_);
    virtual void process_bind (zmq::pipe_t *pipe_);
    virtual void process_activate_read ();
    virtual void process_activate_write (uint64_t msgs_read_);
    virtual void process_hiccup (pipe_: *mut c_void);
    virtual void process_pipe_peer_stats (queue_count_: u64,
                                          zmq::own_t *socket_base_,
                                          endpoint_uri_pair_t *endpoint_pair_);
    virtual void
    process_pipe_stats_publish (outbound_queue_count_: u64,
                                inbound_queue_count_: u64,
                                endpoint_uri_pair_t *endpoint_pair_);
    virtual void process_pipe_term ();
    virtual void process_pipe_term_ack ();
    virtual void process_pipe_hwm (inhwm_: i32, outhwm_: i32);
    virtual void process_term_req (zmq::own_t *object_);
    virtual void process_term (linger_: i32);
    virtual void process_term_ack ();
    virtual void process_term_endpoint (std::string *endpoint_);
    virtual void process_reap (zmq::socket_base_t *socket_);
    virtual void process_reaped ();
    virtual void process_conn_failed ();


    //  Special handler called after a command that requires a seqnum
    //  was processed. The implementation should catch up with its counter
    //  of processed commands here.
    virtual void process_seqnum ();

  // private:
    //  Context provides access to the global state.
    zmq::ZmqContext *const _ctx;

    //  Thread ID of the thread the object belongs to.
    uint32_t _tid;

    void send_command (const command_t &cmd_);

    ZMQ_NON_COPYABLE_NOR_MOVABLE (object_t)
};

zmq::object_t::object_t (ZmqContext *ctx_, uint32_t tid_) : _ctx (ctx_), _tid (tid_)
{
}

zmq::object_t::object_t (object_t *parent_) :
    _ctx (parent_->_ctx), _tid (parent_->_tid)
{
}

zmq::object_t::~object_t ()
{
}

uint32_t zmq::object_t::get_tid () const
{
    return _tid;
}

void zmq::object_t::set_tid (uint32_t id_)
{
    _tid = id_;
}

zmq::ZmqContext *zmq::object_t::get_ctx () const
{
    return _ctx;
}

void zmq::object_t::process_command (const command_t &cmd_)
{
    switch (cmd_.type) {
        case command_t::activate_read:
            process_activate_read ();
            break;

        case command_t::activate_write:
            process_activate_write (cmd_.args.activate_write.msgs_read);
            break;

        case command_t::stop:
            process_stop ();
            break;

        case command_t::plug:
            process_plug ();
            process_seqnum ();
            break;

        case command_t::own:
            process_own (cmd_.args.own.object);
            process_seqnum ();
            break;

        case command_t::attach:
            process_attach (cmd_.args.attach.engine);
            process_seqnum ();
            break;

        case command_t::bind:
            process_bind (cmd_.args.bind.pipe);
            process_seqnum ();
            break;

        case command_t::hiccup:
            process_hiccup (cmd_.args.hiccup.pipe);
            break;

        case command_t::pipe_peer_stats:
            process_pipe_peer_stats (cmd_.args.pipe_peer_stats.queue_count,
                                     cmd_.args.pipe_peer_stats.socket_base,
                                     cmd_.args.pipe_peer_stats.endpoint_pair);
            break;

        case command_t::pipe_stats_publish:
            process_pipe_stats_publish (
              cmd_.args.pipe_stats_publish.outbound_queue_count,
              cmd_.args.pipe_stats_publish.inbound_queue_count,
              cmd_.args.pipe_stats_publish.endpoint_pair);
            break;

        case command_t::pipe_term:
            process_pipe_term ();
            break;

        case command_t::pipe_term_ack:
            process_pipe_term_ack ();
            break;

        case command_t::pipe_hwm:
            process_pipe_hwm (cmd_.args.pipe_hwm.inhwm,
                              cmd_.args.pipe_hwm.outhwm);
            break;

        case command_t::term_req:
            process_term_req (cmd_.args.term_req.object);
            break;

        case command_t::term:
            process_term (cmd_.args.term.linger);
            break;

        case command_t::term_ack:
            process_term_ack ();
            break;

        case command_t::term_endpoint:
            process_term_endpoint (cmd_.args.term_endpoint.endpoint);
            break;

        case command_t::reap:
            process_reap (cmd_.args.reap.socket);
            break;

        case command_t::reaped:
            process_reaped ();
            break;

        case command_t::inproc_connected:
            process_seqnum ();
            break;

        case command_t::conn_failed:
            process_conn_failed ();
            break;

        case command_t::done:
        default:
            zmq_assert (false);
    }
}

int zmq::object_t::register_endpoint (addr_: *const c_char,
                                      const endpoint_t &endpoint_)
{
    return _ctx->register_endpoint (addr_, endpoint_);
}

int zmq::object_t::unregister_endpoint (const std::string &addr_,
                                        socket_base_t *socket_)
{
    return _ctx->unregister_endpoint (addr_, socket_);
}

void zmq::object_t::unregister_endpoints (socket_base_t *socket_)
{
    return _ctx->unregister_endpoints (socket_);
}

zmq::endpoint_t zmq::object_t::find_endpoint (addr_: *const c_char) const
{
    return _ctx->find_endpoint (addr_);
}

void zmq::object_t::pend_connection (const std::string &addr_,
                                     const endpoint_t &endpoint_,
                                     pipe_t **pipes_)
{
    _ctx->pend_connection (addr_, endpoint_, pipes_);
}

void zmq::object_t::connect_pending (addr_: *const c_char,
                                     zmq::socket_base_t *bind_socket_)
{
    return _ctx->connect_pending (addr_, bind_socket_);
}

void zmq::object_t::destroy_socket (socket_base_t *socket_)
{
    _ctx->destroy_socket (socket_);
}

zmq::io_thread_t *zmq::object_t::choose_io_thread (uint64_t affinity_) const
{
    return _ctx->choose_io_thread (affinity_);
}

void zmq::object_t::send_stop ()
{
    //  'stop' command goes always from administrative thread to
    //  the current object.
    command_t cmd;
    cmd.destination = this;
    cmd.type = command_t::stop;
    _ctx->send_command (_tid, cmd);
}

void zmq::object_t::send_plug (own_t *destination_, bool inc_seqnum_)
{
    if (inc_seqnum_)
        destination_->inc_seqnum ();

    command_t cmd;
    cmd.destination = destination_;
    cmd.type = command_t::plug;
    send_command (cmd);
}

void zmq::object_t::send_own (own_t *destination_, own_t *object_)
{
    destination_->inc_seqnum ();
    command_t cmd;
    cmd.destination = destination_;
    cmd.type = command_t::own;
    cmd.args.own.object = object_;
    send_command (cmd);
}

void zmq::object_t::send_attach (session_base_t *destination_,
                                 i_engine *engine_,
                                 bool inc_seqnum_)
{
    if (inc_seqnum_)
        destination_->inc_seqnum ();

    command_t cmd;
    cmd.destination = destination_;
    cmd.type = command_t::attach;
    cmd.args.attach.engine = engine_;
    send_command (cmd);
}

void zmq::object_t::send_conn_failed (session_base_t *destination_)
{
    command_t cmd;
    cmd.destination = destination_;
    cmd.type = command_t::conn_failed;
    send_command (cmd);
}

void zmq::object_t::send_bind (own_t *destination_,
                               pipe_t *pipe_,
                               bool inc_seqnum_)
{
    if (inc_seqnum_)
        destination_->inc_seqnum ();

    command_t cmd;
    cmd.destination = destination_;
    cmd.type = command_t::bind;
    cmd.args.bind.pipe = pipe_;
    send_command (cmd);
}

void zmq::object_t::send_activate_read (pipe_t *destination_)
{
    command_t cmd;
    cmd.destination = destination_;
    cmd.type = command_t::activate_read;
    send_command (cmd);
}

void zmq::object_t::send_activate_write (pipe_t *destination_,
                                         uint64_t msgs_read_)
{
    command_t cmd;
    cmd.destination = destination_;
    cmd.type = command_t::activate_write;
    cmd.args.activate_write.msgs_read = msgs_read_;
    send_command (cmd);
}

void zmq::object_t::send_hiccup (pipe_t *destination_, pipe_: *mut c_void)
{
    command_t cmd;
    cmd.destination = destination_;
    cmd.type = command_t::hiccup;
    cmd.args.hiccup.pipe = pipe_;
    send_command (cmd);
}

void zmq::object_t::send_pipe_peer_stats (pipe_t *destination_,
                                          queue_count_: u64,
                                          own_t *socket_base_,
                                          endpoint_uri_pair_t *endpoint_pair_)
{
    command_t cmd;
    cmd.destination = destination_;
    cmd.type = command_t::pipe_peer_stats;
    cmd.args.pipe_peer_stats.queue_count = queue_count_;
    cmd.args.pipe_peer_stats.socket_base = socket_base_;
    cmd.args.pipe_peer_stats.endpoint_pair = endpoint_pair_;
    send_command (cmd);
}

void zmq::object_t::send_pipe_stats_publish (
  own_t *destination_,
  outbound_queue_count_: u64,
  inbound_queue_count_: u64,
  endpoint_uri_pair_t *endpoint_pair_)
{
    command_t cmd;
    cmd.destination = destination_;
    cmd.type = command_t::pipe_stats_publish;
    cmd.args.pipe_stats_publish.outbound_queue_count = outbound_queue_count_;
    cmd.args.pipe_stats_publish.inbound_queue_count = inbound_queue_count_;
    cmd.args.pipe_stats_publish.endpoint_pair = endpoint_pair_;
    send_command (cmd);
}

void zmq::object_t::send_pipe_term (pipe_t *destination_)
{
    command_t cmd;
    cmd.destination = destination_;
    cmd.type = command_t::pipe_term;
    send_command (cmd);
}

void zmq::object_t::send_pipe_term_ack (pipe_t *destination_)
{
    command_t cmd;
    cmd.destination = destination_;
    cmd.type = command_t::pipe_term_ack;
    send_command (cmd);
}

void zmq::object_t::send_pipe_hwm (pipe_t *destination_,
                                   inhwm_: i32,
                                   outhwm_: i32)
{
    command_t cmd;
    cmd.destination = destination_;
    cmd.type = command_t::pipe_hwm;
    cmd.args.pipe_hwm.inhwm = inhwm_;
    cmd.args.pipe_hwm.outhwm = outhwm_;
    send_command (cmd);
}

void zmq::object_t::send_term_req (own_t *destination_, own_t *object_)
{
    command_t cmd;
    cmd.destination = destination_;
    cmd.type = command_t::term_req;
    cmd.args.term_req.object = object_;
    send_command (cmd);
}

void zmq::object_t::send_term (own_t *destination_, linger_: i32)
{
    command_t cmd;
    cmd.destination = destination_;
    cmd.type = command_t::term;
    cmd.args.term.linger = linger_;
    send_command (cmd);
}

void zmq::object_t::send_term_ack (own_t *destination_)
{
    command_t cmd;
    cmd.destination = destination_;
    cmd.type = command_t::term_ack;
    send_command (cmd);
}

void zmq::object_t::send_term_endpoint (own_t *destination_,
                                        std::string *endpoint_)
{
    command_t cmd;
    cmd.destination = destination_;
    cmd.type = command_t::term_endpoint;
    cmd.args.term_endpoint.endpoint = endpoint_;
    send_command (cmd);
}

void zmq::object_t::send_reap (class socket_base_t *socket_)
{
    command_t cmd;
    cmd.destination = _ctx->get_reaper ();
    cmd.type = command_t::reap;
    cmd.args.reap.socket = socket_;
    send_command (cmd);
}

void zmq::object_t::send_reaped ()
{
    command_t cmd;
    cmd.destination = _ctx->get_reaper ();
    cmd.type = command_t::reaped;
    send_command (cmd);
}

void zmq::object_t::send_inproc_connected (zmq::socket_base_t *socket_)
{
    command_t cmd;
    cmd.destination = socket_;
    cmd.type = command_t::inproc_connected;
    send_command (cmd);
}

void zmq::object_t::send_done ()
{
    command_t cmd;
    cmd.destination = NULL;
    cmd.type = command_t::done;
    _ctx->send_command (ZmqContext::term_tid, cmd);
}

void zmq::object_t::process_stop ()
{
    zmq_assert (false);
}

void zmq::object_t::process_plug ()
{
    zmq_assert (false);
}

void zmq::object_t::process_own (own_t *)
{
    zmq_assert (false);
}

void zmq::object_t::process_attach (i_engine *)
{
    zmq_assert (false);
}

void zmq::object_t::process_bind (pipe_t *)
{
    zmq_assert (false);
}

void zmq::object_t::process_activate_read ()
{
    zmq_assert (false);
}

void zmq::object_t::process_activate_write (uint64_t)
{
    zmq_assert (false);
}

void zmq::object_t::process_hiccup (void *)
{
    zmq_assert (false);
}

void zmq::object_t::process_pipe_peer_stats (uint64_t,
                                             own_t *,
                                             endpoint_uri_pair_t *)
{
    zmq_assert (false);
}

void zmq::object_t::process_pipe_stats_publish (uint64_t,
                                                uint64_t,
                                                endpoint_uri_pair_t *)
{
    zmq_assert (false);
}

void zmq::object_t::process_pipe_term ()
{
    zmq_assert (false);
}

void zmq::object_t::process_pipe_term_ack ()
{
    zmq_assert (false);
}

void zmq::object_t::process_pipe_hwm (int, int)
{
    zmq_assert (false);
}

void zmq::object_t::process_term_req (own_t *)
{
    zmq_assert (false);
}

void zmq::object_t::process_term (int)
{
    zmq_assert (false);
}

void zmq::object_t::process_term_ack ()
{
    zmq_assert (false);
}

void zmq::object_t::process_term_endpoint (std::string *)
{
    zmq_assert (false);
}

void zmq::object_t::process_reap (class socket_base_t *)
{
    zmq_assert (false);
}

void zmq::object_t::process_reaped ()
{
    zmq_assert (false);
}

void zmq::object_t::process_seqnum ()
{
    zmq_assert (false);
}

void zmq::object_t::process_conn_failed ()
{
    zmq_assert (false);
}

void zmq::object_t::send_command (const command_t &cmd_)
{
    _ctx->send_command (cmd_.destination->get_tid (), cmd_);
}
