use libc::EAGAIN;
use crate::defines::fd_t;
use crate::endpoint::endpoint_uri_pair_t;
use crate::msg::msg_t;
use crate::options::options_t;
use crate::stream_engine_base::stream_engine_base_t;
use crate::utils::{get_errno, put_u64};

pub const ZMTP_1_0: i32 = 0;
pub const ZMTP_2_0: i32 = 1;
pub const ZMTP_3_x: i32 = 3;

pub const signature_size: usize = 10;
pub const v2_greeting_size: usize = 12;

pub const v3_greeting_size: usize = 64;

pub const revision_pos: usize = 10;
pub const minor_pos: usize = 11;


pub struct zmtp_engine_t<'a> {
    pub stream_engine_base: stream_engine_base_t<'a>,
    pub _routing_id_msg: msg_t,
    pub _pong_msg: msg_t,
    pub _greeting_size: usize,
    pub _greeting_recv: [u8;v3_greeting_size],
    pub _greeting_send: [u8;v3_greeting_size],
    pub _greeting_bytes_read: u32,
    pub _subscription_required: bool,
    pub _heartbeat_timeout: i32,
}

impl zmtp_engine_t {
    pub unsafe fn new(fd_: fd_t, options_: &options_t, endpoint_uri_pair_: &endpoint_uri_pair_t) -> Self
    {
        let mut out = Self {
            stream_engine_base: stream_engine_base_t::new(fd_,options_,endpoint_uri_pair_,true),
            _routing_id_msg: msg_t::default(),
            _pong_msg: msg_t::default(),
            _greeting_size: v2_greeting_size,
            _greeting_recv: [0;v3_greeting_size],
            _greeting_send: [0;v3_greeting_size],
            _greeting_bytes_read: 0,
            _subscription_required: false,
            _heartbeat_timeout: 0,
        };

        out.stream_engine_base._next_msg = &mut out._routing_id_msg;
        out.stream_engine_base._process_msg = &mut out.process_routing_id_msg;
        out._pong_msg.init2();
        out._routing_id_msg.init2();

        if out._options.heartbeat_interval > 0 {
            out._heartbeat_timeout = out._options.heartbeat_timeout;
            if out._heartbeat_timeout == -1 {
                out._heartbeat = out._options.heartbeat_interval;
            }
        }

        out
    }

    pub unsafe fn plug_internal(&mut self)
    {
        // start optional timer, to prevent handshake hanging on no input
        self.set_handshake_timer ();

        //  Send the 'length' and 'flags' fields of the routing id message.
        //  The 'length' field is encoded in the long format.
        self._outpos = self._greeting_send;
        self._outpos[self._outsize] = u8::MAX;
        self._outsize += 1;
        put_u64 (&self._outpos[self._outsize], self._options.routing_id_size + 1);
        self._outsize += 8;
        self._outpos[self._outsize +=1] = 0x7f;

        self.set_pollin ();
        self.set_pollout ();
        //  Flush all the data that may have been already received downstream.
        self.in_event ();
    }

    pub unsafe fn handshake(&mut self) -> bool {
        // zmq_assert (_greeting_bytes_read < _greeting_size);
        //  Receive the greeting.
        let rc = self.receive_greeting ();
        if (rc == -1) {
            return false;
        }
        let unversioned = rc != 0;

        if (!(self.select_handshake_fun (unversioned, self._greeting_recv[revision_pos],
                                         self._greeting_recv[minor_pos])) ()) {
            return false;
        }

        // Start polling for output if necessary.
        if (self._outsize == 0) {
            self.set_pollout();
        }

        return true;
    }

    pub unsafe fn receive_greeting(&mut self) -> i32 {
        let mut unversioned = false;
        while (self._greeting_bytes_read < self._greeting_size) {
            let mut n = self.read (self._greeting_recv + self._greeting_bytes_read,
                                self._greeting_size - self._greeting_bytes_read);
            if (n == -1) {
                if (get_errno() != EAGAIN) {
                    // error(connection_error);
                }
                return -1;
            }

            self._greeting_bytes_read += n;

            //  We have received at least one byte from the peer.
            //  If the first byte is not 0xff, we know that the
            //  peer is using unversioned protocol.
            if (self._greeting_recv[0] != 0xff) {
                unversioned = true;
                break;
            }

            if (self._greeting_bytes_read < signature_size) {
                continue;
            }

            //  Inspect the right-most bit of the 10th byte (which coincides
            //  with the 'flags' field if a regular message was sent).
            //  Zero indicates this is a header of a routing id message
            //  (i.e. the peer is using the unversioned protocol).
            if (!(self._greeting_recv[9] & 0x01)) {
                unversioned = true;
                break;
            }

            //  The peer is using versioned protocol.
            self.receive_greeting_versioned ();
        }
        return if unversioned { 1 } else { 0 };
    }

    pub unsafe fn receive_greeting_versioned(&mut self) {
        //  Send the major version number.
        if (_outpos + _outsize == _greeting_send + signature_size) {
            if (_outsize == 0) {
                set_pollout();
            }
            _outpos[_outsize++] = 3; //  Major version number
        }

        if (_greeting_bytes_read > signature_size) {
            if (_outpos + _outsize == _greeting_send + signature_size + 1) {
                if (_outsize == 0) {
                    set_pollout();
                }

                //  Use ZMTP/2.0 to talk to older peers.
                if (_greeting_recv[revision_pos] == ZMTP_1_0
                    || _greeting_recv[revision_pos] == ZMTP_2_0) {
                    _outpos[_outsize+ +] = _options. type ;
                }
                else {
                    _outpos[_outsize++] = 1; //  Minor version number
                    memset (_outpos + _outsize, 0, 20);

                    // zmq_assert (_options.mechanism == ZMQ_NULL
                    //             || _options.mechanism == ZMQ_PLAIN
                    //             || _options.mechanism == ZMQ_CURVE
                    //             || _options.mechanism == ZMQ_GSSAPI);

                    if (_options.mechanism == ZMQ_NULL) {
                        memcpy(_outpos + _outsize, "NULL", 4);
                    }
                    else if (_options.mechanism == ZMQ_PLAIN) {
                        memcpy(_outpos + _outsize, "PLAIN", 5);
                    }
                    else if (_options.mechanism == ZMQ_GSSAPI) {
                        memcpy(_outpos + _outsize, "GSSAPI", 6);
                    }
                    else if (_options.mechanism == ZMQ_CURVE) {
                        memcpy(_outpos + _outsize, "CURVE", 5);
                    }
                    _outsize += 20;
                    memset (_outpos + _outsize, 0, 32);
                    _outsize += 32;
                    _greeting_size = v3_greeting_size;
                }
            }
        }
    }
}
