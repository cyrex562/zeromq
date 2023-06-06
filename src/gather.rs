use crate::context::ZmqContext;
use crate::defines::ZMQ_GATHER;
use crate::fq::ZmqFq;
use crate::message::{ZmqMessage, ZMQ_MSG_MORE};
use crate::pipe::ZmqPipe;
use crate::socket_base::ZmqSocketBase;

#[derive(Default, Debug, Clone)]
pub struct ZmqGather {
    //
    //     ZmqGather (ZmqContext *parent_, tid: u32, sid_: i32);
    //     ~ZmqGather ();

    //
    //  Overrides of functions from ZmqSocketBase.
    // void xattach_pipe (pipe: &mut ZmqPipe,
    //                    subscribe_to_all_: bool,
    //                    locally_initiated_: bool);
    // int xrecv (msg: &mut ZmqMessage);
    // bool xhas_in ();
    // void xread_activated (pipe: &mut ZmqPipe);
    // void xpipe_terminated (pipe: &mut ZmqPipe);

    //
    //  Fair queueing object for inbound pipes.
    pub fair_queue: ZmqFq,
    pub socket_base: ZmqSocketBase,
    // // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqGather)
}

impl ZmqGather {
    pub fn new(parent: &mut ZmqContext, tid: u32, sid_: i32) -> Self {
        // :
        //     ZmqSocketBase (parent_, tid, sid_, true)
        // options.type = ZMQ_GATHER;
        let mut out = Self {
            ..Default::default()
        };
        options.type_ = ZMQ_GATHER as i32;
        out
    }

    pub fn xattach_pipe(
        &mut self,
        pipe: &mut ZmqPipe,
        subscribe_to_all_: bool,
        locally_initiated_: bool,
    ) {
        // LIBZMQ_UNUSED (subscribe_to_all_);
        // LIBZMQ_UNUSED (locally_initiated_);

        // zmq_assert (pipe);
        self.fair_queue.attach(pipe);
    }

    pub fn xread_activated(&mut self, pipe: &mut ZmqPipe) {
        self.fair_queue.activated(pipe);
    }

    pub fn xpipe_terminated(&mut self, pipe: &mut ZmqPipe) {
        self.fair_queue.pipe_terminated(pipe);
    }

    pub fn xrecv(&mut self, msg: &mut ZmqMessage) -> anyhow::Result<()> {
        let mut rc = self.fair_queue.recvpipe(msg, None);

        // Drop any messages with more flag
        while msg.flags() & ZMQ_MSG_MORE != 0 {
            // drop all frames of the current multi-frame message
            self.fair_queue.recvpipe(msg, None)?;

            while msg.flags() & ZMQ_MSG_MORE != 0 {
                self.fair_queue.recvpipe(msg, None)?;
            }

            // get the new message
            // if rc == 0 {
            //     rc = self.fair_queue.recvpipe(msg, None)?;
            // }
            self.fair_queue.recvpipe(msg, None)?;
        }

        Ok(())
    }

    pub fn xhas_in(&mut self) -> bool {
        return self.fair_queue.has_in();
    }
}

// ZmqGather::~ZmqGather ()
// {
// }
