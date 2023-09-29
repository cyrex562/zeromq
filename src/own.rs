use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use crate::atomic_counter::atomic_counter_t;
use crate::ctx::ctx_t;
use crate::io_thread::io_thread_t;
use crate::object::object_t;
use crate::options::options_t;

pub struct own_t
{
    pub object: object_t,
    pub options: options_t,
    pub _terminating: bool,
    pub _sent_seqnum: atomic_counter_t,
    pub _processed_seqnum: u64,
    pub _owner: *mut own_t, // really own_t
    pub _owned: HashSet<*mut own_t>,
    pub _term_acks: i32,
}

impl own_t {
    pub fn new(parent_: *mut ctx_t, tid_: u32) -> own_t {
        own_t {
            object: object_t::new(parent_, tid_),
            options: options_t::new(),
            _terminating: false,
            _sent_seqnum: atomic_counter_t::new(0),
            _processed_seqnum: 0,
            _owner: std::ptr::null_mut(),
            _owned: HashSet::new(),
            _term_acks: 0,
        }
    }
    
    pub unsafe fn new2(io_thread_: *mut io_thread_t, options_: &options_t) -> Self {
        Self {
            object: object_t::new2(&mut (*io_thread_).object),
            options: options_t::new(),
            _terminating: false,
            _sent_seqnum: atomic_counter_t::new(0),
            _processed_seqnum: 0,
            _owner: std::ptr::null_mut(),
            _owned: HashSet::new(),
            _term_acks: 0,
        }
    }
    
    pub fn set_owner(&mut self, owner_: *mut Self) {
        self._owner = owner_;
    }
    
    pub fn inc_seqnum(&mut self) {
        self._sent_seqnum.inc();
    }
    
    pub fn process_seqnum (&mut self)
    {
        //  Catch up with counter of processed commands.
        self._processed_seqnum += 1;
    
        //  We may have caught up and still have pending terms acks.
        self.check_term_acks ();
    }
    
    pub unsafe fn launch_child (&mut self, object_: *mut own_t)
    {
        //  Specify the owner of the object.
        (*object_).set_owner (self);
    
        //  Plug the object into the I/O thread.
        self.send_plug (object_);
    
        //  Take ownership of the object.
        self.send_own (self, object_);
    }
    
    pub fn term_child(&mut self, object_: *mut own_t)
    {
        self.process_term_req (object_);
    }
    
    pub fn process_term_req (&mut self, object_: *mut own_t)
    {
        //  When shutting down we can ignore termination requests from owned
        //  objects. The termination request was already sent to the object.
        if (self._terminating) {
            return;
        }
    
        //  If not found, we assume that termination request was already sent to
        //  the object so we can safely ignore the request.
        if (0 == self._owned.erase (object_)) {
            return;
        }
    
        //  If I/O object is well and alive let's ask it to terminate.
        self.register_term_acks (1);
    
        //  Note that this object is the root of the (partial shutdown) thus, its
        //  value of linger is used, rather than the value stored by the children.
        self.send_term (object_, self.options.linger.load ());
    }
    
    pub fn process_own (&mut self, object_: *mut own_t)
    {
        //  If the object is already being shut down, new owned objects are
        //  immediately asked to terminate. Note that linger is set to zero.
        if (self._terminating) {
            self.register_term_acks (1);
            self.send_term (object_, 0);
            return;
        }
    
        //  Store the reference to the owned object.
        self._owned.insert (object_);
    }
    
    pub fn terminate (&mut self)
    {
        //  If termination is already underway, there's no point
        //  in starting it anew.
        if (self._terminating) {
            return;
        }
    
        //  As for the root of the ownership tree, there's no one to terminate it,
        //  so it has to terminate itself.
        if (!self._owner) {
            self.process_term (self.options.linger.load ());
            return;
        }
    
        //  If I am an owned object, I'll ask my owner to terminate me.
        self.send_term_req (self._owner, self);
    }
    
    pub fn is_terminating (&mut self) -> bool
    {
        return self._terminating;
    }
    
    pub fn process_term (&mut self, linger_: i32)
    {
        //  Double termination should never happen.
        // zmq_assert (!_terminating);
    
        //  Send termination request to all owned objects.
        // for (owned_t::iterator it = _owned.begin (), end = _owned.end (); it != end;
        //      ++it)
        for it in self._owned.iter() {
            self.send_term(*it, linger_);
        }
        self.register_term_acks ((self._owned.size ()));
        self._owned.clear ();
    
        //  Start termination process and check whether by chance we cannot
        //  terminate immediately.
        self._terminating = true;
        self.check_term_acks ();
    }
    
    pub fn register_term_acks (&mut self, count_: i32)
    {
        self._term_acks += count_;
    }
    
    pub fn unregister_term_ack (&mut self)
    {
        // zmq_assert (_term_acks > 0);
        self._term_acks -= 1;
    
        //  This may be a last ack we are waiting for before termination...
        self.check_term_acks ();
    }
    
    pub fn process_term_ack (&mut self)
    {
        self.unregister_term_ack ();
    }
    
    pub fn check_term_acks (&mut self)
    {
        if (self._terminating && self._processed_seqnum == self._sent_seqnum.get() as u64
            && self._term_acks == 0) {
            //  Sanity check. There should be no active children at this point.
            // zmq_assert (_owned.empty ());
    
            //  The root object has nobody to confirm the termination to.
            //  Other nodes will confirm the termination to the owner.
            if (self._owner) {
                self.send_term_ack(self._owner);
            }
    
            //  Deallocate the resources.
            self.process_destroy ();
        }
    }
    
    pub fn process_destroy (&mut self)
    {
        // delete this;
    }
}