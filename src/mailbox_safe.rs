#![allow(non_camel_case_types)]
#![allow(non_camel_case_types)]

use std::ptr::null_mut;
use crate::command::command_t;
use crate::condition_variable::condition_variable_t;
use crate::i_mailbox::i_mailbox;
use crate::ypipe::ypipe_t;
use crate::config::command_pipe_granularity;
use crate::mutex::mutex_t;
use crate::signaler::signaler_t;

type cpipe_t = ypipe_t<command_t, command_pipe_granularity>;

pub struct mailbox_safe_t {
    pub _cpipe: cpipe_t,
    pub _cond_var: condition_variable_t,
    pub _sync: *mut mutex_t,
    pub _signalers: Vec<*mut signaler_t>,
}

impl mailbox_safe_t {
    pub unsafe fn new() -> mailbox_safe_t {

        Self {
            _cpipe: cpipe_t::new(),
            _cond_var: condition_variable_t::new(),
            _sync: null_mut(),
            _signalers: Vec::new(),
        }
    }

    pub fn add_signaler(&mut self, signaler_: *mut signaler_t) {
        self._signalers.push(signaler_);
    }

    pub fn remove_signaler(&mut self, signaler_: *mut signaler_t) {
        let index = self._signalers.iter().position(|&x| x == signaler_).unwrap();
        self._signalers.remove(index);
    }

    pub fn clear_signalers(&mut self){
        self._signalers.clear();
    }


}

impl i_mailbox for mailbox_safe_t {
    fn send(&mut self, cmd_: &mut command_t) {
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

    unsafe fn recv(&mut self, cmd_: &mut command_t, timeout_: i32) -> i32 {
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
