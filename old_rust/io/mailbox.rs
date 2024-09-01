use crate::command::ZmqCommand;
use crate::defines::err::ZmqError;
use crate::defines::err::ZmqError::{MailboxError, PipeError};
use crate::defines::mutex::ZmqMutex;
use crate::defines::ZmqConditionVariable;
use crate::io::signaler::ZmqSignaler;
use crate::ypipe::ZmqYPipe;

#[derive(Default, Debug, Clone)]
pub struct ZmqMailbox<'a> {
    pub cpipe: ZmqYPipe<'a, ZmqCommand<'a>>,
    pub signaler: ZmqSignaler,
    pub sync: ZmqMutex,
    pub active: bool,
    pub _cond_var: ZmqConditionVariable,
    pub _signalers: Vec<&'a mut ZmqSignaler>,
}

impl<'a> ZmqMailbox<'a> {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn add_signaler(&mut self, signaler_: &mut ZmqSignaler) {
        self._signalers.push(signaler_);
    }

    pub fn remove_signaler(&mut self, signaler_: &mut ZmqSignaler) {
        let index = self
            ._signalers
            .iter()
            .position(|&x| x == signaler_)
            .unwrap();
        self._signalers.remove(index);
    }

    pub fn clear_signalers(&mut self) {
        self._signalers.clear();
    }

    pub fn send(&mut self, cmd_: &mut ZmqCommand) {
        self.sync.lock();
        self.cpipe.write(cmd_, false);
        let ok = self.cpipe.flush();
        if !ok {
            self._cond_var.broadcast();
            for it in self._signalers.iter() {
                unsafe {
                    (*it).send();
                }
            }
        }
        self.sync.unlock();
    }

    pub fn recv(&mut self, cmd: &mut ZmqCommand, timeout_: i32) -> Result<(),ZmqError> {
        if self.cpipe.read(cmd) {
            return Ok(());
        }

        if timeout_ == 0 {
            self.sync.unlock();
            self.sync.lock();
        } else {
            let rc = self._cond_var.wait(&self.sync, timeout_);
            if rc == -1 {
                return Err(MailboxError("wait failed"));
            }
        }

        let ok = self.cpipe.read(cmd);
        if !ok {
            return Err(PipeError("read failed"));
        }

       Ok(())
    }
}
