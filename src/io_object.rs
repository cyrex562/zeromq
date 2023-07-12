use crate::defines::ZmqHandle;
use crate::defines::ZmqFileDesc;
use crate::thread_context::ZmqThreadContext;


#[derive(Default, Debug, Clone)]
pub struct ZmqIoObject {
    // pub ZmqPollEventsInterface: ZmqPollEventsInterface,
    pub poller: Option<ZmqHandle>,
}

impl ZmqIoObject {
    // ZmqIoObject (ZmqIoThread *io_thread_ = null_mut());
    pub fn new(io_thread_: Option<&mut ZmqThreadContext>) -> Self {
        // : poller (null_mut())
        //     if (io_thread_)
        //         Plug (io_thread_);
        let mut out = Self {
            // ZmqPollEventsInterface: Default::default(),
            poller: None,
        };
        if io_thread_.is_some() {
            out.plug(io_thread_.unwrap());
        }

        out
    }

    // ~ZmqIoObject () ;

    //  When migrating an object from one I/O thread to another, first
    //  unplug it, then migrate it, then Plug it to the new thread.
    // void Plug (ZmqIoThread *io_thread_);
    pub fn plug(&mut self, io_thread: &mut ZmqThreadContext) {
        // zmq_assert (io_thread_);
        // zmq_assert (!poller);
        //  Retrieve the poller from the thread we are running in.
        self.poller = io_thread.get_poller();
    }

    // void unplug ();
    pub fn unplug(&mut self) {
        // zmq_assert (poller);
        //  Forget about old poller in preparation to be migrated
        //  to a different I/O thread.
        self.poller = None;
    }

    //  Methods to access underlying poller object.
    // handle_t add_fd (ZmqFileDesc fd);
    pub fn add_fd(&mut self, fd: ZmqFileDesc) -> ZmqFileDesc {
        return self.poller.add_fd(fd, self);
    }

    // void rm_fd (handle_t handle_);
    pub fn rm_fd(&mut self, handle: ZmqHandle) {
        self.poller.rm_fd(handle_);
    }

    // void set_pollin (handle_t handle_);
    pub fn set_pollin(&mut self, handle_: ZmqFileDesc) {
        self.poller.set_pollin(handle_);
    }

    // void reset_pollin (handle_t handle_);
    pub fn reset_pollin(&mut self, handle_: ZmqFileDesc) {
        self.poller.reset_pollin(handle_);
    }

    // void set_pollout (handle_t handle_);
    pub fn set_pollout(&mut self, handle_: ZmqFileDesc) {
        self.poller.set_pollout(handle_);
    }

    // void reset_pollout (handle_t handle_);
    pub fn reset_pollout(&mut self, handle_: ZmqFileDesc) {
        self.poller.reset_pollout(handle_);
    }

    // void add_timer (timeout: i32, id_: i32);
    pub fn add_timer(&mut self, timeout: i32, id_: i32) {
        self.poller.add_timer(timeout, self, id_);
    }

    // void cancel_timer (id_: i32);
    pub fn cancel_timer(&mut self, id_: i32) {
        self.poller.cancel_timer(self, id_);
    }
}

impl ZmqPollEventsInterface for ZmqIoObject {
    //  i_poll_events interface implementation.
    // void in_event () ;
    fn in_event(&mut self) {
        // zmq_assert (false);
    }

    // void out_event () ;
    fn out_event(&mut self) {
        // zmq_assert (false);
    }

    // void timer_event (id_: i32) ;
    fn timer_event(&mut self, id_: i32) {
        // zmq_assert (false);
    }
}

// ZmqIoObject::~ZmqIoObject ()
// {
// }
