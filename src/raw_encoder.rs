use crate::encoder::ZmqEncoderBase;

pub struct ZmqRawEncoder {
    pub encoder_base: ZmqEncoderBase,

}

impl ZmqRawEncoder {
    pub fn new(&mut self, bufsize_: usize) -> Self {
        let mut out = Self {
            encoder_base: ZmqEncoderBase::new(bufsize_),
        };
        self.encoder_base.next_step(None, 0, true, self.raw_message_ready);
        out
    }
    
    pub unsafe fn raw_message_ready(&mut self) {
        self.encoder_base.next_step((*self.encoder_base.in_progress()).data_mut(), self.encoder_base.in_progress().size(), true, self.raw_message_ready)
    }
}
