use crate::message::{ZmqMessage, ZMQ_MSG_MORE};
use crate::pipe::ZmqPipe;
use anyhow::anyhow;
use std::ptr::null_mut;




//  This class manages a set of outbound pipes. On send it load balances
//  messages fairly among the pipes.
pub struct LoadBalancer {
    //  List of outbound pipes.
    // typedef array_t<ZmqPipe, 2> pipes_t;
    // pipes_t pipes;
    pub pipes: [ZmqPipe; 2],
    //  Number of active pipes. All the active pipes are located at the
    //  beginning of the pipes array.
    // pipes_t::size_type active;
    pub active: usize,
    //  Points to the last pipe that the most recent message was sent to.
    // pipes_t::size_type _current;
    pub _current: usize,
    //  True if last we are in the middle of a multipart message.
    pub more: bool,
    //  True if we are dropping current message.
    pub _dropping: bool,
    // ZMQ_NON_COPYABLE_NOR_MOVABLE (LoadBalancer)
}

impl LoadBalancer {
    // LoadBalancer ();
    pub fn new() -> Self {
        // : active (0), _current (0), more (false), _dropping (false)
        Self {
            pipes: [ZmqPipe::default(), ZmqPipe::default()],
            active: 0,
            _current: 0,
            more: false,
            _dropping: false,
        }
    }

    // ~LoadBalancer ();

    // void Attach (pipe: &mut ZmqPipe);
    pub fn attach(&mut self, pipe: &mut ZmqPipe) {
        self.pipes.push_back(pipe);
        self.activated(pipe);
    }

    // void activated (pipe: &mut ZmqPipe);
    pub fn activated(&mut self, pipe: &mut ZmqPipe) {
        //  Move the pipe to the list of active pipes.
        self.pipes.swap(self.pipes.index(pipe), self.active);
        self.active += 1;
    }

    // void pipe_terminated (pipe: &mut ZmqPipe);
    pub fn pipe_terminated(&mut self, pipe: &mut ZmqPipe) {
        let index = pipe.pipe_index(pipe, &self.pipes);

        //  If we are in the middle of multipart message and current pipe
        //  have disconnected, we have to drop the remainder of the message.
        if (index == self._current && self.more) {
            self._dropping = true;
        }

        //  Remove the pipe from the list; adjust number of active pipes
        //  accordingly.
        if (index < self.active as i32) {
            self.active -= 1;
            pipe.pipe_swap(index as usize, self.active, &mut self.pipes);
            if (self._current == self.active) {
                self._current = 0;
            }
        }
        pipe.pipe_erase(pipe, &self.pipes);
    }

    // int send (msg: &mut ZmqMessage);
    pub fn send(&mut self, msg: &mut ZmqMessage) -> i32 {
        return self.sendpipe(msg, null_mut());
    }

    //  Sends a message and stores the pipe that was used in pipe_.
    //  It is possible for this function to return success but keep pipe_
    //  unset if the rest of a multipart message to a terminated pipe is
    //  being dropped. For the first frame, this will never happen.
    // int sendpipe (msg: &mut ZmqMessage ZmqPipe **pipe);

    pub fn sendpipe(
        &mut self,
        msg: &mut ZmqMessage,
        pipe: Option<&mut &mut [ZmqPipe]>,
    ) -> anyhow::Result<()> {
        //  Drop the message if required. If we are at the end of the message
        //  switch back to non-dropping mode.
        if (self._dropping) {
            self.more = (msg.flags() & ZMQ_MSG_MORE) != 0;
            self._dropping = self.more;

            let mut rc = msg.close();
            // errno_assert (rc == 0);
            msg.init2();
            // errno_assert (rc == 0);
            return Ok(());
        }

        while (self.active > 0) {
            if (self.pipes[self._current].write(msg)) {
                if (pipe) {
                    *pipe = self.pipes[self._current];
                }
                break;
            }

            // If send fails for multi-part msg rollback other
            // parts sent earlier and return EAGAIN.
            // Application should handle this as suitable
            if (self.more) {
                self.pipes[self._current].rollback();
                // At this point the pipe is already being deallocated
                // and the first N frames are unreachable (_outpipe is
                // most likely already NULL so rollback won't actually do
                // anything and they can't be un-written to deliver later).
                // Return EFAULT to socket_base caller to drop current message
                // and any other subsequent frames to avoid them being
                // "stuck" and received when a new client reconnects, which
                // would break atomicity of multi-part messages (in blocking mode
                // socket_base just tries again and again to send the same message)
                // Note that given dropping mode returns 0, the user will
                // never know that the message could not be delivered, but
                // can't really fix it without breaking backward compatibility.
                // -2/EAGAIN will make sure socket_base caller does not re-enter
                // immediately or after a short sleep in blocking mode.
                self._dropping = (msg.flags() & ZMQ_MSG_MORE) != 0;
                self.more = false;
                // errno = EAGAIN;
                return Err(anyhow!("EAGAIN"));
            }

            self.active -= 1;
            if (self._current < self.active) {
                self.pipes.swap(self._current, self.active);
            } else {
                self._current = 0;
            }
        }

        //  If there are no pipes we cannot send the message.
        if (self.active == 0) {
            // errno = EAGAIN;
            return Err(anyhow!("EAGAIN"));
        }

        //  If it's final part of the message we can flush it downstream and
        //  continue round-robining (load balance).
        self.more = (msg.flags() & ZMQ_MSG_MORE) != 0;
        if (!self.more) {
            self.pipes[self._current].flush();

            if (self._current += 1 >= self.active) {
                self._current = 0;
            }
        }

        //  Detach the message from the data buffer.
        msg.init2();
        // errno_assert (rc == 0);

        Ok(())
    }

    // bool has_out ();
    pub fn has_out(&mut self) -> bool {
        //  If one part of the message was already written we can definitely
        //  write the rest of the message.
        if (self.more) {
            return true;
        }

        while (self.active > 0) {
            //  Check whether a pipe has room for another message.
            if (self.pipes[self._current].check_write()) {
                return true;
            }

            //  Deactivate the pipe.
            self.active -= 1;
            self.pipes.swap(self._current, self.active);
            if (self._current == self.active) {
                self._current = 0;
            }
        }
        return false;
    }
}

// LoadBalancer::~LoadBalancer ()
// {
//     // zmq_assert (pipes.empty ());
// }
