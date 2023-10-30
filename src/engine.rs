use crate::address::ZmqAddress;
use crate::decoder::ZmqDecoder;
use crate::defines::{ZmqFd, ZmqHandle};
use crate::encoder::ZmqEncoder;
use crate::endpoint::ZmqEndpointUriPair;
use crate::err::ZmqError;
use crate::io_object::IoObject;
use crate::io_thread::ZmqIoThread;
use crate::mechanism::ZmqMechanism;
use crate::metadata::ZmqMetadata;
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::session_base::ZmqSession;
use crate::socket::ZmqSocket;
use crate::stream_engine::{stream_in_event, stream_out_event, stream_plug, stream_terminate, stream_unplug};
use crate::udp_engine::{udp_in_event, udp_out_event, udp_plug, udp_terminate};
use crate::zmtp_engine::V3_GREETING_SIZE;


pub enum ZmqEngineType {
    Stream,
    Udp,
    Raw,
    Zmtp
}
#[derive(Default,Debug,Clone)]
pub struct ZmqEngine<'a>
{
    pub address: Option<ZmqAddress<'a>>,
    pub decoder: Option<&'a mut ZmqDecoder>,
    pub encoder: Option<&'a mut ZmqEncoder>,
    pub endpoint_uri_pair: Option<ZmqEndpointUriPair>,
    pub engine_type: ZmqEngineType,
    pub fd: ZmqFd,
    pub greeting_size: usize,
    pub greeting_recv: [u8;V3_GREETING_SIZE],
    pub greeting_send: [u8;V3_GREETING_SIZE],
    pub greeting_bytes_read: u32,
    pub handle: ZmqHandle,
    pub handshaking: bool,
    pub has_handshake_stage: bool,
    pub has_handshake_timer: bool,
    pub has_heartbeat_timer: bool,
    pub has_timeout_timer: bool,
    pub has_ttl_timer: bool,
    pub heartbeat_timeout: i32,
    pub in_buffer: Vec<u8>,
    pub in_pos: &'a mut [u8],
    pub in_size: usize,
    pub input_stopped: bool,
    pub io_error: bool,
    pub io_object: IoObject,
    pub mechanism: Option<&'a mut ZmqMechanism>,
    pub metadata: Option<ZmqMetadata>,
    pub out_address: ZmqSockAddr,
    pub out_address_len: usize,
    pub out_buffer: Vec<u8>,
    pub output_stopped: bool,
    pub out_pos: &'a mut [u8],
    pub out_size: usize,
    pub peer_address: String,
    pub plugged: bool,
    pub png_msg: ZmqMsg,
    pub raw_address: ZmqSockAddrIn,
    pub recv_enabled: bool,
    pub routing_id_msg: ZmqMsg,
    pub s: ZmqFd,
    pub send_enabled: bool,
    pub session: Option<&'a mut ZmqSession<'a>>,
    pub subscription_required: bool,
    pub socket: Option<&'a mut ZmqSocket<'a>>,
    pub tx_msg: Option<ZmqMsg>,
    pub process_msg: fn(engine: &mut ZmqEngine, msg: &mut ZmqMsg) -> Result<(),ZmqError>,
    pub next_msg: fn(engine: &mut ZmqEngine, msg: &mut ZmqMsg) -> i32,

}

impl ZmqEngine {
    pub fn plug(&mut self, options: &ZmqOptions, io_thread_: &mut ZmqIoThread, session: &mut ZmqSession)
    {
        match self.engine_type {
            ZmqEngineType::Stream => {
                stream_plug(options,self, io_thread_, session);
            },
            ZmqEngineType::Udp => {
                udp_plug(options, self, io_thread_, session);
            },
            ZmqEngineType::Raw => {
                // raw_plug(engine, io_thread_, session);
                stream_plug(options, self, io_thread_, session);
            },
            ZmqEngineType::Zmtp => {
                // zmtp_plug(engine, io_thread_, session);
                stream_plug(options, self, io_thread_, session);
            },
        }
    }

    pub fn unplug(&mut self) {
        match self.engine_type {
            ZmqEngineType::Stream => stream_unplug(self),
            ZmqEngineType::Udp => {}
            ZmqEngineType::Raw => stream_unplug(self),
            ZmqEngineType::Zmtp => stream_unplug(self),
        }
    }

    pub fn terminate(&mut self) {
        match self.engine_type {
            ZmqEngineType::Stream => stream_terminate(self),
            ZmqEngineType::Udp => udp_terminate(self),
            ZmqEngineType::Raw => stream_terminate(self),
            ZmqEngineType::Zmtp => stream_terminate(self),
        }
    }

    pub fn in_event(&mut self) {
        match self.engine_type {
            ZmqEngineType::Stream => stream_in_event(self),
            ZmqEngineType::Udp => udp_in_event(self),
            ZmqEngineType::Raw => stream_in_event(self),
            ZmqEngineType::Zmtp => stream_in_event(self),
        }
    }

    pub fn out_event(&mut self, options: &ZmqOptions) {
        match self.engine_type {
            ZmqEngineType::Stream => stream_out_event(options, self),
            ZmqEngineType::Udp => udp_out_event(self),
            ZmqEngineType::Raw => stream_out_event(options, self),
            ZmqEngineType::Zmtp => stream_out_event(options, self),
        }
    }

}
