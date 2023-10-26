use crate::defines::ZMQ_REP;
use crate::msg::{MSG_MORE, ZmqMsg};
use crate::options::ZmqOptions;

pub struct ZmqRep {
    pub router: router_t,
    pub _sending_reply: bool,
    pub _request_begins: bool,
}

impl ZmqRep {
    pub fn new(options: &mut ZmqOptions, ctx: &mut crate::ctx::ZmqContext, sid: i32) -> Self {
        let router = router_t::new(ctx, tid, sid);
        options.type_ = ZMQ_REP;
        Self {
            router,
            _sending_reply: false,
            _request_begins: true,
        }
    }

    pub unsafe fn xsend(&mut self, msg_: &mut ZmqMsg) -> i32
    {
        //  If we are in the middle of receiving a request, we cannot send reply.
        if (!self._sending_reply) {
            // errno = EFSM;
            return -1;
        }

        let more = flag_set(msg_.flags(), MSG_MORE);

        //  Push message to the reply pipe.
        let rc = self.router.xsend (msg_);
        if (rc != 0) {
            return rc;
        }

        //  If the reply is complete flip the FSM back to request receiving state.
        if (!more) {
            self._sending_reply = false;
        }

        return 0;
    }

    // int zmq::rep_t::xrecv (msg_t *msg_)
    pub unsafe fn xrecv(&mut self, msg_: &mut ZmqMsg) -> i32
    {
        //  If we are in middle of sending a reply, we cannot receive next request.
        if (self._sending_reply) {
            // errno = EFSM;
            return -1;
        }

        //  First thing to do when receiving a request is to copy all the labels
        //  to the reply pipe.
        if (self._request_begins) {
            loop {
                let rc = self.router.xrecv (msg_);
                if (rc != 0) {
                    return rc;
                }

                if ((msg_.flags () & MSG_MORE)) {
                    //  Empty message part delimits the traceback stack.
                    let bottom = (msg_.size () == 0);

                    //  Push it to the reply pipe.
                    rc = self.router.xsend (msg_);
                    // errno_assert (rc == 0);

                    if (bottom) {
                        break;
                    }
                } else {
                    //  If the traceback stack is malformed, discard anything
                    //  already sent to pipe (we're at end of invalid message).
                    rc = self.router.rollback ();
                    // errno_assert (rc == 0);
                }
            }
            self._request_begins = false;
        }

        //  Get next message part to return to the user.
        let rc = self.router.xrecv (msg_);
        if (rc != 0) {
            return rc;
        }

        //  If whole request is read, flip the FSM to reply-sending state.
        if (!(msg_.flags () & MSG_MORE)) {
            self._sending_reply = true;
            self._request_begins = true;
        }

        return 0;
    }

    // bool zmq::rep_t::xhas_in ()
    pub unsafe fn xhas_in(&mut self) -> bool
    {
        if (self._sending_reply) {
            return false;
        }

        return self.router.xhas_in ();
    }

    // bool zmq::rep_t::xhas_out ()
    pub unsafe fn xhas_out(&mut self) -> bool
    {
        if (!self._sending_reply) {
            return false;
        }

        return self.router.xhas_out ();
    }
}
