use crate::msg::msg_t;

pub trait i_encoder {
    fn encode(&mut self, data_: *mut *mut u8, size_: usize);

    fn load_msg(&mut self, msg_: *mut msg_t);
}
