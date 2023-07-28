use crate::command::command_t;

pub struct i_mailbox
{
    pub send: fn(cmd_: &command_t),
    pub recv: fn(cmd_: *mut command_t, timeout_: i32) -> i32,
    #[cfg(feature="fork")]
    pub forked: fn(),
}