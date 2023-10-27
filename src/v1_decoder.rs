use crate::decoder::ZmqDecoderBase;
use crate::msg::ZmqMsg;
use crate::utils::get_u64;

pub struct V1Decoder {
    pub decoder_base: ZmqDecoderBase<V1Decoder>,
    pub _tmpbuf: [u8;8],
    pub _in_progress: ZmqMsg,
    pub _max_msg_size: i64,
}

impl V1Decoder {
    pub fn new(bufsize: usize, max_msg_size: i64) -> Self {
        let mut out = Self {
            decoder_base: ZmqDecoderBase::new(bufsize),
            _tmpbuf: [0;8],
            _in_progress: ZmqMsg::default(),
            _max_msg_size: 0,
        };

        let rc = self._in_progress.init2();
        out.next_step(out._tmpbuf, out.one_byte_size_ready);
        out
    }

   // int zmq::v1_decoder_t::one_byte_size_ready (unsigned char const *)
    pub unsafe fn one_byte_size_ready(&mut self, buf: &[u8]) -> i32
    {
        //  First byte of size is read. If it is UCHAR_MAX (0xff) read 8-byte size.
        //  Otherwise allocate the buffer for message data and read the
        //  message data into it.
        if (*self._tmpbuf == u8::MAX) {
            self.next_step(self._tmpbuf, 8, &V1Decoder::eight_byte_size_ready);
        }
        else {
            //  There has to be at least one byte (the flags) in the message).
            if (!*self._tmpbuf) {
                // errno = EPROTO;
                return -1;
            }

            if self._max_msg_size >= 0 &&(self._tmpbuf[0] - 1) > self._max_msg_size as u8 {
                // errno = EMSGSIZE;
                return -1;
            }

            let mut rc = self._in_progress.close ();
            // assert (rc == 0);
            rc = self._in_progress.init_size (*self._tmpbuf - 1);
            if (rc != 0) {
                // errno_assert (errno == ENOMEM);
                rc = self._in_progress.init ();
                // errno_assert (rc == 0);
                // errno = ENOMEM;
                return -1;
            }

            self.next_step (self._tmpbuf, 1, &V1Decoder::flags_ready);
        }
        return 0;
    }

    pub fn msg(&mut self) -> &mut ZmqMsg {
        self._in_progress.refm()
    }

    // int zmq::v1_decoder_t::eight_byte_size_ready (unsigned char const *)
    pub unsafe fn eight_byte_size_ready(&mut self, buf: &[u8]) -> i32
    {
        //  8-byte payload length is read. Allocate the buffer
        //  for message body and read the message data into it.
        let payload_length = get_u64(self._tmpbuf.as_ptr());

        //  There has to be at least one byte (the flags) in the message).
        if payload_length == 0 {
            // errno = EPROTO;
            return -1;
        }

        //  Message size must not exceed the maximum allowed size.
        if self._max_msg_size >= 0 && payload_length - 1 > (self._max_msg_size) as u64 {
            // errno = EMSGSIZE;
            return -1;
        }

    // #ifndef __aarch64__
    //     //  Message size must fit within range of size_t data type.
    //     if (payload_length - 1 > std::numeric_limits<size_t>::max ()) {
    //         errno = EMSGSIZE;
    //         return -1;
    //     }
    // #endif

        let msg_size = (payload_length - 1);

        let mut rc = self._in_progress.close ();
        // assert (rc == 0);
        rc = self._in_progress.init_size (msg_size);
        if (rc != 0) {
            // errno_assert (errno == ENOMEM);
            rc = self._in_progress.init2 ();
            // errno_assert (rc == 0);
            // errno = ENOMEM;
            return -1;
        }

        self.next_step (self._tmpbuf, 1, &V1Decoder::flags_ready);
        return 0;
    }

    // int zmq::v1_decoder_t::flags_ready (unsigned char const *)
    pub unsafe fn flags_ready(&mut self, buf: &[u8]) -> i32
    {
        //  Store the flags from the wire into the message structure.
        self._in_progress.set_flags (self._tmpbuf[0] & ZmqMsg::more);

        self.next_step (self._in_progress.data_mut(), self._in_progress.size (),
                        &V1Decoder::message_ready);

        return 0;
    }

    // int zmq::v1_decoder_t::message_ready (unsigned char const *)
    pub unsafe fn message_ready(&mut self, buf: &[u8]) -> i32
    {
        //  Message is completely read. Push it further and start reading
        //  new message. (in_progress is a 0-byte message after this point.)
        self.next_step (self._tmpbuf, 1, &V1Decoder::one_byte_size_ready);
        return 1;
    }
}
