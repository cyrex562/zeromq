pub struct ZmqMutex {
    pub _mutex: std::sync::Mutex<bool>,
    // pthread_mutex_t _mutex;
    // pthread_mutexattr_t _attr;
}

impl ZmqMutex {
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
            }
            Err(_) => return false,
        }
    }

    pub fn get_mutex(&mut self) -> &mut std::sync::Mutex<bool> {
        &mut self._mutex
    }
}

pub struct ZmqScopedLock<'a> {
    pub _mutex: &'a mut ZmqMutex,
}

impl<'a> ZmqScopedLock<'a> {
    pub fn new(mutex: &mut ZmqMutex) -> Self {
        Self {
            _mutex: mutex,
        }
    }
}

pub struct ZmqScopedOptionalLock {
    pub _mutex: *mut ZmqMutex,
}

impl ZmqScopedOptionalLock {
    pub fn new(mutex: *mut ZmqMutex) -> Self {
        Self {
            _mutex: mutex,
        }
    }
}
