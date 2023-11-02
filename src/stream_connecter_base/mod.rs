use std::ptr::null_mut;
use crate::address::ZmqAddress;
use crate::defines::{ZmqFd, ZmqHandle};
use crate::endpoint::{ZmqEndpointUriPair, make_unconnected_connect_endpoint_pair};
use crate::endpoint::ZmqEndpointType::EndpointTypeConnect;
use crate::defines::RETIRED_FD;
use crate::i_engine::IEngine;
use crate::io::io_object::IoObject;
use crate::io_thread::ZmqIoThread;
use crate::options::ZmqOptions;
use crate::own::ZmqOwn;
use crate::session_base::ZmqSession;
use crate::socket::ZmqSocket;

mod tcp_connecter;

pub const RECONNECT_TIMER_ID: i32 = 1;

pub struct ZmqStreamConnecterBase<'a> {
    pub own: ZmqOwn<'a>,
    pub io_object: IoObject,
    pub _addr: ZmqAddress,
    pub _s: ZmqFd,
    pub _handle: ZmqHandle,
    pub _endpoint: String,
    pub _socket: &'a ZmqSocket<'a>,
    pub _session: &'a ZmqSession<'a>,
    pub _delayed_start: bool,
    pub _reconnect_timer_started: bool,
    pub _current_reconnect_ivl: i32,
}

impl ZmqStreamConnecterBase {
    pub unsafe fn new(io_thread_: &mut ZmqIoThread, session_: &mut ZmqSession, options_: &ZmqOptions, addr_: ZmqAddress, delayed_start_: bool) -> Self {
        let mut out = Self {
            own: ZmqOwn::from_io_thread(io_thread_, options_),
            io_object: IoObject::new(io_thread_),
            _addr: addr_,
            _s: RETIRED_FD,
            _handle: null_mut(),
            _endpoint: "".to_string(),
            _socket: session_.get_socket(),
            _session: session_,
            _delayed_start: delayed_start_,
            _reconnect_timer_started: false,
            _current_reconnect_ivl: 0,
        }
    }

    pub fn process_plug(&mut self) {
        if self._delayed_start {
            self.add_reconnect_timer();
        } else {
            self.start_connecting();
        }
    }

    pub fn process_term(&mut self, linger_: i32) {
        if self._reconnect_timer_started {
            self.cancel_timer(RECONNECT_TIMER_ID);
            self._reconnect_timer_started = false;
        }

        if self._handle != null_mut() {
            self.rm_handle();
        }

        if self._s != RETIRED_FD {
            self.close();
        }

        self.own.process_term(linger_);
    }

    pub unsafe fn add_reconnect_timer(&mut self) {
        if self.options.reconnect_ivl > 0 {
            let interval = self.get_new_reconnect_ivl();
            self.add_timer(interval, RECONNECT_TIMER_ID);
            self._socket.event_connect_retried(
                make_unconnected_connect_endpoint_pair(self._endpoint), interval
            );
            self._reconnect_timer_started = true;
        }
    }

    pub unsafe fn get_new_reconnect_ivl(&mut self) -> i32 {
        if (self.options.reconnect_ivl_max > 0) {
            let mut candidate_interval = 0;
            if (self._current_reconnect_ivl == -1) {
                candidate_interval = self.options.reconnect_ivl;
            }
            else if (self._current_reconnect_ivl > i32::MAX / 2)
                candidate_interval = i32::MAX;
            else
                candidate_interval = self._current_reconnect_ivl * 2;

            if (candidate_interval > self.options.reconnect_ivl_max) {
                self._current_reconnect_ivl = self.options.reconnect_ivl_max;
            }
            else {
                self._current_reconnect_ivl = candidate_interval;
            }
            return self._current_reconnect_ivl;
        } else {
            if (self._current_reconnect_ivl == -1) {
                self._current_reconnect_ivl = self.options.reconnect_ivl;
            }
            //  The new interval is the base interval + random value.
                let random_jitter = generate_random () % self.options.reconnect_ivl;
            let interval =
              if self._current_reconnect_ivl < i32::MAX - random_jitter {
                  self._current_reconnect_ivl + random_jitter
              } else {
                  i32::MAX
              };

            return interval;
        }
    }

    pub fn rm_handle(&mut self) {
        self.io_object.rm_fd(self._handle);
        self._handle = null_mut();
    }
    
    pub unsafe fn close(&mut self) {
        #[cfg(target_os="windows")]
        {
            closeseocket(self._s);
        }
        #[cfg(not(target_os="windows"))]
        {
            libc::close(self._s);
        }
        self._socket.event_closed(make_unconnected_connect_endpoint_pair(self._endpoint));
        self._s = RETIRED_FD
    }
    
    pub unsafe fn in_event(&mut self)
    {
        self.out_event();
    }
    
    pub unsafe fn create_engine(&mut self, fd_: ZmqFd, local_address_: &str) {
        // const endpoint_uri_pair_t endpoint_pair (local_address_, _endpoint,
        //                                      EndpointTypeConnect);
        let mut endpoint_pair = ZmqEndpointUriPair::new();
        endpoint_pair.local = local_address_.to_string();
        endpoint_pair.remote = self._endpoint.clone();
        endpoint_pair.local_type = EndpointTypeConnect;
    
        //  Create the engine object for this connection.
        // i_engine *engine;
        let mut engine: dyn IEngine;
        if (self.options.raw_socket) {
            // engine = new (std::nothrow) raw_engine_t (fd_, options, endpoint_pair);}
            engine = raw_engine_t::new(fd_, options, endpoint_pair);
        }
        else{
            // engine = new (std::nothrow) zmtp_engine_t (fd_, options, endpoint_pair);
            engine = zmtp_engine_t::new(fd_, options, endpoint_pair);
            }
        // alloc_assert (engine);
    
        //  Attach the engine to the corresponding session object.
        self.send_attach (self._session, engine);
    
        //  Shut the connecter down.
        self.terminate ();
    
        self._socket.event_connected (&endpoint_pair, fd_);
    }
    
    pub fn timer_event(&mut self id_: i32) {
        if id_ == RECONNECT_TIMER_ID {
            self._reconnect_timer_started = false;
            self.start_connecting();
        }
    }
}