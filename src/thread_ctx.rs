use std::sync::Mutex;
use std::collections::HashSet;
use crate::defines::{ZMQ_THREAD_PRIORITY_DFLT, ZMQ_THREAD_SCHED_POLICY_DFLT};

#[derive(Default,Debug,Clone)]
pub struct ThreadCtx {
    //
    //  Synchronisation of access to context options.
    // mutex_t _opt_sync;
    pub _opt_sync: Mutex<u8>,
    //
    //  Thread parameters.
    pub _thread_priority: i32,
    pub _thread_sched_policy: i32,
    // std::set<int> _thread_affinity_cpus;
    pub _thread_affinity_cpus: HashSet<i32>,
    // std::string _thread_name_prefix;
    pub _thread_name_prefix: String,
}

impl ThreadCtx {
    // ThreadCtx ();

    //  Start a new thread with proper scheduling parameters.
    // void start_thread (thread_t &thread_,
    //                    thread_fn *tfn_,
    //                    arg_: *mut c_void,
    //                    const char *name = NULL) const;

    // int set (option_: i32, const optval_: *mut c_void, optvallen_: usize);
    // int get (option_: i32, optval_: *mut c_void, const optvallen_: *mut usize);

pub fn new()-> Self {
    Self {
        _opt_sync: Mutex::new(0),
        _thread_priority: ZMQ_THREAD_PRIORITY_DFLT,
        _thread_sched_policy: ZMQ_THREAD_SCHED_POLICY_DFLT,
        _thread_affinity_cpus: Default::default(),
        _thread_name_prefix: "".to_string(),
    }

}


pub fn start_thread (thread: &mut thread_t,
                                      tfn: thread_fn,
                                      arg_: &mut [u8],
                                      name: &str) const
{
    thread_.setSchedulingParameters (_thread_priority, _thread_sched_policy,
                                     _thread_affinity_cpus);

    char namebuf[16] = "";
    snprintf (namebuf, sizeof (namebuf), "%s%sZMQbg%s%s",
              _thread_name_prefix.is_empty() ? "" : _thread_name_prefix,
              _thread_name_prefix.is_empty() ? "" : "/", name ? "/" : "",
              name ? name : "");
    thread_.start (tfn_, arg_, namebuf);
}

int set (option_: i32, const opt_val: &mut [u8], optvallen_: usize)
{
    const bool is_int = (optvallen_ == sizeof );
    int value = 0;
    if (is_int)
        memcpy (&value, optval_, sizeof );

    switch (option_) {
        ZMQ_THREAD_SCHED_POLICY =>
            if (is_int && value >= 0) {
                scoped_lock_t locker (_opt_sync);
                _thread_sched_policy = value;
                return 0;
            }


        ZMQ_THREAD_AFFINITY_CPU_ADD =>
            if (is_int && value >= 0) {
                scoped_lock_t locker (_opt_sync);
                _thread_affinity_cpus.insert (value);
                return 0;
            }


        ZMQ_THREAD_AFFINITY_CPU_REMOVE =>
            if (is_int && value >= 0) {
                scoped_lock_t locker (_opt_sync);
                if (0 == _thread_affinity_cpus.erase (value)) {
                    errno = EINVAL;
                    return -1;
                }
                return 0;
            }


        ZMQ_THREAD_PRIORITY =>
            if (is_int && value >= 0) {
                scoped_lock_t locker (_opt_sync);
                _thread_priority = value;
                return 0;
            }


        ZMQ_THREAD_NAME_PREFIX =>
            // start_thread() allows max 16 chars for thread name
            if (is_int) {
                std::ostringstream s;
                s << value;
                scoped_lock_t locker (_opt_sync);
                _thread_name_prefix = s.str ();
                return 0;
            } else if (optvallen_ > 0 && optvallen_ <= 16) {
                scoped_lock_t locker (_opt_sync);
                _thread_name_prefix.assign (static_cast<const char *> (optval_),
                                            optvallen_);
                return 0;
            }

    }

    errno = EINVAL;
    return -1;
}

int get (option_: i32,
                            optval_: &mut [u8],
                            const optvallen_: *mut usize)
{
    const bool is_int = (*optvallen_ == sizeof );
    int *value = static_cast<int *> (optval_);

    switch (option_) {
        ZMQ_THREAD_SCHED_POLICY =>
            if (is_int) {
                scoped_lock_t locker (_opt_sync);
                *value = _thread_sched_policy;
                return 0;
            }


        ZMQ_THREAD_NAME_PREFIX =>
            if (is_int) {
                scoped_lock_t locker (_opt_sync);
                *value = atoi (_thread_name_prefix.c_str ());
                return 0;
            } else if (*optvallen_ >= _thread_name_prefix.size ()) {
                scoped_lock_t locker (_opt_sync);
                memcpy (optval_, _thread_name_prefix.data (),
                        _thread_name_prefix.size ());
                return 0;
            }

    }

    errno = EINVAL;
    return -1;
}

}
