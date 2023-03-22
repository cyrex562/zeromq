use crate::pipe::ZmqPipe;
use std::collections::HashMap;

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct inprocs_t {
    // typedef std::multimap<std::string, ZmqPipe *> map_t;
    //         map_t _inprocs;
    pub _inprocs: HashMap<String, ZmqPipe>,
}

impl inprocs_t {
    // void emplace (endpoint_uri_: *const c_char, pipe_: &mut ZmqPipe);
    pub fn emplace(&mut self, endpoint_uri_: &str, pipe: &mut ZmqPipe) {
        // self._inprocs.ZMQ_MAP_INSERT_OR_EMPLACE (endpoint_uri_), pipe_);
        let _old = self
            ._inprocs
            .insert(String::from(endpoint_uri_), pipe.clone());
    }

    // int erase_pipes (endpoint_uri_str_: &str);
    pub fn erase_pipes(&mut self, endpoint_uri_str_: &str) -> i32 {
        // const std::pair<map_t::iterator, map_t::iterator> range =
        //   _inprocs.equal_range (endpoint_uri_str_);
        // if (range.first == range.second) {
        //     errno = ENOENT;
        //     return -1;
        // }
        if self._inprocs.is_empty() {
            return -1;
        }

        // for (map_t::iterator it = range.first; it != range.second; ++it) {
        //     it->second->send_disconnect_msg ();
        //     it->second->terminate (true);
        // }
        for (_, pipe) in self._inprocs.iter_mut() {
            pipe.send_disconnect_msg();
            pipe.terminate(true);
        }

        // _inprocs.erase (range.first, range.second);
        self._inprocs.clear();
        return 0;
    }

    // void erase_pipe (const pipe_: &mut ZmqPipe);
    pub fn erase_pipe(&mut self, pipe: &mut ZmqPipe) {
        // for (map_t::iterator it = _inprocs.begin (), end = _inprocs.end ();
        //      it != end; ++it)
        //     if (it->second == pipe_) {
        //         _inprocs.erase (it);
        //         break;
        //     }
        self._inprocs.retain(|_, x| x != pipe);
    }
}
