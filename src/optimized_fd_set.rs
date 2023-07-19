use libc::fd_set;

// #if defined ZMQ_HAVE_WINDOWS
// struct fd_set {
//  u_int   fd_count;
//  SOCKET  fd_array[1];
// };
// NOTE: offsetof(fd_set, fd_array)==mem::size_of::<SOCKET>() on both x86 and x64
//       due to alignment bytes for the latter.
#[derive(Default, Debug, Clone)]
pub struct OptimizedFdSet {
    // fast_vector_t<SOCKET, 1 + ZMQ_POLLITEMS_DFLT> _fd_set;
    pub _fd_set: Vec<fd_set>,
}

impl OptimizedFdSet {
    // explicit OptimizedFdSet (nevents_: usize) : _fd_set (1 + nevents_) {}

    // fd_set *get () { return reinterpret_cast<fd_set *> (&_fd_set[0]); }
}

#[derive(Default, Debug, Clone)]
pub struct ResizableOptimizedFdSet {
    // resizable_fast_vector_t<SOCKET, 1 + ZMQ_POLLITEMS_DFLT> _fd_set;
    pub _fd_set: Vec<fd_set>,
}

impl ResizableOptimizedFdSet {
    // void resize (nevents_: usize) { _fd_set.resize (1 + nevents_); }

    // fd_set *get () { return reinterpret_cast<fd_set *> (&_fd_set[0]); }
    // pub fn get(&mut self) -> &mut fd_set {
    //     self.fd
    // }
}
