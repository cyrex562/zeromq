

pub trait IPollEvents
{
    fn in_event(&mut self);
    fn out_event(&mut self);
    fn timer_event(&mut self, id_: i32);
}
