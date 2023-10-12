use std::ptr::null_mut;
use crate::address::address_t;
use crate::defines::{fd_t, handle_t};
use crate::endpoint::{endpoint_uri_pair_t, make_unconnected_connect_endpoint_pair};
use crate::endpoint::endpoint_type_t::endpoint_type_connect;
use crate::fd::retired_fd;
use crate::i_engine::i_engine;
use crate::io_object::io_object_t;
use crate::io_thread::io_thread_t;
use crate::options::options_t;
use crate::own::own_t;
use crate::session_base::session_base_t;
use crate::socket_base::socket_base_t;

pub const reconnect_timer_id: i32 = 1;

pub struct stream_connecter_base_t<'a> {
    pub own: own_t<'a>,
    pub io_object: io_object_t,
    pub _addr: address_t,
    pub _s: fd_t,
    pub _handle: handle_t,
    pub _endpoint: String,
    pub _socket: &'a socket_base_t<'a>,
    pub _session: &'a session_base_t<'a>,
    pub _delayed_start: bool,
    pub _reconnect_timer_started: bool,
    pub _current_reconnect_ivl: i32,
}

impl stream_connecter_base_t {
    pub unsafe fn new(io_thread_: &mut io_thread_t, session_: &mut session_base_t, options_: &options_t, addr_: address_t, delayed_start_: bool) -> Self {
        let mut out = Self {
            own: own_t::new2(io_thread_, options_),
            io_object: io_object_t::new(io_thread_),
            _addr: addr_,
            _s: retired_fd,
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
            self.cancel_timer(reconnect_timer_id);
            self._reconnect_timer_started = false;
        }

        if self._handle != null_mut() {
            self.rm_handle();
        }

        if self._s != retired_fd {
            self.close();
        }

        self.own.process_term(linger_);
    }

    pub unsafe fn add_reconnect_timer(&mut self) {
        if self.options.reconnect_ivl > 0 {
            let interval = self.get_new_reconnect_ivl();
            self.add_timer(interval, reconnect_timer_id);
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
        self._s = retired_fd
    }
    
    pub unsafe fn in_event(&mut self)
    {
        self.out_event();
    }
    
    pub unsafe fn create_engine(&mut self, fd_: fd_t, local_address_: &str) {
        // const endpoint_uri_pair_t endpoint_pair (local_address_, _endpoint,
        //                                      endpoint_type_connect);
        let mut endpoint_pair = endpoint_uri_pair_t::new();
        endpoint_pair.local = local_address_.to_string();
        endpoint_pair.remote = self._endpoint.clone();
        endpoint_pair.local_type = endpoint_type_connect;
    
        //  Create the engine object for this connection.
        // i_engine *engine;
        let mut engine: dyn i_engine;
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
        if id_ == reconnect_timer_id {
            self._reconnect_timer_started = false;
            self.start_connecting();
        }
    }
}
