use crate::transport::ZmqTransport;

pub struct ZmqAddress {
    pub protocol: ZmqTransport,
    pub address: String,
}