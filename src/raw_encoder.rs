use crate::encoder::encoder_base_t;

pub struct raw_encoder_t {
    pub encoder_base: encoder_base_t,

}

impl raw_encoder_t {
    pub fn new(&mut self, bufsize_: usize) -> Self {
        let mut out = Self {
            encoder_base: encoder_base_t::new(bufsize_),
        };
        self.encoder_base.next_step(None, 0, true, self.raw_message_ready);
        out
    }
    
    pub unsafe fn raw_message_ready(&mut self) {
        self.encoder_base.next_step((*self.encoder_base.in_progress()).data(), self.encoder_base.in_progress().size(), true, self.raw_message_ready)
    }
}
