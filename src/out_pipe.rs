use std::collections::HashMap;
use crate::pipe::ZmqPipe;

pub struct ZmqOutpipe<'a> {
    pub pipe: &'a mut ZmqPipe<'a>,
    pub active: bool,
}

pub type ZmqOutPipes = HashMap<u32, ZmqOutpipe>;
