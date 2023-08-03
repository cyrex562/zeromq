
pub struct mutex_t
{
    pub _mutex: std::sync::Mutex<bool>,
    // pthread_mutex_t _mutex;
    // pthread_mutexattr_t _attr;

}

impl mutex_t {
    pub fn new() -> Self {
        Self {
            _mutex: std::sync::Mutex::new(false),
        }
    }

    pub fn lock(&mut self) {
        let mut guard = self._mutex.lock().unwrap();
        *guard = true;
    }

    pub fn unlock(&mut self) {
        let mut guard = self._mutex.lock().unwrap();
        *guard = false;
    }

    pub fn try_lock(&mut self) -> bool {
        let mut guard = self._mutex.lock();
        match guard {
            Ok(g) => {
                *g = true;
            return true;
            },
            Err(_) => return false,
        }
    }

    pub fn get_mutex(&mut self)-> &mut std::sync::Mutex<bool> {
        &mut self._mutex
    }

}

pub struct scoped_lock_t<'a> {
    pub _mutex: &'a mut mutex_t,
}

impl <'a>scoped_lock_t<'a> {
    pub fn new(mutex: &mut mutex_t) -> Self {
        Self {
            _mutex: mutex,
        }
    }
}

pub struct scoped_optional_lock_t {
    pub _mutex: *mut mutex_t,
}

impl scoped_optional_lock_t {
    pub fn new(mutex: *mut mutex_t) -> Self {
        Self {
            _mutex: mutex,
        }
    }
}