use crate::ctx::ctx_t;
use crate::defines::{ZMQ_SUB, ZMQ_SUBSCRIBE, ZMQ_UNSUBSCRIBE};
use crate::msg::{close_and_return, msg_t};
use crate::options::options_t;
use crate::xsub::xsub_t;

pub struct sub_t<'a> {
    pub xsub: xsub_t<'a>,
}

impl sub_t {
    pub unsafe fn new(options: &mut options_t, parent_: &mut ctx_t, tid_: u32, sid_: i32) -> Self {
        options.type_ = ZMQ_SUB;
        options.filter = true;
        Self {
            xsub: xsub_t::new(options, parent_, tid_, sid_),
        }
    }

    pub unsafe fn xsetsockopt(option_: i32, optval_: &[u8], optvallen_: usize) -> i32 {
        if (option_ != ZMQ_SUBSCRIBE && option_ != ZMQ_UNSUBSCRIBE) {
            // errno = EINVAL;
            return -1;
        }

        //  Create the subscription message.
        // msg_t msg;
        let mut msg = msg_t::default();
        // int rc;
        let mut rc = 0i32;
        let data = (optval_);
        if (option_ == ZMQ_SUBSCRIBE) {
            rc = msg.init_subscribe(optvallen_, data);
        } else {
            rc = msg.init_cancel(optvallen_, data);
        }
        // errno_assert (rc == 0);

        //  Pass it further on in the stack.
        rc = xsub_t::xsend(&mut msg);
        return close_and_return(&mut msg, rc);
    }

    // int zmq::sub_t::xsend (msg_t *)
    pub unsafe fn xsend(&mut self, msg: &mut msg_t) -> i32 {
        //  Override the XSUB's send.
        // errno = ENOTSUP;
        return -1;
    }

    // bool zmq::sub_t::xhas_out ()
    pub unsafe fn xhas_out(&mut self) -> bool {
        //  Override the XSUB's send.
        return false;
    }
}
