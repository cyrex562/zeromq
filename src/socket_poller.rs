use std::ffi::c_void;
use std::ptr::null_mut;
use windows::Win32::Networking::WinSock::FD_SET;
use crate::defines::ZMQ_FD;
use crate::fd::fd_t;
use crate::polling_util::resizable_optimized_fd_set_t;
use crate::select::fd_set;
use crate::signaler::signaler_t;
use crate::socket_base::socket_base_t;

pub type event_t = zmq_poller_event_t;

pub struct item_t {
    pub socket: *mut socket_base_t,
    pub fd: fd_t,
    pub user_data: *mut c_void,
    pub events: i16,
    pub pollfd_index: i32,
}

pub type items_t = Vec<item_t>;

pub struct socket_poller_t {
    pub _tag: u32,
    pub _signaler: *mut signaler_t,
    pub _items: items_t,
    pub _need_rebuild: bool,
    pub _use_signaler: bool,
    pub _pollset_size: i32,
    #[cfg(feature = "poll")]
    pub _pollfds: *mut pollfd,
    #[cfg(feature = "select")]
    pub _pollset_in: resizable_optimized_fd_set_t,
    #[cfg(feature = "select")]
    pub _pollset_out: resizable_optimized_fd_set_t,
    #[cfg(feature = "select")]
    pub _pollset_err: resizable_optimized_fd_set_t,
    #[cfg(feature = "select")]
    pub _max_fd: fd_t,
}


impl socket_poller_t {
    pub fn new() -> Self {
        Self {
            _tag: 0xdeadbeef,
            _signaler: null_mut(),
            _items: vec![],
            _need_rebuild: false,
            _use_signaler: false,
            _pollset_size: 0,
            _pollfds: null_mut(),
            _pollset_in: fd_set { fd_count: 0, fd_array: [] },
            _pollset_out: fd_set { fd_count: 0, fd_array: [] },
            _pollset_err: fd_set { fd_count: 0, fd_array: [] },
            _max_fd: 0,
        }
    }

    pub fn check_tag(&mut self) -> bool {
        self._tag == 0xCAFEBABE
    }

    pub unsafe fn signaler_fd(&mut self, fd_: *mut fd_t) -> i32 {
        if (self._signaler) {
            *fd_ = self._signaler.get_fd();
            return 0;
        }
        // Only thread-safe socket types are guaranteed to have a signaler.
        // errno = EINVAL;
        return -1;
    }

    pub fn add(&mut self, socket_: *mut socket_base_t,
               user_data_: *mut c_void,
               events_: i16) -> i32 {
        // if (find_if2 (self._items.begin (), _items.end (), socket_, &is_socket)
        //     != _items.end ()) {
        //     errno = EINVAL;
        //     return -1;
        // }

        if is_thread_safe(*socket_) {
            if self._signaler == null_mut() {
                self._signaler = &mut signaler_t::new();
                if !self._signaler {
                    // errno = ENOMEM;
                    return -1;
                }
                if !self._signaler.valid() {
                    // delete _signaler;
                    self._signaler = null_mut();
                    // errno = EMFILE;
                    return -1;
                }
            }

            socket_.add_signaler(self._signaler);
        }

        let mut item: item_t = item_t {
            socket_,
            0,
            user_data_,
            events_
            // #if defined ZMQ_POLL_BASED_ON_POLL,
            -1,
            // #endif
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

        return 0;
    }

    pub fn add_fd(&mut self, fd_: fd_t, user_data_: *mut c_void, events_: i16) -> i32 {
        //      if (find_if2 (_items.begin (), _items.end (), fd_, &is_fd)
        //     != _items.end ()) {
        //     errno = EINVAL;
        //     return -1;
        // }

        let mut item: item_t = item_t {
            null_mut(),
            fd_,
            user_data_,
            events_
// #if defined ZMQ_POLL_BASED_ON_POLL,
            -1,
// #endif
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

        return 0;
    }

    pub fn modify(&mut self, socket_: *mut socket_base_t, events_: i16) -> i32 {
         // let it = items_t. const items_t::iterator it =
          // find_if2 (_items.begin (), _items.end (), socket_, &is_socket);

        if (it == self._items.end ()) {
            // errno = EINVAL;
            return -1;
        }

        it.events = events_;
        self._need_rebuild = true;

        return 0;
    }

    pub fn modify_fd(&mut self, fd_: fd_t, events_: i16) -> i32 {
        // let it = items_t. const items_t::iterator it =
        // find_if2 (_items.begin (), _items.end (), fd_, &is_fd);

        if (it == self._items.end ()) {
            // errno = EINVAL;
            return -1;
        }

        it.events = events_;
        self._need_rebuild = true;

        return 0;
    }

    pub fn remove_fd(&mut self, fd_: fd_t) -> i32 {
        const items_t::iterator it =
            find_if2 (_items.begin (), _items.end (), fd_, &is_fd);

        if (it == _items.end ()) {
            errno = EINVAL;
            return -1;
        }

        _items.erase (it);
        _need_rebuild = true;

        return 0;


    }

    pub unsafe fn rebuild(&mut self) -> i32 {
        self._use_signaler = false;
    self._pollset_size = 0;
    self._need_rebuild = false;

// #if defined ZMQ_POLL_BASED_ON_POLL
#[cfg(feature="poll")]{
    if (self._pollfds) {
        // free(_pollfds);
        // _pollfds = NULL;
    }

    // for (items_t::iterator it = _items.begin(), end = _items.end();
    // it != end;
    // + +it) {
    for it in self._items.iter() {
        if (it.events) {
            if (it.socket != null_mut() && is_thread_safe(&mut *it.socket)) {
                if (!self_use_signaler) {
                    self._use_signaler = true;
                    self._pollset_size += 1;
                }
            } else {
                _pollset_size += 1;
            }
        }
    }

    if (_pollset_size == 0) {
        return 0;
    }

    self._pollfds = libc::malloc(_pollset_size * size_of_val(self._pollfds) as libc::size_t;
        //static_cast < pollfd * > (malloc(_pollset_size * sizeof(pollfd)));

    if (!_pollfds) {
        errno = ENOMEM;
        self._need_rebuild = true;
        return -1;
    }

    let mut item_nbr = 0;

    if (_use_signaler) {
        item_nbr = 1;
        self._pollfds[0].fd = self._signaler.get_fd();
        self._pollfds[0].events = POLLIN;
    }

    // for (items_t::iterator it = _items.begin(), end = _items.end();
    // it != end;
    // + +it)
    for it in self._items.iter()
        {
        if (it.events) {
            if (it.socket) {
                if (!is_thread_safe(&mut *it.socket)) {
                    let mut fd_size = mem::size_of::<fd_t>();
                    let rc = it.socket.getsockopt(
                        ZMQ_FD, &self._pollfds[item_nbr].fd, &fd_size);
                    // zmq_assert(rc == 0);

                    self._pollfds[item_nbr].events = POLLIN;
                    item_nbr += 1;
                }
            } else {
                self._pollfds[item_nbr].fd = it.fd;
                // self._pollfds[item_nbr].events = (it.events & ZMQ_POLLIN?
                // POLLIN: 0) | (it.events & ZMQ_POLLOUT?
                // POLLOUT: 0) | (it.events & ZMQ_POLLPRI?
                // POLLPRI: 0);
                it.pollfd_index = item_nbr;
                item_nbr += 1;
            }
        }
    }
}
// #elif defined ZMQ_POLL_BASED_ON_SELECT
#[cfg(feature="select")]{
        //  Ensure we do not attempt to select () on more than FD_SETSIZE
        //  file descriptors.
        // zmq_assert (_items.size () < = FD_SETSIZE);

        self._pollset_in.resize (self._items.size ());
        self._pollset_out.resize (self._items.size ());
        self._pollset_err.resize (self._items.size ());

        FD_ZERO (self._pollset_in.get ());
        FD_ZERO (self._pollset_out.get ());
        FD_ZERO (self._pollset_err.get ());

        // for (items_t::iterator it = _items.begin (), end = _items.end (); it != end; + + it)
    for it in self._items.iter()
    {
        if (it.socket & & is_thread_safe ( *it.socket) && it.events) {
            self._use_signaler = true;
            FD_SET (self._signaler.get_fd (), _pollset_in.get ());
            self._pollset_size = 1; break;
        }
        }

        self._max_fd = 0;

        //  Build the fd_sets for passing to select (). for (items_t::iterator it = _items.begin (), end = _items.end (); it != end; ++ it) {
        if (it.events) {
        //  If the poll item is a 0MQ socket we are interested in input on the
        //  notification file descriptor retrieved by the ZMQ_FD socket option. if (it -> socket) {
        if ( ! is_thread_safe ( * it.socket)) {
        // zmq::fd_t notify_fd;
            let mut notify_fd: zmq::fd_t = 0;
            let fd_size = sizeof (zmq::fd_t);
            let rc = it.socket.getsockopt (ZMQ_FD, & notify_fd, &fd_size);
            // zmq_assert (rc == 0);

        FD_SET (notify_fd, _pollset_in.get ());
            if (_max_fd < notify_fd) {
                _max_fd = notify_fd;
            }

        _pollset_size += 1;
        }
        }
        //  Else, the poll item is a raw file descriptor. Convert the poll item
        //  events to the appropriate fd_sets. else {
        if (it -> events & ZMQ_POLLIN)
        FD_SET (it -> fd, _pollset_in.get ()); if (it -> events & ZMQ_POLLOUT)
        FD_SET (it -> fd, _pollset_out.get ()); if (it -> events & ZMQ_POLLERR)
        FD_SET (it -> fd, _pollset_err.get ()); if (_max_fd < it -> fd)
        _max_fd = it -> fd;

        _pollset_size + +;
        }
        }
        }}

// #endif

    return 0;
    }
}

pub fn is_thread_safe(socket_: &mut socket_base_t) -> bool {
    // do not use getsockopt here, since that would fail during context termination
    return socket_.is_thread_safe();
}
