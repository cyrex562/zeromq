use anyhow::bail;
use libc::{close, unlink};
use crate::address::{get_socket_name, ZmqAddress, ZmqSocketEnd};
use crate::defines::{retired_fd, ZmqHandle};
use crate::endpoint::{EndpointUriPair, make_unconnected_bind_endpoint_pair};
use crate::endpoint::EndpointType::endpoint_type_bind;
use crate::engine::ZmqEngine;
use crate::fd::ZmqFileDesc;
use crate::io_object::ZmqIoObject;
use crate::ip::create_ipc_wildcard_address;
use crate::ipc::{ipc_accept, ipc_close, ipc_in_event, ipc_resolve_address, ipc_set_local_address};
use crate::session_base::ZmqSessionBase;
use crate::socket::ZmqSocket;
use crate::tcp::{tcp_accept, tcp_create_socket, tcp_in_event};
use crate::thread_context::ZmqThreadContext;
use crate::transport::ZmqTransport;

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
}

impl<'a> ZmqListener<'a> {
    pub fn new(
        io_thread: &mut ZmqThreadContext,
        socket: &mut ZmqSocket,
    ) -> Self {
        Self {
            io_object: ZmqIoObject::new(Some(io_thread)),
            socket,
            fd: retired_fd as ZmqFileDesc,
            handle: ZmqHandle::default(),
            endpoint: String::new(),
            has_file: false,
            tmp_socket_dirname: "".to_string(),
            filename: "".to_string(),
            address: Default::default(),
        }
    }

    pub fn get_local_address(&mut self, addr: &mut String) -> anyhow::Result<()>
    {
        *addr = get_socket_name(self.fd, ZmqSocketEnd::SocketEndLocal)?;
        Ok(())
    }

    pub fn set_local_address(&mut self, addr: &mut str) -> anyhow::Result<()>
    {
        if self.socket.destination.protocol == ZmqTransport::ZmqIpc {
            ipc_set_local_address(self, addr)
        } else {
            Ok(())
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
            _ => bail!("Unsupported protocol"),
        }
    }


    pub fn create_engine(&mut self) -> anyhow::Result<()> {
        let endpoint_pair = EndpointUriPair::new(
            &get_socket_name(self.fd, ZmqSocketEnd::SocketEndLocal).unwrap(),
            &get_socket_name(self.fd, ZmqSocketEnd::SocketEndRemote).unwrap(),
            endpoint_type_bind,
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

    pub fn get_socket_name(&mut self, socket_end: ZmqSocketEnd) -> anyhow::Result<String> {
        get_socket_name(self.fd, socket_end)
    }

    pub fn in_event(&mut self) -> anyhow::Result<()> {
        match self.socket.destination.protocol {
            ZmqTransport::ZmqIpc => ipc_in_event(self),
            ZmqTransport::ZmqTcp => tcp_in_event(self),
            _ => bail!("unsupported protocol")
        }
    }

    pub fn filter(&mut self) -> anyhow::Result<()> {
        match self.socket.destination.protocol {
            #[cfg(not(windows))]
            ZmqTransport::ZmqIpc => ipc_filter(self),
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
            _ => bail!("unsupported protocol")
        }
    }
}