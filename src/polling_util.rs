

// #if defined ZMQ_POLL_BASED_ON_POLL
// typedef int timeout_t;
//
// timeout_t
// compute_timeout (first_pass_: bool, long timeout, now_: u64, u64 end_);
// // #endif
// #if (!defined ZMQ_POLL_BASED_ON_POLL && defined ZMQ_POLL_BASED_ON_SELECT)      \
//   || defined ZMQ_HAVE_PPOLL
// #if defined ZMQ_HAVE_WINDOWS
use libc::fd_set;

fn valid_pollset_bytes (pollset_: &fd_set) -> usize
{
    // On Windows we don't need to copy the whole fd_set.
    // SOCKETS are continuous from the beginning of fd_array in fd_set.
    // We just need to copy fd_count elements of fd_array.
    // We gain huge memcpy() improvement if number of used SOCKETs is much lower than FD_SETSIZE.
    return  (
             &pollset_.fd_array[pollset_.fd_count])
           -  (&pollset_);
}
// #else
// inline size_t valid_pollset_bytes (const fd_set & /*pollset_*/)
// {
//     return mem::size_of::<fd_set>();
// }
// #endif


// #if defined ZMQ_HAVE_WINDOWS
// struct fd_set {
//  u_int   fd_count;
//  SOCKET  fd_array[1];
// };
// NOTE: offsetof(fd_set, fd_array)==mem::size_of::<SOCKET>() on both x86 and x64
//       due to alignment bytes for the latter.
#[derive(Default,Debug,Clone)]
pub struct optimized_fd_set_t
{
    // fast_vector_t<SOCKET, 1 + ZMQ_POLLITEMS_DFLT> _fd_set;
    pub _fd_set: Vec<fd_set>,
}

impl optimized_fd_set_t {
    explicit optimized_fd_set_t (nevents_: usize) : _fd_set (1 + nevents_) {}

    fd_set *get () { return reinterpret_cast<fd_set *> (&_fd_set[0]); }
}

#[derive(Default,Debug,Clone)]
pub struct resizable_optimized_fd_set_t
{
    // resizable_fast_vector_t<SOCKET, 1 + ZMQ_POLLITEMS_DFLT> _fd_set;
    pub _fd_set: Vec<fd_set>,
}

impl resizable_optimized_fd_set_t {
    // void resize (nevents_: usize) { _fd_set.resize (1 + nevents_); }

    // fd_set *get () { return reinterpret_cast<fd_set *> (&_fd_set[0]); }
    pub fn get(&mut self) -> &mut fd_set {
        self.fd
    }

}

// #else
pub struct optimized_fd_set_t
{
//
    explicit optimized_fd_set_t (size_t /*nevents_*/) {}

    fd_set *get () { return &_fd_set; }

  //
    fd_set _fd_set;
};
pub struct resizable_optimized_fd_set_t : public optimized_fd_set_t
{
//
    resizable_optimized_fd_set_t () : optimized_fd_set_t (0) {}

    void resize (size_t /*nevents_*/) {}
};
// #endif
// #endif

pub fn compute_timeout (first_pass_: bool,
                                      timeout: i32,
                                      now_: u64,
                                      end_: u64) -> i32
{
    if (first_pass_) {
        return 0;
    }

    if (timeout < 0) {
        return -1;
    }

    // return (
    //   std::min<u64> (end_ - now_, INT_MAX));
    i32::min((end_ - now_) as i32, i32::max_value())
}
// #endif
