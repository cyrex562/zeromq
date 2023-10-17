use crate::ctx::ctx_t;
use crate::defines::ZMQ_PAIR;
use crate::msg::{MSG_MORE, msg_t};
use crate::options::options_t;
use crate::pipe::pipe_t;
use crate::socket_base::socket_base_t;

pub struct pair_t<'a> {
    pub socket_base: socket_base_t<'a>,
    pub _pipe: Option<&'a mut pipe_t<'a>>,
}

impl pair_t {
    pub unsafe fn new(options: &mut options_t, parent_: &mut ctx_t, tid_: u32, sid_: i32) -> pair_t {
        let mut out = Self {
            socket_base: socket_base_t::new(parent_, tid_, sid_),
            _pipe: pipe_t::default(),
        };
        options.type_ = ZMQ_PAIR;
        out
    }

    pub unsafe fn xattach_pipe(&mut self, pipe_: &mut pipe_t, subscribe_to_all_: bool, locally_initiated_: bool) {
        if self._pipe.is_none() {
            self._pipe = Some(pipe_);
        } else {
            self._pipe.as_mut().unwrap().terminate(false);
        }
    }

    pub unsafe fn xpipe_terminated(&mut self, pipe_: &mut pipe_t) {
        if pipe_ == self._pipe {
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
        if (!self._pipe || !self._pipe.write (msg_)) {
            // errno = EAGAIN;
            return -1;
        }

        if msg_.flag_clear(MSG_MORE) == true{
        self._pipeflush ();}

        //  Detach the original message from the data buffer.
        let rc = msg_.init2 ();
        // errno_assert (rc == 0);

        return 0;
    }

    pub unsafe fn xrecv(&mut self, msg_: &mut msg_t) -> i32 {
        //  Deallocate old content of the message.
        let rc = msg_.close ();
        // errno_assert (rc == 0);

        if (!self._pipe.is_none() || !self._pipe.read (msg_)) {
            //  Initialise the output parameter to be a 0-byte message.
            rc = msg_.init2();
            // errno_assert (rc == 0);

            // errno = EAGAIN;
            return -1;
        }
        return 0;
    }

    pub unsafe fn xhas_in (&mut self) -> bool
    {
        if (self._pipe.is_none()) {
            return false;
        }

        return self._pipe.check_read ();
    }

    pub unsafe fn xhas_out (&mut self) -> bool
    {
        if (self._pipe.is_none()) {
            return false;
        }

        return self._pipe.check_write ();
    }
}
