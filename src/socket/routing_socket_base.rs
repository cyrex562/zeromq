use std::collections::HashMap;
use std::ffi::c_void;
use crate::ctx::ZmqContext;
use crate::defines::ZMQ_CONNECT_ROUTING_ID;
use crate::pipe::out_pipe::ZmqOutpipe;
use crate::pipe::ZmqPipe;
use crate::socket::ZmqSocket;

impl ZmqRoutingSocketBase {
    pub unsafe fn new(parent_: &mut ZmqContext, tid_: u32, sid_: i32) -> Self {
        Self {
            base: ZmqSocket::new(parent_, tid_, sid_, false),
            _out_pipes: HashMap::new(),
            _connect_routing_id: String::new(),
        }
    }

    pub unsafe fn xsetsockopt(&mut self, option_: i32, optval: *const c_void, optvallen_: usize) -> i32 {
        if option_ == ZMQ_CONNECT_ROUTING_ID {
            if optvallen_ > 255 {
                // errno = EINVAL;
                return -1;
            }
            self._connect_routing_id = String::from_raw_parts(optval as *mut u8, optvallen_, optvallen_);
            return 0;
        }

        return self.base.xsetsockopt(option_, optval, optvallen_);
    }

    pub unsafe fn xwrite_activated(&mut self, pipe_: &mut ZmqPipe) {
        for it in self._out_pipes.iter_mut() {
            if it.1.pipe == *pipe_ {
                it.1.active = true;
                break;
            }
        }
    }

    pub unsafe fn extract_connect_routing_id(&mut self) -> String {
        // std::string res = ZMQ_MOVE (_connect_routing_id);
        let res = self._connect_routing_id.clone();
        self._connect_routing_id.clear();
        return res;
    }

    pub unsafe fn connect_routing_id_is_set(&mut self) -> bool {
        return !self._connect_routing_id.empty();
    }

    pub unsafe fn add_out_pipe(&mut self, routing_id_: Vec<u8>, pipe_: &mut ZmqPipe) {
        //  Add the record into output pipes lookup table
        // const out_pipe_t outpipe = {pipe_, true};
        let outpipe = ZmqOutpipe {
            pipe: &mut pipe_.clone(),
            active: true,
        };
        let ok = self._out_pipes.insert((routing_id_.clone()), outpipe);
    }

    pub unsafe fn has_out_pipe(&mut self, routing_id_: Vec<u8>) -> bool {
        return self._out_pipes.contains_key(&routing_id_);
    }

    pub unsafe fn lookup_out_pipe(&mut self, routing_id_: Vec<u8>) -> *mut ZmqOutpipe {
        // out_pipes_t::iterator it = _out_pipes.find (routing_id_);
        // if (it == _out_pipes.end ())
        //     return NULL;
        // return &it.1;
        return self._out_pipes.get_mut(&routing_id_).unwrap();
    }

    pub unsafe fn erase_out_pipe(&mut self, pipe_: &mut ZmqPipe) {
        let erased = self._out_pipes.erase(pipe_.get_routing_id());
    }

    pub unsafe fn try_erase_out_pipe(&mut self, routing_id_: &Vec<u8>) -> ZmqOutpipe {
        // out_pipes_t::iterator it = _out_pipes.find (routing_id_);
        // if (it == _out_pipes.end ())
        //     return out_pipe_t ();
        // out_pipe_t outpipe = it.1;
        // _out_pipes.erase (it);
        // return outpipe;
        return self._out_pipes.remove(routing_id_).unwrap();
    }
}

#[derive(Default,Debug,Clone)]
pub struct ZmqRoutingSocketBase<'a> {
    pub base: ZmqSocket<'a>,
    pub _out_pipes: HashMap<Vec<u8>, ZmqOutpipe<'a>>,
    pub _connect_routing_id: String,
}
