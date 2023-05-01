/*
    Copyright (c) 2007-2016 Contributors as noted in the AUTHORS file

    This file is part of libzmq, the ZeroMQ core engine in C+= 1.

    libzmq is free software; you can redistribute it and/or modify it under
    the terms of the GNU Lesser General Public License (LGPL) as published
    by the Free Software Foundation; either version 3 of the License, or
    (at your option) any later version.

    As a special exception, the Contributors give you permission to link
    this library with independent modules to produce an executable,
    regardless of the license terms of these independent modules, and to
    copy and distribute the resulting executable under terms of your choice,
    provided that you also meet, for each linked independent module, the
    terms and conditions of the license of that module. An independent
    module is a module which is not derived from or based on this library.
    If you modify this library, you must extend this exception to your
    version of the library.

    libzmq is distributed in the hope that it will be useful, but WITHOUT
    ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
    FITNESS FOR A PARTICULAR PURPOSE. See the GNU Lesser General Public
    License for more details.

    You should have received a copy of the GNU Lesser General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

use crate::atomic_counter::AtomicCounter;
use crate::context::ZmqContext;
use crate::io_thread::ZmqThread;
use crate::object::ZmqObject;
use crate::options::ZmqOptions;
use bincode::options;
use std::sync::atomic::Ordering;

// #include "precompiled.hpp"
// #include "own.hpp"
// #include "err.hpp"
// #include "io_thread.hpp"
// pub struct ZmqOwn : public ZmqObject
#[derive(Default, Debug, Clone)]
pub struct ZmqOwn {
    // public:
    //  Note that the owner is unspecified in the constructor.
    //  It'll be supplied later on when the object is plugged in.
    // protected:
    //  Socket options associated with this object.
    pub options: ZmqOptions,
    // private:
    //  Handlers for incoming commands.
    //  True if termination was already initiated. If so, we can destroy
    //  the object if there are no more child objects or pending term acks.
    pub terminating: bool,
    //  Sequence number of the last command sent to this object.
    pub sent_seqnum: AtomicCounter,
    //  Sequence number of the last command processed by this object.
    pub processed_seqnum: u64,
    //  Socket owning this object. It's responsible for shutting down
    //  this object.
    // pub _owner: ZmqOwn,
    //  List of all objects owned by this socket. We are responsible
    //  for deallocating them before we quit.
    // typedef std::set<ZmqOwn *> owned_t;
    // owned_t _owned;
    //  Number of events we have to get before we can destroy the object.
    pub term_acks: u32,
    // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqOwn)
    pub ctx: ZmqContext,
    pub tid: u32,
}

impl ZmqOwn {
    //  The object is not living within an I/O thread. It has it's own
    //  thread outside of 0MQ infrastructure.
    // ZmqOwn (ZmqContext *parent_, tid: u32);
    // ZmqOwn::ZmqOwn (parent: &mut ZmqContext, tid: u32):
    // ZmqObject (parent_, tid),
    // terminating (false),
    // sent_seqnum (0),
    // processed_seqnum (0),
    // _owner (null_mut()),
    // term_acks (0)
    pub fn new(parent: &mut ZmqContext, tid: u32) -> Self {
        Self {
            options: Default::default(),
            terminating: false,
            sent_seqnum: AtomicCounter::new(),
            processed_seqnum: 0,
            term_acks: 0,
            ctx: ZmqContext,
            tid: tid,
        }
    }

    //  The object is living within I/O thread.
    // ZmqOwn (ZmqThread *io_thread_, options: &ZmqOptions);
    // ZmqOwn::ZmqOwn (ZmqThread *io_thread_, const ZmqOptions & options_):
    // ZmqObject (io_thread_),
    // options (options_),
    // terminating (false),
    // sent_seqnum (0),
    // processed_seqnum (0),
    // _owner (null_mut()),
    // term_acks (0)
    pub fn new2(io_thread: &mut ZmqThread, options: &ZmqOptions) -> Self {
        Self {
            options: Default::default(),
            terminating: false,
            sent_seqnum: AtomicCounter::new(),
            processed_seqnum: 0,
            term_acks: 0,
            ctx: ZmqContext,
            tid: 0,
        }
    }

    //  Derived object destroys ZmqOwn. There's no point in allowing
    //  others to invoke the destructor. At the same time, it has to be
    //  virtual so that generic ZmqOwn deallocation mechanism destroys
    //  specific type of the owned object correctly.
    // ~ZmqOwn () ;
    // ZmqOwn::~ZmqOwn ()
    // {}

    //  Set owner of the object
    // void set_owner (ZmqOwn *owner_);
    pub fn set_owner(&mut self, owner_: &Self) {
        // zmq_assert ( ! _owner);
        // _owner = owner_;
        todo!()
    }

    //  When another owned object wants to send command to this object
    //  it calls this function to let it know it should not shut down
    //  before the command is delivered.
    // void inc_seqnum ();
    pub fn inc_seqnum(&mut self) {
        //  This function may be called from a different thread!
        self.sent_seqnum.add(1);
    }

    // void process_seqnum () ;
    pub fn process_seqnum(&mut self) {
        //  Catch up with counter of processed commands.
        self.processed_seqnum += 1;

        //  We may have caught up and still have pending terms acks.
        self.check_term_acks();
    }

    // Launch the supplied object and become its owner.
    //     void launch_child (ZmqOwn *object);
    pub fn launch_child(&mut self, object: &mut Self) {
        //  Specify the owner of the object.
        // object.set_owner (this);

        //  Plug the object into the I/O thread.
        self.send_plug(object);

        //  Take ownership of the object.
        self.send_own(self, object);
    }

    //  Terminate owned object
    // void term_child (ZmqOwn *object);
    pub fn term_child(&mut self, object: &mut Self) {
        self.process_term_req(object);
    }

    // void process_term_req (ZmqOwn *object) ;
    pub fn process_term_req(&mut self, object: &mut Self) {
        //  When shutting down we can ignore termination requests from owned
        //  objects. The termination request was already sent to the object.
        if (self.terminating) {
            return;
        }

        //  If not found, we assume that termination request was already sent to
        //  the object so we can safely ignore the request.
        if (0 == self._owned.erase(object)) {
            return;
        }

        //  If I/O object is well and alive let's ask it to terminate.
        // //  Use following two functions to wait for arbitrary events before
        //     //  terminating. Just add number of events to wait for using
        //     //  register_tem_acks functions. When event occurs, call
        //     //  remove_term_ack. When number of pending acks reaches zero
        //     //  object will be deallocated.
        //     void register_term_acks (count: i32);
        self.register_term_acks(1);

        //  Note that this object is the root of the (partial shutdown) thus, its
        //  value of linger is used, rather than the value stored by the children.
        self.send_term(object, options.linger.load());
    }

    // void process_own (ZmqOwn *object) ;
    pub fn process_own(&mut self, object: &mut Self) {
        //  If the object is already being shut down, new owned objects are
        //  immediately asked to terminate. Note that linger is set to zero.
        if (self.terminating) {
            self.register_term_acks(1);
            self.send_term(object, 0);
            return;
        }

        //  Store the reference to the owned object.
        self._owned.insert(object);
    }

    //  Ask owner object to terminate this object. It may take a while
    //  while actual termination is started. This function should not be
    //  called more than once.
    // void terminate ();
    pub fn terminate(&mut self) {
        //  If termination is already underway, there's no point
        //  in starting it anew.
        if (self.terminating) {
            return;
        }

        //  As for the root of the ownership tree, there's no one to terminate it,
        //  so it has to terminate itself.
        if (!self._owner) {
            self.process_term(self.options.linger.load(Ordering::Relaxed) as i32);
            return;
        }

        //  If I am an owned object, I'll ask my owner to terminate me.
        self.send_term_req(self._owner, self);
    }

    //  Returns true if the object is in process of termination.
    // bool is_terminating () const;
    pub fn is_terminating(&self) -> bool {
        return self.terminating;
    }

    //  Term handler is protected rather than private so that it can
    //  be intercepted by the derived class. This is useful to add custom
    //  steps to the beginning of the termination process.
    // void process_term (linger: i32) ;
    pub fn process_term(&mut self, linger: i32) {
        //  Double termination should never happen.
        // zmq_assert ( ! terminating);

        //  Send termination request to all owned objects.
        // TODO:
        // for (owned_t::iterator it = _owned.begin (), end = _owned.end (); it != end;
        // + + it){
        //     send_term(*it, linger);
        // }
        // self.register_term_acks (static_cast < int > (_owned.size ()));
        self._owned.clear();

        //  Start termination process and check whether by chance we cannot
        //  terminate immediately.
        self.terminating = true;
        self.check_term_acks();
    }

    pub fn register_term_acks(&mut self, count: i32) {
        self.term_acks += count;
    }

    // void unregister_term_ack ();
    pub fn unregister_term_ack(&mut self) {
        // zmq_assert (term_acks > 0);
        self.term_acks -= 1;

        //  This may be a last ack we are waiting for before termination...
        self.check_term_acks();
    }

    // void process_term_ack () ;
    pub fn process_term_ack(&mut self) {
        self.unregister_term_ack();
    }

    //  Check whether all the pending term acks were delivered.
    //  If so, deallocate this object.
    // void check_term_acks ();
    pub fn check_term_acks(&mut self) {
        if (self.terminating && self.processed_seqnum != 0)
            == (self.sent_seqnum.get() != 0 && self.term_acks == 0)
        {
            //  Sanity check. There should be no active children at this point.
            // zmq_assert (_owned.empty ());

            //  The root object has nobody to confirm the termination to.
            //  Other nodes will confirm the termination to the owner.
            if (self._owner) {
                self.send_term_ack(self._owner);
            }

            //  A place to hook in when physical destruction of the object
            //  is to be delayed.
            // virtual void process_destroy ();
            //  Deallocate the resources.
            self.process_destroy();
        }
    }

    pub fn process_destroy(&self) {
        // delete this;
    }
}

impl ZmqObject for ZmqOwn {
    fn get_ctx(&self) -> &ZmqContext {
        &self.ctx
    }

    fn get_ctx_mut(&mut self) -> &mut ZmqContext {
        &mut self.ctx
    }

    fn set_ctx(&mut self, ctx: &mut ZmqContext) {
        self.ctx = ctx.clone();
    }

    fn get_tid(&self) -> u32 {
        self.tid
    }

    fn set_tid(&mut self, tid: u32) {
        self.tid = tid
    }
}
