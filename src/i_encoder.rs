use crate::msg::ZmqMsg;

pub trait IEncoder {
    fn encode(&mut self, data_: *mut *mut u8, size_: usize);

    fn load_msg(&mut self, msg_: *mut ZmqMsg);
}
