use crate::atomic_counter::atomic_counter_t;

pub union trie_next {
    pub node: *mut trie_t,
    pub table: *mut *mut trie_t,
}

pub struct trie_t {
    pub _next: trie_next,
    pub _refcnt: u32,
    pub _min: u8,
    pub _count: u16,
    pub _live_nodes: u16,
}

pub struct trie_with_size_t {
    pub _num_prefixes: atomic_counter_t,
    pub _trie: trie_t,
}

impl trie_with_size_t {
    pub fn new() -> Self {
        trie_with_size_t {
            _num_prefixes: atomic_counter_t::new(),
            _trie: trie_t {
                _next: trie_next {
                    node: std::ptr::null_mut(),
                },
                _refcnt: 0,
                _min: 0,
                _count: 0,
                _live_nodes: 0,
            },
        }
    }

    pub unsafe fn add(&mut self, prefix_: &str, size_: usize) -> bool {
        if self._trie.add(prefix_, size_) {
            self._num_prefixes.inc();
            true
        }
        false
    }

    pub unsafe fn rm(&mut self, prefix_: &str, size_: usize) -> bool {
        if self._trie.rm(prefix_, size_) {
            self._num_prefixes.dec();
            true
        }
        false
    }

    pub unsafe fn check(&mut self, data: &[u8], size_: usize) -> bool {
        self._trie.check(data, size_)
    }

    pub unsafe fn apply(&mut self, func: fn(&mut [u8], usize, &mut [u8]), arg: &mut [u8]) {
        self._trie.apply(func, arg)
    }
}
