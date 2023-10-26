use std::ptr::null_mut;
use crate::ctx::ZmqContext;
use crate::defines::ZMQ_CHANNEL;
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;
use crate::socket_base::ZmqSocketBase;


pub struct ZmqChannel<'a>
{
    pub base: ZmqSocketBase<'a>,
    pub _pipe: &'a mut ZmqPipe<'a>,
}

impl ZmqChannel
{
    pub unsafe fn new(options: &mut ZmqOptions, parent: &mut ZmqContext, tid_: u32, sid_: i32) -> Self
    {
        options.type_ = ZMQ_CHANNEL;
        Self {
            base: ZmqSocketBase::new(parent, tid_, sid_, true),
            _pipe: null_mut(),
        }
    }

    pub fn xattach_pipe(&mut self, pipe_: &mut ZmqPipe, subscribe_to_all_: bool, local_initiated_: bool) {
        if self._pipe == null_mut() {
            self._pipe == pipe_;
        }
    }

    pub fn xpipe_terminated(&mut self, pipe_: &mut ZmqPipe) {
        if self._pipe == pipe_ {
            self._pipe = null_mut();
        }
    }

    pub fn xread_activated(&mut self, pipe: &mut ZmqPipe)
    {
        unimplemented!()
    }

    pub fn xwrite_activated(&mut self, pipe: &mut ZmqPipe)
    {
        unimplemented!()
    }

    pub unsafe fn xsend(&mut self, msg: &mut ZmqMsg) -> i32
    {
        if msg.flags() & ZmqMsg::more > 0 {
            return -1;
        }

        if self._pipe == null_mut() || !(*self._pipe).write(msg)
        {
            return -1;
        }

        self._pipe.flush();

        let rc = (*msg).init2();

        return 0;

    }

    pub unsafe fn xrecv(&mut self, msg: &mut ZmqMsg) -> i32
    {
        let mut rc = (*msg).close();

        if (!self._pipe) {
            rc = (*msg).init2();
            return -1;
        }

        let mut read = (*self._pipe).read(msg);
        
        while read && msg.flags() & ZmqMsg::more > 0
        {
            read = (*self._pipe).read(msg);
            while read && msg.flags() & ZmqMsg::more > 0
            {
                read = (*self._pipe).read(msg);
            }

            if read {
                read = (*self._pipe).read(msg);
            }
        }

        if !read {
            rc = (*msg).init2();
            return -1;
        }

        return 0;
    }

    pub fn xhas_in(&mut self) -> bool
    {
        if !self._pipe {
            return false;
        }
        return self._pipe.check_read();
    }

    pub fn xhas_out(&mut self) -> bool
    {
        if !self._pipe {
            return false;
        }
        return self._pipe.check_write();
    }
}
