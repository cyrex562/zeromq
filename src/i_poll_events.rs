pub struct i_poll_events
{
    pub in_event: fn(),
    pub out_event: fn(),
    pub timer_event: fn(id_: i32),
}