#![allow(non_camel_case_types)]

pub trait i_poll_events
{
    fn in_event(&mut self);
    fn out_event(&mut self);
    fn timer_event(&mut self, id_: i32);
}
