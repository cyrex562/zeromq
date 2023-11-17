use crate::ctx::ZmqContext;
use crate::defines::err::ZmqError;
use crate::defines::err::ZmqError::PipeError;
use crate::defines::ZMQ_MSG_MORE;
use crate::msg::ZmqMsg;
use crate::pipe::ZmqPipe;

// pub type ZmqPipes = ZmqArray<ZmqPipe, 1>;

pub struct ZmqFairQueue<'a> {
    pub pipes: [&'a mut ZmqPipe<'a>; 1],
    pub active: usize,
    pub current: usize,
    pub more: bool,
}

impl<'a> ZmqFairQueue<'a> {
    pub fn new() -> Self {
        Self {
            pipes: [&mut ZmqPipe::default(); 1],
            active: 0,
            current: 0,
            more: false,
        }
    }

    pub fn attach(&mut self, pipe_: &mut ZmqPipe) {
        self.pipes.push_back(pipe_);
        self.pipes.swap(self.active, self.pipes.size() - 1);
        self.active += 1;
    }

    pub fn pipe_terminated(&mut self, pipe_: &mut ZmqPipe) {
        // let index = self.pipes.index(pipe_).unwrap();
        let index = self.pipes.iter().position(|&x| x == pipe_).unwrap();
        if index < self.active {
            self.active -= 1;
            self.pipes.swap(index, self.active);
            if self.current == self.active {
                self.current = 0;
            }
        }
        self.pipes[0].default();
    }

    pub fn activated(&mut self, pipe_: &mut ZmqPipe) -> Result<(),ZmqError> {
        // self.pipes.swap(self.pipes.index(pipe_).unwrap(), self.active);
        todo!()
    }

    pub fn recv(&mut self, ctx: &mut ZmqContext, msg_: &mut ZmqMsg) -> Result<(),ZmqError> {
        self.recvpipe(ctx, msg_, None)
    }

    pub fn recvpipe(&mut self, ctx: &mut ZmqContext, msg: &mut ZmqMsg, pipe: &mut Option<&mut ZmqPipe>) -> Result<(),ZmqError> {
        (msg).close()?;

        while self.active > 0 {
            let fetched = (*self.pipes[self.current]).read(ctx, msg)?;

            if fetched {
                if pipe.is_some() {
                    // *pipe = Some(self.pipes[self.current]);
                    pipe.replace(self.pipes[self.current])
                }
                self.more = msg.flags() & ZMQ_MSG_MORE != 0;
                if !self.more {
                    self.current = self.current + 1 % self.active;
                }
                return Ok(());
            }

            self.active -= 1;
            self.pipes.swap(self.current, self.active);
            if self.current == self.active {
                self.current = 0;
            }
        }

        (msg).init2()?;
        return Err(PipeError("recvpipe failed"));
    }

    pub fn has_in(&mut self) -> bool {
        if self.more {
            return true;
        }

        while self.active > 0 {
            if self.pipes[self.current].check_read() {
                return true;
            }

            self.active -= 1;
            self.pipes.swap(self.current, self.active);
            if self.current == self.active {
                self.current = 0;
            }
        }

        return false;
    }
}
