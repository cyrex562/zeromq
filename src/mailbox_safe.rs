


use std::ptr::null_mut;
use crate::command::ZmqCommand;
use crate::condition_variable::ZmqConditionVariable;
use crate::i_mailbox::IMailbox;
use crate::ypipe::ZmqYPipe;
use crate::config::COMMAND_PIPE_GRANULARITY;
use crate::mutex::ZmqMutex;
use crate::signaler::ZmqSignaler;

type ZmqCPipe = ZmqYPipe<ZmqCommand, COMMAND_PIPE_GRANULARITY>;

pub struct ZmqMailboxSafe {
    pub _cpipe: ZmqCPipe,
    pub _cond_var: ZmqConditionVariable,
    pub _sync: *mut ZmqMutex,
    pub _signalers: Vec<*mut ZmqSignaler>,
}

impl ZmqMailboxSafe {
    pub fn new() -> ZmqMailboxSafe {

        Self {
            _cpipe: ZmqCPipe::new(),
            _cond_var: ZmqConditionVariable::new(),
            _sync: null_mut(),
            _signalers: Vec::new(),
        }
    }

    pub fn add_signaler(&mut self, signaler_: *mut ZmqSignaler) {
        self._signalers.push(signaler_);
    }

    pub fn remove_signaler(&mut self, signaler_: *mut ZmqSignaler) {
        let index = self._signalers.iter().position(|&x| x == signaler_).unwrap();
        self._signalers.remove(index);
    }

    pub fn clear_signalers(&mut self){
        self._signalers.clear();
    }


}

impl IMailbox for ZmqMailboxSafe {
    fn send(&mut self, cmd_: &mut ZmqCommand) {
        self._sync.lock();
        self._cpipe.write(cmd_, false);
        let ok = self._cpipe.flush();
        if !ok {
            self._cond_var.broadcast();
            for it in self._signalers.iter() {
                unsafe {
                    (*it).send();
                }
            }
        }
        self._sync.unlock();
    }

    unsafe fn recv(&mut self, cmd_: &mut ZmqCommand, timeout_: i32) -> i32 {
        if self._cpipe.read(cmd) {
            return 0;
        }
        
       if timeout_ == 0 {
           self._sync.unlock();
           self._sync.lock();
       }  else {
           let rc = self._cond_var.wait(self._sync, timeout_);
           if rc ==-1 {
               return -1;
           }
       }
        
        let ok = self._cpipe.read(cmd_);
        if !ok {
            return -1;
        }
        
        return 0;
    }
}
