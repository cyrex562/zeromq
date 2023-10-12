use crate::address::get_socket_name;
use crate::address::socket_end_t::{socket_end_local, socket_end_remote};
use crate::defines::{fd_t, handle_t};
use crate::endpoint::endpoint_type_t::endpoint_type_bind;
use crate::endpoint::{endpoint_uri_pair_t, make_unconnected_connect_endpoint_pair};
use crate::fd::retired_fd;
use crate::i_engine::i_engine;
use crate::io_object::io_object_t;
use crate::io_thread::io_thread_t;
use crate::options::options_t;
use crate::own::own_t;
use crate::session_base::session_base_t;
use crate::socket_base::socket_base_t;
use std::ptr::null_mut;

pub struct stream_listener_base_t<'a> {
    pub own: own_t<'a>,
    pub io_object: io_object_t,
    pub _s: fd_t,
    pub _handle: handle_t,
    pub _socket: &'a mut socket_base_t<'a>,
    pub _endpoint: String,
}

impl stream_listener_base_t {
    pub fn new(
        io_thread_: &mut io_thread_t,
        socket_: &mut socket_base_t,
        options_: &options_t,
    ) -> Self {
        Self {
            own: own_t::new2(io_thread_, options_),
            io_object: io_object_t::new(io_thread_),
            _s: retired_fd,
            _handle: null_mut(),
            _socket: socket_,
            _endpoint: "".to_string(),
        }
    }

    pub unsafe fn get_local_address(&mut self, addr_: &mut String) -> i32 {
        *addr_ = get_socket_name(self._s, socket_end_local);
        if addr_.is_empty() {
            return -1;
        }
        return 0;
    }

    pub unsafe fn process_plug(&mut self) {
        self._handle = self.add_fd(self._s);
        self.set_pollin();
    }

    pub unsafe fn process_term(&mut self, linger_: i32) {
        self.rm_fd(self._handle);
        self.close();
        self._handle = null_mut();
        self.own.process_term(linger_);
    }

    pub unsafe fn close(&mut self) {
        #[cfg(target_os = "windows")]
        {
            closeseocket(self._s);
        }
        #[cfg(not(target_os = "windows"))]
        {
            libc::close(self._s);
        }
        self._socket
            .event_closed(make_unconnected_connect_endpoint_pair(self._endpoint));
        self._s = retired_fd
    }

    pub unsafe fn create_engine(&mut self, fd_: fd_t) {
        let endpoint_pair = endpoint_uri_pair_t::new(
            get_socket_name(fd_, socket_end_local),
            get_socket_name(fd_, socket_end_remote),
            endpoint_type_bind,
        );

        // i_engine *engine;
        let mut engine: dyn i_engine;
        if (self.options.raw_socket) {
            engine = raw_engine_t::new(fd_, self.options, endpoint_pair);
        } else {
            engine = zmtp_engine_t::new(fd_, self.options, endpoint_pair);
        }
        // alloc_assert (engine);

        //  Choose I/O thread to run connecter in. Given that we are already
        //  running in an I/O thread, there must be at least one available.
        let io_thread = self.choose_io_thread(self.options.affinity);
        // zmq_assert (io_thread);

        //  Create and launch a session object.
        let mut session =
            session_base_t::create(io_thread, false, self._socket, self.options, None);
        // errno_assert (session);
        session.inc_seqnum();
        self.launch_child(session);
        self.send_attach(session, engine, false);

        self._socket.event_accepted(endpoint_pair, fd_);
    }
}
