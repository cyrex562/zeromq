use crate::address::get_socket_name;
use crate::address::SocketEnd::{SocketEndLocal, SocketEndRemote};
use crate::defines::RETIRED_FD;
use crate::defines::{ZmqFd, ZmqHandle};
use crate::endpoint::ZmqEndpointType::endpoint_type_bind;
use crate::endpoint::{make_unconnected_connect_endpoint_pair, ZmqEndpointUriPair};
use crate::i_engine::IEngine;
use crate::io_object::IoObject;
use crate::io_thread::ZmqIoThread;
use crate::options::ZmqOptions;
use crate::own::ZmqOwn;
use crate::session_base::ZmqSessionBase;
use crate::socket_base::ZmqSocket;
use std::ptr::null_mut;

pub struct ZmqStreamListenerBase<'a> {
    pub own: ZmqOwn<'a>,
    pub io_object: IoObject,
    pub _s: ZmqFd,
    pub _handle: ZmqHandle,
    pub _socket: &'a mut ZmqSocket<'a>,
    pub _endpoint: String,
}

impl ZmqStreamListenerBase {
    pub fn new(
        io_thread_: &mut ZmqIoThread,
        socket_: &mut ZmqSocket,
        options_: &ZmqOptions,
    ) -> Self {
        Self {
            own: ZmqOwn::new2(io_thread_, options_),
            io_object: IoObject::new(io_thread_),
            _s: RETIRED_FD,
            _handle: null_mut(),
            _socket: socket_,
            _endpoint: "".to_string(),
        }
    }

    pub unsafe fn get_local_address(&mut self, addr_: &mut String) -> i32 {
        *addr_ = get_socket_name(self._s, SocketEndLocal);
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
        self._s = RETIRED_FD
    }

    pub unsafe fn create_engine(&mut self, fd_: ZmqFd) {
        let endpoint_pair = ZmqEndpointUriPair::new(
            get_socket_name(fd_, SocketEndLocal),
            get_socket_name(fd_, SocketEndRemote),
            endpoint_type_bind,
        );

        // i_engine *engine;
        let mut engine: dyn IEngine;
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
            ZmqSessionBase::create(io_thread, false, self._socket, self.options, None);
        // errno_assert (session);
        session.inc_seqnum();
        self.launch_child(session);
        self.send_attach(session, engine, false);

        self._socket.event_accepted(endpoint_pair, fd_);
    }
}
