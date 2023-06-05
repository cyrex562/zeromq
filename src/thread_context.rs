use crate::command::ZmqCommand;
use crate::context::ZmqContext;
use crate::defines::ZmqHandle;
use crate::devpoll::ZmqPoller;
use crate::endpoint::{EndpointUriPair, ZmqEndpoint};
use crate::mailbox::ZmqMailbox;
use crate::object::ZmqObject;
use crate::own::ZmqOwn;
use crate::pipe::ZmqPipe;
use crate::session_base::ZmqSessionBase;
use crate::socket_base::ZmqSocketBase;
use libc::EINTR;
use std::collections::HashSet;
use std::sync::Mutex;
use std::{mem, thread};

pub const DEFAULT_PRIORITY: i32 = 100;
pub const DEFAULT_OPTIONS: i32 = 0;
pub const DEFAULT_STACK_SIZE: i32 = 4000;

#[derive(Default, Debug, Clone)]
pub struct ZmqThreadContext {
    //
    //  I/O thread accesses incoming commands via this mailbox.
    pub mailbox: Option<ZmqMailbox>,
    //  Handle associated with mailbox' file descriptor.
    pub mailbox_handle: Option<ZmqHandle>,
    //  I/O multiplexing is performed using a poller object.
    pub poller: Option<ZmqPoller>,
    // pub ctx: &'a mut ZmqContext,
    pub tid: u32,
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
    // DWORD _type;
    pub _type: u32,
    // LPCSTR _name;
    pub _name: String,
    // DWORD _thread_id;
    pub _thread_id: u32,
    // DWORD _flags;
    pub _flags: u32,
    pub _tfn: Option<thread_fn>,
    pub _arg: Vec<u8>,
    pub _started: bool,
    pub thread_join_handle: Option<thread::JoinHandle<()>>,
}

impl ZmqThreadContext {
    pub fn new(tid: u32) -> Self {
        let mut out = Self {
            mailbox: None,
            poller: None,
            mailbox_handle: None,
            // ctx: ctx,
            tid: 0,
            _opt_sync: Mutex::new(0),
            _thread_priority: 0,
            _thread_sched_policy: 0,
            _thread_affinity_cpus: Default::default(),
            _thread_name_prefix: "".to_string(),
            _type: 0,
            _name: "".to_string(),
            _thread_id: 0,
            _flags: 0,
            _tfn: None,
            _arg: vec![],
            _started: false,
            thread_join_handle: None,
        };
        // TODO
        // if out.mailbox.get_fd() != retired_fd {
        //     out.poller.add_fd(out.mailbox.get_fd(), &mut out);
        //     out.mailbox_handle = out.mailbox.get_fd();
        //     out.poller.set_pollin(&out.mailbox_handle.unwrap());
        // }
        out
    }

    // mailbox_handle (static_cast<Poller::handle_t> (null_mut()))
    // {
    // poller = new (std::nothrow) Poller (*ctx);
    // alloc_assert (poller);
    //
    // if (mailbox.get_fd () != retired_fd) {
    // mailbox_handle = poller.add_fd (mailbox.get_fd (), this);
    // poller.set_pollin (mailbox_handle);
    // }
    // }

    //  Clean-up. If the thread was started, it's necessary to call 'stop'
    //  before invoking destructor. Otherwise the destructor would hang up.
    //  Launch the physical thread.
    // void start ();
    pub fn start(&mut self) {
        let mut name: String = String::new();
        // snprintf (name, mem::size_of::<name>(), "IO/%u",
        //           get_tid () - ZmqContext::REAPER_TID - 1);
        // name = format!("IO/{}", get_tid() - ZmqContext::REAPER_TID - 1);
        //  Start the underlying I/O thread.
        if self.poller.is_some() {
            self.poller.as_mut().unwrap().start(name);
        } else if self._tfn.is_some() {
            self.thread_join_handle = Some(thread::spawn(move || {
                self._tfn.unwrap()(self._arg.clone());
            }));
        }
    }

    pub fn is_current_thread(&mut self) -> bool {
        thread::current().id() == self.thread_join_handle.unwrap().thread().id()
    }

    //  Ask underlying thread to stop.
    // void stop ();
    pub fn stop(&mut self) {
        if self.poller.is_some() {
            self.send_stop();
        } else if self._tfn.is_some() && self.thread_join_handle.is_some() {
            self.thread_join_handle
                .unwrap()
                .join()
                .expect("thread join failed");
        }
    }

    pub fn setSchedulingParameters(
        &mut self,
        priority_: i32,
        scheduling_policy_: i32,
        affinity_cps_: &HashSet<i32>,
    ) {
        // not implemented
        // LIBZMQ_UNUSED (priority_);
        // LIBZMQ_UNUSED (scheduling_policy_);
        // LIBZMQ_UNUSED (affinity_cpus_);
        unimplemented!("setSchedulingParameters")
    }

    pub fn applySchedulingParameters() // to be called in secondary thread context
    {
        // not implemented
        unimplemented!("applySchedulingParameters")
    }

    //  i_poll_events implementation.
    pub fn in_event(&mut self) {
        //  TODO: Do we want to limit number of commands I/O thread can
        //  process in a single go?

        let mut cmd: ZmqCommand = ZmqCommand::default();
        let rc = mailbox.recv(&cmd, 0);

        while (rc == 0 || errno == EINTR) {
            if (rc == 0) {
                cmd.destination.process_command(&cmd);
            }
            rc = mailbox.recv(&cmd, 0);
        }
    }

    pub fn out_event(&mut self) {
        //  We are never polling for POLLOUT here. This function is never called.
        unimplemented!()
    }

    pub fn timer_event(&mut self) {
        //  No timers here. This function is never called.
        unimplemented!()
    }

    //  Command handlers.
    pub fn process_stop(&mut self) {
        // zmq_assert (mailbox_handle);
        self.poller.rm_fd(&self.mailbox_handle.unwrap());
        self.poller.stop();
    }

    //  Returns load experienced by the I/O thread.
    pub fn get_load(&mut self) {
        return self.poller.base.get_load();
    }

    pub fn set(&mut self, option_: i32, opt_val: &mut [u8], optvallen_: usize) -> i32 {
        // const bool is_int = (optvallen_ == sizeof );
        let mut value = 0;
        // if (is_int) {
        //     memcpy(&value, optval_, sizeof);
        // }

        match (option_) {
            ZMQ_THREAD_SCHED_POLICY => {
                // if (is_int && value >= 0) {
                //     scoped_lock_t
                //     locker(_opt_sync);
                //     _thread_sched_policy = value;
                //     return 0;
                // }
            }

            ZMQ_THREAD_AFFINITY_CPU_ADD => {
                // if (is_int && value >= 0) {
                //     scoped_lock_t
                //     locker(_opt_sync);
                //     _thread_affinity_cpus.insert(value);
                //     return 0;
                // }
            }

            ZMQ_THREAD_AFFINITY_CPU_REMOVE => {
                // if (is_int && value >= 0) {
                //     scoped_lock_t
                //     locker(_opt_sync);
                //     if (0 == _thread_affinity_cpus.erase(value)) {
                //         errno = EINVAL;
                //         return -1;
                //     }
                //     return 0;
                // }
            }

            ZMQ_THREAD_PRIORITY => {
                // if (is_int && value >= 0) {
                //     scoped_lock_t
                //     locker(_opt_sync);
                //     _thread_priority = value;
                //     return 0;
                // }
            }

            ZMQ_THREAD_NAME_PREFIX => {
                // start_thread() allows max 16 chars for thread name
                if (is_int) {
                    // std::ostringstream
                    // s;
                    // s << value;
                    // scoped_lock_t
                    // locker(_opt_sync);
                    // _thread_name_prefix = s.str();
                    return 0;
                } else if (optvallen_ > 0 && optvallen_ <= 16) {
                    // scoped_lock_t
                    // locker(_opt_sync);
                    // _thread_name_prefix.assign(static_cast < const char * > (optval_),
                    //                            optvallen_);
                    return 0;
                }
            }
            _ => {}
        }

        errno = EINVAL;
        return -1;
    }

    pub fn get(&mut self, option_: i32) -> anyhow::Result<Vec<u8>> {
        // let is_int = (*optvallen_ == sizeof );
        // int *value = static_cast<int *> (optval_);

        match option_ {
            ZMQ_THREAD_SCHED_POLICY => {
                // if (is_int) {
                //     scoped_lock_t
                //     locker(_opt_sync);
                //     *value = _thread_sched_policy;
                //     return 0;
                // }
                return Ok(());
            }

            ZMQ_THREAD_NAME_PREFIX => {
                // if (is_int) {
                //     scoped_lock_t
                //     locker(_opt_sync);
                //     *value = atoi(_thread_name_prefix.c_str());
                //     return 0;
                // } else if (*optvallen_ >= _thread_name_prefix.size()) {
                //     scoped_lock_t
                //     locker(_opt_sync);
                //     memcpy(optval_, _thread_name_prefix.data(),
                //            _thread_name_prefix.size());
                //     return 0;
                // }
                return Ok(());
            }

            _ => {}
        }

        // errno = EINVAL;
        // return -1;
        Err(anyhow::anyhow!("EINVAL"))
    }
}
