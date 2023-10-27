use crate::encoder::ZmqEncoderBase;
use crate::defines::MSG_MORE;
use crate::utils::put_u64;

pub struct V1Encoder {
    pub encoder_base: ZmqEncoderBase<V1Encoder>,
    pub _tmpbuf: [u8;11],
}

impl V1Encoder
{
    pub fn new(bufsize_: usize) -> Self {
        let mut out = Self {
            encoder_base: ZmqEncoderBase::new(bufsize_),
            _tmpbuf: [0;11],
        };
        out.next_step(None, 0, out.message_ready, true);
    }

    pub unsafe fn size_ready(&mut self)  {
        self.next_step(self.in_progress().data(), self.in_progress().size(), self.message_ready, true);
    }

    pub unsafe fn message_ready(&mut self) {
        let mut header_size = 2; // flags byte + size byte
        //  Get the message size.
        let mut size = self.in_progress().size ();

        //  Account for the 'flags' byte.
        size += 1;

        //  Account for the subscribe/cancel byte.
        if (self.in_progress().is_subscribe() || self.in_progress().is_cancel()){
            size += 1;
        }

        //  For messages less than 255 bytes long, write one byte of message size.
        //  For longer messages write 0xff escape character followed by 8-byte
        //  message size. In both cases 'flags' field follows.
        if (size < u8::MAX) {
            self._tmpbuf[0] = size as u8;
            self._tmpbuf[1] = (self.in_progress().flags() & MSG_MORE);
        } else {
            self._tmpbuf[0] = u8::MAX;
            put_u64 (self._tmpbuf + 1, size);
            self._tmpbuf[9] = (self.in_progress().flags () & MSG_MORE);
            header_size = 10;
        }

        //  Encode the subscribe/cancel byte. This is Done in the encoder as
        //  opposed to when the subscribe message is created to allow different
        //  protocol behaviour on the wire in the v3.1 and legacy encoders.
        //  It results in the work being Done multiple times in case the sub
        //  is sending the subscription/cancel to multiple pubs, but it cannot
        //  be avoided. This processing can be moved to xsub once support for
        //  ZMTP < 3.1 is dropped.
        if (self.in_progress ().is_subscribe ()){
            self._tmpbuf[header_size+ +] = 1;
        }
        else if (self.in_progress ().is_cancel ()) {
            self._tmpbuf[header_size] = 0;
            header_size += 1;
        }

        self.next_step (self._tmpbuf, header_size, &V1Encoder::size_ready, false);
    }
}
