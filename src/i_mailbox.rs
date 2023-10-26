use crate::command::ZmqCommand;

pub trait IMailbox {
    fn send(&mut self, cmd_: &ZmqCommand);
    fn recv(&mut self, cmd_: &ZmqCommand, timeout_: i32) -> i32;
    #[cfg(feature="forked")]
    fn forked();
}
