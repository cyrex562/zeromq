use crate::context::ZmqContext;
use crate::fq::ZmqFq;
use crate::proxy::ZmqSocketBase;
use crate::socket_base::ZmqSocketBase;

#[derive(Default,Debug,Clone)]
pub struct ZmqGather
{
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

    pub fn new(parent: &mut ZmqContext, tid: u32, sid_: i32) -> Self
    {
        // :
        //     ZmqSocketBase (parent_, tid, sid_, true)
        // options.type = ZMQ_GATHER;
        let mut out = Self {
            socket_base: ZmqSocketBase::new()
            ..Default::default()
        };
        out.socket_base.
    }
}



ZmqGather::~ZmqGather ()
{
}

void ZmqGather::xattach_pipe (pipe: &mut ZmqPipe,
                                  subscribe_to_all_: bool,
                                  locally_initiated_: bool)
{
    LIBZMQ_UNUSED (subscribe_to_all_);
    LIBZMQ_UNUSED (locally_initiated_);

    // zmq_assert (pipe);
    fair_queue.attach (pipe);
}

void ZmqGather::xread_activated (pipe: &mut ZmqPipe)
{
    fair_queue.activated (pipe);
}

void ZmqGather::xpipe_terminated (pipe: &mut ZmqPipe)
{
    fair_queue.pipe_terminated (pipe);
}

int ZmqGather::xrecv (msg: &mut ZmqMessage)
{
    int rc = fair_queue.recvpipe (msg, null_mut());

    // Drop any messages with more flag
    while (rc == 0 && msg.flags () & ZMQ_MSG_MORE) {
        // drop all frames of the current multi-frame message
        rc = fair_queue.recvpipe (msg, null_mut());

        while (rc == 0 && msg.flags () & ZMQ_MSG_MORE)
            rc = fair_queue.recvpipe (msg, null_mut());

        // get the new message
        if (rc == 0)
            rc = fair_queue.recvpipe (msg, null_mut());
    }

    return rc;
}

bool ZmqGather::xhas_in ()
{
    return fair_queue.has_in ();
}
