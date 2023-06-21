use std::collections::HashMap;
use libc::EINVAL;
use serde::{Deserialize, Serialize};
use crate::context::ZmqContext;
use crate::defines::ZMQ_CONNECT_ROUTING_ID;
use crate::out_pipe::ZmqOutPipe;
use crate::pipe::ZmqPipe;
use crate::socket::ZmqSocket;

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct routing_socket_base_t {
    //
    // methods from ZmqSocketBase
    // Own methods
    //
    //  Outbound pipes indexed by the peer IDs.
    // typedef std::map<Blob, ZmqOutPipe> out_pipes_t;
    // out_pipes_t _out_pipes;
    pub _out_pipes: HashMap<Blob, ZmqOutPipe>,
    pub base: ZmqSocket,
    // Next assigned name on a zmq_connect() call used by ROUTER and STREAM socket types
    // std::string _connect_routing_id;
    pub _connect_routing_id: String,
}

impl routing_socket_base_t {
    // routing_socket_base_t (parent: &mut ZmqContext, tid: u32, sid_: i32);
    // routing_socket_base_t::routing_socket_base_t (parent: &mut ZmqContext,
    // tid: u32,
    // sid_: i32) :
    // ZmqSocketBase (parent_, tid, sid_)
    // {
    // }
    pub fn new(parent_: &mut ZmqContext, options: &mut ZmqContext, tid: u32, sid_: i32) -> Self {
        Self {
            _out_pipes: HashMap::new(),
            base: ZmqSocket::new(parent_, options, tid, sid_, false),
            _connect_routing_id: String::new(),
        }
    }

    // ~routing_socket_base_t () ;
    // routing_socket_base_t::~routing_socket_base_t ()
    // {
    // zmq_assert (_out_pipes.empty ());
    // }

    // int xsetsockopt (option_: i32, const optval_: *mut c_void, ptvallen_: usize) ;
    pub fn xsetsockopt(&mut self, option_: i32, optval_: &[u8], optvallen_: usize) -> i32 {
        match (option_) {
            ZMQ_CONNECT_ROUTING_ID => {
                // TODO why isn't it possible to set an empty connect_routing_id
                //   (which is the default value)
                if (optvallen_ > 0) {
                    // self._connect_routing_id.assign(static_cast <const char
                    // * > (optval_),
                    // optvallen_);
                    self._connect_routing_id += String::from_utf8_lossy(optval_).to_string().as_str();
                    return 0;
                }
            }
            _ => {}
        }
        errno = EINVAL;
        return -1;
    }

    // void xwrite_activated (pipe_: &mut ZmqPipe) ;
    pub fn xwrite_activated(&mut self, pipe: &mut ZmqPipe) {
        // const out_pipes_t::iterator end = _out_pipes.end ();
        let (end_blob, end_pipe) = self._out_pipes.last().unwrap();
        // out_pipes_t::iterator it;
        // for (it = _out_pipes.begin (); it != end; += 1it)
        // let mut pipe_ref: &mut ZmqPipe
        for (it_blob, it_pipe) in self._out_pipes.iter_mut() {
            // if (it.second.pipe == pipe_) {
            //     break;
            // }
            if it_pipe == pipe && it_pipe != end_pipe && it_pipe.active == false {
                it_pipe.active = true;
                break;
            }
        }

        // zmq_assert (it != end);
        // zmq_assert (!it.second.active);
        // it.second.active = true;
    }

    // std::string extract_connect_routing_id ();
    pub fn extract_connect_routing_id(&mut self) -> String {
        let res = self._connect_routing_id.clone();
        self._connect_routing_id.clear();
        return res;
    }

    // bool connect_routing_id_is_set () const;
    pub fn connect_routing_id_is_set(&mut self) -> bool {
        return !self._connect_routing_id.is_empty();
    }

    // void add_out_pipe (Blob routing_id_, pipe_: &mut ZmqPipe);
    pub fn add_out_pipe(&mut self, routing_id_: Blob, pipe: &mut ZmqPipe) {
        //  Add the record into output pipes lookup table
        let outpipe = ZmqOutPipe::new(pipe, true);
        let ok = self._out_pipes.ZMQ_MAP_INSERT_OR_EMPLACE(routing_id_, outpipe).second;
        // zmq_assert (ok);
    }

    // bool has_out_pipe (const Blob &routing_id_) const;
    pub fn has_out_pipe(&mut self, routing_id_: &mut Blob) -> bool {
        return 0 != _out_pipes.count(routing_id_);
    }

    // ZmqOutPipe *lookup_out_pipe (const Blob &routing_id_);
    pub fn lookup_out_pipe(&mut self, routing_id_: &mut Blob) -> Option<ZmqOutPipe> {
        // TODO we could probably avoid constructor a temporary Blob to call this function
        // out_pipes_t::iterator it = _out_pipes.find (routing_id_);
        // return it == _out_pipes.end () ? null_mut() : &it.second;
        let result = self._out_pipes.iter().find(routing_id_);
        if result.is_some() {
            Some(result.unwrap())
        }
        None
    }

    // const ZmqOutPipe *lookup_out_pipe (const Blob &routing_id_) const;

    // void erase_out_pipe (const pipe_: &mut ZmqPipe);
    pub fn erase_out_pipe(&mut self, pipe: &mut ZmqPipe) {
        let erased = _out_pipes.erase(pipe.get_routing_id());
        // zmq_assert (erased);
    }

    // ZmqOutPipe try_erase_out_pipe (const Blob &routing_id_);
    pub fn try_erase_out_pipe(&mut self, routing_id_: &mut Blob) -> Option<ZmqOutPipe> {
        // const out_pipes_t::iterator it = _out_pipes.find (routing_id_);
        let result = self._out_pipes.remove(routing_id_);
        //     if result.is_some() {
        //
        //     }
        //     ZmqOutPipe res = {null_mut(), false};
        // if (it != _out_pipes.end ()) {
        // res = it.second;
        // _out_pipes.erase (it);
        // }
        // return res;
        // }
        result
    }

    // template <typename Func> bool any_of_out_pipes (Func func_)
    pub fn any_of_out_pipes(&mut self) -> bool {
        let mut res = false;
        // for (out_pipes_t::iterator it = _out_pipes.begin (),
        //                            end = _out_pipes.end ();
        //      it != end && !res; += 1it) {
        //     res |= func_ (*it->second.pipe);
        // }
        for pipe in self._out_pipes.iter_mut() {}

        return res;
    }
}
