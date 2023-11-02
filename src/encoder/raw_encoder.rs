use crate::encoder::ZmqEncoder;

pub struct ZmqRawEncoder {
    pub encoder_base: ZmqEncoder,

}

impl ZmqRawEncoder {
    pub fn new(bufsize_: usize) -> Self {
        let mut out = Self {
            encoder_base: ZmqEncoder::new(bufsize_),
        };
        out.encoder_base.next_step(None, 0, true, out.raw_message_ready);
        out
    }
    
    pub unsafe fn raw_message_ready(&mut self) {
        self.encoder_base.next_step((*self.encoder_base.in_progress()).data_mut(), self.encoder_base.in_progress().size(), true, self.raw_message_ready)
    }
}
