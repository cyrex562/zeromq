/*
    Copyright (c) 2007-2016 Contributors as noted in the AUTHORS file

    This file is part of libzmq, the ZeroMQ core engine in C+= 1.

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

// #include <new>

use std::mem;
use libc::EINTR;
use crate::command::ZmqCommand;
use crate::context::ZmqContext;
use crate::devpoll::Poller;
use crate::endpoint::{EndpointUriPair, ZmqEndpoint};
use crate::mailbox::ZmqMailbox;
use crate::object::ZmqObject;
use crate::own::ZmqOwn;
use crate::pipe::ZmqPipe;
use crate::session_base::ZmqSessionBase;
use crate::socket_base::ZmqSocketBase;

// #include "macros.hpp"
// #include "io_thread.hpp"
// #include "err.hpp"
// #include "ctx.hpp"
// pub struct ZmqThread  : public ZmqObject, public i_poll_events
#[derive(Default, Debug, Clone)]
pub struct ZmqThread {
    //
    //  I/O thread accesses incoming commands via this mailbox.
    pub mailbox: Option<ZmqMailbox>,
    //  Handle associated with mailbox' file descriptor.
    pub mailbox_handle: Option<handle_t>,
    //  I/O multiplexing is performed using a poller object.
    pub poller: Poller,
    // // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqThread)
    pub ctx: ZmqContext,
    //
    pub tid: u32,
}

impl ZmqThread {
    //
    // ZmqThread (ctx: &mut ZmqContext, tid: u32);
    // ZmqObject (ctx, tid),
    pub fn new(ctx: &mut ZmqContext, tid: u32) -> Self {
        let mut out = Self {
            mailbox: None,
            poller: Poller::new(ctx),
            mailbox_handle: None,
            ctx: ctx.clone(),
            tid: 0,

        };
        if out.mailbox.get_fd() != retired_fd {
            out.poller.add_fd(out.mailbox.get_fd(), &mut out);
            out.mailbox_handle = out.mailbox.get_fd();
            out.poller.set_pollin(&out.mailbox_handle);
        }
        out
    }

    // mailbox_handle (static_cast<Poller::handle_t> (null_mut()))
    // {
    // poller = new (std::nothrow) Poller (*ctx);
    // alloc_assert (poller);
    //
    // if (mailbox.get_fd () != retired_fd) {
    // mailbox_handle = poller.add_fd (mailbox.get_fd (), this);
    // poller.set_pollin (mailbox_handle);
    // }
    // }

    //  Clean-up. If the thread was started, it's necessary to call 'stop'
    //  before invoking destructor. Otherwise the destructor would hang up.
    // ~ZmqThread ();

    //  Launch the physical thread.
    // void start ();
    pub fn start(&mut self) {
        let mut name: String = String::new();
        // snprintf (name, mem::size_of::<name>(), "IO/%u",
        //           get_tid () - ZmqContext::REAPER_TID - 1);
        name = format!("IO/{}", get_tid() - ZmqContext::REAPER_TID - 1);
        //  Start the underlying I/O thread.
        self.poller.start(name);
    }

    //  Ask underlying thread to stop.
    // void stop ();
    pub fn stop(&mut self) {
        self.send_stop();
    }

    //  Returns mailbox associated with this I/O thread.
    // mailbox_t *get_mailbox ();
    pub fn get_mailbox(&mut self) -> &mut Mailbox {
        return &mut self.mailbox;
    }

    //  i_poll_events implementation.
    // void in_event ();
    pub fn in_event(&mut self) {
        //  TODO: Do we want to limit number of commands I/O thread can
        //  process in a single go?

        let mut cmd: ZmqCommand = ZmqCommand::default();
        let rc = mailbox.recv(&cmd, 0);

        while (rc == 0 || errno == EINTR) {
            if (rc == 0) {
                cmd.destination.process_command(&cmd);
            }
            rc = mailbox.recv(&cmd, 0);
        }

        // errno_assert (rc != 0 && errno == EAGAIN);
    }

    // void out_event ();
    pub fn out_event(&mut self) {
        //  We are never polling for POLLOUT here. This function is never called.
        // zmq_assert (false);
    }

    // void timer_event (id_: i32);
    pub fn timer_event(&mut self) {
        //  No timers here. This function is never called.
        // zmq_assert (false);
    }

    //  Used by io_objects to retrieve the associated poller object.
    // Poller *get_poller () const;
    pub fn get_poller(&mut self) -> &mut Poller {
        // zmq_assert (poller);
        return &mut self.poller;
    }

    //  Command handlers.
    // void process_stop ();
    pub fn process_stop(&mut self) {
        // zmq_assert (mailbox_handle);
        self.poller.rm_fd(self.mailbox_handle.unwrap());
        self.poller.stop();
    }

    //  Returns load experienced by the I/O thread.
    // int get_load () const;
    pub fn get_load(&mut self) {
        return self.poller.get_load();
    }
}

impl ZmqObject for ZmqThread {
    fn get_ctx(&self) -> &ZmqContext {
        todo!()
    }

    fn get_ctx_mut(&mut self) -> &mut ZmqContext {
        todo!()
    }

    fn set_ctx(&mut self, ctx: &mut ZmqContext) {
        todo!()
    }

    fn get_tid(&self) -> u32 {
        todo!()
    }

    fn set_tid(&mut self, tid: u32) {
        todo!()
    }

    fn process_command(&mut self, cmd: &ZmqCommand) {
        todo!()
    }

    fn register_endpoint(&mut self, addr: &str, endpoint: &mut ZmqEndpoint) -> anyhow::Result<()> {
        todo!()
    }

    fn unregister_endpoint(&mut self, addr: &str, sock_base: &mut ZmqSocketBase) -> anyhow::Result<()> {
        todo!()
    }

    fn unregister_endpoints(&mut self, sock_base: &mut ZmqSocketBase) {
        todo!()
    }

    fn find_endpoint(&self, addr: &str) -> Option<ZmqEndpoint> {
        todo!()
    }

    fn pend_connection(&mut self, addr: &str, endpoint: &ZmqEndpoint, pipes: &[ZmqPipe]) {
        todo!()
    }

    fn connect_pending(&self, addr: &str, bind_socket: &mut ZmqSocketBase) {
        todo!()
    }

    fn destroy_socket(&mut self, socket: &mut ZmqSocketBase) {
        todo!()
    }

    fn log(msg: &str) {
        todo!()
    }

    fn send_inproc_connected(&mut self, socket: &mut ZmqSocketBase) {
        todo!()
    }

    fn send_bind(&mut self, destination: &mut ZmqSocketBase, pipe: &mut ZmqPipe, inc_seqnum: bool) {
        todo!()
    }

    fn choose_io_thread(&mut self, affinity: u64) -> Option<ZmqThread> {
        todo!()
    }

    fn send_stop(&mut self) {
        todo!()
    }

    fn send_plug(&mut self, destination: &mut ZmqOwn, inc_seqnum: bool) {
        todo!()
    }

    fn send_own(&mut self, destination: &mut ZmqOwn, object: &mut ZmqOwn) {
        todo!()
    }

    fn send_attach(&mut self, destination: &mut ZmqSessionbase, engine: &mut ZmqEngineInterface, inc_seqnum: bool) {
        todo!()
    }

    fn send_activate_read(&mut self, destination: &mut ZmqPipe) {
        todo!()
    }

    fn send_activate_write(&mut self, destination: &mut ZmqPipe, msgs_read: u64) {
        todo!()
    }

    fn send_hiccup(&mut self, destination: &mut ZmqPipe, pipe: &mut [u8]) {
        todo!()
    }

    fn send_pipe_peer_stats(&mut self, destination: &mut ZmqPipe, queue_count: u64, socket_base: &mut ZmqOwn, endpoint_pair: &mut EndpointUriPair) {
        todo!()
    }

    fn send_pipe_stats_publish(&mut self, destination: &mut ZmqOwn, outbound_queue_count: u64, inbound_queue_count: u64, endpoint_pair: &mut EndpointUriPair) {
        todo!()
    }

    fn send_pipe_term(&mut self, destination: &mut ZmqPipe) {
        todo!()
    }

    fn send_pipe_term_ack(&mut self, destination: &mut ZmqPipe) {
        todo!()
    }

    fn send_pipe_hwm(&mut self, destination: &mut ZmqPipe, inhwm: i32, outhwm: i32) {
        todo!()
    }

    fn send_term_req(&mut self, destination: &mut ZmqOwn, object: &mut ZmqOwn) {
        todo!()
    }

    fn send_term(&mut self, destination: &mut ZmqOwn, linger: i32) {
        todo!()
    }

    fn send_term_ack(&mut self, destination: &mut ZmqOwn) {
        todo!()
    }

    fn send_term_endpoint(&mut self, destination: &mut ZmqOwn, endpoint: &str) {
        todo!()
    }

    fn send_reap(&mut self, socket: &mut ZmqSocketBase) {
        todo!()
    }

    fn send_reaped(&mut self) {
        todo!()
    }

    fn send_done(&mut self) {
        todo!()
    }

    fn send_conn_failed(&mut self, destination: &mut ZmqSessionBase) {
        todo!()
    }

    fn process_stop(&mut self) {
        todo!()
    }

    fn process_plug(&mut self) {
        todo!()
    }

    fn process_own(&mut self, object: &mut ZmqOwn) {
        todo!()
    }

    fn process_attached(&mut self, engine: &mut ZmqEngineInterface) {
        todo!()
    }

    fn process_bind(&mut self, pipe: &mut ZmqPipe) {
        todo!()
    }

    fn process_activate_read(&mut self) {
        todo!()
    }

    fn process_activate_write(&mut self, msgs_read: u64) {
        todo!()
    }

    fn process_hiccup(&mut self, pipe: &mut [u8]) {
        todo!()
    }

    fn process_pipe_peer_stats(&mut self, queue_count: u64, socket_base: &mut ZmqOwn, endpoint_pair: &mut EndpointUriPair) {
        todo!()
    }

    fn process_pipe_stats_publish(&mut self, outbound_queue_count: u64, inbound_queue_count: u64, endpoint_pair: &mut EndpointUriPair) {
        todo!()
    }

    fn process_pipe_term(&mut self) {
        todo!()
    }

    fn process_pipe_term_ack(&mut self) {
        todo!()
    }

    fn process_pipe_hwm(&mut self, inhwm: i32, outhwm: i32) {
        todo!()
    }

    fn process_term_req(&mut self, object: &mut ZmqOwn) {
        todo!()
    }

    fn process_term(&mut self, linger: i32) {
        todo!()
    }

    fn process_term_ack(&mut self) {
        todo!()
    }

    fn process_term_endpoint(&mut self, endpoint: &str) {
        todo!()
    }

    fn process_reap(&mut self, socket: &mut ZmqSocketBase) {
        todo!()
    }

    fn process_reaped(&mut self) {
        todo!()
    }

    fn process_conn_failed(&mut self) {
        todo!()
    }

    fn process_seqnum(&mut self) {
        todo!()
    }

    fn send_command(&mut self, cmd: &mut ZmqCommand) -> anyhow::Result<()> {
        todo!()
    }
}


// ZmqThread::~ZmqThread ()
// {
//     LIBZMQ_DELETE (poller);
// }


















