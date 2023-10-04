use std::collections::HashSet;
use std::ptr::null_mut;
use crate::address::address_t;
use crate::endpoint::endpoint_uri_pair_t;
use crate::i_engine::i_engine;
use crate::io_object::io_object_t;
use crate::io_thread::io_thread_t;
use crate::msg::{command, more, msg_t};
use crate::object::object_t;
use crate::options::options_t;
use crate::own::own_t;
use crate::pipe::{i_pipe_events, pipe_t, pipepair};
use crate::socket_base::socket_base_t;

pub struct session_base_t
{
    pub own: own_t,
    pub io_object: io_object_t,
    pub _active: bool,
    pub _pipe: *mut pipe_t,
    pub _zap_pipe: *mut pipe_t,
    pub _terminating_pipes: HashSet<*mut pipe_t>,
    pub _incomplete_in: bool,
    pub _pending: bool,
    pub _socket: *mut socket_base_t,
    pub _io_thread: *mut io_thread_t,

}

impl i_pipe_events for session_base_t {
    fn read_activated(&self, pipe: &pipe_t) {
        unimplemented!()
    }
    fn write_activated(&self, pipe: &pipe_t) {
        unimplemented!()
    }
    fn hiccuped(&self, pipe: &pipe_t) {
        unimplemented!()
    }
    fn pipe_terminated(&self, pipe: &pipe_t) {
        unimplemented!()
    }
}

impl i_engine for session_base_t
{
    fn has_handshake_stage(&mut self) -> bool {
        todo!()
    }

    fn plug(&mut self, io_thread_: *mut io_thread_t, session_: *mut session_base_t) {
        todo!()
    }

    fn terminate(&mut self) {
        todo!()
    }

    fn restart_input(&mut self) {
        todo!()
    }

    fn restart_output(&mut self) {
        todo!()
    }

    fn zap_msg_available(&mut self) {
        todo!()
    }

    fn get_endpoint(&mut self) -> &mut endpoint_uri_pair_t {
        todo!()
    }
}

impl session_base_t{
    pub unsafe fn create(io_thread_: *mut io_thread_t, active_: bool, socket_: *mut socket_base_t, options_: &options_t, addr_: *mut address_t) -> *mut session_base_t {
        let mut s: *mut session_base_t = null_mut();
        match options_.type_ {
            ZMQ_REQ => {
                // s = &mut req_session_t::new(io_thread_, active_, socket_, options_, addr_);
            },
            ZMQ_RADIO => {
                // s = &mut radio_session_t::new(io_thread_, active_, socket_, options_, addr_);
            },
            ZMQ_DISH => {
                // s = &mut dish_session_t::new(io_thread_, active_, socket_, options_, addr_);
            },
            _ => {
                if options_.can_send_hello_msg && options_.hello_msg.len() > 0 {
                    // s = &mut hello_session_t::new(io_thread_, active_, socket_, options_, addr_);
                } else {
                    s = &mut session_base_t::new(io_thread_, active_, socket_, options_, addr_);
                }
        }
        }
        return s;
    }

    pub unsafe fn new(io_thread_: *mut io_thread_t, active_: bool, socket_: *mut socket_base_t, options_: &options_t, addr_: *mut address_t) -> Self {
        Self {
            own: own_t::new2(io_thread_, options_),
            io_object: io_object_t::new(io_thread_),
            _active: active_,
            _pipe: null_mut(),
            _zap_pipe: null_mut(),
            _terminating_pipes: HashSet::new(),
            _incomplete_in: false,
            _pending: false,
            _socket: socket_,
            _io_thread: io_thread_,
        }
    }

    pub fn get_endpoint(&mut self) -> &mut endpoint_uri_pair_t
    {
        return self.get_endpoint()
    }

    pub fn attach_pipe(&mut self, pipe_: *mut pipe_t)
    {
        self._pipe = pipe_;
        self._pipe.set_event_risk(self)
    }

    pub unsafe fn pull_msg(&mut self, msg_: *mut msg_t) -> i32
    {
        if self._pipe == null_mut() || !(*self._pipe).read(msg_)  {
            return -1;
        }

        self._incomplete_in = msg_.flags() & more != 0;
        return 0;
    }

    pub unsafe fn push_msg(&mut self, msg_: *mut msg_t) -> i32 {
        if msg_.flags() & command !=0 && !msg_.is_subscribe() && !msg_.is_cancel() {
            return 0;
        }
        if self._pipe != null_mut() && (*self._pipe).write(msg_) {
            let mut rc = msg_.init2();
            return 0;
        }

        return -1;
    }

    pub unsafe fn read_zap_msg(&mut self, msg_: *mut msg_t) -> i32
    {
        if self._zap_pipe == null_mut() || !(*self._zap_pipe).read(msg_) {
            return -1;
        }
        return 0;
    }

    pub unsafe fn write_zap_msg(&mut self, msg_: *mut msg_t) -> i32
    {
        if self._zap_pipe == null_mut() && !(*self._zap_pipe).write(msg_) {
            return -1;
        }

        if msg_.flags() & more == 0 {
            self._zap_pipe.flush()
        }

        let rc = msg_.init2();

        return 0;
    }

    pub fn reset(&mut self) {
        unimplemented!()
    }

    pub unsafe fn flush(&mut self) {
        if self._pipe != null_mut() {
            self._pipe.flush();
        }
    }

    pub unsafe fn rollback(&mut self)
    {
        if self._pipe != null_mut() {
            self._pipe.rollback();
        }
    }

    pub unsafe fn clean_pipes(&mut self)
    {
        self._pipe.rollback();
        self._pipe.flush();

        while (self._incomplete_in) {
            let mut msg = msg_t::new();
            self.pull_msg(&mut msg);
            msg.close();
        }
    }

    pub unsafe fn pipe_terminated(&mut self, pipe_: *mut pipe_t)
    {
        if pipe_ == self._pipe {
            self._pipe = null_mut();
            if self._has_linger_timer {
                self.io_object.cancel_timer(self._linger_timer_id);
                self._has_linger_timer = false;
            }
        } else if pipe_ == self._zap_pipe {
            self._zap_pipe = null_mut();
        } else {
            self._terminating_pipes.insert(pipe_);
        }

        if !self.is_terminating() && self.options.raw_socket {
            if self._engine != null_mut() {
                self._engine.terminate();
                self._engine = null_mut();
            }
            self.terminate();
        }

        if self._pending && self._pipe == null_mut() && self._zap_pipe == null_mut() && self._terminating_pipes.len() == 0 {
            self._pending = false;
            self.io_object.signal();
        }
    }

    pub fn read_activated(&mut self, pipe_: *mut pipe_t)
    {
        if pipe_ != self._pipe && pipe_ != self._zap_pipe {
            return;
        }

        if self._engine == null_mut() {
            if self._pipe {
                self._pipe.check_read()
            }
            return;
        }

        if pipe_ == self._pipe {
            self._engine.restart_input();
        } else {
            self._engine.zap_msg_available();
        }
    }

    pub fn write_activated(&mut self, pipe_: *mut pipe_t) {
        if self._pipe != pipe_ {
            return;
        }

        if self._engine != null_mut() {
            self._engine.restart_output();
        }
    }

    pub fn hiccuped(&mut self, pipe_: *mut pipe_t) {
        unimplemented!()
    }

    pub fn get_socket(&mut self) -> *mut socket_base_t {
        return self._socket;
    }

    pub fn process_plug(&mut self) {
        if self._active {
            self.start_connecting(false)
        }
    }

    pub unsafe fn zap_connect(&mut self) -> i32 {
        if (self._zap_pipe != null_mut) {
            return 0;
        }

        let mut peer = self.find_endpoint ("inproc://zeromq.zap.01");
        if (peer.socket == null_mut()) {
            // errno = ECONNREFUSED;
            return -1;
        }
        // zmq_assert (peer.options.type == ZMQ_REP || peer.options.type == ZMQ_ROUTER
        //             || peer.options.type == ZMQ_SERVER);

        //  Create a bi-directional pipe that will connect
        //  session with zap socket.
        let mut parents: [*mut object_t;2] = {self, peer.socket};
        let mut new_pipes: [*mut pipe_t;2] = [null_mut(), null_mut()];
        let mut hwms: [i32;2] = [0, 0];
        let mut conflates: [bool;2] = [false, false];
        let rc = pipepair (parents, &mut new_pipes, hwms, conflates);
        // errno_assert (rc == 0);

        //  Attach local end of the pipe to this socket object.
        self._zap_pipe = new_pipes[0];
        self._zap_pipe.set_nodelay ();
        self._zap_pipe.set_event_sink (self);

        self.send_bind (peer.socket, new_pipes[1], false);

        //  Send empty routing id if required by the peer.
        if (peer.options.recv_routing_id) {
            let mut id = msg_t::default();
            let rc = id.init ();
            // errno_assert (rc == 0);
            id.set_flags (msg_t::routing_id);
            let ok = (*self._zap_pipe).write (id);
            // zmq_assert (ok);
            self._zap_pipe.flush ();
        }

        return 0;
    }

    1q
}
