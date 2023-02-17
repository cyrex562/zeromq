#[derive(Default,Debug,Clone)]
pub struct ZmqPollerEvent
{
    pub socket: Vec<u8>,
    pub fd: i32,
    pub user_data: Vec<u8>,
    pub events: i16,
}
