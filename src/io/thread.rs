use std::collections::HashSet;
use std::ffi::c_void;
use std::thread;
use std::thread::ThreadId;

#[cfg(target_os = "windows")]
use crate::defines::ZmqHandle;

pub const DEFAULT_PRIORITY: u32 = 100;
pub const DEFAULT_OPTIONS: u32 = 0;
pub const DEFAULT_STACK_SIZE: u32 = 4000;

pub type ZmqThreadFn = fn(&[u8]);

// TODO: make serializable
pub struct ZmqThread<'a> {
    pub _arg: &'a mut [u8],
    pub _tfn: Option<ZmqThreadFn>,
    pub _name: String,
    pub _started: bool,
    #[cfg(target_os = "windows")]
    pub _descriptor: Option<ZmqHandle>,
    pub _thread_id: ThreadId,
    pub _thread_priority: i32,
    pub _thread_sched_policy: i32,
    pub _thread_affinity_cpus: HashSet<i32>,
    pub _join_handle: thread::JoinHandle<()>,
    pub _builder: thread::Builder,
}

impl<'a> Default for ZmqThread<'a> {
    fn default() -> Self {
        Self {
            _arg: &mut [0u8],
            _tfn: None,
            _name: "".to_string(),
            _started: false,
            #[cfg(target_os = "windows")]
            _descriptor: None,
            _thread_id: thread::current().id(),
            _thread_priority: -1,
            _thread_sched_policy: -1,
            _thread_affinity_cpus: HashSet::new(),
            _join_handle: thread::Builder::new().spawn(|| {}).unwrap(),
            _builder: thread::Builder::new(),
        }
    }
}

#[cfg(target_os = "windows")]
pub struct ThreadInfoT {
    pub _type: u16,
    pub _name: *mut libc::c_char,
    pub _thread_id: u32,
    pub _flags: u32,
}


pub fn thread_routine(arg: &mut [u8]) {
    // TODO deserialize ZmqThread object from arg_
    // let self_ = unsafe { &mut *(arg_ as &mut ZmqThread) };
    let mut thread = ZmqThread::default();
    thread.apply_scheduling_parameters();
    thread.apply_thread_name();
    thread._tfn(thread._arg);
}

impl<'a> ZmqThread<'a> {
    pub fn get_started(&self) -> bool {
        self._started
    }

    pub fn start(&mut self, tfn_: ZmqThreadFn, arg: &mut [u8], name: &str) {
        self._tfn = Some(tfn_);
        self._arg = arg;
        // if name != null_mut() {
        //     unsafe {
        //         strncpy(self._name.as_mut_ptr(), name, self._name.len() - 1);
        //     }
        // }

        if name.is_empty() == false {
            self._name = name.to_string();
        }

        let mut stack = 0usize;
        #[cfg(target_arch = "x86_64")]
        {
            stack = 0x400000;
        }

        self._builder = thread::Builder::new().stack_size(stack);

        let handle = self._builder.spawn(move || {
            // TODO serialize self into [u8]
            ZmqThread::thread_routine(self);
        }).unwrap();
        self._started = true;
        self._join_handle = handle;
    }

    pub fn is_current_thread(&self) -> bool {
        let curr_thread = thread::current();
        self._thread_id == curr_thread.id()
    }

    pub fn stop(&mut self) {
        self._join_handle.join();
        self._started = false;
    }

    pub fn set_scheduling_parameters(&mut self) {
        unimplemented!()
    }

    pub fn apply_scheduling_parameters(&mut self) {
        unimplemented!()
    }
    pub fn apply_thread_name(&mut self) {
        let name_string: String = unsafe {
            String::from_raw_parts(
                self._name.as_mut_ptr() as *mut u8,
                self._name.len(),
                self._name.len(),
            )
        };
        self._builder.name(name_string);
    }
}
