use crate::command::command_t;
use crate::i_mailbox::i_mailbox;
use crate::ypipe::ypipe_t;
use crate::config::command_pipe_granularity;
use crate::singaler::signaler_t;

#[derive()]
pub struct mailbox_t
{
    pub interface: i_mailbox,
    pub _cpipe: ypipe_t<command_t, command_pipe_granularity>,
    pub _signaler: signaler_t,
    pub _sync: mutex_t, 
    pub _active: bool,
}