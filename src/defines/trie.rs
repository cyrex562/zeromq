use std::cmp::min;
use std::mem;
use std::os::raw::c_void;
use std::ptr::null_mut;

use libc::size_t;

// use crate::defines::atomic_counter::ZmqAtomicCounter;

// pub union TrieNext {
//     pub node: *mut ZmqTrie,
//     pub table: *mut *mut ZmqTrie,
// }

// pub struct ZmqTrie {
//     pub _next: TrieNext,
//     pub _refcnt: u32,
//     pub _min: u8,
//     pub _count: u16,
//     pub _live_nodes: u16,
// }

// impl ZmqTrie {
//     pub fn new() -> Self {
//         ZmqTrie {
//             _next: TrieNext {
//                 node: std::ptr::null_mut(),
//             },
//             _refcnt: 0,
//             _min: 0,
//             _count: 0,
//             _live_nodes: 0,
//         }
//     }
//
//     // bool zmq::trie_t::add (unsigned char *prefix_, size_t size_)
//     pub unsafe fn add(&mut self, prefix_: &str, size_: usize) -> bool {
//         //  We are at the node corresponding to the prefix. We are Done.
//         if (!size_) {
//             self._refcnt += 1;
//             return self._refcnt == 1;
//         }
//
//         let c = prefix_.as_bytes()[0];
//         if c < self._min || c >= self._min + self._count {
//             //  The character is out of range of currently handled
//             //  characters. We have to extend the table.
//             if (!self._count) {
//                 self._min = c;
//                 self._count = 1;
//                 self._next.node = null_mut()
//             } else if self._count == 1 {
//                 let oldc = self._min;
//                 let oldp = self._next.node;
//                 self._count = ((if self._min < c {
//                     c - self._min
//                 } else {
//                     self._min - c
//                 }) + 1) as u16;
//                 self._next.table = libc::malloc(std::mem::size_of::<*mut ZmqTrie>() * self._count)
//                     as *mut *mut ZmqTrie;
//                 // static_cast<trie_t **> (malloc (sizeof (trie_t *) * _count));
//                 // alloc_assert (_next.table);
//                 // for (unsigned short i = 0; i != _count; ++i)
//                 for i in 0..self._count {
//                     self._next.table[i] = 0;
//                 }
//                 self._min = min(self._min, c);
//                 self._next.table[oldc - self._min] = oldp;
//             } else if self._min < c {
//                 //  The new character is above the current character range.
//                 let old_count = self._count;
//                 self._count = (c - self._min + 1) as u16;
//                 self._next.table = libc::realloc(
//                     self._next.table as *mut c_void,
//                     std::mem::size_of::<*mut ZmqTrie>() * self._count,
//                 ) as *mut *mut ZmqTrie;
//                 //static_cast<trie_t **> (        realloc (_next.table, sizeof (trie_t *) * _count));
//                 // zmq_assert (_next.table);
//                 // for (unsigned short i = old_count; i != _count; i++)
//                 for i in old_count..self._count {
//                     self._next.table[i] = null_mut();
//                 }
//             } else {
//                 //  The new character is below the current character range.
//                 let old_count = self._count;
//                 self._count = ((self._min + old_count) - c) as u16;
//                 self._next.table = libc::realloc(
//                     self._next.table as *mut c_void,
//                     std::mem::size_of::<*mut ZmqTrie>() * self._count,
//                 ) as *mut *mut ZmqTrie;
//                 // static_cast<trie_t **> (
//                 // realloc (_next.table, sizeof (trie_t *) * _count));
//                 // zmq_assert (_next.table);
//                 libc::memmove(
//                     self._next.table + self._min - c,
//                     self._next.table as *const c_void,
//                     (old_count * std::mem::size_of::<*mut ZmqTrie>()) as size_t,
//                 );
//                 // for (unsigned short i = 0; i != _min - c; i++)
//                 for i in 0..self._min {
//                     self._next.table[i] = null_mut();
//                 }
//                 self._min = c;
//             }
//         }
//
//         //  If next node does not exist, create one.
//         if self._count == 1 {
//             if !self._next.node {
//                 self._next.node = &mut ZmqTrie::new(); //new (std::nothrow) trie_t;
//                                                        // alloc_assert (_next.node);
//                                                        // ++_live_nodes;
//                 self._live_nodes += 1;
//                 // zmq_assert (_live_nodes == 1);
//             }
//             return (*self._next.node).add(&prefix_[1..], size_ - 1);
//         }
//         if !self._next.table[c - self._min] {
//             self._next.table[c - self._min] = &mut ZmqTrie::new();
//             // alloc_assert (_next.table[c - _min]);
//             // ++_live_nodes;
//             self._live_nodes += 1;
//             // zmq_assert (_live_nodes > 1);
//         }
//         return self._next.table[c - self._min].add(&prefix_[1..], size_ - 1);
//     }
//
//     // bool zmq::trie_t::rm (unsigned char *prefix_, size_t size_)
//     pub unsafe fn rm(&mut self, prefix_: &str, size_: usize) -> bool {
//         //  TODO: Shouldn't an Error be reported if the key does not exist?
//         if !size_ {
//             if !self._refcnt {
//                 return false;
//             }
//             self._refcnt -= 1;
//             return self._refcnt == 0;
//         }
//         let c = prefix_.as_bytes()[0];
//         if self._count == 0 || c < self._min || c >= self._min + self._count {
//             return false;
//         }
//
//         let mut next_node = if self._count == 1 {
//             self._next.node
//         } else {
//             self._next.table[c - self._min]
//         };
//
//         if (!next_node) {
//             return false;
//         }
//
//         let ret = next_node.rm(&prefix_[1..], size_ - 1);
//
//         //  Prune redundant nodes
//         if next_node.is_redundant() {
//             // LIBZMQ_DELETE (next_node);
//             // zmq_assert (_count > 0);
//
//             if self._count == 1 {
//                 //  The just pruned node is was the only live node
//                 self._next.node = null_mut();
//                 self._count = 0;
//                 self._live_nodes -= 1;
//                 // zmq_assert (_live_nodes == 0);
//             } else {
//                 self._next.table[c - self._min] = 0;
//                 // zmq_assert (_live_nodes > 1);
//                 self._live_nodes -= 1;
//
//                 //  Compact the table if possible
//                 if self._live_nodes == 1 {
//                     //  We can switch to using the more compact single-node
//                     //  representation since the table only contains one live node
//                     // trie_t *node = 0;
//                     let mut node: *mut ZmqTrie = null_mut();
//                     //  Since we always compact the table the pruned node must
//                     //  either be the left-most or right-most ptr in the node
//                     //  table
//                     if c == self._min {
//                         //  The pruned node is the left-most node ptr in the
//                         //  node table => keep the right-most node
//                         node = self._next.table[self._count - 1];
//                         self._min += self._count - 1;
//                     } else if c == self._min + self._count - 1 {
//                         //  The pruned node is the right-most node ptr in the
//                         //  node table => keep the left-most node
//                         node = self._next.table[0];
//                     }
//                     // zmq_assert (node);
//                     // free (_next.table);
//                     self._next.node = node;
//                     self._count = 1;
//                 } else if c == self._min {
//                     //  We can compact the table "from the left".
//                     //  Find the left-most non-null node ptr, which we'll use as
//                     //  our new min
//                     let mut new_min = self._min;
//                     // for (unsigned short i = 1; i < _count; ++i)
//                     for i in 1..self._count {
//                         if self._next.table[i] {
//                             new_min = (i + self._min) as u8;
//                             break;
//                         }
//                     }
//                     // zmq_assert (new_min != _min);
//
//                     let old_table = self._next.table;
//                     // zmq_assert (new_min > _min);
//                     // zmq_assert (_count > new_min - _min);
//
//                     self._count = self._count - (new_min - self._min);
//                     self._next.table =
//                         libc::malloc(std::mem::size_of::<*mut ZmqTrie>() * self._count)
//                             as *mut *mut ZmqTrie;
//                     // static_cast<trie_t **> (malloc (sizeof (trie_t *) * _count));
//                     // alloc_assert (_next.table);
//
//                     libc::memmove(
//                         self._next.table as *mut c_void,
//                         old_table + (new_min - self._min),
//                         mem::size_of::<*mut ZmqTrie>() * self._count,
//                     );
//                     // free (old_table);
//
//                     self._min = new_min;
//                 } else if c == self._min + self._count - 1 {
//                     //  We can compact the table "from the right".
//                     //  Find the right-most non-null node ptr, which we'll use to
//                     //  determine the new table size
//                     let mut new_count = self._count;
//                     // for (unsigned short i = 1; i < _count; ++i)
//                     for i in 1.. {
//                         if self._next.table[self._count - 1 - i] {
//                             new_count = self._count - i;
//                             break;
//                         }
//                     }
//                     // zmq_assert (new_count != _count);
//                     self._count = new_count;
//
//                     let old_table = self._next.table;
//                     self._next.table =
//                         libc::malloc(std::mem::size_of::<*mut ZmqTrie>() * self._count)
//                             as *mut *mut ZmqTrie;
//                     // static_cast<trie_t **> (malloc (sizeof (trie_t *) * _count));
//                     // alloc_assert (_next.table);
//
//                     libc::memmove(
//                         self._next.table as *mut c_void,
//                         old_table as *const c_void,
//                         mem::size_of::<*mut ZmqTrie>() * self._count,
//                     );
//                     // free (old_table);
//                 }
//             }
//         }
//         return ret;
//     }
//
//     // bool zmq::trie_t::check (const unsigned char *data_, size_t size_) const
//     pub unsafe fn check(&mut self, mut data_: &[u8], mut size_: usize) -> bool {
//         //  This function is on critical path. It deliberately doesn't use
//         //  recursion to get a bit better performance.
//         let mut current = self;
//         loop {
//             //  We've found a corresponding subscription!
//             if (current._refcnt) {
//                 return true;
//             }
//
//             //  We've checked all the data and haven't found matching subscription.
//             if (!size_) {
//                 return false;
//             }
//
//             //  If there's no corresponding slot for the first character
//             //  of the prefix, the message does not match.
//             let c = data_[0];
//             if c < current._min || c >= current._min + current._count {
//                 return false;
//             }
//
//             //  Move to the next character.
//             if current._count == 1 {
//                 // TODO
//                 // current = current._next.node;
//             } else {
//                 current = current._next.table[c - current._min];
//                 if (!current) {
//                     return false;
//                 }
//             }
//             data_ = &data_[1..];
//             size_ -= 1;
//         }
//     }
//
//     // void zmq::trie_t::apply (void (*func_) (unsigned char *data_, size_t size_, void *arg_), void *arg_)
//     pub unsafe fn apply(&mut self, func_: fn(&mut [u8], usize, arg_: &mut [u8]), arg_: &mut [u8]) {
//         // unsigned char *buff = NULL;
//         let buff: &mut [u8] = &mut [];
//         self.apply_helper(buff, 0, 0, func_, arg_);
//         // free (buff);
//     }
//
//     // void zmq::trie_t::apply_helper (unsigned char **buff_,
//     //                             size_t buffsize_,
//     //                             size_t maxbuffsize_,
//     //                             void (*func_) (unsigned char *data_,
//     //                                            size_t size_,
//     //                                            void *arg_),
//     //                             void *arg_) const
//     pub unsafe fn apply_helper(
//         &mut self,
//         buff_: &mut [u8],
//         mut buffsize_: usize,
//         mut maxbuffsize_: usize,
//         func_: fn(&mut [u8], usize, &mut [u8]),
//         arg_: &mut [u8],
//     ) {
//         //  If this node is a subscription, apply the function.
//         if (self._refcnt) {
//             func_(buff_, buffsize_, arg_);
//         }
//
//         //  Adjust the buffer.
//         if (buffsize_ >= maxbuffsize_) {
//             maxbuffsize_ = buffsize_ + 256;
//             // todo
//             // *buff_ = static_cast<unsigned char *> (realloc (*buff_, maxbuffsize_));
//             // zmq_assert (*buff_);
//         }
//
//         //  If there are no subnodes in the trie, return.
//         if (self._count == 0) {
//             return;
//         }
//
//         //  If there's one subnode (optimisation).
//         if (self._count == 1) {
//             (buff_)[buffsize_] = self._min;
//             buffsize_ += 1;
//             self._next
//                 .node
//                 .apply_helper(buff_, buffsize_, maxbuffsize_, func_, arg_);
//             return;
//         }
//
//         //  If there are multiple subnodes.
//         // for (unsigned short c = 0; c != _count; c++)
//         for c in 0..self._count {
//             (*buff_)[buffsize_] = self._min + c;
//             if (self._next.table[c]) {
//                 self._next.table[c].apply_helper(buff_, buffsize_ + 1, maxbuffsize_, func_, arg_);
//             }
//         }
//     }
//
//     // bool zmq::trie_t::is_redundant () const
//     pub unsafe fn is_redundant(&mut self) -> bool {
//         return self._refcnt == 0 && self._live_nodes == 0;
//     }
// }

// pub struct trie_with_size_t {
//     pub _num_prefixes: ZmqAtomicCounter,
//     pub _trie: ZmqTrie,
// }

// impl trie_with_size_t {
//     pub fn new() -> Self {
//         trie_with_size_t {
//             _num_prefixes: ZmqAtomicCounter::new(0),
//             _trie: ZmqTrie {
//                 _next: TrieNext {
//                     node: std::ptr::null_mut(),
//                 },
//                 _refcnt: 0,
//                 _min: 0,
//                 _count: 0,
//                 _live_nodes: 0,
//             },
//         }
//     }
//
//     pub unsafe fn add(&mut self, prefix_: &str, size_: usize) -> bool {
//         if self._trie.add(prefix_, size_) {
//             self._num_prefixes.inc();
//             true
//         }
//         false
//     }
//
//     pub unsafe fn rm(&mut self, prefix_: &str, size_: usize) -> bool {
//         if self._trie.rm(prefix_, size_) {
//             self._num_prefixes.dec();
//             true
//         }
//         false
//     }
//
//     pub unsafe fn check(&mut self, data: &[u8], size_: usize) -> bool {
//         self._trie.check(data, size_)
//     }
//
//     pub unsafe fn apply(&mut self, func: fn(&mut [u8], usize, &mut [u8]), arg: &mut [u8]) {
//         self._trie.apply(func, arg)
//     }
//
//     pub unsafe fn num_prefixes(&mut self) -> u32 {
//         self._num_prefixes.get() as u32
//     }
// }
