use crate::pipe::ZmqPipe;

#[derive(Default,Debug,Clone)]
pub struct ZmqPipes<'a> {
    pub pipes: [ZmqPipe<'a>; 2]
}

impl<'a> ZmqPipes<'a>
{
    pub fn new() -> Self {
        Self {
            pipes: [ZmqPipe::default(), ZmqPipe::default()]
        }
    }

    pub fn has_item(&mut self, pipe_: &mut ZmqPipe) -> bool {
        self.pipes.contains(pipe_)
    }

    pub fn index(&mut self, pipe_: &mut ZmqPipe) -> Option<usize> {
        for i in 0..self.pipes.len() {
            if self.pipes[i] == *pipe_ {
                return Some(i);
            }
        }
        None
    }

    pub fn push_back(&mut self, pipe_: &mut ZmqPipe) {
        self.pipes.push(pipe_);
    }

    pub fn swap(&mut self, index1: usize, index2: usize) {
        self.pipes.swap(index1, index2);
    }

    pub fn erase(&mut self, pipe_: &mut ZmqPipe) {
        self.pipes.remove(self.index(pipe_).unwrap());
    }

    pub fn size(&mut self) -> usize {
        self.pipes.len()
    }
}
