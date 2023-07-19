use anyhow::bail;
use libc::{close, unlink};
use crate::address::{get_socket_name, ZmqAddress, ZmqSocketEnd};
use crate::defines::{RETIRED_FD, ZmqHandle};
use crate::endpoint::make_unconnected_bind_endpoint_pair;
use crate::endpoint::EndpointType::Bind;
use crate::engine::ZmqEngine;
use crate::defines::ZmqFileDesc;
use crate::endpoint_uri::EndpointUriPair;
use crate::io_object::ZmqIoObject;
use crate::ip::create_ipc_wildcard_address;
use crate::ipc::{ipc_accept, ipc_close, ipc_in_event, ipc_resolve_address, ipc_set_local_address, ipc_filter};
use crate::session_base::ZmqSessionBase;
use crate::socket::ZmqSocket;
use crate::tcp::{tcp_accept, tcp_create_socket, tcp_in_event, tcp_set_local_address};
use crate::thread_context::ZmqThreadContext;
use crate::tipc::{tipc_accept, tipc_in_event, tipc_set_local_address};
use crate::transport::ZmqTransport;
use crate::vmci::{vmci_accept, vmci_in_event, vmci_set_local_address};
use crate::ws::{ws_create_engine, ws_create_socket, ws_set_local_address};

#[derive(Default, Debug, Clone)]
pub struct ZmqListener<'a> {
    pub io_object: ZmqIoObject,
    pub socket: &'a mut ZmqSocket<'a>,
    pub fd: ZmqFileDesc,
    pub handle: ZmqHandle,
    pub endpoint: String,
    pub has_file: bool,
    pub tmp_socket_dirname: String,
    pub filename: String,
    pub address: ZmqAddress,
    pub wss: bool,
    pub tls_cred: Vec<u8>
}

impl<'a> ZmqListener<'a> {
    pub fn new(
        io_thread: &mut ZmqThreadContext,
        socket: &mut ZmqSocket,
    ) -> Self {
        Self {
            io_object: ZmqIoObject::new(Some(io_thread)),
            socket,
            fd: RETIRED_FD as ZmqFileDesc,
            handle: ZmqHandle::default(),
            endpoint: String::new(),
            has_file: false,
            tmp_socket_dirname: "".to_string(),
            filename: "".to_string(),
            address: Default::default(),
            wss: false,
            tls_cred: vec![],
        }
    }

    pub fn get_local_address(&mut self, addr: &mut String) -> anyhow::Result<()>
    {
        *addr = get_socket_name(self.fd, ZmqSocketEnd::SocketEndLocal)?;
        Ok(())
    }

    pub fn set_local_address(&mut self, addr: &mut str) -> anyhow::Result<()>
    {
        match self.socket.destination.protocol {
            ZmqTransport::ZmqTcp => tcp_set_local_address(self, addr),
            ZmqTransport::ZmqIpc => ipc_set_local_address(self, addr),
            ZmqTransport::ZmqTipc => tipc_set_local_address(self, &mut addr.to_string()),
            ZmqTransport::ZmqVmci => vmci_set_local_address(self, &mut addr.to_string()),
            ZmqTransport::ZmqWs => ws_set_local_address(self, &mut addr.to_string()),
            _ => bail!("Unsupported protocol"),
        }
    }

    pub fn process_plug(&mut self)
    {
        self.handle = self.io_object.add_fd(self.fd);
        self.io_object.set_pollin(self.handle);
    }

    pub fn process_term(&mut self, linger: i32)
    {
        self.io_object.rm_fd(self.handle);
        self.handle = ZmqHandle::default();
        self.close();
        self.io_object.process_term();
    }

    pub fn create_socket(&mut self, addr: &mut str) -> anyhow::Result<()>
    {
        match self.address.protocol {
            ZmqTransport::ZmqTcp => tcp_create_socket(self, addr),
            ZmqTransport::ZmqWs => ws_create_socket(self, addr),
            _ => bail!("Unsupported protocol"),

        }
    }


    pub fn create_engine(&mut self) -> anyhow::Result<()> {
        match self.address.protocol {
            ZmqTransport::ZmqWs => {
                ws_create_engine(self, self.fd)
            },
            _ => {
                let endpoint_pair = EndpointUriPair::new(
                    &get_socket_name(self.fd, ZmqSocketEnd::SocketEndLocal).unwrap(),
                    &get_socket_name(self.fd, ZmqSocketEnd::SocketEndRemote).unwrap(),
                    Bind,
                );

                let mut engine = ZmqEngine::new();
                let io_thread = self.chosen_io_thread();
                let mut session = ZmqSessionBase::create(
                    io_thread,
                    false,
                    self.socket,
                    None,
                )?;

                session.inc_seqnum();
                self.own.launch_child(session);
                self.own.send_attach(&mut session, engine, false);
                self.socket.event_accepted(&endpoint_pair, self.fd);
                Ok(())
            }
        }
    }

    pub fn get_socket_name(&mut self, socket_end: ZmqSocketEnd) -> anyhow::Result<String> {
        get_socket_name(self.fd, socket_end)
    }

    pub fn in_event(&mut self) -> anyhow::Result<()> {
        match self.socket.destination.protocol {
            ZmqTransport::ZmqIpc => ipc_in_event(self),
            ZmqTransport::ZmqTcp => tcp_in_event(self),
            ZmqTransport::ZmqTipc => tipc_in_event(self),
            ZmqTransport::ZmqVmci => vmci_in_event(self),
            _ => bail!("unsupported protocol")
        }
    }

    pub fn filter(&mut self) -> anyhow::Result<()> {
        match self.socket.destination.protocol {
            #[cfg(not(windows))]
            ZmqTransport::ZmqIpc => ipc_filter(self, 0),
            _ => bail!("unsupported protocol")
        }
    }

    // pub fn close(&mut self) -> anyhow::Result<()>
    //     {
    //         let rc = unsafe { close(self.fd) };
    //         if rc != 0 {
    //             anyhow::bail!("close failed");
    //         }
    //         self.socket.event_closed(&make_unconnected_bind_endpoint_pair(&self.endpoint), self.fd);
    //         Ok(())
    //     }
    pub fn close(&mut self) -> anyhow::Result<()> {
        match self.socket.destination.protocol {
            ZmqTransport::ZmqIpc => ipc_close(self),
            _ => bail!("unsupported protocol")
        }
    }

    pub fn accept(&mut self) -> anyhow::Result<ZmqFileDesc>
    {
        match self.socket.destination.protocol {
            ZmqTransport::ZmqIpc => ipc_accept(self),
            ZmqTransport::ZmqTcp => tcp_accept(self),
            ZmqTransport::ZmqTipc => tipc_accept(self),
            ZmqTransport::ZmqVmci => vmci_accept(self),
            _ => bail!("unsupported protocol")
        }
    }
}