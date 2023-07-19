use std::error::Error;
use anyhow::{anyhow};
use crate::address::{get_socket_name, ZmqAddress, ZmqSocketEnd};
use crate::address_family::AF_UNIX;
use crate::defines::{RETIRED_FD, ZMQ_RECONNECT_STOP_AFTER_DISCONNECT};
use crate::err::{wsa_error_to_errno, ZmqError};
use crate::defines::ZmqFileDesc;
use crate::ip::{ip_open_socket, unblock_socket};

use crate::session_base::ZmqSessionBase;
use crate::stream_connecter_base::StreamConnecterBase;
use crate::thread_context::ZmqThreadContext;
use libc::{c_char, close, connect, getsockopt, open, ECONNREFUSED, EHOSTUNREACH, EINPROGRESS, EINTR, ENETDOWN, ENETUNREACH, ENOPROTOOPT, ETIMEDOUT, c_int};
use windows::Win32::Networking::WinSock::{
    WSAGetLastError, SOCK_STREAM, SOL_SOCKET, SO_ERROR, WSAEINPROGRESS, WSAEWOULDBLOCK,
};

use crate::context::ZmqContext;

pub struct IpcConnecter<'a> {
    // : public stream_connecter_base_t
    pub base: StreamConnecterBase<'a>,
}

impl<'a> IpcConnecter<'a> {
    //
    //  If 'delayed_start' is true connecter first waits for a while,
    //  then starts connection process.
    // IpcConnecter (ZmqIoThread *io_thread_,
    //                 ZmqSessionBase *session_,
    //              options: &ZmqOptions,
    //                 Address *addr_,
    //              delayed_start_: bool);
    pub fn new(
        ctx: &mut ZmqContext,
        io_thread_: &mut ZmqThreadContext,
        session: &mut ZmqSessionBase,
        addr: &mut ZmqAddress,
        delayed_start_: bool,
    ) -> Self {
        // stream_connecter_base_t (
        //       io_thread_, session_, options_, addr_, delayed_start_)
        // zmq_assert (_addr.protocol == protocol_name::ipc);
        Self {
            base: Default::default(),
        }
    }

    //
    //  Handlers for I/O events.
    // void out_event ();
    pub fn out_event(&mut self) -> anyhow::Result<()> {
        // TODO
        // let fd = unsafe { connect() };
        let mut fd = 0 as i32;
        self.base.rm_handle();

        //  Handle the error condition by attempt to reconnect.
        if (fd == RETIRED_FD) {
            unsafe { close(fd) };
            self.base.add_reconnect_timer();
            return Ok(());
        }

        self.base.create_engine(fd as ZmqFileDesc,
                                get_socket_name(fd as ZmqFileDesc,
                                                ZmqSocketEnd::SocketEndLocal)?.as_str());
        Ok(())
    }

    //  Internal function to start the actual connection establishment.
    // void start_connecting ();
    pub fn start_connecting(&mut self) -> anyhow::Result<()> {
        //  Open the connecting socket.
        let rc = self.open();


        match self.open()
        {
            Ok(_) => {
                self.base._handle = self.base.add_fd(self.base._s);
                self.out_event();
                return Ok(());
            }
            Err(e) => {
                match e {
                    ZmqError::InProgress(_) => {
                        self.base._handle = self.base.add_fd(self.base._s);
                        self.base.set_pollout(self.base._handle);
                        // TODO
                        // self._socket.event_connect_delayed(
                        //     make_unconnected_connect_endpoint_pair(_endpoint),
                        //     // zmq_errno(),
                        // );
                    }
                    ZmqError::ConnectionRefused(_) => {
                        if self.options.reconnect_stop & ZMQ_RECONNECT_STOP_AFTER_DISCONNECT && self.base._socket.is_disconnected() {
                            if self.base._s != RETIRED_FD as usize {
                                unsafe { close(self.base._s as c_int) };
                                self.base._s = RETIRED_FD as ZmqFileDesc;
                            }
                        }
                    }
                    _ => {
                        if self.base._s != RETIRED_FD as usize {
                            unsafe { close(self.base._s as c_int) };
                            self.base._s = RETIRED_FD as ZmqFileDesc;
                        }
                        self.base.add_reconnect_timer();
                    }
                }
            }
        }

        return Err(anyhow!("failed to connect"));
    }

    pub fn open(&mut self) -> Result<(), ZmqError> {
        // zmq_assert (_s == retired_fd);

        //  Create the socket.
        self.base._s = ip_open_socket(AF_UNIX as i32, SOCK_STREAM as i32, 0)?;
        if (self.base._s == RETIRED_FD as usize) {
            return Err(anyhow!("failed to create socket"));
        }

        //  Set the non-blocking flag.
        unblock_socket(self.base._s);

        //  Connect to the remote peer.
        let rc: i32 = unsafe {
            connect(
                self.base._s,
                self.base._addr.resolved.ipc_addr.addr(),
                self.base._addr.resolved.ipc_addr.addrlen(),
            )
        };

        //  Connect was successful immediately.
        if (rc == 0) {
            return Ok(());
        }

        //  Translate other error codes indicating asynchronous connect has been
        //  launched to a uniform EINPROGRESS.
        // #ifdef ZMQ_HAVE_WINDOWS
        let mut errno = 0i32;
        if cfg!(target_os = "windows") {
            let last_error = unsafe { WSAGetLastError() };
            if (last_error == WSAEINPROGRESS || last_error == WSAEWOULDBLOCK) {
              // errno = EINPROGRESS;
            } else {
              // errno = wsa_error_to_errno(last_error);
            }
        }
        // #else
        else {
            if (rc == -1 && errno == EINTR) {
              // errno = EINPROGRESS;
            }
        }

        if errno == EINPROGRESS {
            return Err(ZmqError::InProgress("connect not finished yet".to_string()));
        }// #endif
        if errno == ECONNREFUSED {
            return Err(ZmqError::ConnectionRefused("connection refused".to_string()));
        }

        //  Forward the error.
        return Err(anyhow!("failed to connect to socket"));
    }

    //  Open IPC connecting socket. Returns -1 in case of error,
    //  0 if connect was successful immediately. Returns -1 with
    //  EAGAIN errno if async connect was launched.
    // int open ();

    //  Get the file descriptor of newly created connection. Returns
    //  retired_fd if the connection was unsuccessful.
    // ZmqFileDesc connect ();
    pub fn connect(&mut self) -> Result<ZmqFileDesc, ZmqError> {
        //  Following code should handle both Berkeley-derived socket
        //  implementations and Solaris.
        let mut err = 0;
        let mut len = 4;
        let rc: i32 = unsafe {
            getsockopt(
                self.base._s,
                SOL_SOCKET,
                SO_ERROR,
                (&mut err.to_le_bytes() as &mut c_char),
                &mut len,
            )
        };

        if rc == -1 {
            let mut errno = 0i32;
            #[cfg(target_os = "linux")]
            {
              // errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
            }
            #[cfg(target_os = "windows")]
            {
                let wsa_err = unsafe { WSAGetLastError() };
              // errno = wsa_error_to_errno(wsa_err);
            }
            if errno = ENOPROTOOPT {
              // errno = 0;
            }
            return Err(anyhow!("failed to get socket options"));
        }

        let result = self.base._s;
        self.base._s = RETIRED_FD as ZmqFileDesc;
        return Ok(result);
    }
}

// #endif
