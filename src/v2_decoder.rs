use crate::decoder::decoder_base_t;
use crate::decoder_allocators::shared_message_memory_allocator;
use crate::msg::{MSG_COMMAND, msg_t};
use crate::utils::get_u64;

pub struct v2_decoder_t {
    pub decoder_base: decoder_base_t<v2_decoder_t, shared_message_memory_allocator>,
    pub _tmpbuf: [u8; 8],
    pub _msg_flags: u8,
    pub _in_progress: msg_t,
    pub _zero_copy: bool,
    pub _max_msg_size: i64,
}

impl v2_decoder_t {
    pub fn msg(&mut self) -> &mut msg_t {
        &mut self._in_progress
    }

    pub fn new(bufsize_: usize, maxmsgsize_: i64, zero_copy_: bool) -> Self {
        let mut out = Self {
            decoder_base: decoder_base_t::new(bufsize_),
            _tmpbuf: [0; 8],
            _msg_flags: 0,
            _in_progress: msg_t::default(),
            _zero_copy: zero_copy_,
            _max_msg_size: maxmsgsize_,
        };
        out._in_progress.init2();
        out.next_step(out._tmpbuf, out.one_byte_size_ready);
        out
    }

    pub unsafe fn flags_ready(&mut self, buf: &[u8]) -> i32 {
        self._msg_flags = 0;
        if (self._tmpbuf[0] & more_flag) {
            self._msg_flags |= msg_t::more;
        }
        if (self._tmpbuf[0] & command_flag) {
            self._msg_flags |= MSG_COMMAND;
        }

        //  The payload length is either one or eight bytes,
        //  depending on whether the 'large' bit is set.
        if (self._tmpbuf[0] & large_flag) {
            self.next_step(self._tmpbuf, 8, eight_byte_size_ready);
        } else {
            self.next_step(self._tmpbuf, 1, one_byte_size_ready);
        }

        return 0;
    }

    // int zmq::v2_decoder_t::one_byte_size_ready (unsigned char const *read_from_)
    pub unsafe fn one_byte_size_ready(&mut self, read_from_: &[u8]) -> i32 {
        return self.size_ready(self._tmpbuf[0], read_from_);
    }

    // int zmq::v2_decoder_t::eight_byte_size_ready (unsigned char const *read_from_)
    pub unsafe fn eight_byte_size_ready(&mut self, read_from_: &[u8]) -> i32 {
        //  The payload size is encoded as 64-bit unsigned integer.
        //  The most significant byte comes first.
        let msg_size = get_u64(self._tmpbuf);

        return self.size_ready(msg_size, read_from_);
    }

    // int zmq::v2_decoder_t::size_ready (uint64_t msg_size_, unsigned char const *read_pos_)
    pub unsafe fn size_ready(&mut self, msg_size_: u64, read_pos_: &[u8]) -> i32 {
        //  Message size must not exceed the maximum allowed size.
        if (self._max_msg_size >= 0) {
            if ((msg_size_ > (self._max_msg_size))) {
                // errno = EMSGSIZE;
                return -1;
            }
        }

        //  Message size must fit into size_t data type.
        if ((msg_size_ != (msg_size_))) {
            // errno = EMSGSIZE;
            return -1;
        }

        let rc = self._in_progress.close();
        // assert (rc == 0);

        // the current message can exceed the current buffer. We have to copy the buffer
        // data into a new message and complete it in the next receive.

        let mut allocator = self.get_allocator();
        if ((!self._zero_copy || msg_size_ > (
            allocator.data() + allocator.size() - read_pos_))) {
            // a new message has started, but the size would exceed the pre-allocated arena
            // this happens every time when a message does not fit completely into the buffer
            rc = self._in_progress.init_size((msg_size_));
        } else {
            // construct message using n bytes from the buffer as storage
            // increase buffer ref count
            // if the message will be a large message, pass a valid refcnt memory location as well
            rc = self._in_progress.init((read_pos_),
                                        (msg_size_),
                                        call_dec_ref,
                                        allocator.buffer(), allocator.provide_content());

            // For small messages, data has been copied and refcount does not have to be increased
            if (self._in_progress.is_zcmsg()) {
                allocator.advance_content();
                allocator.inc_ref();
            }
        }

        if ((rc)) {
            // errno_assert (errno == ENOMEM);
            rc = self._in_progress.init();
            // errno_assert (rc == 0);
            // errno = ENOMEM;
            return -1;
        }

        self._in_progress.set_flags(self._msg_flags);
        // this sets read_pos to
        // the message data address if the data needs to be copied
        // for small message / messages exceeding the current buffer
        // or
        // to the current start address in the buffer because the message
        // was constructed to use n bytes from the address passed as argument
        self.next_step(self._in_progress.data(), self._in_progress.size(),
                       &v2_decoder_t::message_ready);

        return 0;
    }

    // int zmq::v2_decoder_t::message_ready (unsigned char const *)
    pub unsafe fn message_ready(&mut self, buf: &[u8]) -> i32 {
        //  Message is completely read. Signal this to the caller
        //  and prepare to decode next message.
        self.next_step(self._tmpbuf, 1, &v2_decoder_t::flags_ready);
        return 1;
    }
}
