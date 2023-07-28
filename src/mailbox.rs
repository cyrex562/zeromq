use crate::command::command_t;
use crate::i_mailbox::i_mailbox;

pub struct mailbox_t
{
    pub interface: i_mailbox,
    pub _cpipe: ypipe<command_t, command_pipe_granularity>,
    pub _signaler: signaler_t,
    pub _sync: mutex_t, 
    pub _active: bool,
}