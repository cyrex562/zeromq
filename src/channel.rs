use std::ptr::null_mut;
use crate::ctx::ctx_t;
use crate::defines::ZMQ_CHANNEL;
use crate::msg::msg_t;
use crate::options::options_t;
use crate::socket_base::socket_base_t;


pub struct channel_t
{
    base: socket_base_t,
    _pipe: *mut pipe_t,
}

impl channel_t
{
    pub fn new(options: &mut options_t, parent: *mut ctx_t, tid_: u32, sid_: i32) -> Self
    {
        options.type_ = ZMQ_CH+ANNEL;
        Self {
            base: socket_base_t::new(parent, tid_, sid_, true),
            _pipe: null_mut(),
        }
    }

    pub fn xattach_pipe(&mut self, pipe_: *mut pipe_t, subscribe_to_all_: bool, local_initiated_: bool) {
        if self._pipe == null_mut() {
            self._pipe == pipe_;
        }
    }

    pub fn xpipe_terminated(&mut self, pipe_: *mut pipe_t) {
        if self._pipe == pipe_ {
            self._pipe = null_mut();
        }
    }

    pub fn xread_activated(&mut self, pipe: *mut pipe_t)
    {
        unimplemented!()
    }

    pub fn xwrite_activated(&mut self, pipe: *mut pipe_t)
    {
        unimplemented!()
    }

    pub unsafe fn xsend(&mut self, msg: *mut msg_t) -> i32
    {
        if msg.flags() & msg_t::more > 0 {
            return -1;
        }

        if self._pipe == null_mut() || !self._pipe.write(msg_)
        {
            return -1;
        }

        self._pipe.flush();

        let rc = msg.init();

        return 0;

    }

    pub unsafe fn xrecv(&mut self, msg: *mut msg_t) -> i32
    {
        let mut rc = msg.close();

        if (!self._pipe) {
            rc = msg.init();
            return -1;
        }

        let read = self._pipe.read(msg);
        
        while(read && msg.flags() & msg_t::more > 0)
        {
            read = self._pipe.read(msg);
            while(read && msg.flags() & msg_t::more > 0)
            {
                read = self._pipe.read(msg);
            }

            if read {
                read = self._pipe.read(msg_);
            }
        }

        if !read {
            rc = msg.init();
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