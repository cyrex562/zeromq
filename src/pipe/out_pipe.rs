use std::collections::HashMap;
use crate::pipe::ZmqPipe;

pub struct ZmqOutpipe<'a> {
    pub pipe: &'a mut ZmqPipe<'a>,
    pub active: bool,
}

pub type ZmqOutPipes<'a> = HashMap<u32, ZmqOutpipe<'a>>;

// pub struct out_pipe_t<'a> {
//     pub pipe: ZmqPipe<'a>,
//     pub active: bool,
// }
