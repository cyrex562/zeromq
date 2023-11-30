use std::ffi::{c_uint, c_void};
use std::mem;
use std::ptr::null_mut;
use libc::{EINTR, time_t, timeval};

#[cfg(not(target_os = "windows"))]
use libc::{suseconds_t};

#[cfg(target_os = "windows")]
use windows::Win32::Networking::WinSock::{FD_SET, POLLIN, POLLOUT, POLLPRI};
use windows::Win32::Networking::WinSock::SOCKET_ERROR;
#[cfg(target_os = "windows")]
use windows::Win32::System::Threading::{INFINITE, Sleep};

use crate::defines::{RETIRED_FD, ZMQ_EVENTS, ZMQ_FD, ZMQ_POLLERR, ZMQ_POLLIN, ZMQ_POLLOUT, ZMQ_POLLPRI, ZmqFd, ZmqPollFd};
use crate::defines::clock::ZmqClock;
use crate::defines::err::ZmqError;
use crate::defines::err::ZmqError::PollerError;
use crate::defines::time::ZmqTimeval;
use crate::io::signaler::ZmqSignaler;
use crate::platform::platform_select;
use crate::options::ZmqOptions;
use crate::poll::poller_event::ZmqPollerEvent;
use crate::poll::polling_util::{ResizableOptimizedFdSetT, valid_pollset_bytes};
use crate::poll::select::{fd_set, FD_SET, FD_ZERO};
use crate::socket::ZmqSocket;
use crate::utils::{FD_ISSET, get_errno};

pub type ZmqEvent<'a> = ZmqPollerEvent<'a>;

#[derive(Default, Debug)]
pub struct ZmqItem<'a> {
    pub socket: Option<&'a mut ZmqSocket<'a>>,
    pub fd: ZmqFd,
    pub user_data: Vec<u8>,
    pub events: i16,
    pub pollfd_index: i32,
}

pub type ZmqItems<'a> = Vec<ZmqItem<'a>>;

#[derive(Default, Debug, Clone)]
pub struct ZmqSocketPoller<'a> {
    pub _tag: u32,
    pub _signaler: &'a mut ZmqSignaler,
    pub _items: ZmqItems<'a>,
    pub _need_rebuild: bool,
    pub _use_signaler: bool,
    pub _pollset_size: i32,
    #[cfg(feature = "poll")]
    pub _pollfds: Vec<ZmqPollFd>,
    #[cfg(feature = "select")]
    pub _pollset_in: ResizableOptimizedFdSetT,
    #[cfg(feature = "select")]
    pub _pollset_out: ResizableOptimizedFdSetT,
    #[cfg(feature = "select")]
    pub _pollset_err: ResizableOptimizedFdSetT,
    #[cfg(feature = "select")]
    pub _max_fd: ZmqFd,
}


impl<'a> ZmqSocketPoller<'a> {
    pub fn new() -> Self {
        let mut out = ZmqSocketPoller {
            _tag: 0xdeadbeef,
            _signaler: &mut ZmqSignaler::default(),
            _items: vec![],
            _need_rebuild: false,
            _use_signaler: false,
            _pollset_size: 0,

            ..Default::default()
        };

        #[cfg(feature = "poll")]
        {
            out._pollfds = vec![];
        }

        #[cfg(feature = "select")]
        {
            out._pollset_in = fd_set { fd_count: 0, fd_array: [0; 64] };
            out._pollset_out = fd_set { fd_count: 0, fd_array: [0; 64] };
            out._pollset_err = fd_set { fd_count: 0, fd_array: [0; 64] };
            out._max_fd = 0;
        }
        out
    }

    pub fn check_tag(&mut self) -> bool {
        self._tag == 0xCAFEBABE
    }

    pub fn signaler_fd(&mut self, fd_: &mut ZmqFd) -> Result<(), ZmqError> {
        if self._signaler {
            *fd_ = self._signaler.get_fd();
            return Ok(());
        }
        // Only thread-safe socket types are guaranteed to have a signaler.
        // errno = EINVAL;
        return Err(PollerError("EINVAL"));
    }

    pub fn add(&mut self, socket_: &mut ZmqSocket,
               user_data_: &mut [u8],
               events_: i16) -> Result<(), ZmqError> {
        // if (find_if2 (self._items.begin (), _items.end (), socket_, &is_socket)
        //     != _items.end ()) {
        //     errno = EINVAL;
        //     return -1;
        // }

        if is_thread_safe(socket_) {
            if self._signaler == null_mut() {
                self._signaler = &mut ZmqSignaler::new();
                if !self._signaler {
                    // errno = ENOMEM;
                    return Err(PollerError("ENOMEM"));
                }
                if !self._signaler.valid() {
                    // delete _signaler;
                    self._signaler = &mut ZmqSignaler::default();
                    // errno = EMFILE;
                    return Err(PollerError("EMFILE"));
                }
            }

            socket_.add_signaler(self._signaler);
        }

        let item: ZmqItem = ZmqItem {
            socket: Some(socket_),
            fd: 0,
            user_data: user_data_.into_vec(),
            events: events_,
            // socket_,
            // 0,
            // user_data_,
            // events_
            // // #if defined ZMQ_POLL_BASED_ON_POLL,
            // -1,
            // // #endif
            pollfd_index: -1,
        };
        // try {
        //     _items.push_back (item);
        // }
        // catch (const std::bad_alloc &) {
        //     errno = ENOMEM;
        //     return -1;
        // }
        self._items.push_back(item);
        self._need_rebuild = true;

        return Ok(());
    }

    pub fn add_fd(&mut self, fd_: ZmqFd, user_data_: &mut [u8], events_: i16) -> Result<(), ZmqError> {
        //      if (find_if2 (_items.begin (), _items.end (), fd_, &is_fd)
        //     != _items.end ()) {
        //     errno = EINVAL;
        //     return -1;
        // }

        let mut item: ZmqItem = ZmqItem {
            socket: None,
            fd: fd_,
            user_data: user_data_.into_vec(),
            events: events_,
//             null_mut(),
//             fd_,
//             user_data_,
//             events_
// // #if defined ZMQ_POLL_BASED_ON_POLL,
//             -1,
// #endif
            pollfd_index: -1,
        };
        // try {
        //     _items.push_back (item);
        // }
        // catch (const std::bad_alloc &) {
        //     errno = ENOMEM;
        //     return -1;
        // }
        self._items.push_back(item);
        self._need_rebuild = true;

        return Ok(());
    }

    pub fn modify(&mut self, socket_: &mut ZmqSocket, events_: i16) -> Result<(), ZmqError> {
        // let it = items_t. const items_t::iterator it =
        // find_if2 (_items.begin (), _items.end (), socket_, &is_socket);
        for it in self._items.iter_mut() {
            if it.socket.unwrawp() == socket_ {
                it.events = events_;
                self._need_rebuild = true;
                return Ok(());
            }
        }
        return Err(PollerError("EINVAL"));

        // if it == self._items.end() {
        //     // errno = EINVAL;
        //     return -1;
        // }
        //
        // it.events = events_;
        // self._need_rebuild = true;
        //
        // return 0;
    }

    pub fn modify_fd(&mut self, fd_: ZmqFd, events_: i16) -> Result<(), ZmqError> {
        // let it = items_t. const items_t::iterator it =
        // find_if2 (_items.begin (), _items.end (), fd_, &is_fd);

        for it in self._items.iter_mut() {
            if it.fd == fd_ {
                it.events = events_;
                self._need_rebuild = true;
                return Ok(());
            }
        }
        return Err(PollerError("EINVAL"));

        // if it == self._items.end() {
        //     // errno = EINVAL;
        //     return -1;
        // }
        //
        // it.events = events_;
        // self._need_rebuild = true;
        //
        // return 0;
    }

    pub fn remove_fd(&mut self, fd_: ZmqFd) -> Result<(), ZmqError> {
        // const items_t
        // ::iterator
        // it = find_if2(_items.begin(), _items.end(), fd_, &is_fd);

        for it in self._items.iter_mut() {
            if it.fd == fd_ {
                self._items.erase(it);
                self._need_rebuild = true;
                return Ok(());
            }
        }
        return Err(PollerError("EINVAL"));

        // if (it == _items.end()) {
        //     errno = EINVAL;
        //     return -1;
        // }
        //
        // _items.erase(it);
        // _need_rebuild = true;
        //
        // return 0;
    }

    pub fn rebuild(&mut self, options: &ZmqOptions) -> Result<(), ZmqError> {
        self._use_signaler = false;
        self._pollset_size = 0;
        self._need_rebuild = false;

// #if defined ZMQ_POLL_BASED_ON_POLL
        #[cfg(feature = "poll")]{
            if self._pollfds {
                // free(_pollfds);
                // _pollfds = NULL;
            }

            // for (items_t::iterator it = _items.begin(), end = _items.end();
            // it != end;
            // + +it) {
            for it in self._items.iter() {
                if it.events {
                    if it.socket.is_some() && is_thread_safe(&mut *it.socket) {
                        if !self._use_signaler {
                            self._use_signaler = true;
                            self._pollset_size += 1;
                        }
                    } else {
                        self._pollset_size += 1;
                    }
                }
            }

            if self._pollset_size == 0 {
                return Ok(());
            }

            // self._pollfds = libc::malloc(self._pollset_size * size_of_val(self._pollfds) as libc::size_t;
            //static_cast < pollfd * > (malloc(_pollset_size * sizeof(pollfd)));
            self._pollfds = vec![];

            // if (!self._pollfds) {
            //     // errno = ENOMEM;
            //     self._need_rebuild = true;
            //     return -1;
            // }

            let mut item_nbr = 0;

            if self._use_signaler {
                item_nbr = 1;
                self._pollfds[0].fd = self._signaler.get_fd();
                self._pollfds[0].events = POLLIN.0;
            }

            // for (items_t::iterator it = _items.begin(), end = _items.end();
            // it != end;
            // + +it)
            for it in self._items.iter_mut() {
                if it.events {
                    if it.socket {
                        if !is_thread_safe(&mut *it.socket) {
                            let mut fd_size = mem::size_of::<ZmqFd>();
                            let res = it.socket.getsockopt(options, ZMQ_FD)?;
                            let res_fd: ZmqFd = u32::from_le_bytes([res[0], res[1], res[2], res[3]]) as ZmqFd;
                            self._pollfds[item_nbr].fd = res_fd;

                            // zmq_assert(rc == 0);


                            self._pollfds[item_nbr].events = POLLIN.0;
                            item_nbr += 1;
                        }
                    } else {
                        self._pollfds[item_nbr].fd = it.fd;
                        // self._pollfds[item_nbr].events = (it.events & ZMQ_POLLIN?
                        // POLLIN: 0) | (it.events & ZMQ_POLLOUT?
                        // POLLOUT: 0) | (it.events & ZMQ_POLLPRI?
                        // POLLPRI: 0);
                        it.pollfd_index = item_nbr as i32;
                        item_nbr += 1;
                    }
                }
            }
        }
// #elif defined ZMQ_POLL_BASED_ON_SELECT
        #[cfg(feature = "select")]{
            //  Ensure we do not attempt to select () on more than FD_SETSIZE
            //  file descriptors.
            // zmq_assert (_items.size () < = FD_SETSIZE);

            self._pollset_in.resize(self._items.size());
            self._pollset_out.resize(self._items.size());
            self._pollset_err.resize(self._items.size());

            FD_ZERO(self._pollset_in.get());
            FD_ZERO(self._pollset_out.get());
            FD_ZERO(self._pollset_err.get());

            // for (items_t::iterator it = _items.begin (), end = _items.end (); it != end; + + it)
            let mut it: &ZmqItem = &ZmqItem::default();
            for it in self._items.iter() {
                if it.socket.is_some() && is_thread_safe(it.socket.unwrap()) && it.events != 0 {
                    self._use_signaler = true;
                    FD_SET(self._signaler.get_fd(), self._pollset_in.get());
                    self._pollset_size = 1;
                    break;
                }
            }

            self._max_fd = 0;

            //  Build the fd_sets for passing to select (). for (items_t::iterator it = _items.begin (), end = _items.end (); it != end; ++ it) {
            if it.events {
                //  If the poll item is a 0MQ socket we are interested in input on the
                //  notification file descriptor retrieved by the ZMQ_FD socket option. if (it -> socket) {
                if !is_thread_safe(*it.socket) {
                    // zmq::fd_t notify_fd;
                    let mut notify_fd: ZmqFd = 0;
                    let fd_size = mem::size_of::<ZmqFd>();
                    let rc = it.socket.getsockopt(ZMQ_FD, &notify_fd, &fd_size);
                    // zmq_assert (rc == 0);

                    FD_SET(notify_fd, self._pollset_in.get());
                    if self._max_fd < notify_fd {
                        self._max_fd = notify_fd;
                    }

                    self._pollset_size += 1;
                }
            }
            //  Else, the poll item is a raw file descriptor. Convert the poll item
            //  events to the appropriate fd_sets. else {
            if it.events & ZMQ_POLLIN {
                FD_SET(it.fd, self._pollset_in.get());
            }
            if it.events & ZMQ_POLLOUT {
                FD_SET(it.fd, self._pollset_out.get());
            }
            if it.events & ZMQ_POLLERR {
                FD_SET(it.fd, self._pollset_err.get());
            }
            if self._max_fd < it.fd {
                self._max_fd = it.fd;
            }

            self._pollset_size += 1;
        }

        Ok(())
    }
// }

// #endif

// return 0; }

    pub fn zero_trail_events(&mut self, events_: &[ZmqEvent], n_events_: i32, found_: i32) {
        // for (i32 i = found_; i < n_events_; ++i) {
        //     events_[i].socket = NULL;
        //     events_[i].fd = 0;
        //     events_[i].events = 0;
        //     events_[i].user_data = NULL;
        // }
        for i in found_..n_events_ {
            events_[i].socket = null_mut();
            events_[i].fd = 0;
            events_[i].events = 0;
            events_[i].user_data = null_mut();
        }
    }

    // #[cfg(feature = "poll")]
    // pub unsafe fn check_events(&mut self, events_: *mut ZmqEvent, n_events_: i32) -> i32
    #[cfg(feature = "select")]
    pub fn check_events(&mut self, events_: &mut [ZmqEvent], n_events_: i32, inset_: Option<&mut fd_set>, outset_: Option<&mut fd_set>, errset_: Option<&mut fd_set>) -> Result<i32, ZmqError> {
        let mut found = 0usize;
        // for (items_t::iterator it = _items.begin (), end = _items.end ();
        //      it != end && found < n_events_; ++it)
        for it in self._items.iter_mut() {
            let mut events: u32 = 0;
            if found >= n_events_ {
                break;
            }
            //  The poll item is a 0MQ socket. Retrieve pending events
            //  using the ZMQ_EVENTS socket option.
            if it.socket {
                let events_size = 4;
                if it.socket.getsockopt(ZMQ_EVENTS, &events, &events_size) == -1 {
                    return Err(PollerError("EINVAL"));
                }

                if it.events & events {
                    events_[found].socket = it.socket.unwrap();
                    events_[found].fd = RETIRED_FD;
                    events_[found].user_data = it.user_data;
                    events_[found].events = it.events & events;
                    found += 1;
                }
            }
            //  Else, the poll item is a raw file descriptor, simply convert
            //  the events to zmq_pollitem_t-style format.
            else if it.events {
                // #if defined ZMQ_POLL_BASED_ON_POLL
                #[cfg(feature = "poll")]{
                    // zmq_assert (it.pollfd_index >= 0);
                    let revents = self._pollfds[it.pollfd_index].revents;
                    let mut events = 0;

                    if revents & POLLIN {
                        events |= ZMQ_POLLIN;
                    }
                    if revents & POLLOUT {
                        events |= ZMQ_POLLOUT;
                    }
                    if revents & POLLPRI {
                        events |= ZMQ_POLLPRI;
                    }
                    if revents & !(POLLIN | POLLOUT | POLLPRI) {
                        events |= ZMQ_POLLERR;
                    }
                }
                // #elif defined ZMQ_POLL_BASED_ON_SELECT
                #[cfg(feature = "select")] {
                    // let events = 0;

                    if FD_ISSET(it.fd, inset_.unwrap()) {
                        events |= ZMQ_POLLIN;
                    }
                    if FD_ISSET(it.fd, outset_.unwrap()) {
                        events |= ZMQ_POLLOUT;
                    }
                    if FD_ISSET(it.fd, errset_.unwrap()) {
                        events |= ZMQ_POLLERR;
                    }
                }
                // #endif //POLL_SELECT

                if events {
                    events_[found].socket = null_mut();
                    events_[found].fd = it.fd;
                    events_[found].user_data = it.user_data;
                    events_[found].events = events;
                    found += 1;
                }
            }
        }

        return Ok(found);
    }

    pub fn adjust_timeout(&mut self, clock_: &mut ZmqClock, timeout_: i32, now_: &mut u64, end_: &mut u64, first_pass_: &mut bool) -> Result<(), ZmqError> {
        //  If socket_poller_t::timeout is zero, exit immediately whether there
        //  are events or not.
        if timeout_ == 0 {
            return Ok(());
        }

        //  At this point we are meant to wait for events but there are none.
        //  If timeout is infinite we can just loop until we get some events.
        if timeout_ < 0 {
            if first_pass_ {
                *first_pass_ = false;
            }
            return Err(PollerError("EAGAIN"));
        }

        //  The timeout is finite and there are no events. In the first pass
        //  we get a timestamp of when the polling have begun. (We assume that
        //  first pass have taken negligible time). We also compute the time
        //  when the polling should time out.
        *now_ = clock_.now_ms();
        if first_pass_ {
            *end_ = *now_ + *timeout_;
            *first_pass_ = false;
            return Err(PollerError("EAGAIN"));
        }

        //  Find out whether timeout have expired.
        if now_ >= end_ {
            return Ok(());
        }

        return return Err(PollerError("EAGAIN"));;
    }

    pub fn wait(&mut self, options: &ZmqOptions, events_: &mut [ZmqEvent], n_events_: i32, timeout_: i32) -> Result<i32, ZmqError> {
        if self._items.empty() && timeout_ < 0 {
            // errno = EFAULT;
            return Err(PollerError("EFAULT"));
        }

        if self._need_rebuild {
            self.rebuild(options)?;
        }

        if self._pollset_size == 0 {
            if timeout_ < 0 {
                // Fail instead of trying to sleep forever
                // errno = EFAULT;
                return Err(PollerError("EFAULT"));
            }
            // We'll report an Error (timed out) as if the list was non-empty and
            // no event occurred within the specified timeout. Otherwise the caller
            // needs to check the return value AND the event to avoid using the
            // nullified event data.
            // errno = EAGAIN;
            if timeout_ == 0 {
                return Err(PollerError("EAGAIN"));
            }
            // #if defined ZMQ_HAVE_WINDOWS
            #[cfg(target_os = "windows")]{
                unsafe { Sleep(if timeout_ > 0 { timeout_ } else { INFINITE } as u32) };
                return Err(PollerError("EAGAIN"));
            }
            // #elif defined ZMQ_HAVE_ANDROID
            //         usleep (timeout_ * 1000);
            //         return -1;
            // #elif defined ZMQ_HAVE_OSX
            //         usleep (timeout_ * 1000);
            //         errno = EAGAIN;
            //         return -1;
            // #elif defined ZMQ_HAVE_VXWORKS
            //         struct timespec ns_;
            //         ns_.tv_sec = timeout_ / 1000;
            //         ns_.tv_nsec = timeout_ % 1000 * 1000000;
            //         nanosleep (&ns_, 0);
            //         return -1;
            // #else
            #[cfg(not(target_os = "windows"))]{
                unsafe { libc::usleep((timeout_ * 1000) as c_uint) };
                return Err(PollerError("EAGAIN"));
            }
            // #endif
        }

        // #if defined ZMQ_POLL_BASED_ON_POLL
        #[cfg(feature = "poll")]{
            let mut clock = ZmqClock::default();
            let mut now = 0u64;
            let mut end = 0u64;

            let mut first_pass = true;

            loop {
                //  Compute the timeout for the subsequent poll.
                let mut timeout = 0;
                if first_pass {
                    timeout = 0;
                } else if timeout_ < 0 {
                    timeout = 0;
                } else {
                    // timeout = static_cast < int > (std::min < uint64_t > (end - now, INT_MAX));
                    let timeout = std::cmp::min(end - now, u64::MAX);
                }

                //  Wait for events.
                platform_poll(self._pollfds.as_slice(), self._pollset_size as u32, timeout)?;
                // errno_assert (rc >= 0);

                //  Receive the signal from pollfd
                if self._use_signaler && self._pollfds[0].revents & POLLIN {
                    self._signaler.recv();
                }

                //  Check for the events.
                let found = self.check_events(events_, n_events_, None, None, None);
                if (found) {
                    if (found > 0) {
                        self.zero_trail_events(events_, n_events_, found);
                    }
                    return found;
                }

                //  Adjust timeout or break
                if self.adjust_timeout(clock, timeout_, &mut now, &mut end, &mut first_pass) == 0 {
                    break;
                }
            }
            // errno = EAGAIN;
            return -1;
        }
        // #elif defined ZMQ_POLL_BASED_ON_SELECT
        #[cfg(feature = "select")]
        {
            // zmq::clock_t clock;
            let mut clock = ZmqClock::default();
            // uint64_t now = 0;
            let mut now = 0u64;
            // uint64_t end = 0;
            let mut end = 0u64;

            // bool first_pass = true;
            let mut first_pass = true;

            // optimized_fd_set_t inset (_pollset_size);
            let mut inset = fd_set { fd_count: 0, fd_array: [0; 64] };
            // optimized_fd_set_t outset (_pollset_size);
            let mut outset = fd_set { fd_count: 0, fd_array: [0; 64] };
            // optimized_fd_set_t errset (_pollset_size);
            let mut errset = fd_set { fd_count: 0, fd_array: [0; 64] };

            loop {
                //  Compute the timeout for the subsequent poll.
                // timeval timeout;
                // let mut timeout = timeval { tv_sec: 0, tv_usec: 0 };
                // // timeval *ptimeout;
                // // let mut ptimeout: *mut timeval = null_mut();
                // if first_pass {
                //     timeout.tv_sec = 0;
                //     timeout.tv_usec = 0;
                //     // ptimeout = &mut timeout;
                // } else if timeout_ < 0 {
                //     // ptimeout = null_mut();
                // } else {
                //     timeout.tv_sec = ((end - now) / 1000) as time_t;
                //     timeout.tv_usec = ((end - now) % 1000 * 1000) as suseconds_t;
                //     // ptimeout = &mut timeout;
                // }
                let mut timeout = ZmqTimeval::default();
                if first_pass {
                    timeout.tv_sec = 0;
                    timeout.tv_usec = 0;
                } else if timeout_ < 0 {
                    timeout = ZmqTimeval::default();
                } else {
                    timeout.tv_sec = ((end - now) / 1000) as i32;
                    timeout.tv_usec = ((end - now) % 1000 * 1000) as i32;
                }

                //  Wait for events. Ignore interrupts if there's infinite timeout.
                // libc::memcpy(
                //     inset.get(),
                //     self._pollset_in.get(),
                //     valid_pollset_bytes(*self._pollset_in.get())
                // );
                inset = self._pollset_in.clone();


                // libc::memcpy(
                //     outset.get(),
                //     self._pollset_out.get(),
                //     valid_pollset_bytes(*self._pollset_out.get())
                // );
                outset = self._pollset_out.clone();


                // libc::memcpy(
                //     errset.get(),
                //     self._pollset_err.get(),
                //     valid_pollset_bytes(*self._pollset_err.get())
                // );
                errset = self._pollset_err.clone();

                // let rc = select((_max_fd + 1), inset.get(),
                //                 outset.get(), errset.get(), ptimeout);
                platform_select(self._max_fd + 1, Some(&mut inset),
                                Some(&mut outset), Some(&mut errset), if timeout_ < 0 { None } else { &mut timeout })?;
                // #if defined ZMQ_HAVE_WINDOWS
                // #[cfg(target_os = "windows")]{
                //     if ((rc == SOCKET_ERROR)) {
                //         // errno = wsa_error_to_errno(WSAGetLastError());
                //         // wsa_assert(errno == ENOTSOCK);
                //         return -1;
                //     }
                // }
                // #else
                // #[cfg(not(target_os = "windows"))]{
                //     if rc == -1 {
                //         // errno_assert(errno == EINTR || errno == EBADF);
                //         return -1;
                //     }
                // }
                // #endif

                if self._use_signaler && FD_ISSET(
                    self._signaler.get_fd(),
                    inset.get(),
                ) {
                    self._signaler.recv();
                }

                //  Check for the events.
                let found = self.check_events(
                    events_,
                    n_events_,
                    *inset.get(),
                    *outset.get(),
                    *errset.get(),
                )?;
                if found {
                    if found > 0 {
                        self.zero_trail_events(events_, n_events_, found);
                    }
                    return Ok(found);
                }

                //  Adjust timeout or break
                if self.adjust_timeout(
                    clock,
                    timeout_,
                    &mut now,
                    &mut end,
                    &mut first_pass,
                ).is_ok() {
                    break;
                }
            }

            // errno = EAGAIN;
            return Err(PollerError("EAGAIN"));
        }
        // #else
        //
        //     //  Exotic platforms that support neither poll() nor select().
        //     errno = ENOTSUP;
        //     return -1;
        //
        // #endif
    }
} // impl socket_poller_t

pub fn is_thread_safe(socket_: &mut ZmqSocket) -> bool {
    // do not use getsockopt here, since that would fail during context termination
    return socket_.is_thread_safe();
}
