use crate::ctx::ZmqContext;
use crate::options::ZmqOptions;
use std::collections::HashSet;
use std::sync::atomic::AtomicU32;
use crate::io::io_thread::ZmqIoThread;
use crate::object::{obj_send_term, obj_send_term_ack};

#[derive(Default, Debug, Clone)]
pub struct ZmqOwn<'a> {
    // pub object: ZmqObject<'a>,
    // pub options: ZmqOptions,
    pub terminating: bool,
    pub sent_seqnum: AtomicU32,
    pub processed_seqnum: u64,
    pub owner: Option<&'a mut ZmqOwn<'a>>,
    pub owned: HashSet<&'a mut ZmqOwn<'a>>,
    pub term_acks: i32,
}

impl<'a> ZmqOwn<'a> {
    pub fn new(parent_: &mut ZmqContext, tid_: u32) -> ZmqOwn<'a> {
        ZmqOwn {
            // object: ZmqObject::new(parent_, tid_),
            // options: ZmqOptions::new(),
            terminating: false,
            sent_seqnum: AtomicU32::new(0),
            processed_seqnum: 0,
            owner: None,
            owned: HashSet::new(),
            term_acks: 0,
        }
    }

    pub fn from_io_thread(io_thread_: &mut ZmqIoThread) -> Self {
        Self {
            // object: ZmqObject::new2(&mut (*io_thread_).object),
            // options: ZmqOptions::new(),
            terminating: false,
            sent_seqnum: AtomicU32::new(0),
            processed_seqnum: 0,
            owner: None,
            owned: HashSet::new(),
            term_acks: 0,
        }
    }
}

pub fn own_set_owner(own: &mut ZmqOwn, owner_: &mut ZmqOwn) {
    own.owner = Some(owner_);
}

pub fn own_inc_seqnum(own: &mut ZmqOwn) {
    own.sent_seqnum.inc();
}

pub fn own_process_seqnum(own: &mut ZmqOwn) {
    //  Catch up with counter of processed commands.
    own.processed_seqnum += 1;

    //  We may have caught up and still have pending terms acks.
    own.check_term_acks();
}

pub fn own_launch_child(own: &mut ZmqOwn, object_: &mut ZmqOwn) {
    //  Specify the owner of the object.
    (object_).set_owner(own);

    //  Plug the object into the I/O thread.
    own.send_plug(object_);

    //  Take ownership of the object.
    own.send_own(own, object_);
}

pub fn own_term_child(own: &mut ZmqOwn, options: &ZmqOptions, object_: &mut ZmqOwn) {
    own.process_term_req(options, object_);
}

pub fn own_process_term_req(own: &mut ZmqOwn, options: &ZmqOptions, object_: &mut ZmqOwn) {
    //  When shutting down we can ignore termination requests from owned
    //  objects. The termination request was already sent to the object.
    if own.terminating {
        return;
    }

    //  If not found, we assume that termination request was already sent to
    //  the object so we can safely ignore the request.
    if 0 == own.owned.erase(object_) {
        return;
    }

    //  If I/O object is well and alive let's ask it to terminate.
    own.register_term_acks(1);

    //  Note that this object is the root of the (partial shutdown) thus, its
    //  value of linger is used, rather than the value stored by the children.
    own.send_term(object_, options.linger.load());
}

pub fn own_process_own(own: &mut ZmqOwn, object_: &mut ZmqOwn) {
    //  If the object is already being shut down, new owned objects are
    //  immediately asked to terminate. Note that linger is set to zero.
    if own.terminating {
        own.register_term_acks(1);
        own.send_term(object_, 0);
        return;
    }

    //  Store the reference to the owned object.
    own.owned.insert(object_);
}

pub fn own_terminate(own: &mut ZmqOwn, options: &ZmqOptions) {
    //  If termination is already underway, there's no point
    //  in starting it anew.
    if own.terminating {
        return;
    }

    //  As for the root of the ownership tree, there's no one to terminate it,
    //  so it has to terminate itself.
    if own.owner.is_none() {
        own.process_term(options.linger.load());
        return;
    }

    //  If I am an owned object, I'll ask my owner to terminate me.
    own.send_term_req(options, &own.owner, own);
}

pub fn own_is_terminating(own: &mut ZmqOwn) -> bool {
    return own.terminating;
}

pub fn own_process_term(
    ctx: &mut ZmqContext,
    owned: &mut HashSet<&mut ZmqOwn>,
    terminating: &mut bool,
    term_acks: &mut i32,
    linger_: i32,
) {
    //  Double termination should never happen.
    // zmq_assert (!_terminating);

    //  Send termination request to all owned objects.
    // for (owned_t::iterator it = _owned.begin (), end = _owned.end (); it != end;
    //      ++it)
    for it in owned.iter() {
        obj_send_term(ctx, *it, linger_);
    }
    own_register_term_acks(term_acks, owned.len());
    owned.clear();

    //  Start termination process and check whether by chance we cannot
    //  terminate immediately.
    *terminating = true;
    // TODO: fix up correct args
    own_check_term_acks(
        ctx,
        terminating,
        &mut 0,
        &mut ZmqAtomicCounter::new(0),
        term_acks,
        &mut None,
    );
}

pub fn own_register_term_acks(term_acks: &mut i32, count_: usize) {
    *term_acks += count_;
}

pub fn own_unregister_term_ack(
    ctx: &mut ZmqContext,
    terminating: &mut bool,
    processed_seqnum: &mut u64,
    sent_seqnum: &mut ZmqAtomicCounter,
    term_acks: &mut i32,
    owner: &mut Option<&mut ZmqOwn>,
) {
    // zmq_assert (_term_acks > 0);
    *term_acks -= 1;

    //  This may be a last ack we are waiting for before termination...
    own_check_term_acks(ctx, terminating, processed_seqnum, sent_seqnum, term_acks, owner);
}

pub fn own_process_term_ack(
    ctx: &mut ZmqContext,
    terminating: &mut bool,
    processed_seqnum: &mut u64,
    sent_seqnum: &mut ZmqAtomicCounter,
    term_acks: &mut i32,
    owner: &mut Option<&mut ZmqOwn>,
) {
    own_unregister_term_ack(ctx, terminating, processed_seqnum, sent_seqnum, term_acks, owner);
}

pub fn own_check_term_acks(
    ctx: &mut ZmqContext,
    terminating: &mut bool,
    processed_seqnum: &mut u64,
    sent_seqnum: &mut ZmqAtomicCounter,
    term_acks: &mut i32,
    owner: &mut Option<&mut ZmqOwn>,
) {
    if *terminating && processed_seqnum == sent_seqnum.get() as u64 && term_acks == 0 {
        //  Sanity check. There should be no Active children at this point.
        // zmq_assert (_owned.empty ());

        //  The root object has nobody to confirm the termination to.
        //  Other nodes will confirm the termination to the owner.
        if owner.is_some() {
            obj_send_term_ack(ctx, owner.unwrap());
        }

        //  Deallocate the resources.
        own_process_destroy();
    }
}

pub fn own_process_destroy() {
    // delete this;
}
