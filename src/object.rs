use crate::ctx::ctx_t;

pub struct object_t
{
    pub _ctx: *const ctx_t,
    pub _tid: u32,
}

impl object_t
{
    pub fn new(ctx_: *mut ctx_t, tid_: u32) -> Self{
        Self { _ctx: ctx_, _tid: tid_ }
    }
    
    pub unsafe fn new2(parent: *mut Self) -> Self {
        Self { _ctx: (*parent)._ctx, _tid: (*parent)._tid }
    }
    
    pub fn get_tid(&self) -> u32 {
        self._tid
    }
    
    pub fn set_tid(&mut self, id_: u32) {
        self._tid = id_;
    }
    
    pub fn process_command(&mut self, cmd_: &command_t)
    {
        match cmd_._type {
            
        }
    }
}