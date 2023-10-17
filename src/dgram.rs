use crate::msg::{MSG_MORE, msg_t};
use crate::pipe::pipe_t;
use crate::socket_base::socket_base_t;

pub struct dgram_t<'a> {
    pub socket_base: socket_base_t<'a>,
    pub _pipe: Option<&'a mut pipe_t<'a>>,
    pub _more_out: bool,
}

impl dgram_t {
    pub unsafe fn new(options: &mut options_t,
                      parent_: &mut ctx_t,
                      tid_: u32,
                      sid_: i32) -> dgram_t {
        dgram_t {
            socket_base: socket_base_t::new(parent_, tid_, sid_, false),
            _pipe: None,
            _more_out: false,
        }
    }

    pub unsafe fn xattach_pipe(&mut self, pipe_: &mut pipe_t, subscribe_to_all: bool, locally_initiated: bool) {
        if self._pipe.is_none() {
            self._pipe = Some(pipe_);
        } else {
            pipe_.terminate(false);
        }
    }

    pub unsafe fn xpipe_terminated(&mut self, pipe_: &mut pipe_t) {
        if self._pipe.is_some() && self._pipe.unwrap() == pipe_ {
            self._pipe = None;
        }
    }

    pub unsafe fn xread_activated(&mut self, pipe_: &mut pipe_t) {
        unimplemented!()
    }

    pub unsafe fn xwrite_activated(&mut self, pipe_: &mut pipe_t) {
        unimplemented!()
    }

    pub unsafe fn xsend(&mut self, msg_: &mut msg_t) -> i32 {
        // If there's no out pipe, just drop it.
        if (!self._pipe) {
            let mut rc = msg_.close();
            // errno_assert (rc == 0);
            return -1;
        }

        //  If this is the first part of the message it's the ID of the
        //  peer to send the message to.
        if (!self._more_out) {
            if (!(msg_.flags() & MSG_MORE != 0)) {
                // errno = EINVAL;
                return -1;
            }
        } else {
            //  dgram messages are two part only, reject part if more is set
            if (msg_.flags() & MSG_MORE) {
                // errno = EINVAL;
                return -1;
            }
        }

        // Push the message into the pipe.
        if (!self._pipe.write(msg_)) {
            // errno = EAGAIN;
            return -1;
        }

        if (!(msg_.flags() & MSG_MORE))
        self._pipe.flush();

        // flip the more flag
        self._more_out = !self._more_out;

        //  Detach the message from the data buffer.
        let rc = msg_.init();
        // errno_assert (rc == 0);

        return 0;
    }

    pub unsafe fn xrecv(&mut self, msg_: &mut msg_t) -> i32 {
        //  Deallocate old content of the message.
        let mut rc = msg_.close();
        // errno_assert (rc == 0);

        if (!self._pipe || !self._pipe.read(msg_)) {
            //  Initialise the output parameter to be a 0-byte message.
            rc = msg_.init2();
            // errno_assert (rc == 0);

            // errno = EAGAIN;
            return -1;
        }

        return 0;
    }

    pub unsafe fn xhas_in(&mut self) -> bool {
        if (self._pipe.is_none()) {
            return false;
        }

        return self._pipe.check_read();
    }

    pub unsafe fn xhas_out(&mut self) -> bool {
        if (self._pipe.is_none()) {
            return false;
        }

        return self._pipe.check_write();
    }
}
