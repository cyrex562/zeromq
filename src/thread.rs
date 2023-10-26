

use std::collections::HashSet;
use std::ffi::{c_char, c_void};
use std::os::windows::raw::HANDLE;
use std::ptr::null_mut;
use libc::strncpy;
use windows::imp::{CloseHandle, WaitForSingleObject};
use windows::Win32::System::Threading::{CreateThread, GetCurrentThreadId, THREAD_CREATION_FLAGS};
use std::thread;
use std::thread::ThreadId;

pub const DEFAULT_PRIORITY: u32 = 100;
pub const DEFAULT_OPTIONS: u32 = 0;
pub const DEFAULT_STACK_SIZE: u32 = 4000;

pub type ZmqThreadFn = fn(*mut c_void);

pub struct ZmqThread {
    pub _arg: *mut c_void,
    pub _tfn: ZmqThreadFn,
    pub _name: [c_char; 16],
    pub _started: bool,
    #[cfg(target_os = "windows")]
    pub _descriptor: HANDLE,
    pub _thread_id: ThreadId,
    pub _thread_priority: i32,
    pub _thread_sched_policy: i32,
    pub _thread_affinity_cpus: HashSet<i32>,
    pub _join_handle: thread::JoinHandle<()>,
    pub _builder: thread::Builder,
}


#[cfg(target_os = "windows")]
pub struct thread_info_t {
    pub _type: u16,
    pub _name: *mut c_char,
    pub _thread_id: u32,
    pub _flags: u32,
}

impl ZmqThread {
    pub fn get_started(&self) -> bool {
        self._started
    }

    pub fn thread_routine(arg_: *mut c_void)
    {
        let self_ = unsafe { &mut *(arg_ as *mut ZmqThread) };
        self_.apply_scheduling_parameters();
        self_.apply_thread_name();
        self_._tfn(self_._arg);
    }

    pub unsafe fn start(&mut self, tfn_: ZmqThreadFn, arg: *mut c_void, name: *const c_char) {
        self._tfn = tfn_;
        self._arg = arg;
        if name != null_mut() {
            unsafe {
                strncpy(self._name.as_mut_ptr(), name, self._name.len() - 1);
            }
        }
        let mut stack = 0usize;
        #[cfg(target_arch = "x86_64")]{
            stack = 0x400000;
        }

        self._builder = thread::Builder::new().stack_size(stack);

        let handle = self._builder.spawn(move || {
            ZmqThread::thread_routine(self as *mut c_void);
        }).unwrap();
        self._started = true;
        self._join_handle = handle;
    }

    pub unsafe fn is_current_thread(&self) -> bool {
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
        let name_string: String = unsafe { String::from_raw_parts(self._name.as_mut_ptr() as *mut u8, self._name.len(), self._name.len()) };
        self._builder.name(name_string);
    }

}
