use crate::message::{ZmqMessage, ZMQ_MSG_MORE};
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;
use crate::socket_base::{ZmqContext, ZmqSocketBase};
use crate::socket_base_ops::ZmqSocketBaseOps;
use anyhow::anyhow;
use libc::socket;

#[derive(Default, Debug, Clone)]
pub struct ZmqChannel
//: public ZmqSocketBase
{
    // public:
    //     channel_t (ZmqContext *parent_, uint32_t tid, sid_: i32);
    //     ~channel_t ();
    //
    //     //  Overrides of functions from ZmqSocketBase.
    //     void xattach_pipe (ZmqPipe *pipe_,
    //                        bool subscribe_to_all_,
    //                        bool locally_initiated_);
    //     int xsend (ZmqMessage *msg);
    //     int xrecv (ZmqMessage *msg);
    //     bool xhas_in ();
    //     bool xhas_out ();
    //     void xread_activated (ZmqPipe *pipe_);
    //     void xwrite_activated (ZmqPipe *pipe_);
    //     void xpipe_terminated (ZmqPipe *pipe_);

    // private:
    //   ZmqPipe *_pipe;

    // ZMQ_NON_COPYABLE_NOR_MOVABLE (channel_t)
    pipe: Option<ZmqPipe>,
    base: ZmqSocketBase,
}

impl ZmqChannel {
    pub fn new(parent: &mut ZmqContext, options: &mut ZmqOptions, tid: u32, sid: i32) -> Self {
        let mut out = Self {
            pipe: Default::default(),
            base: ZmqSocketBase::new(parent, options, tid, sid, true),
        };

        out
    }

    // channel_t::~channel_t ()
    // {
    //     zmq_assert (!_pipe);
    // }

    // channel_t::channel_t (class ZmqContext *parent_, tid: u32, sid_: i32) :
    // ZmqSocketBase (parent_, tid, sid_, true), _pipe (null_mut())
    // {
    // options.type = ZMQ_CHANNEL;
    // }
}

impl ZmqSocketBaseOps for ZmqChannel {
    fn xattach_pipe(
        &mut self,
        skt_base: &mut ZmqSocketBase,
        in_pipe: &mut ZmqPipe,
        subscribe_to_all: bool,
        locally_initiated: bool,
    ) {
        // LIBZMQ_UNUSED (subscribe_to_all_);
        // LIBZMQ_UNUSED (locally_initiated_);

        // zmq_assert (pipe_ != null_mut());

        //  ZMQ_PAIR socket can only be connected to a single peer.
        //  The socket rejects any further connection requests.
        // if (_pipe == null_mut())
        // _pipe = pipe_;
        // else
        // pipe_.terminate (false);
        // }
        if self.pipe.is_none() {
            self.pipe = Some(in_pipe.clone());
        } else {
            in_pipe.terminate(false);
        }
    }

    fn xpipe_terminated(&mut self, skt_base: &mut ZmqSocketBase, pipe: &mut ZmqPipe) {
        if (pipe == self.pipe.unwrap()) {
            self.pipe = None;
        }
    }

    fn xwrite_activated(&mut self, skt_base: &mut ZmqSocketBase, pipe: &mut ZmqPipe) {
        //  There's just one pipe. No lists of active and inactive pipes.
        //  There's nothing to do here.
        unimplemented!()
    }

    fn xsend(&mut self, skt_base: &mut ZmqSocketBase, msg: &mut ZmqMessage) -> anyhow::Result<()> {
        //  CHANNEL sockets do not allow multipart data (ZMQ_SNDMORE)
        if (msg.flags() & ZMQ_MSG_MORE) {
            // errno = EINVAL;
            // return -1;
            return Err(anyhow!(
                "invalid state: channel sockets do not allow multipart data"
            ));
        }

        if (self.pipe.is_none() || !self.pipe.unwrap().write(msg)) {
            return Err(anyhow!("EAGAIN"));
        }

        self.pipe.flush();

        //  Detach the original message from the data buffer.
        let rc: i32 = msg.init2();
        // errno_assert (rc == 0);

        Ok(())
    }

    fn xrecv(&mut self, skt_base: &mut ZmqSocketBase, msg: &mut ZmqMessage) -> anyhow::Result<()> {
        //  Deallocate old content of the message.
        let mut rc = msg.close();
        errno_assert(rc == 0);

        if (self.pipe.is_none()) {
            //  Initialise the output parameter to be a 0-byte message.
            rc = msg.init2();
            errno_assert(rc == 0);
            return Err(anyhow!("error EAGAIN"));
        }

        // Drop any messages with more flag
        let read = self.pipe.unwrap().read(msg);
        while (read && msg.flags() & ZMQ_MSG_MORE) {
            // drop all frames of the current multi-frame message
            read = self.pipe.unwrap().read(msg);
            while (read && msg.flags() & ZMQ_MSG_MORE) {
                read = self.pipe.unwrap().read(msg);
            }

            // get the new message
            if (read) {
                read = self.pipe.unwrap().read(msg);
            }
        }

        if (!read) {
            //  Initialise the output parameter to be a 0-byte message.
            rc = msg.init2();
            errno_assert(rc == 0);
            return Err(anyhow!("EAGAIN"));
            // errno = EAGAIN;
            // return -1;
        }

        // return 0;
        Ok(())
    }

    fn xhas_in(&mut self, skt_base: &mut ZmqSocketBase) -> bool {
        if (self.pipe.is_none()) {
            return false;
        }

        return self.pipe.unwrap().check_read();
    }

    fn xhas_out(&mut self, skt_base: &mut ZmqSocketBase) -> bool {
        if (self.pipe.is_none()) {
            return false;
        }

        return self.pipe.unwrap().check_write();
    }
}
