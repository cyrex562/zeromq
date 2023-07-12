

pub struct channel_t
{
    base: socket_base_t,
    _pipe: *mut pipe_t,
}

impl channel_t
{
    pub fn new(options: &mut options_t, parent: *mut ctx_t, tid_: u32, sid_: i32) -> Self
    {
        options.type_ = ZMQ_CHANNEL;
        Self {
            base: socket_base_t::new(parent, tid_, sid_, true),
            _pipe: null_mut(),
        }
    }

    pub fn xattach_pipe(&mut self, pipe_: *mut pipe_t, subscribe_to_all_: bool, local_initiated_: bool) {
        if self._pipe 
    }
}