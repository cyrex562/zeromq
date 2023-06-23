#[derive(Default, Debug, Clone)]
pub struct ZmqSocketStats {
    // u64 msg_in;
    pub msg_in: u64,
    // u64 bytes_in;
    pub bytes_in: u64,
    // u64 msg_out;
    pub msg_out: u64,
    // u64 bytes_out;
    pub bytes_out: u64,
}
