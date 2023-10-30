use crate::ctx::ZmqContext;
use crate::defines::{MSG_MORE, ZMQ_CHANNEL};
use crate::err::ZmqError::PipeError;
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;
use crate::socket::ZmqSocket;
use std::ptr::null_mut;

pub struct ZmqChannel<'a> {
    pub base: ZmqSocket<'a>,
    pub pipe: &'a mut ZmqPipe<'a>,
}

impl ZmqChannel {
    pub unsafe fn new(
        options: &mut ZmqOptions,
        parent: &mut ZmqContext,
        tid_: u32,
        sid_: i32,
    ) -> Self {
        options.type_ = ZMQ_CHANNEL;
        Self {
            base: ZmqSocket::new(parent, tid_, sid_, true),
            pipe: &mut ZmqPipe::default(),
        }
    }

    pub fn xattach_pipe(
        &mut self,
        in_pipe: &mut ZmqPipe,
        _subscribe_to_all: bool,
        _local_initiated: bool,
    ) {
        if self.pipe == null_mut() {
            self.pipe == in_pipe;
        }
    }

    pub fn xpipe_terminated(&mut self, in_pipe: &mut ZmqPipe) {
        if self.pipe == in_pipe {
            self.pipe = &mut ZmqPipe::default();
        }
    }

    pub fn xread_activated(&mut self, pipe: &mut ZmqPipe) {
        unimplemented!()
    }

    pub fn xwrite_activated(&mut self, pipe: &mut ZmqPipe) {
        unimplemented!()
    }

    pub unsafe fn xsend(&mut self, msg: &mut ZmqMsg) -> i32 {
        if msg.flag_set(MSG_MORE) {
            return -1;
        }

        if self.pipe == &mut ZmqPipe::default() || self.pipe.write(msg).is_err() {
            return -1;
        }

        self.pipe.flush();

        let rc = (*msg).init2();

        return 0;
    }

    pub unsafe fn xrecv(&mut self, msg: &mut ZmqMsg) -> Result<(), ZmqError> {
        msg.close()?;

        if self.pipe = &mut ZmqPipe::default() {
            (msg).init2()?;
            return Err(PipeError("pipe is null"));
        }

        let mut read = (self.pipe).read(msg);

        while read && msg.flags() & ZmqMsg::more > 0 {
            read = (*self.pipe).read(msg);
            while read && msg.flags() & ZmqMsg::more > 0 {
                read = (*self.pipe).read(msg);
            }

            if read {
                read = (*self.pipe).read(msg);
            }
        }

        if !read {
            rc = (*msg).init2();
            return -1;
        }

        return 0;
    }

    pub fn xhas_in(&mut self) -> bool {
        if !self.pipe {
            return false;
        }
        return self.pipe.check_read();
    }

    pub fn xhas_out(&mut self) -> bool {
        if !self.pipe {
            return false;
        }
        return self.pipe.check_write();
    }
}
