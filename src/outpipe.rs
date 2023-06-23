use crate::pipe::ZmqPipe;

#[derive(Default,Debug,Clone)]
pub struct ZmqOutpipe
{
    // ZmqPipe *pipe;
    pub pipe: ZmqPipe,
    // active: bool
    pub active: bool,
}
