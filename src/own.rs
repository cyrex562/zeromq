

use std::collections::{HashSet};
use crate::atomic_counter::ZmqAtomicCounter;
use crate::ctx::ZmqContext;
use crate::io_thread::ZmqIoThread;
use crate::object::ZmqObject;
use crate::options::ZmqOptions;

pub struct ZmqOwn<'a>
{
    pub object: ZmqObject<'a>,
    pub options: ZmqOptions,
    pub terminating: bool,
    pub sent_seqnum: ZmqAtomicCounter,
    pub processed_seqnum: u64,
    pub owner: Option<&'a mut ZmqOwn<'a>>, // really own_t
    pub owned: HashSet<&'a mut ZmqOwn<'a>>,
    pub term_acks: i32,
}

impl ZmqOwn {
    pub fn new(parent_: &mut ZmqContext, tid_: u32) -> ZmqOwn {
        ZmqOwn {
            object: ZmqObject::new(parent_, tid_),
            options: ZmqOptions::new(),
            terminating: false,
            sent_seqnum: ZmqAtomicCounter::new(0),
            processed_seqnum: 0,
            owner: None,
            owned: HashSet::new(),
            term_acks: 0,
        }
    }
    
    pub unsafe fn new2(io_thread_: *mut ZmqIoThread, options_: &ZmqOptions) -> Self {
        Self {
            object: ZmqObject::new2(&mut (*io_thread_).object),
            options: ZmqOptions::new(),
            terminating: false,
            sent_seqnum: ZmqAtomicCounter::new(0),
            processed_seqnum: 0,
            owner: None,
            owned: HashSet::new(),
            term_acks: 0,
        }
    }
    
    pub fn set_owner(&mut self, owner_: &mut Self) {
        self.owner = Some(owner_);
    }
    
    pub fn inc_seqnum(&mut self) {
        self.sent_seqnum.inc();
    }
    
    pub fn process_seqnum (&mut self)
    {
        //  Catch up with counter of processed commands.
        self.processed_seqnum += 1;
    
        //  We may have caught up and still have pending terms acks.
        self.check_term_acks ();
    }
    
    pub unsafe fn launch_child (&mut self, object_: *mut ZmqOwn)
    {
        //  Specify the owner of the object.
        (*object_).set_owner (self);
    
        //  Plug the object into the I/O thread.
        self.send_plug (object_);
    
        //  Take ownership of the object.
        self.send_own (self, object_);
    }
    
    pub fn term_child(&mut self, object_: *mut ZmqOwn)
    {
        self.process_term_req (object_);
    }
    
    pub fn process_term_req (&mut self, object_: *mut ZmqOwn)
    {
        //  When shutting down we can ignore termination requests from owned
        //  objects. The termination request was already sent to the object.
        if (self.terminating) {
            return;
        }
    
        //  If not found, we assume that termination request was already sent to
        //  the object so we can safely ignore the request.
        if (0 == self.owned.erase (object_)) {
            return;
        }
    
        //  If I/O object is well and alive let's ask it to terminate.
        self.register_term_acks (1);
    
        //  Note that this object is the root of the (partial shutdown) thus, its
        //  value of linger is used, rather than the value stored by the children.
        self.send_term (object_, self.options.linger.load ());
    }
    
    pub fn process_own (&mut self, object_: &mut ZmqOwn)
    {
        //  If the object is already being shut down, new owned objects are
        //  immediately asked to terminate. Note that linger is set to zero.
        if self.terminating {
            self.register_term_acks (1);
            self.send_term (object_, 0);
            return;
        }
    
        //  Store the reference to the owned object.
        self.owned.insert (object_);
    }
    
    pub fn terminate (&mut self)
    {
        //  If termination is already underway, there's no point
        //  in starting it anew.
        if self.terminating {
            return;
        }
    
        //  As for the root of the ownership tree, there's no one to terminate it,
        //  so it has to terminate itself.
        if self.owner.is_none() {
            self.process_term (self.options.linger.load ());
            return;
        }
    
        //  If I am an owned object, I'll ask my owner to terminate me.
        self.send_term_req (self.owner, self);
    }
    
    pub fn is_terminating (&mut self) -> bool
    {
        return self.terminating;
    }
    
    pub fn process_term (&mut self, linger_: i32)
    {
        //  Double termination should never happen.
        // zmq_assert (!_terminating);
    
        //  Send termination request to all owned objects.
        // for (owned_t::iterator it = _owned.begin (), end = _owned.end (); it != end;
        //      ++it)
        for it in self.owned.iter() {
            self.send_term(*it, linger_);
        }
        self.register_term_acks ((self.owned.size ()));
        self.owned.clear ();
    
        //  Start termination process and check whether by chance we cannot
        //  terminate immediately.
        self.terminating = true;
        self.check_term_acks ();
    }
    
    pub fn register_term_acks (&mut self, count_: i32)
    {
        self.term_acks += count_;
    }
    
    pub fn unregister_term_ack (&mut self)
    {
        // zmq_assert (_term_acks > 0);
        self.term_acks -= 1;
    
        //  This may be a last ack we are waiting for before termination...
        self.check_term_acks ();
    }
    
    pub fn process_term_ack (&mut self)
    {
        self.unregister_term_ack ();
    }
    
    pub fn check_term_acks (&mut self)
    {
        if self.terminating && self.processed_seqnum == self.sent_seqnum.get() as u64
            && self.term_acks == 0 {
            //  Sanity check. There should be no Active children at this point.
            // zmq_assert (_owned.empty ());
    
            //  The root object has nobody to confirm the termination to.
            //  Other nodes will confirm the termination to the owner.
            if self.owner {
                self.send_term_ack(self.owner);
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
