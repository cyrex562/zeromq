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

impl ZmqFairQueue {
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

    pub fn activated(&mut self, pipe_: &mut ZmqPipe) {
        // self.pipes.swap(self.pipes.index(pipe_).unwrap(), self.active);
        todo!()
    }

    pub unsafe fn recv(&mut self, msg_: &mut ZmqMsg) -> i32 {
        self.recvpipe(msg_, None)
    }

    pub unsafe fn recvpipe(&mut self, msg_: &mut ZmqMsg, pipe_: Option<&mut ZmqPipe>) -> i32 {
        let mut rc = (*msg_).close();

        while self.active > 0 {
            let fetched = (*self.pipes[self.current]).read(msg_);

            if fetched {
                if pipe_.is_some() {
                    *pipe_ = self.pipes[self.current];
                }
                self.more = msg_.flags() & ZMQ_MSG_MORE != 0;
                if !self.more {
                    self.current = self.current + 1 % self.active;
                }
                return 0;
            }

            self.active -= 1;
            self.pipes.swap(self.current, self.active);
            if self.current == self.active {
                self.current = 0;
            }
        }

        rc = (*msg_).init2();
        return -1;
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
