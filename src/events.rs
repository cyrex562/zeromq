pub struct ZmqEvents {
    
}

impl ZmqEvents {
    // Called by I/O thread when file descriptor is ready for reading.
    pub fn in_event(&mut self) {
        
    }
    // Called by I/O thread when file descriptor is ready for writing.
    pub fn out_event(&mut self) {
        
    }
    // Called when timer expires.
    pub fn timer_event(&mut self, id: i32) {
        
    }
}