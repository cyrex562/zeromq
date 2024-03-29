use std::ptr::null_mut;
use libc::EAGAIN;
use crate::config::PROXY_BURST_SIZE;
use crate::context::ZmqContext;
use crate::defines::{ZMQ_DONTWAIT, ZMQ_POLLIN, ZMQ_POLLOUT, ZMQ_RCVMORE, ZMQ_SNDMORE};
use crate::message::ZmqMessage;

use crate::poller_event::ZmqPollerEvent;
use crate::socket::ZmqSocket;
use crate::socket_poller::ZmqSocketPoller;
use crate::socket_stats::ZmqSocketStats;
use crate::utils::{cmp_bytes, copy_bytes};


// Utility functions

pub fn capture(options: &mut ZmqContext, capture_: &mut ZmqSocket, msg: &mut ZmqMessage, more_: i32) -> i32 {
    //  Copy message to capture socket if any
    if capture_ {
        let mut ctrl = ZmqMessage::default();
        let mut rc = ctrl.init2();
        if rc < 0 {
            return -1;
        }
        // rc = ctrl.copy (*msg);
        // if rc < 0 {
        //     return -1;
        // }
        ctrl = msg.clone();
        capture_.send(&mut ctrl,  if more_ { ZMQ_SNDMORE } else { 0 });
        if rc < 0 {
            return -1;
        }
    }
    return 0;
}

pub fn forward(options: &mut ZmqContext, from_: &mut ZmqSocket, from_stats_: &mut ZmqSocketStats, to_: &mut ZmqSocket, to_stats: &mut ZmqSocketStats, capture: &mut ZmqSocket, msg: &mut ZmqMessage) -> anyhow::Result<()> {
    // Forward a burst of messages
    // for (unsigned int i = 0; i < proxy_burst_size; i+= 1)
    for i in 0..PROXY_BURST_SIZE {
        let mut more = 0i32;
        let mut moresz = 0usize;
        let mut complete_msg_size = 0usize;

        // Forward all the parts of one message
        loop {
            *msg = from_.recv(  ZMQ_DONTWAIT as i32)?;

            complete_msg_size += msg.size();

            moresz = 4;
            from_.getsockopt( ZMQ_RCVMORE as i32)?;

            //  Copy message to capture socket if any
            // TODO
            // let rc = capture::new(capture, msg, &more);
            // if rc < 0 {
            //     return -1;
            // }

            to_.send(msg,  if more { ZMQ_SNDMORE } else { 0 })?;


            if more == 0 {
                break;
            }
        }

        // A multipart message counts as 1 packet:
        from_stats_.msg_in += 1;
        from_stats_.bytes_in += complete_msg_size;
        to_stats.msg_out += 1;
        to_stats.bytes_out += &complete_msg_size;
    }

    Ok(())
}

pub fn loop_and_send_multipart_stat(options: &mut ZmqContext,
                                    control_: &mut ZmqSocket,
                                    stat_: u64,
                                    first_: bool,
                                    more_: bool) -> i32 {
    let mut rc: i32 = 0;
    let mut msg = ZmqMessage::default();

    //  VSM of 8 bytes can't fail to init
    msg.init_size(8);
    copy_bytes(msg.data_mut(), 0, stat_.bytes, 0, 8);

    //  if the first message is handed to the pipe successfully then the HWM
    //  is not full, which means failures are due to interrupts (on Windows pipes
    //  are TCP sockets), so keep retrying
    loop {
        rc = control_.send(&mut msg,  if more_ { ZMQ_SNDMORE } else { 0 });
        if !(!first_.clone() && rc != 0) {
            break;
        }
    }

    return rc;
}

pub fn reply_stats(options: &mut ZmqContext,
                   control_: &mut ZmqSocket,
                   frontend_stats_: &ZmqSocketStats,
                   backend_stats_: &ZmqSocketStats) -> i32 {
    // first part: frontend stats - the first send might fail due to HWM
    if loop_and_send_multipart_stat(options, control_, frontend_stats_.msg_in.clone(), true, true) != 0 {
        return -1;
    }

    loop_and_send_multipart_stat(options, control_, frontend_stats_.bytes_in.clone(), false,
                                 true);
    loop_and_send_multipart_stat(options, control_, frontend_stats_.msg_out.clone(), false,
                                 true);
    loop_and_send_multipart_stat(options, control_, frontend_stats_.bytes_out.clone(), false,
                                 true);

    // second part: backend stats
    loop_and_send_multipart_stat(options, control_, backend_stats_.msg_in.clone(), false,
                                 true);
    loop_and_send_multipart_stat(options, control_, backend_stats_.bytes_in.clone(), false,
                                 true);
    loop_and_send_multipart_stat(options, control_, backend_stats_.msg_out.clone(), false,
                                 true);
    loop_and_send_multipart_stat(options, control_, backend_stats_.bytes_out.clone(), false,
                                 false);

    return 0;
}

enum ProxyState {
    Active,
    Paused,
    Terminated,
}
// #ifdef ZMQ_HAVE_POLLER

pub fn proxy(options: &mut ZmqContext,
             frontend_: &mut ZmqSocket,
             backend_: &mut ZmqSocket,
             capture_: &mut ZmqSocket,
             control_: Option<&mut ZmqSocket>) -> i32 {
    let mut msg = ZmqMessage::default();
    msg.init2();
    // if (rc != 0)
    //     return -1;

    //  The algorithm below assumes ratio of requests and replies processed
    //  under full load to be 1:1.

    let mut more = 0i32;
    // size_t moresz = mem::size_of::<more>();
    let mut moresz = 4usize;
    //  Proxy can be in these three states
    let mut state: ProxyState = ProxyState::Active;

    let mut frontend_equal_to_backend = false;
    let mut frontend_in = false;
    let mut frontend_out = false;
    let mut backend_in = false;
    let mut backend_out = false;
    let mut control_in = false;
    // ZmqSocketPoller::event_t events[3];
    let mut events: [ZmqPollerEvent; 3] = [ZmqPollerEvent::default(); 3];
    // ZmqSocketStats frontend_stats;
    let mut frontend_stats = ZmqSocketStats::default();
    // ZmqSocketStats backend_stats;
    let mut backend_stats = ZmqSocketStats::default();
    // memset (&frontend_stats, 0, mem::size_of::<frontend_stats>());
    // memset (&backend_stats, 0, mem::size_of::<backend_stats>());

    //  Don't allocate these pollers from stack because they will take more than 900 kB of stack!
    //  On Windows this blows up default stack of 1 MB and aborts the program.
    //  I wanted to use std::shared_ptr here as the best solution but that requires C+= 111...
    let mut poller_all = ZmqSocketPoller::default(); //  Poll for everything.
    let mut poller_in = ZmqSocketPoller::default(); //  Poll only 'ZMQ_POLLIN' on all sockets. Initial blocking poll in loop.
    let mut poller_control = ZmqSocketPoller::default(); //  Poll only for 'ZMQ_POLLIN' on 'control_', when proxy is paused.
    let mut poller_receive_blocked = ZmqSocketPoller::default(); //  All except 'ZMQ_POLLIN' on 'frontend_'.

    //  If frontend_==backend_ 'poller_send_blocked' and 'poller_receive_blocked' are the same, 'ZMQ_POLLIN' is ignored.
    //  In that case 'poller_send_blocked' is not used. We need only 'poller_receive_blocked'.
    //  We also don't need 'poller_both_blocked', 'poller_backend_only' nor 'poller_frontend_only' no need to initialize it.
    //  We save some RAM and time for initialization.
    // ZmqSocketPoller *poller_send_blocked = null_mut(); //  All except 'ZMQ_POLLIN' on 'backend_'.
    // ZmqSocketPoller *poller_both_blocked = null_mut(); //  All except 'ZMQ_POLLIN' on both 'frontend_' and 'backend_'.
    // ZmqSocketPoller *poller_frontend_only = null_mut(); //  Only 'ZMQ_POLLIN' and 'ZMQ_POLLOUT' on 'frontend_'.
    // ZmqSocketPoller *poller_backend_only = null_mut(); //  Only 'ZMQ_POLLIN' and 'ZMQ_POLLOUT' on 'backend_'.

    if frontend_ != backend_ {
        // poller_send_blocked = ZmqSocketPoller::default(); //  All except 'ZMQ_POLLIN' on 'backend_'.
        // poller_both_blocked = ZmqSocketPoller::default(); //  All except 'ZMQ_POLLIN' on both 'frontend_' and 'backend_'.
        // poller_frontend_only = ZmqSocketPoller::default(); //  Only 'ZMQ_POLLIN' and 'ZMQ_POLLOUT' on 'frontend_'.
        // poller_backend_only = ZmqSocketPoller::default(); //  Only 'ZMQ_POLLIN' and 'ZMQ_POLLOUT' on 'backend_'.
        frontend_equal_to_backend = false;
    } else {
        frontend_equal_to_backend = true;
    }

    // if poller_all == null_mut() || poller_in == null_mut() || poller_control == null_mut() || poller_receive_blocked == null_mut() || ((poller_send_blocked == null_mut() || poller_both_blocked == null_mut()) && !frontend_equal_to_backend) {
    //     // PROXY_CLEANUP ();
    //     // return close_and_return (&msg, -1);
    //     return -1;
    // }

    let poller_wait = poller_in; //  Poller for blocking wait, initially all 'ZMQ_POLLIN'.

    //  Register 'frontend_' and 'backend_' with pollers.
    let rc = poller_all.add(frontend_, null_mut(), ZMQ_POLLIN | ZMQ_POLLOUT); //  Everything.
    // CHECK_RC_EXIT_ON_FAILURE();
    rc = poller_in.add(frontend_, null_mut(), ZMQ_POLLIN); //  All 'ZMQ_POLLIN's.
    // CHECK_RC_EXIT_ON_FAILURE();

    if frontend_equal_to_backend {
        //  If frontend_==backend_ 'poller_send_blocked' and 'poller_receive_blocked' are the same,
        //  so we don't need 'poller_send_blocked'. We need only 'poller_receive_blocked'.
        //  We also don't need 'poller_both_blocked', no need to initialize it.
        rc = poller_receive_blocked.add(frontend_, null_mut(), ZMQ_POLLOUT);
        // CHECK_RC_EXIT_ON_FAILURE();
    } else {
        rc = poller_all.add(backend_, null_mut(),
                            ZMQ_POLLIN | ZMQ_POLLOUT); //  Everything.
        // CHECK_RC_EXIT_ON_FAILURE();
        rc = poller_in.add(backend_, null_mut(), ZMQ_POLLIN); //  All 'ZMQ_POLLIN's.
        // CHECK_RC_EXIT_ON_FAILURE();
        // rc = poller_both_blocked.add(
        //     frontend_, null_mut(), ZMQ_POLLOUT); //  Waiting only for 'ZMQ_POLLOUT'.
        // CHECK_RC_EXIT_ON_FAILURE();
        // rc = poller_both_blocked.add(
        //     backend_, null_mut(), ZMQ_POLLOUT); //  Waiting only for 'ZMQ_POLLOUT'.
        // CHECK_RC_EXIT_ON_FAILURE();
        // rc = poller_send_blocked.add(
        //     backend_, null_mut(),
        //     ZMQ_POLLOUT); //  All except 'ZMQ_POLLIN' on 'backend_'.
        // CHECK_RC_EXIT_ON_FAILURE();
        // rc = poller_send_blocked.add(
        //     frontend_, null_mut(),
        //     ZMQ_POLLIN | ZMQ_POLLOUT); //  All except 'ZMQ_POLLIN' on 'backend_'.
        // CHECK_RC_EXIT_ON_FAILURE();
        rc = poller_receive_blocked.add(
            frontend_, null_mut(),
            ZMQ_POLLOUT); //  All except 'ZMQ_POLLIN' on 'frontend_'.
        // CHECK_RC_EXIT_ON_FAILURE();
        rc = poller_receive_blocked.add(
            backend_, null_mut(),
            ZMQ_POLLIN | ZMQ_POLLOUT); //  All except 'ZMQ_POLLIN' on 'frontend_'.
        // CHECK_RC_EXIT_ON_FAILURE();
        // rc = poller_frontend_only.add(frontend_, null_mut(), ZMQ_POLLIN | ZMQ_POLLOUT);
        // CHECK_RC_EXIT_ON_FAILURE();
        // rc = poller_backend_only.add(backend_, null_mut(), ZMQ_POLLIN | ZMQ_POLLOUT);
        // CHECK_RC_EXIT_ON_FAILURE();
    }

    //  Register 'control_' with pollers.
    if (control_ != null_mut()) {
        rc = poller_all.add(control_, null_mut(), ZMQ_POLLIN);
        // CHECK_RC_EXIT_ON_FAILURE();
        rc = poller_in.add(control_, null_mut(), ZMQ_POLLIN);
        // CHECK_RC_EXIT_ON_FAILURE();
        rc = poller_control.add(
            control_, null_mut(),
            ZMQ_POLLIN); //  When proxy is paused we wait only for ZMQ_POLLIN on 'control_' socket.
        // CHECK_RC_EXIT_ON_FAILURE();
        rc = poller_receive_blocked.add(control_, null_mut(), ZMQ_POLLIN);
        // CHECK_RC_EXIT_ON_FAILURE();
        if (!frontend_equal_to_backend.clone()) {
            // rc = poller_send_blocked.add(control_, null_mut(), ZMQ_POLLIN);
            // CHECK_RC_EXIT_ON_FAILURE();
            // rc = poller_both_blocked.add(control_, null_mut(), ZMQ_POLLIN);
            // CHECK_RC_EXIT_ON_FAILURE();
            // rc = poller_frontend_only.add(control_, null_mut(), ZMQ_POLLIN);
            // CHECK_RC_EXIT_ON_FAILURE();
            // rc = poller_backend_only.add(control_, null_mut(), ZMQ_POLLIN);
            // CHECK_RC_EXIT_ON_FAILURE();
        }
    }

    let mut request_processed = false;
    let mut reply_processed = false;

    while state != ProxyState::Terminated {
        //  Blocking wait initially only for 'ZMQ_POLLIN' - 'poller_wait' points to 'poller_in'.
        //  If one of receiving end's queue is full ('ZMQ_POLLOUT' not available),
        //  'poller_wait' is pointed to 'poller_receive_blocked', 'poller_send_blocked' or 'poller_both_blocked'.
        rc = poller_wait.wait(events, 3, -1);
        if rc < 0 {
            rc = 0;
        }
        // CHECK_RC_EXIT_ON_FAILURE();

        //  Some of events waited for by 'poller_wait' have arrived, now poll for everything without blocking.
        rc = poller_all.wait(events, 3, 0);
        if rc < 0{
            rc = 0;
        }
        // CHECK_RC_EXIT_ON_FAILURE();

        //  Process events.
        // for (int i = 0; i < rc; i+= 1)
        for i in 0..rc {
            if events[i].socket == frontend_ {
                frontend_in = (events[i].events & ZMQ_POLLIN) != 0;
                frontend_out = (events[i].events & ZMQ_POLLOUT) != 0;
            } else {
                //  This 'if' needs to be after check for 'frontend_' in order never
                //  to be reached in case frontend_==backend_, so we ensure backend_in=false in that case.
                if events[i].socket == backend_ {
                    backend_in = (events[i].events & ZMQ_POLLIN) != 0;
                    backend_out = (events[i].events & ZMQ_POLLOUT) != 0;
                } else if events[i].socket == control_ {
                    control_in = (events[i].events & ZMQ_POLLIN) != 0;
                }
            }
        }


        //  Process a control command if any.
        if control_in {
            rc = control_.recv(&mut msg, options, 0);
            // CHECK_RC_EXIT_ON_FAILURE();
            rc = control_.getsockopt(options, ZMQ_RCVMORE as i32);
            if (rc < 0) || more != 0 {
                // PROXY_CLEANUP ();
                // return close_and_return (&msg, -1);
                return -1;
            }

            //  Copy message to capture socket if any.
            rc = capture(options, capture_, &mut msg, more.clone());
            // CHECK_RC_EXIT_ON_FAILURE();

            if msg.size() == 5 && cmp_bytes(msg.data(), 0, b"PAUSE", 0, 5) == 0 {
                state = ProxyState::Paused;
                poller_wait = poller_control;
            } else if msg.size() == 6 && cmp_bytes(msg.data(), 0, b"RESUME", 0, 6) == 0 {
                state = ProxyState::Active;
                poller_wait = poller_in;
            } else {
                if msg.size() == 9 && cmp_bytes(msg.data(), 0, b"TERMINATE", 0, 9) == 0 {
                    state = ProxyState::Terminated;
                } else {
                    if msg.size() == 10 && cmp_bytes(msg.data(), 0, b"STATISTICS", 0, 10) == 0 {
                        rc = reply_stats(options, control_, &frontend_stats,
                                         &backend_stats);
                        // CHECK_RC_EXIT_ON_FAILURE();
                    } else {
                        //  This is an API error, we assert
                        // puts ("E: invalid command sent to proxy");
                        // zmq_assert (false);
                    }
                }
            }
            control_in = false;
        }

        if state == ProxyState::Active {
            //  Process a request, 'ZMQ_POLLIN' on 'frontend_' and 'ZMQ_POLLOUT' on 'backend_'.
            //  In case of frontend_==backend_ there's no 'ZMQ_POLLOUT' event.
            if frontend_in.clone() && (backend_out.clone() || frontend_equal_to_backend.clone()) {
                rc = forward(options, frontend_, &mut frontend_stats, backend_,
                             &mut backend_stats, capture_, &mut msg);
                // CHECK_RC_EXIT_ON_FAILURE();
                request_processed = true;
                frontend_in = false;
                backend_out = false;
            } else {
                request_processed = false;
            }

            //  Process a reply, 'ZMQ_POLLIN' on 'backend_' and 'ZMQ_POLLOUT' on 'frontend_'.
            //  If 'frontend_' and 'backend_' are the same this is not needed because previous processing
            //  covers all of the cases. 'backend_in' is always false if frontend_==backend_ due to
            //  design in 'for' event processing loop.
            if (backend_in.clone() && frontend_out.clone()) {
                rc = forward(options, backend_, &mut backend_stats, frontend_,
                             &mut frontend_stats, capture_, &mut msg);
                // CHECK_RC_EXIT_ON_FAILURE();
                reply_processed = true;
                backend_in = false;
                frontend_out = false;
            } else {
                reply_processed = false;
            }

            if request_processed || reply_processed {
                //  If request/reply is processed that means we had at least one 'ZMQ_POLLOUT' event.
                //  Enable corresponding 'ZMQ_POLLIN' for blocking wait if any was disabled.
                if poller_wait != poller_in {
                    if request_processed { //  'frontend_' -> 'backend_'
                        // if poller_wait == poller_both_blocked {
                        //     poller_wait = poller_send_blocked;
                        // } else if poller_wait == poller_receive_blocked || poller_wait == poller_frontend_only {
                        //     poller_wait = poller_in;
                        // }
                    }
                    if reply_processed { //  'backend_' -> 'frontend_'
                        // if poller_wait == poller_both_blocked {
                        //     poller_wait = poller_receive_blocked;
                        // } else if poller_wait == poller_send_blocked || poller_wait == poller_backend_only {
                        //     poller_wait = poller_in;
                        // }
                    }
                }
            } else {
                //  No requests have been processed, there were no 'ZMQ_POLLIN' with corresponding 'ZMQ_POLLOUT' events.
                //  That means that out queue(s) is/are full or one out queue is full and second one has no messages to process.
                //  Disable receiving 'ZMQ_POLLIN' for sockets for which there's no 'ZMQ_POLLOUT',
                //  or wait only on both 'backend_''s or 'frontend_''s 'ZMQ_POLLIN' and 'ZMQ_POLLOUT'.
                if frontend_in {
                    if frontend_out {
                        // If frontend_in and frontend_out are true, obviously backend_in and backend_out are both false.
                        // In that case we need to wait for both 'ZMQ_POLLIN' and 'ZMQ_POLLOUT' only on 'backend_'.
                        // We'll never get here in case of frontend_==backend_ because then frontend_out will always be false.
                        // poller_wait = poller_backend_only;
                    } else {
                        // if poller_wait == poller_send_blocked {
                        //     poller_wait = poller_both_blocked;
                        // } else if poller_wait == poller_in {
                        //     poller_wait = poller_receive_blocked;
                        // }
                    }
                }
                if (backend_in) {
                    //  Will never be reached if frontend_==backend_, 'backend_in' will
                    //  always be false due to design in 'for' event processing loop.
                    if (backend_out) {
                        // If backend_in and backend_out are true, obviously frontend_in and frontend_out are both false.
                        // In that case we need to wait for both 'ZMQ_POLLIN' and 'ZMQ_POLLOUT' only on 'frontend_'.
                        // poller_wait = poller_frontend_only;
                    } else {
                        // if (poller_wait == poller_receive_blocked) {
                        //     poller_wait = poller_both_blocked;
                        // } else if (poller_wait == poller_in) {
                        //     poller_wait = poller_send_blocked;
                        // }
                    }
                }
            }
        }
    }
    // PROXY_CLEANUP();
    // return close_and_return (&msg, 0);
    return 0;
}

// #else //  ZMQ_HAVE_POLLER
//
// int proxy (class ZmqSocketBase *frontend_,
// pub struct ZmqSocketBase *backend_,
// pub struct ZmqSocketBase *capture_,
// pub struct ZmqSocketBase *control_)
// {
// let mut msg = ZmqMessage::default();
//     int rc = msg.init ();
//     if (rc != 0)
//         return -1;
//
//     //  The algorithm below assumes ratio of requests and replies processed
//     //  under full load to be 1:1.
//
//     more: i32;
//     moresz: usize;
//     ZmqPollItem items[] = {{frontend_, 0, ZMQ_POLLIN, 0},
//                               {backend_, 0, ZMQ_POLLIN, 0},
//                               {control_, 0, ZMQ_POLLIN, 0}};
//     int qt_poll_items = (control_ ? 3 : 2);
//     ZmqPollItem itemsout[] = {{frontend_, 0, ZMQ_POLLOUT, 0},
//                                  {backend_, 0, ZMQ_POLLOUT, 0}};
//
//     ZmqSocketStats frontend_stats;
//     memset (&frontend_stats, 0, mem::size_of::<frontend_stats>());
//     ZmqSocketStats backend_stats;
//     memset (&backend_stats, 0, mem::size_of::<backend_stats>());
//
//     //  Proxy can be in these three states
//     enum
//     {
//         active,
//         paused,
//         terminated
//     } state = active;
//
//     while (state != terminated) {
//         //  Wait while there are either requests or replies to process.
//         rc = zmq_poll (&items[0], qt_poll_items, -1);
//         if ( (rc < 0))
//             return close_and_return (&msg, -1);
//
//         //  Get the pollout separately because when combining this with pollin it maxes the CPU
//         //  because pollout shall most of the time return directly.
//         //  POLLOUT is only checked when frontend and backend sockets are not the same.
//         if (frontend_ != backend_) {
//             rc = zmq_poll (&itemsout[0], 2, 0);
//             if ( (rc < 0)) {
//                 return close_and_return (&msg, -1);
//             }
//         }
//
//         //  Process a control command if any
//         if (control_ && items[2].revents & ZMQ_POLLIN) {
//             rc = control_.recv (&msg, 0);
//             if ( (rc < 0))
//                 return close_and_return (&msg, -1);
//
//             moresz = sizeof more;
//             rc = control_.getsockopt (ZMQ_RCVMORE, &more, &moresz);
//             if ( (rc < 0) || more)
//                 return close_and_return (&msg, -1);
//
//             //  Copy message to capture socket if any
//             rc = capture (capture_, &msg);
//             if ( (rc < 0))
//                 return close_and_return (&msg, -1);
//
//             if (msg.size () == 5 && memcmp (msg.data (), "PAUSE", 5) == 0)
//                 state = paused;
//             else if (msg.size () == 6 && memcmp (msg.data (), "RESUME", 6) == 0)
//                 state = active;
//             else if (msg.size () == 9
//                      && memcmp (msg.data (), "TERMINATE", 9) == 0)
//                 state = terminated;
//             else {
//                 if (msg.size () == 10
//                     && memcmp (msg.data (), "STATISTICS", 10) == 0) {
//                     rc =
//                       reply_stats (control_, &frontend_stats, &backend_stats);
//                     if ( (rc < 0))
//                         return close_and_return (&msg, -1);
//                 } else {
//                     //  This is an API error, we assert
//                     puts ("E: invalid command sent to proxy");
//                     // zmq_assert (false);
//                 }
//             }
//         }
//         //  Process a request
//         if (state == active && items[0].revents & ZMQ_POLLIN
//             && (frontend_ == backend_ || itemsout[1].revents & ZMQ_POLLOUT)) {
//             rc = forward (frontend_, &frontend_stats, backend_, &backend_stats,
//                           capture_, &msg);
//             if ( (rc < 0))
//                 return close_and_return (&msg, -1);
//         }
//         //  Process a reply
//         if (state == active && frontend_ != backend_
//             && items[1].revents & ZMQ_POLLIN
//             && itemsout[0].revents & ZMQ_POLLOUT) {
//             rc = forward (backend_, &backend_stats, frontend_, &frontend_stats,
//                           capture_, &msg);
//             if ( (rc < 0))
//                 return close_and_return (&msg, -1);
//         }
//     }
//
//     return close_and_return (&msg, 0);
// }

// #endif //  ZMQ_HAVE_POLLER
