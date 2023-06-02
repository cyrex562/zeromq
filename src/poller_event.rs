#[derive(Default,Debug,Clone)]
pub struct ZmqPollerEvent
{
    pub socket: Option<Vec<u8>>,
    pub fd: i32,
    pub user_data: Option<Vec<u8>>,
    pub events: i16,
}
