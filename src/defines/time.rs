use libc;
use windows::Win32::Networking::WinSock::TIMEVAL;

#[derive(Default, Debug, Clone)]
pub struct ZmqTimeval {
    pub tv_sec: i32,
    pub tv_usec: i32,
}

pub fn zmq_timeval_to_timeval(tv: &ZmqTimeval) -> libc::timeval {
    libc::timeval {
        tv_sec: tv.tv_sec as libc::time_t,
        tv_usec: tv.tv_usec as libc::suseconds_t,
    }
}

pub fn zmq_timeval_to_ms_timeval(tv: &ZmqTimeval) -> TIMEVAL {
    TIMEVAL {
        tv_sec: tv.tv_sec,
        tv_usec: tv.tv_usec,
    }
}

pub fn timeval_to_zmq_timeval(tv: &libc::timeval) -> ZmqTimeval {
    ZmqTimeval {
        tv_sec: tv.tv_sec as i32,
        tv_usec: tv.tv_usec as i32,
    }
}
