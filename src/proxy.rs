use std::mem::size_of_val;
use crate::socket_base::socket_base_t;

pub enum proxy_state = {
active,
    paused,
    terminated
}

pub fn proxy(frontend_: *mut socket_base_t,
             backend_: *mut socket_base_t,
             capture_: *mut socket_base_t) -> i32 {
    
     // msg_t msg;
    let mut msg = msg_t::new();
    let rc = msg.init ();
    if (rc != 0) {
        return -1;
    }

    //  The algorithm below assumes ratio of requests and replies processed
    //  under full load to be 1:1.

    //  Proxy can be in these three states
    // enum
    // {
    //     active,
    //     paused,
    //     terminated
    // } state = active;
    let mut state = proxy_state::active;

    let mut frontend_equal_to_backend = false;
    let mut frontend_in = false;
    let mut frontend_out = false;
    let mut backend_in = false;
    let mut  backend_out = false;
    // zmq::socket_poller_t::event_t events[3];
    let mut events: [event_t;3] = [event_t::new(); 3];
    
    //  Don't allocate these pollers from stack because they will take more than 900 kB of stack!
    //  On Windows this blows up default stack of 1 MB and aborts the program.
    //  I wanted to use std::shared_ptr here as the best solution but that requires C++11...
    zmq::socket_poller_t *poller_all =
      new (std::nothrow) zmq::socket_poller_t; //  Poll for everything.
    zmq::socket_poller_t *poller_in = new (std::nothrow) zmq::
      socket_poller_t; //  Poll only 'ZMQ_POLLIN' on all sockets. Initial blocking poll in loop.
    zmq::socket_poller_t *poller_receive_blocked = new (std::nothrow)
      zmq::socket_poller_t; //  All except 'ZMQ_POLLIN' on 'frontend_'.

    //  If frontend_==backend_ 'poller_send_blocked' and 'poller_receive_blocked' are the same, 'ZMQ_POLLIN' is ignored.
    //  In that case 'poller_send_blocked' is not used. We need only 'poller_receive_blocked'.
    //  We also don't need 'poller_both_blocked', 'poller_backend_only' nor 'poller_frontend_only' no need to initialize it.
    //  We save some RAM and time for initialization.
    zmq::socket_poller_t *poller_send_blocked =
      NULL; //  All except 'ZMQ_POLLIN' on 'backend_'.
    zmq::socket_poller_t *poller_both_blocked =
      NULL; //  All except 'ZMQ_POLLIN' on both 'frontend_' and 'backend_'.
    zmq::socket_poller_t *poller_frontend_only =
      NULL; //  Only 'ZMQ_POLLIN' and 'ZMQ_POLLOUT' on 'frontend_'.
    zmq::socket_poller_t *poller_backend_only =
      NULL; //  Only 'ZMQ_POLLIN' and 'ZMQ_POLLOUT' on 'backend_'.

    if (frontend_ != backend_) {
        poller_send_blocked = new (std::nothrow)
          zmq::socket_poller_t; //  All except 'ZMQ_POLLIN' on 'backend_'.
        poller_both_blocked = new (std::nothrow) zmq::
          socket_poller_t; //  All except 'ZMQ_POLLIN' on both 'frontend_' and 'backend_'.
        poller_frontend_only = new (std::nothrow) zmq::
          socket_poller_t; //  Only 'ZMQ_POLLIN' and 'ZMQ_POLLOUT' on 'frontend_'.
        poller_backend_only = new (std::nothrow) zmq::
          socket_poller_t; //  Only 'ZMQ_POLLIN' and 'ZMQ_POLLOUT' on 'backend_'.
        frontend_equal_to_backend = false;
    } else
        frontend_equal_to_backend = true;

    if (poller_all == NULL || poller_in == NULL
        || poller_receive_blocked == NULL
        || ((poller_send_blocked == NULL || poller_both_blocked == NULL)
            && !frontend_equal_to_backend)) {
        PROXY_CLEANUP ();
        return close_and_return (&msg, -1);
    }

    zmq::socket_poller_t *poller_wait =
      poller_in; //  Poller for blocking wait, initially all 'ZMQ_POLLIN'.

    //  Register 'frontend_' and 'backend_' with pollers.
    rc = poller_all->add (frontend_, NULL,
                          ZMQ_POLLIN | ZMQ_POLLOUT); //  Everything.
    CHECK_RC_EXIT_ON_FAILURE ();
    rc = poller_in->add (frontend_, NULL, ZMQ_POLLIN); //  All 'ZMQ_POLLIN's.
    CHECK_RC_EXIT_ON_FAILURE ();

    if (frontend_equal_to_backend) {
        //  If frontend_==backend_ 'poller_send_blocked' and 'poller_receive_blocked' are the same,
        //  so we don't need 'poller_send_blocked'. We need only 'poller_receive_blocked'.
        //  We also don't need 'poller_both_blocked', no need to initialize it.
        rc = poller_receive_blocked->add (frontend_, NULL, ZMQ_POLLOUT);
        CHECK_RC_EXIT_ON_FAILURE ();
    } else {
        rc = poller_all->add (backend_, NULL,
                              ZMQ_POLLIN | ZMQ_POLLOUT); //  Everything.
        CHECK_RC_EXIT_ON_FAILURE ();
        rc = poller_in->add (backend_, NULL, ZMQ_POLLIN); //  All 'ZMQ_POLLIN's.
        CHECK_RC_EXIT_ON_FAILURE ();
        rc = poller_both_blocked->add (
          frontend_, NULL, ZMQ_POLLOUT); //  Waiting only for 'ZMQ_POLLOUT'.
        CHECK_RC_EXIT_ON_FAILURE ();
        rc = poller_both_blocked->add (
          backend_, NULL, ZMQ_POLLOUT); //  Waiting only for 'ZMQ_POLLOUT'.
        CHECK_RC_EXIT_ON_FAILURE ();
        rc = poller_send_blocked->add (
          backend_, NULL,
          ZMQ_POLLOUT); //  All except 'ZMQ_POLLIN' on 'backend_'.
        CHECK_RC_EXIT_ON_FAILURE ();
        rc = poller_send_blocked->add (
          frontend_, NULL,
          ZMQ_POLLIN | ZMQ_POLLOUT); //  All except 'ZMQ_POLLIN' on 'backend_'.
        CHECK_RC_EXIT_ON_FAILURE ();
        rc = poller_receive_blocked->add (
          frontend_, NULL,
          ZMQ_POLLOUT); //  All except 'ZMQ_POLLIN' on 'frontend_'.
        CHECK_RC_EXIT_ON_FAILURE ();
        rc = poller_receive_blocked->add (
          backend_, NULL,
          ZMQ_POLLIN | ZMQ_POLLOUT); //  All except 'ZMQ_POLLIN' on 'frontend_'.
        CHECK_RC_EXIT_ON_FAILURE ();
        rc =
          poller_frontend_only->add (frontend_, NULL, ZMQ_POLLIN | ZMQ_POLLOUT);
        CHECK_RC_EXIT_ON_FAILURE ();
        rc =
          poller_backend_only->add (backend_, NULL, ZMQ_POLLIN | ZMQ_POLLOUT);
        CHECK_RC_EXIT_ON_FAILURE ();
    }

    bool request_processed, reply_processed;

    while (state != terminated) {
        //  Blocking wait initially only for 'ZMQ_POLLIN' - 'poller_wait' points to 'poller_in'.
        //  If one of receiving end's queue is full ('ZMQ_POLLOUT' not available),
        //  'poller_wait' is pointed to 'poller_receive_blocked', 'poller_send_blocked' or 'poller_both_blocked'.
        rc = poller_wait->wait (events, 3, -1);
        if (rc < 0 && errno == EAGAIN)
            rc = 0;
        CHECK_RC_EXIT_ON_FAILURE ();

        //  Some of events waited for by 'poller_wait' have arrived, now poll for everything without blocking.
        rc = poller_all->wait (events, 3, 0);
        if (rc < 0 && errno == EAGAIN)
            rc = 0;
        CHECK_RC_EXIT_ON_FAILURE ();

        //  Process events.
        for (int i = 0; i < rc; i++) {
            if (events[i].socket == frontend_) {
                frontend_in = (events[i].events & ZMQ_POLLIN) != 0;
                frontend_out = (events[i].events & ZMQ_POLLOUT) != 0;
            } else
                //  This 'if' needs to be after check for 'frontend_' in order never
                //  to be reached in case frontend_==backend_, so we ensure backend_in=false in that case.
                if (events[i].socket == backend_) {
                    backend_in = (events[i].events & ZMQ_POLLIN) != 0;
                    backend_out = (events[i].events & ZMQ_POLLOUT) != 0;
                }
        }

        if (state == active) {
            //  Process a request, 'ZMQ_POLLIN' on 'frontend_' and 'ZMQ_POLLOUT' on 'backend_'.
            //  In case of frontend_==backend_ there's no 'ZMQ_POLLOUT' event.
            if (frontend_in && (backend_out || frontend_equal_to_backend)) {
                rc = forward (frontend_, backend_, capture_, &msg);
                CHECK_RC_EXIT_ON_FAILURE ();
                request_processed = true;
                frontend_in = backend_out = false;
            } else
                request_processed = false;

            //  Process a reply, 'ZMQ_POLLIN' on 'backend_' and 'ZMQ_POLLOUT' on 'frontend_'.
            //  If 'frontend_' and 'backend_' are the same this is not needed because previous processing
            //  covers all of the cases. 'backend_in' is always false if frontend_==backend_ due to
            //  design in 'for' event processing loop.
            if (backend_in && frontend_out) {
                rc = forward (backend_, frontend_, capture_, &msg);
                CHECK_RC_EXIT_ON_FAILURE ();
                reply_processed = true;
                backend_in = frontend_out = false;
            } else
                reply_processed = false;

            if (request_processed || reply_processed) {
                //  If request/reply is processed that means we had at least one 'ZMQ_POLLOUT' event.
                //  Enable corresponding 'ZMQ_POLLIN' for blocking wait if any was disabled.
                if (poller_wait != poller_in) {
                    if (request_processed) { //  'frontend_' -> 'backend_'
                        if (poller_wait == poller_both_blocked)
                            poller_wait = poller_send_blocked;
                        else if (poller_wait == poller_receive_blocked
                                 || poller_wait == poller_frontend_only)
                            poller_wait = poller_in;
                    }
                    if (reply_processed) { //  'backend_' -> 'frontend_'
                        if (poller_wait == poller_both_blocked)
                            poller_wait = poller_receive_blocked;
                        else if (poller_wait == poller_send_blocked
                                 || poller_wait == poller_backend_only)
                            poller_wait = poller_in;
                    }
                }
            } else {
                //  No requests have been processed, there were no 'ZMQ_POLLIN' with corresponding 'ZMQ_POLLOUT' events.
                //  That means that out queue(s) is/are full or one out queue is full and second one has no messages to process.
                //  Disable receiving 'ZMQ_POLLIN' for sockets for which there's no 'ZMQ_POLLOUT',
                //  or wait only on both 'backend_''s or 'frontend_''s 'ZMQ_POLLIN' and 'ZMQ_POLLOUT'.
                if (frontend_in) {
                    if (frontend_out)
                        // If frontend_in and frontend_out are true, obviously backend_in and backend_out are both false.
                        // In that case we need to wait for both 'ZMQ_POLLIN' and 'ZMQ_POLLOUT' only on 'backend_'.
                        // We'll never get here in case of frontend_==backend_ because then frontend_out will always be false.
                        poller_wait = poller_backend_only;
                    else {
                        if (poller_wait == poller_send_blocked)
                            poller_wait = poller_both_blocked;
                        else if (poller_wait == poller_in)
                            poller_wait = poller_receive_blocked;
                    }
                }
                if (backend_in) {
                    //  Will never be reached if frontend_==backend_, 'backend_in' will
                    //  always be false due to design in 'for' event processing loop.
                    if (backend_out)
                        // If backend_in and backend_out are true, obviously frontend_in and frontend_out are both false.
                        // In that case we need to wait for both 'ZMQ_POLLIN' and 'ZMQ_POLLOUT' only on 'frontend_'.
                        poller_wait = poller_frontend_only;
                    else {
                        if (poller_wait == poller_receive_blocked)
                            poller_wait = poller_both_blocked;
                        else if (poller_wait == poller_in)
                            poller_wait = poller_send_blocked;
                    }
                }
            }
        }
    }
    PROXY_CLEANUP ();
    return close_and_return (&msg, 0);
}

pub unsafe fn capture (capture_: *mut socket_base_t, msg_: *mut msg_t, more_: i32) -> i32 {
    //  Copy message to capture socket if any
    if (capture_) {
        // zmq::msg_t ctrl;
        let mut ctrl = msg_t::new ();
        let rc = ctrl.init ();
        if ( (rc < 0)) {
            return -1;
        }
        rc = ctrl.copy (*msg_);
        if ( (rc < 0)) {
            return -1;
        }
        rc = capture_.send (&ctrl, more_ ? ZMQ_SNDMORE : 0);
        if ( (rc < 0)) {
            return -1;
        }
    }
    return 0;
}

pub unsafe fn forward(from_: *mut socket_base_t, to_: *mut socket_base_t, capture_: *mut socket_base_t) -> i32
{
    // Forward a burst of messages
    // for (unsigned int i = 0; i < zmq::proxy_burst_size; i++)
    {
        let mut more = 0i32;
        let mut moresz = 0usize;

        // Forward all the parts of one message
        loop
        { 
            let rc = from_.recv (msg_, ZMQ_DONTWAIT);
            if (rc < 0) {
                if ( ( i > 0)) {
                    return 0; // End of burst}

                return -1;
            }

            moresz = size_of_val(more) ;
            rc = from_.getsockopt (ZMQ_RCVMORE, &more, &moresz);
            if ( (rc < 0)) {
                return -1;
            }

            //  Copy message to capture socket if any
            rc = capture (capture_, msg_, more);
            if ( (rc < 0)) {
                return -1;
            }

            rc = to_->send (msg_, more ? ZMQ_SNDMORE : 0);
            if ( (rc < 0)) {
                return -1;
            }

            if (more == 0) {
                break;
            }
        }
    }

    return 0;
}
