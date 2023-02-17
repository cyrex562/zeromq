use libc::socket;
use crate::pipe::pipe_t;
use crate::socket_base::{ZmqSocketBase, ZmqContext};

#[derive!(Default,Debug,Clone)]
pub struct channel_t //: public ZmqSocketBase
{
// public:
//     channel_t (ZmqContext *parent_, uint32_t tid_, sid_: i32);
//     ~channel_t ();
//
//     //  Overrides of functions from ZmqSocketBase.
//     void xattach_pipe (pipe_t *pipe_,
//                        bool subscribe_to_all_,
//                        bool locally_initiated_);
//     int xsend (ZmqMessage *msg);
//     int xrecv (ZmqMessage *msg);
//     bool xhas_in ();
//     bool xhas_out ();
//     void xread_activated (pipe_t *pipe_);
//     void xwrite_activated (pipe_t *pipe_);
//     void xpipe_terminated (pipe_t *pipe_);

  // private:
  //   pipe_t *_pipe;

    // ZMQ_NON_COPYABLE_NOR_MOVABLE (channel_t)
    pipe: pipe_t,
    base: ZmqSocketBase,
}

impl channel_t {
    pub fn new(parent: *mut ZmqContext, tid: u32, sid: i32) -> Self {
        Self {
            pipe: Default::default(),
            base: ZmqSocketBase {
                parent,
                tid,
                sid,
                true

            }
        }
    }
}


channel_t::channel_t (class ZmqContext *parent_, uint32_t tid_, sid_: i32) :
    ZmqSocketBase (parent_, tid_, sid_, true), _pipe (NULL)
{
    options.type = ZMQ_CHANNEL;
}

channel_t::~channel_t ()
{
    zmq_assert (!_pipe);
}

void channel_t::xattach_pipe (pipe_t *pipe_,
                                   bool subscribe_to_all_,
                                   bool locally_initiated_)
{
    LIBZMQ_UNUSED (subscribe_to_all_);
    LIBZMQ_UNUSED (locally_initiated_);

    zmq_assert (pipe_ != NULL);

    //  ZMQ_PAIR socket can only be connected to a single peer.
    //  The socket rejects any further connection requests.
    if (_pipe == NULL)
        _pipe = pipe_;
    else
        pipe_->terminate (false);
}

void channel_t::xpipe_terminated (pipe_t *pipe_)
{
    if (pipe_ == _pipe)
        _pipe = NULL;
}

void channel_t::xread_activated (pipe_t *)
{
    //  There's just one pipe. No lists of active and inactive pipes.
    //  There's nothing to do here.
}

void channel_t::xwrite_activated (pipe_t *)
{
    //  There's just one pipe. No lists of active and inactive pipes.
    //  There's nothing to do here.
}

int channel_t::xsend (ZmqMessage *msg)
{
    //  CHANNEL sockets do not allow multipart data (ZMQ_SNDMORE)
    if (msg->flags () & ZmqMessage::more) {
        errno = EINVAL;
        return -1;
    }

    if (!_pipe || !_pipe->write (msg)) {
        errno = EAGAIN;
        return -1;
    }

    _pipe->flush ();

    //  Detach the original message from the data buffer.
    let rc: i32 = msg->init ();
    errno_assert (rc == 0);

    return 0;
}

int channel_t::xrecv (ZmqMessage *msg)
{
    //  Deallocate old content of the message.
    int rc = msg->close ();
    errno_assert (rc == 0);

    if (!_pipe) {
        //  Initialise the output parameter to be a 0-byte message.
        rc = msg->init ();
        errno_assert (rc == 0);

        errno = EAGAIN;
        return -1;
    }

    // Drop any messages with more flag
    bool read = _pipe->read (msg);
    while (read && msg->flags () & ZmqMessage::more) {
        // drop all frames of the current multi-frame message
        read = _pipe->read (msg);
        while (read && msg->flags () & ZmqMessage::more)
            read = _pipe->read (msg);

        // get the new message
        if (read)
            read = _pipe->read (msg);
    }

    if (!read) {
        //  Initialise the output parameter to be a 0-byte message.
        rc = msg->init ();
        errno_assert (rc == 0);

        errno = EAGAIN;
        return -1;
    }

    return 0;
}

bool channel_t::xhas_in ()
{
    if (!_pipe)
        return false;

    return _pipe->check_read ();
}

bool channel_t::xhas_out ()
{
    if (!_pipe)
        return false;

    return _pipe->check_write ();
}
