use crate::defines::err::ZmqError;
use crate::pipe::ZmqPipe;

#[derive(Default,Debug,Clone)]
pub struct PipeEvent {

}

impl PipeEvent {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn read_activated(&mut self, pipe: &mut ZmqPipe) -> Result<(),ZmqError> {
        unimplemented!()
    }

    pub fn write_activated(&mut self, pipe: &mut ZmqPipe) -> Result<(),ZmqError> {
        unimplemented!()
    }

    pub fn hiccuped(&mut self, pipe: &mut ZmqPipe) -> Result<(),ZmqError> {
        unimplemented!()
    }

    pub fn pipe_terminated(&mut self, pipe: &mut ZmqPipe) -> Result<(),ZmqError> {
        unimplemented!()
    }
}
