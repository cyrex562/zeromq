use crate::command::command_t;

pub trait i_mailbox {
    fn send(&mut self, cmd_: &mut command_t);
    fn recv(&mut self, cmd_: &mut command_t, timeout_: i32) -> i32;
    #[cfg(feature="forked")]
    fn forked();
}