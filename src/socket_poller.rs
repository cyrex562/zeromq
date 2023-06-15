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

// #include "precompiled.hpp"
// #include "socket_poller.hpp"
// #include "err.hpp"
// #include "polling_util.hpp"
// #include "macros.hpp"

// #include <limits.h>

use std::mem;
use std::ptr::null_mut;
use anyhow::anyhow;
use libc::{clock_t, EAGAIN, EFAULT, EINTR, EINVAL, EMFILE, ENOMEM, ENOTSOCK, ENOTSUP, free, INT_MAX, malloc, memcpy, size_t, timeval};
use windows::Win32::Networking::WinSock::{FD_SET, POLLIN, POLLOUT, POLLPRI, select, SOCKET_ERROR, WSAGetLastError};
use windows::Win32::System::Threading::Sleep;
use crate::defines::{ZMQ_EVENTS, ZMQ_FD, ZMQ_POLLERR, ZMQ_POLLIN, ZMQ_POLLOUT, ZMQ_POLLPRI};
use crate::err::wsa_error_to_errno;
use crate::defines::ZmqFileDesc;
use crate::poller_event;
use crate::poller_event::ZmqPollerEvent;
use crate::polling_util::{OptimizedFdSet, ResizableOptimizedFdSet};
use crate::signaler::ZmqSignaler;
use crate::socket::ZmqSocket;

#[derive(Default, Debug, Clone)]
struct ZmqItem {
    // ZmqSocketBase *socket;
    pub socket: Option<ZmqSocket>,
    // ZmqFileDesc fd;
    pub fd: ZmqFileDesc,
    // user_data: *mut c_void;
    pub user_data: Option<Vec<u8>>,
    // short events;
    pub events: i16,
    // #if defined ZMQ_POLL_BASED_ON_POLL
    pollfd_index: i32,
// #endif
}

#[derive(Default, Debug, Clone)]
pub struct ZmqSocketPoller {
//
//     ZmqSocketPoller ();
//     ~ZmqSocketPoller ();

    // typedef ZmqPollerEvent event_t;

    // int add (ZmqSocketBase *socket, user_data_: &mut [u8], short events_);
    // int modify (const ZmqSocketBase *socket, short events_);
    // int remove (ZmqSocketBase *socket);
    // int add_fd (fd: ZmqFileDesc, user_data_: &mut [u8], short events_);
    // int modify_fd (fd: ZmqFileDesc, short events_);
    // int remove_fd (ZmqFileDesc fd);
    // Returns the signaler's fd if there is one, otherwise errors.
    // int signaler_fd (ZmqFileDesc *fd) const;
    // int wait (event_t *events_, n_events_: i32, long timeout);
    // int size () const { return  (_items.size ()); };
    //  Return false if object is not a socket.
    // bool check_tag () const;
    // static void zero_trail_events (ZmqSocketPoller::event_t *events_,
    //                                n_events_: i32,
    //                                found_: i32);
// #if defined ZMQ_POLL_BASED_ON_POLL
//     int check_events (ZmqSocketPoller::event_t *events_, n_events_: i32);
// #elif defined ZMQ_POLL_BASED_ON_SELECT
//     int check_events (ZmqSocketPoller::event_t *events_,
//                       n_events_: i32,
//                       fd_set &inset_,
//                       fd_set &outset_,
//                       fd_set &errset_);
// #endif
//     static int adjust_timeout (clock_t &clock_,
//                                long timeout,
//                                u64 &now_,
//                                u64 &end_,
//                                bool &first_pass_);
//     int rebuild ();
    //  Used to check whether the object is a socket_poller.
    pub _tag: u32,

    //  Signaler used for thread safe sockets polling
    // ZmqSignaler *signaler;
    pub signaler: ZmqSignaler,

    //  List of sockets
    // typedef std::vector<ZmqItem> items_t;
    // items_t _items;
    pub _items: Vec<ZmqItem>,

    //  Does the pollset needs rebuilding?
    pub _need_rebuild: bool,

    //  Should the signaler be used for the thread safe polling?
    pub _use_signaler: bool,

    //  Size of the pollset
    pub _pollset_size: i32,

    // #if defined ZMQ_POLL_BASED_ON_POLL
//     pollfd *_pollfds;
    pub _pollfds: Vec<pollfd>,
    // #elif defined ZMQ_POLL_BASED_ON_SELECT
// #elif defined ZMQ_POLL_BASED_ON_SELECT
//     ResizableOptimizedFdSet _pollset_in;
    pub _pollset_in: ResizableOptimizedFdSet,
    // ResizableOptimizedFdSet _pollset_out;
    pub _pollset_out: ResizableOptimizedFdSet,
    // ResizableOptimizedFdSet _pollset_err;
    pub _pollset_err: ResizableOptimizedFdSet,
    // ZmqFileDesc _max_fd;
    pub _max_fd: ZmqFileDesc,
// #endif

    // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqSocketPoller)
}

impl ZmqSocketPoller {
    pub fn is_socket(&mut self, item: &ZmqItem, socket: &mut ZmqSocket) -> bool {
        return item.socket == Some(socket.clone());
    }

    pub fn is_fd(&mut self, item: &ZmqItem, fd: ZmqFileDesc) -> bool {
        return item.socket.is_some() && item.fd == fd;
    }

    pub fn is_thread_safe(&mut self, socket: &ZmqSocket) -> bool {
        // do not use getsockopt here, since that would fail during context termination
        return socket.is_thread_safe();
    }

    pub fn new() -> Self {

// #endif {
// _tag (0xCAFEBABE),
//     signaler (null_mut())
// // #if defined ZMQ_POLL_BASED_ON_POLL
//     ,
//     _pollfds (null_mut())
// #elif defined ZMQ_POLL_BASED_ON_SELECT
//     ,
//     _max_fd (0)
        let mut out = Self {
            _tag: 0xCAFEBABE,
            signaler: Default::default(),
            _items: vec![],
            _need_rebuild: false,
            _use_signaler: false,
            _pollset_size: 0,
            _pollfds: vec![],
            _pollset_in: Default::default(),
            _pollset_out: Default::default(),
            _pollset_err: Default::default(),
            _max_fd: 0,
        };
        rebuild();
        out
    }

    pub fn check_tag(&mut self) -> bool {
        return _tag == 0xCAFEBABE;
    }

    pub fn signaler_fd(&mut self) -> anyhow::Result<ZmqFileDesc> {
        if (self.signaler) {
            Ok(self.signaler.get_fd())
        }
        // Only thread-safe socket types are guaranteed to have a signaler.
        Err(anyhow!("EINVAL"))
    }


    pub fn add(&mut self, socket: &mut ZmqSocket,
               user_data_: Option<&mut [u8]>,
               events_: i16) -> anyhow::Result<()> {
        if find_if2(_items.begin(), _items.end(), socket, &is_socket) != _items.end()
        {
            return Err(anyhow!("EINVAL"));
        }

        if (is_thread_safe(socket)) {
            if (signaler == null_mut()) {
                signaler = ZmqSignaler();
                if (!signaler) {
                    return Err(anyhow!("ENOMEM"));
                }
                if (!signaler.valid()) {
                    // delete signaler;
                    // signaler = null_mut();
                    return Err(anyhow!("EMFILE"));
                }
            }

            socket.add_signaler(signaler);
        }

        let item = ZmqItem {
            socket: Some(socket.clone()),
            fd: 0,
            user_data: user_data_.to_vec(),
            events: events_,
            pollfd_index: 0,
        };

        self._items.push_back(item);


        self._need_rebuild = true;

        Ok(())
    }


    pub fn add_fd(&mut self, fd: ZmqFileDesc, user_data_: Option<&mut [u8]>, events_: i16) -> i32 {
        if (find_if2(_items.begin(), _items.end(), fd, &is_fd) != _items.end()) {
            errno = EINVAL;
            return -1;
        }

        let item = ZmqItem {
            socket: None,
            fd: fd,
            user_data: user_data_.to_vec(),
            events: events_,
            pollfd_index: -1,
        };
        _items.push_back(item);


        // catch (const std::bad_alloc &) {
        // errno = ENOMEM;
        // return -1;
        // }
        _need_rebuild = true;

        return 0;
    }

    pub fn modify(&mut self, socket: &ZmqSocket, events_: i16) -> anyhow::Result<()> {
        let it = find_if2(_items.begin(), _items.end(), socket, &is_socket);

        if (it == _items.end()) {
            // errno = EINVAL;
            // return -1;
            return Err(anyhow!("EINVAL"));
        }

        it.events = events_;
        _need_rebuild = true;

        Ok(())
    }

    pub fn modify_fd(&mut self, fd: ZmqFileDesc, events_: i16) -> i32 {
        let it = find_if2(_items.begin(), _items.end(), fd, &is_fd);

        if (it == _items.end()) {
            errno = EINVAL;
            return -1;
        }

        it.events = events_;
        _need_rebuild = true;

        return 0;
    }

    pub fn remove(&mut self, socket: &mut ZmqSocket) -> i32 {
        let it = find_if2(_items.begin(), _items.end(), socket, &is_socket);

        if (it == _items.end()) {
            errno = EINVAL;
            return -1;
        }

        _items.erase(it);
        _need_rebuild = true;

        if (is_thread_safe(socket)) {
            socket.remove_signaler(signaler);
        }

        return 0;
    }

    pub fn remove_fd(&mut self, fd: ZmqFileDesc) -> i32 {
        let it = find_if2(_items.begin(), _items.end(), fd, &is_fd);

        if (it == _items.end()) {
            errno = EINVAL;
            return -1;
        }

        _items.erase(it);
        _need_rebuild = true;

        return 0;
    }


    pub fn rebuild(&mut self) -> i32 {
        _use_signaler = false;
        _pollset_size = 0;
        _need_rebuild = false;

// #if defined ZMQ_POLL_BASED_ON_POLL

        if (_pollfds) {
            // free (_pollfds);
            // _pollfds = null_mut();
        }

        // for (items_t::iterator it = _items.begin (), end = _items.end (); it != end;
        //     += 1it)
        for it in _items.iter() {
            if (it.events) {
                if (it.socket && is_thread_safe(*it.socket)) {
                    if (!_use_signaler) {
                        _use_signaler = true;
                        _pollset_size += 1;
                    }
                } else {
                    _pollset_size += 1;
                }
            }
        }

        if (_pollset_size == 0) {
            return 0;
        }

        // TODO
        // _pollfds = static_cast<pollfd *> (malloc (_pollset_size * mem::size_of::<pollfd>()));

        if (!_pollfds) {
            errno = ENOMEM;
            _need_rebuild = true;
            return -1;
        }

        let mut item_nbr = 0;

        if (_use_signaler) {
            item_nbr = 1;
            _pollfds[0].fd = signaler.get_fd();
            _pollfds[0].events = POLLIN;
        }

        // for (items_t::iterator it = _items.begin (), end = _items.end (); it != end;
        //     += 1it)
        for it in _items.iter() {
            if (it.events) {
                if (it.socket) {
                    if (!is_thread_safe(*it.socket)) {
                        let fd_size = mem::sizeof::<ZmqFileDesc>();
                        let rc: i32 = it.socket.getsockopt(
                            ZMQ_FD, &_pollfds[item_nbr].fd, &fd_size);
                        // zmq_assert (rc == 0);

                        _pollfds[item_nbr].events = POLLIN;
                        item_nbr += 1;
                    }
                } else {
                    _pollfds[item_nbr].fd = it.fd;
                    _pollfds[item_nbr].events = (if it.events & ZMQ_POLLIN { POLLIN } else { 0 }) | (if it.events & ZMQ_POLLOUT { POLLOUT } else { 0 }) | (if it.events & ZMQ_POLLPRI { POLLPRI } else { 0 });
                    it.pollfd_index = item_nbr;
                    item_nbr += 1;
                }
            }
        }

// #elif defined ZMQ_POLL_BASED_ON_SELECT

        //  Ensure we do not attempt to select () on more than FD_SETSIZE
        //  file descriptors.
        // zmq_assert (_items.size () <= FD_SETSIZE);

        _pollset_in.resize(_items.size());
        _pollset_out.resize(_items.size());
        _pollset_err.resize(_items.size());

        FD_ZERO(_pollset_in.get());
        FD_ZERO(_pollset_out.get());
        FD_ZERO(_pollset_err.get());

        // for (items_t::iterator it = _items.begin (), end = _items.end (); it != end;
        //     += 1it)
        for it in _items.iter() {
            if (it.socket && is_thread_safe(*it.socket) && it.events) {
                _use_signaler = true;
                FD_SET(signaler.get_fd(), _pollset_in.get());
                _pollset_size = 1;
                break;
            }
        }

        _max_fd = 0;

        //  Build the fd_sets for passing to select ().
        // for (items_t::iterator it = _items.begin (), end = _items.end (); it != end;
        //     += 1it)
        {
            if (it.events) {
                //  If the poll item is a 0MQ socket we are interested in input on the
                //  notification file descriptor retrieved by the ZMQ_FD socket option.
                if (it.socket) {
                    if (!is_thread_safe(*it.socket)) {
                        let mut notify_fd: ZmqFileDesc = 0;
                        let fd_size = mem::sizeof::<ZmqFileDesc>();
                        let rc = it.socket.getsockopt(ZMQ_FD, &notify_fd, &fd_size);
                        // zmq_assert (rc == 0);

                        FD_SET(notify_fd, _pollset_in.get());
                        if (_max_fd < notify_fd) {
                            _max_fd = notify_fd;
                        }

                        _pollset_size += 1;
                    }
                }
                //  Else, the poll item is a raw file descriptor. Convert the poll item
                //  events to the appropriate fd_sets.
                else {
                    if (it.events & ZMQ_POLLIN) {
                        FD_SET(it.fd, _pollset_in.get());
                    }
                    if (it.events & ZMQ_POLLOUT) {
                        FD_SET(it.fd, _pollset_out.get());
                    }
                    if (it.events & ZMQ_POLLERR) {
                        FD_SET(it.fd, _pollset_err.get());
                    }
                    if (_max_fd < it.fd) {
                        _max_fd = it.fd;
                    }

                    _pollset_size += 1;
                }
            }
        }

// #endif

        return 0;
    }

    pub fn zero_trail_events(&mut self, events: &[ZmqPollerEvent], n_events_: i32, found_: i32) {
        // for (int i = found_; i < n_events_; += 1i)
        for i in found_..n_events_ {
            events_[i].socket = null_mut();
            events_[i].fd = retired_fd;
            events_[i].user_data = null_mut();
            events_[i].events = 0;
        }
    }


    // #if defined ZMQ_POLL_BASED_ON_POLL
    pub fn check_events(&mut self, events: &mut [poller_event],
                        n_events_: i32) -> i32 {
// #elif defined ZMQ_POLL_BASED_ON_SELECT
// int ZmqSocketPoller::check_events (ZmqSocketPoller::event_t *events_,
//                                         n_events_: i32,
//                                         fd_set &inset_,
//                                         fd_set &outset_,
//                                         fd_set &errset_)
// #endif {
        let mut found = 0;
        // for (items_t::iterator it = _items.begin (), end = _items.end ();
        //      it != end && found < n_events_; += 1it)
        for it in _items.iter() {
            //  The poll item is a 0MQ socket. Retrieve pending events
            //  using the ZMQ_EVENTS socket option.
            if (it.socket) {
                let events_size = 4;
                let mut events = 0u32;
                if (it.socket.getsockopt(ZMQ_EVENTS, &events, &events_size) == -1) {
                    return -1;
                }

                if (it.events & events) {
                    events_[found].socket = it.socket;
                    events_[found].fd = retired_fd;
                    events_[found].user_data = it.user_data;
                    events_[found].events = it.events & events;
                    found += 1;
                }
            }
            //  Else, the poll item is a raw file descriptor, simply convert
            //  the events to ZmqPollItem-style format.
            else if (it.events) {
// #if defined ZMQ_POLL_BASED_ON_POLL
                // zmq_assert (it.pollfd_index >= 0);
                let revents = _pollfds[it.pollfd_index].revents;
                let mut events = 0;

                if (revents & POLLIN) {
                    events |= ZMQ_POLLIN;
                }
                if (revents & POLLOUT) {
                    events |= ZMQ_POLLOUT;
                }
                if (revents & POLLPRI) {
                    events |= ZMQ_POLLPRI;
                }
                if (revents & !(POLLIN | POLLOUT | POLLPRI)) {
                    events |= ZMQ_POLLERR;
                }

// #elif defined ZMQ_POLL_BASED_ON_SELECT
//
//             short events = 0;
//
//             if (FD_ISSET (it.fd, &inset_))
//                 events |= ZMQ_POLLIN;
//             if (FD_ISSET (it.fd, &outset_))
//                 events |= ZMQ_POLLOUT;
//             if (FD_ISSET (it.fd, &errset_))
//                 events |= ZMQ_POLLERR;
// // #endif //POLL_SELECT

                if (events) {
                    events_[found].socket = null_mut();
                    events_[found].fd = it.fd;
                    events_[found].user_data = it.user_data;
                    events_[found].events = events;
                    found += 1;
                }
            }
        }

        return found;
    }

    pub fn adjust_timeout(&mut self, clock_: &clock_t,
                          timeout: i32,
                          now_: &mut u64,
                          end_: &mut u64,
                          first_pass_: &mut bool) -> i32 {
        //  If ZmqSocketPoller::timeout is zero, exit immediately whether there
        //  are events or not.
        if (timeout == 0) {
            return 0;
        }

        //  At this point we are meant to wait for events but there are none.
        //  If timeout is infinite we can just loop until we get some events.
        if (timeout < 0) {
            if (first_pass_) {
                *first_pass_ = false;
            }
            return 1;
        }

        //  The timeout is finite and there are no events. In the first pass
        //  we get a timestamp of when the polling have begun. (We assume that
        //  first pass have taken negligible time). We also compute the time
        //  when the polling should time out.
        *now_ = clock_.now_ms();
        if (first_pass_) {
            *end_ = now_ + timeout;
            *first_pass_ = false;
            return 1;
        }

        //  Find out whether timeout have expired.
        if (now_ >= end_) {
            return 0;
        }

        return 1;
    }


    pub fn wait(events_: &mut [poller_event],
                n_events_: i32,
                mut imeout: i32) -> i32 {
        if (_items.is_empty() && timeout < 0) {
            errno = EFAULT;
            return -1;
        }

        if (_need_rebuild) {
            let rc: i32 = rebuild();
            if (rc == -1) {
                return -1;
            }
        }

        if ((_pollset_size == 0)) {
            if (timeout < 0) {
                // Fail instead of trying to sleep forever
                errno = EFAULT;
                return -1;
            }
            // We'll report an error (timed out) as if the list was non-empty and
            // no event occurred within the specified timeout. Otherwise the caller
            // needs to check the return value AND the event to avoid using the
            // nullified event data.
            errno = EAGAIN;
            if (timeout == 0) {
                return -1;
            }
// // #if defined ZMQ_HAVE_WINDOWS
//         Sleep (timeout > 0 ? timeout : INFINITE);
//         return -1;
// #elif defined ZMQ_HAVE_ANDROID
//         usleep (timeout * 1000);
//         return -1;
// #elif defined ZMQ_HAVE_OSX
//         usleep (timeout * 1000);
//         errno = EAGAIN;
//         return -1;
// #elif defined ZMQ_HAVE_VXWORKS
//         struct timespec ns_;
//         ns_.tv_sec = timeout / 1000;
//         ns_.tv_nsec = timeout % 1000 * 1000000;
//         nanosleep (&ns_, 0);
//         return -1;
// // #else
//         usleep (timeout * 1000);
//         return -1;
// // #endif
        }

// #if defined ZMQ_POLL_BASED_ON_POLL
        let clock = clock_t {};
        let mut now = 0u64;
        let mut end = 0u64;

        let first_pass = true;

        loop {
            //  Compute the timeout for the subsequent poll.
            timeout: i32;
            if (first_pass) {
                timeout = 0;
            } else if (timeout < 0) {
                timeout = -1;
            } else {
                timeout = u64::min(end - now, u64::MAX);
            }

            //  Wait for events.
            let rc: i32 = poll(_pollfds, _pollset_size, timeout);
            if (rc == -1 && errno == EINTR) {
                return -1;
            }
            // errno_assert (rc >= 0);

            //  Receive the signal from pollfd
            if (_use_signaler && _pollfds[0].revents & POLLIN) {
                signaler.recv();
            }

            //  Check for the events.
            let found: i32 = check_events(events_, n_events_);
            if (found) {
                if (found > 0) {
                    zero_trail_events(events_, n_events_, found);
                }
                return found;
            }

            //  Adjust timeout or break
            if (adjust_timeout(clock, timeout, now, end, first_pass) == 0) {
                break;
            }
        }
        errno = EAGAIN;
        return -1;

// #elif defined ZMQ_POLL_BASED_ON_SELECT
//
//     clock_t clock;
//     u64 now = 0;
//     u64 end = 0;
//
//     bool first_pass = true;
//
//     OptimizedFdSet inset (_pollset_size);
//     OptimizedFdSet outset (_pollset_size);
//     OptimizedFdSet errset (_pollset_size);
//
//     while (true) {
//         //  Compute the timeout for the subsequent poll.
//         timeval timeout;
//         timeval *ptimeout;
//         if (first_pass) {
//             timeout.tv_sec = 0;
//             timeout.tv_usec = 0;
//             ptimeout = &timeout;
//         } else if (timeout < 0)
//             ptimeout = null_mut();
//         else {
//             timeout.tv_sec = static_cast<long> ((end - now) / 1000);
//             timeout.tv_usec = static_cast<long> ((end - now) % 1000 * 1000);
//             ptimeout = &timeout;
//         }
//
//         //  Wait for events. Ignore interrupts if there's infinite timeout.
//         memcpy (inset.get (), _pollset_in.get (),
//                 valid_pollset_bytes (*_pollset_in.get ()));
//         memcpy (outset.get (), _pollset_out.get (),
//                 valid_pollset_bytes (*_pollset_out.get ()));
//         memcpy (errset.get (), _pollset_err.get (),
//                 valid_pollset_bytes (*_pollset_err.get ()));
//         let rc: i32 = select ( (_max_fd + 1), inset.get (),
//                                outset.get (), errset.get (), ptimeout);
// // #if defined ZMQ_HAVE_WINDOWS
//         if ( (rc == SOCKET_ERROR)) {
//             errno = wsa_error_to_errno (WSAGetLastError ());
//             wsa_assert (errno == ENOTSOCK);
//             return -1;
//         }
// // #else
//         if ( (rc == -1)) {
//             // errno_assert (errno == EINTR || errno == EBADF);
//             return -1;
//         }
// // #endif
//
//         if (_use_signaler && FD_ISSET (signaler.get_fd (), inset.get ()))
//             signaler.recv ();
//
//         //  Check for the events.
//         let found: i32 = check_events (events_, n_events_, *inset.get (),
//                                         *outset.get (), *errset.get ());
//         if (found) {
//             if (found > 0)
//                 zero_trail_events (events_, n_events_, found);
//             return found;
//         }
//
//         //  Adjust timeout or break
//         if (adjust_timeout (clock, timeout, now, end, first_pass) == 0)
//             break;
//     }
//
//     errno = EAGAIN;
//     return -1;
//
// // #else
//
//     //  Exotic platforms that support neither poll() nor select().
//     errno = ENOTSUP;
//     return -1;
//
// // #endif
    }
} // end of impl


//Return 0 if timeout is expired otherwise 1

