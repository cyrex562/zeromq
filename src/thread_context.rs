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
use libc::{EINTR, EINVAL};
use std::collections::HashSet;
use std::sync::Mutex;
use std::{mem, thread};
use crate::defines::ZMQ_THREAD_SCHED_POLICY;
use crate::defines::ZMQ_THREAD_NAME_PREFIX;
use crate::defines::ZMQ_THREAD_PRIORITY;
use crate::defines::ZMQ_THREAD_AFFINITY_CPU_ADD;
use crate::defines::ZMQ_THREAD_AFFINITY_CPU_REMOVE;

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
    pub opt_sync: Mutex<u8>,
    //
    //  Thread parameters.
    pub thread_priority: i32,
    pub thread_sched_policy: i32,
    // std::set<int> _thread_affinity_cpus;
    pub thread_affinity_cpus: HashSet<i32>,
    // std::string _thread_name_prefix;
    pub thread_name_prefix: String,
    // DWORD _type;
    pub type_: u32,
    // LPCSTR _name;
    pub name: String,
    // DWORD _thread_id;
    pub thread_id: u32,
    // DWORD _flags;
    pub flags: u32,
    pub tfn: Option<thread_fn>,
    pub arg: Vec<u8>,
    pub started: bool,
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
            opt_sync: Mutex::new(0),
            thread_priority: 0,
            thread_sched_policy: 0,
            thread_affinity_cpus: Default::default(),
            thread_name_prefix: "".to_string(),
            type_: 0,
            name: "".to_string(),
            thread_id: 0,
            flags: 0,
            tfn: None,
            arg: vec![],
            started: false,
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
        } else if self.tfn.is_some() {
            self.thread_join_handle = Some(thread::spawn(move || {
                self.tfn.unwrap()(self.arg.clone());
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
        } else if self.tfn.is_some() && self.thread_join_handle.is_some() {
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
        self.poller.rm_fd(&self.mailbox_handle.unwrap());
        self.poller.stop();
    }

    //  Returns load experienced by the I/O thread.
    pub fn get_load(&mut self) -> u64 {
        self.poller.unwrap().base.base.get_load()
    }

    pub fn set(&mut self, option_: i32, opt_val: &mut [u8], optvallen_: usize) -> i32 {
        match (option_) {
            ZMQ_THREAD_SCHED_POLICY => {
                self.thread_sched_policy = i32::from_le_bytes(opt_val.clone())
            }
            ZMQ_THREAD_AFFINITY_CPU_ADD => {
                self.thread_affinity_cpus.insert(i32::from_le_bytes(opt_val.clone()));
            }
            ZMQ_THREAD_AFFINITY_CPU_REMOVE => {
                self.thread_affinity_cpus.remove(&i32::from_le_bytes(opt_val.clone()))
            }

            ZMQ_THREAD_PRIORITY => {
                self.thread_priority = i32::from_le_bytes(opt_val.clone());
            }

            ZMQ_THREAD_NAME_PREFIX => {
                self.thread_name_prefix = String::from_utf8(opt_val.to_vec()).unwrap();
            }
            _ => {}
        }

        errno = EINVAL;
        return -1;
    }

    pub fn get(&mut self, opt: i32) -> anyhow::Result<Vec<u8>> {
        match opt {
            ZMQ_THREAD_SCHED_POLICY => {
                let tsc = self.thread_sched_policy;
                let tsc_bytes = tsc.to_le_bytes();
                return Ok(tsc_bytes.to_vec());
            }

            ZMQ_THREAD_NAME_PREFIX => {
                let tnp = self.thread_name_prefix.clone();
                return Ok(tnp.into_bytes());
            }

            _ => {}
        }
        Err(anyhow::anyhow!("EINVAL"))
    }
}
