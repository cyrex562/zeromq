/*
    Copyright (c) 2007-2016 Contributors as noted in the AUTHORS file

    This file is part of libzmq, the ZeroMQ core engine in C++.

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

// #include "precompiled.hpp"
// #include "own.hpp"
// #include "err.hpp"
// #include "io_thread.hpp"
pub struct own_t : public object_t
{
// public:
    //  Note that the owner is unspecified in the constructor.
    //  It'll be supplied later on when the object is plugged in.

    //  The object is not living within an I/O thread. It has it's own
    //  thread outside of 0MQ infrastructure.
    own_t (zmq::ZmqContext *parent_, uint32_t tid_);

    //  The object is living within I/O thread.
    own_t (zmq::io_thread_t *io_thread_, const ZmqOptions &options_);

    //  When another owned object wants to send command to this object
    //  it calls this function to let it know it should not shut down
    //  before the command is delivered.
    void inc_seqnum ();

    //  Use following two functions to wait for arbitrary events before
    //  terminating. Just add number of events to wait for using
    //  register_tem_acks functions. When event occurs, call
    //  remove_term_ack. When number of pending acks reaches zero
    //  object will be deallocated.
    void register_term_acks (count_: i32);
    void unregister_term_ack ();

  protected:
    //  Launch the supplied object and become its owner.
    void launch_child (own_t *object_);

    //  Terminate owned object
    void term_child (own_t *object_);

    //  Ask owner object to terminate this object. It may take a while
    //  while actual termination is started. This function should not be
    //  called more than once.
    void terminate ();

    //  Returns true if the object is in process of termination.
    bool is_terminating () const;

    //  Derived object destroys own_t. There's no point in allowing
    //  others to invoke the destructor. At the same time, it has to be
    //  virtual so that generic own_t deallocation mechanism destroys
    //  specific type of the owned object correctly.
    ~own_t () ZMQ_OVERRIDE;

    //  Term handler is protected rather than private so that it can
    //  be intercepted by the derived class. This is useful to add custom
    //  steps to the beginning of the termination process.
    void process_term (linger_: i32) ZMQ_OVERRIDE;

    //  A place to hook in when physical destruction of the object
    //  is to be delayed.
    virtual void process_destroy ();

    //  Socket options associated with this object.
    ZmqOptions options;

  // private:
    //  Set owner of the object
    void set_owner (own_t *owner_);

    //  Handlers for incoming commands.
    void process_own (own_t *object_) ZMQ_OVERRIDE;
    void process_term_req (own_t *object_) ZMQ_OVERRIDE;
    void process_term_ack () ZMQ_OVERRIDE;
    void process_seqnum () ZMQ_OVERRIDE;

    //  Check whether all the pending term acks were delivered.
    //  If so, deallocate this object.
    void check_term_acks ();

    //  True if termination was already initiated. If so, we can destroy
    //  the object if there are no more child objects or pending term acks.
    bool _terminating;

    //  Sequence number of the last command sent to this object.
    AtomicCounter _sent_seqnum;

    //  Sequence number of the last command processed by this object.
    u64 _processed_seqnum;

    //  Socket owning this object. It's responsible for shutting down
    //  this object.
    own_t *_owner;

    //  List of all objects owned by this socket. We are responsible
    //  for deallocating them before we quit.
    typedef std::set<own_t *> owned_t;
    owned_t _owned;

    //  Number of events we have to get before we can destroy the object.
    _term_acks: i32;

    ZMQ_NON_COPYABLE_NOR_MOVABLE (own_t)
};

impl own_t {

}

zmq::own_t::own_t (class ZmqContext *parent_, uint32_t tid_) :
    object_t (parent_, tid_),
    _terminating (false),
    _sent_seqnum (0),
    _processed_seqnum (0),
    _owner (NULL),
    _term_acks (0)
{
}

zmq::own_t::own_t (io_thread_t *io_thread_, const ZmqOptions &options_) :
    object_t (io_thread_),
    options (options_),
    _terminating (false),
    _sent_seqnum (0),
    _processed_seqnum (0),
    _owner (NULL),
    _term_acks (0)
{
}

zmq::own_t::~own_t ()
{
}

void zmq::own_t::set_owner (own_t *owner_)
{
    zmq_assert (!_owner);
    _owner = owner_;
}

void zmq::own_t::inc_seqnum ()
{
    //  This function may be called from a different thread!
    _sent_seqnum.add (1);
}

void zmq::own_t::process_seqnum ()
{
    //  Catch up with counter of processed commands.
    _processed_seqnum++;

    //  We may have caught up and still have pending terms acks.
    check_term_acks ();
}

void zmq::own_t::launch_child (own_t *object_)
{
    //  Specify the owner of the object.
    object_->set_owner (this);

    //  Plug the object into the I/O thread.
    send_plug (object_);

    //  Take ownership of the object.
    send_own (this, object_);
}

void zmq::own_t::term_child (own_t *object_)
{
    process_term_req (object_);
}

void zmq::own_t::process_term_req (own_t *object_)
{
    //  When shutting down we can ignore termination requests from owned
    //  objects. The termination request was already sent to the object.
    if (_terminating)
        return;

    //  If not found, we assume that termination request was already sent to
    //  the object so we can safely ignore the request.
    if (0 == _owned.erase (object_))
        return;

    //  If I/O object is well and alive let's ask it to terminate.
    register_term_acks (1);

    //  Note that this object is the root of the (partial shutdown) thus, its
    //  value of linger is used, rather than the value stored by the children.
    send_term (object_, options.linger.load ());
}

void zmq::own_t::process_own (own_t *object_)
{
    //  If the object is already being shut down, new owned objects are
    //  immediately asked to terminate. Note that linger is set to zero.
    if (_terminating) {
        register_term_acks (1);
        send_term (object_, 0);
        return;
    }

    //  Store the reference to the owned object.
    _owned.insert (object_);
}

void zmq::own_t::terminate ()
{
    //  If termination is already underway, there's no point
    //  in starting it anew.
    if (_terminating)
        return;

    //  As for the root of the ownership tree, there's no one to terminate it,
    //  so it has to terminate itself.
    if (!_owner) {
        process_term (options.linger.load ());
        return;
    }

    //  If I am an owned object, I'll ask my owner to terminate me.
    send_term_req (_owner, this);
}

bool zmq::own_t::is_terminating () const
{
    return _terminating;
}

void zmq::own_t::process_term (linger_: i32)
{
    //  Double termination should never happen.
    zmq_assert (!_terminating);

    //  Send termination request to all owned objects.
    for (owned_t::iterator it = _owned.begin (), end = _owned.end (); it != end;
         ++it)
        send_term (*it, linger_);
    register_term_acks (static_cast<int> (_owned.size ()));
    _owned.clear ();

    //  Start termination process and check whether by chance we cannot
    //  terminate immediately.
    _terminating = true;
    check_term_acks ();
}

void zmq::own_t::register_term_acks (count_: i32)
{
    _term_acks += count_;
}

void zmq::own_t::unregister_term_ack ()
{
    zmq_assert (_term_acks > 0);
    _term_acks--;

    //  This may be a last ack we are waiting for before termination...
    check_term_acks ();
}

void zmq::own_t::process_term_ack ()
{
    unregister_term_ack ();
}

void zmq::own_t::check_term_acks ()
{
    if (_terminating && _processed_seqnum == _sent_seqnum.get ()
        && _term_acks == 0) {
        //  Sanity check. There should be no active children at this point.
        zmq_assert (_owned.empty ());

        //  The root object has nobody to confirm the termination to.
        //  Other nodes will confirm the termination to the owner.
        if (_owner)
            send_term_ack (_owner);

        //  Deallocate the resources.
        process_destroy ();
    }
}

void zmq::own_t::process_destroy ()
{
    delete this;
}
