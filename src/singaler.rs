use libc::EAGAIN;
use crate::fd::fd_t;
use crate::utils::get_errno;

pub struct signaler_t
{
    pub _w: fd_t,
    pub _r: fd_t,
}

impl signaler_t
{
    pub fn new() -> Self {
        let mut out = Self {
          _r: 0,
            _W: 0
        };
        let mut rc = make_fdpair(&mut out._r, &mut out._w);
        if rc == 0 {
            unblock_socket(out._r);
            unblock_socket(out._w);
        }
        
    }
}

pub  unsafe fn close_wait_ms(fd_: i32, max_ms_: u32) -> i32 {
    let mut ms_so_far = 0i32;
    let min_step_ms = 1u32;
    let max_step_ms = 100u32;
    let step_ms = u32::min(u32::max(min_step_ms, max_ms_ / 10), max_step_ms);

    let mut rc = 0i32;
    while ms_so_far < max_ms_ as i32 && rc == -1 && get_errno() == EAGAIN
    {
        if rc == -1 && get_errno() == EAGAIN {
            std::thread::sleep(std::time::Duration::from_millis(step_ms.into()));
            ms_so_far += step_ms as i32;
        }
        rc = libc::close(fd_);
    }
    
    rc
}

