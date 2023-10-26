use crate::command::ZmqCommand;
use crate::i_mailbox::IMailbox;
use crate::ypipe::ZmqYPipe;
use crate::config::COMMAND_PIPE_GRANULARITY;
use crate::signaler::ZmqSignaler;

#[derive()]
pub struct ZmqMailbox
{
    pub interface: IMailbox,
    pub _cpipe: ZmqYPipe<ZmqCommand, COMMAND_PIPE_GRANULARITY>,
    pub _signaler: ZmqSignaler,
    pub _sync: mutex_t, 
    pub _active: bool,
}
