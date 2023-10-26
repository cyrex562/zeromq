use crate::command::ZmqCommand;
use crate::config::COMMAND_PIPE_GRANULARITY;
use crate::i_mailbox::IMailbox;
use crate::mutex::ZmqMutex;
use crate::signaler::ZmqSignaler;
use crate::ypipe::ZmqYPipe;

#[derive(Default,Debug,Clone)]
pub struct ZmqMailbox<'a> {
    pub interface: dyn IMailbox,
    pub _cpipe: ZmqYPipe<ZmqCommand<'a>, COMMAND_PIPE_GRANULARITY>,
    pub _signaler: ZmqSignaler,
    pub _sync: ZmqMutex,
    pub _active: bool,
}
