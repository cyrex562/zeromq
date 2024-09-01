// use crate::defines::atomic_counter::ZmqAtomicCounter;
// use libc::size_t;
// use std::cmp;
// use std::collections::HashSet;
// use std::ffi::c_void;
// use std::ptr::null_mut;

// enum RmResult {
//     NotFound,
//     LastValueRemoved,
//     ValuesRemain,
// }

// pub type Prefix<'a> = &'a mut [u8];

// pub union MtrieNext<T> {
//     pub node: *mut GenericMtrie<T>,
//     pub table: *mut *mut GenericMtrie<T>,
// }

// pub struct MtrieIter<T> {
//     pub node: *mut GenericMtrie<T>,
//     pub next_node: *mut GenericMtrie<T>,
//     pub prefix: Vec<u8>,
//     pub size: usize,
//     pub current_child: u16,
//     pub new_min: u8,
//     pub new_max: u8,
//     pub processed_for_removal: bool,
// }

// pub struct GenericMtrie<T> {
//     pub pipes: HashSet<*mut T>,
//     pub _num_prefixes: ZmqAtomicCounter,
//     pub _min: u8,
//     pub _count: u16,
//     pub _live_nodes: u16,
//     pub _next: MtrieNext<T>,
// }

// impl<T> GenericMtrie<T> {
//     pub fn new() -> GenericMtrie<T> {
//         GenericMtrie {
//             pipes: HashSet::new(),
//             _num_prefixes: ZmqAtomicCounter::new(0),
//             _min: 0,
//             _count: 0,
//             _live_nodes: 0,
//             _next: MtrieNext::<T> {
//                 node: std::ptr::null_mut(),
//             },
//         }
//     }
//
//     pub fn add(
//         &mut self,
//         mut prefix_: Option<&mut [u8]>,
//         mut size_: usize,
//         pipe: *mut T,
//     ) -> bool {
//         let mut it = self;
//         while size_ {
//             let c = *prefix_;
//
//             if c < it._min || c > it._min + it._count {
//                 if !it._count {
//                     it._min = c;
//                     it._count = 1;
//                     it._next.node = null_mut();
//                 } else if it._count == 1 {
//                     let oldc = it._min;
//                     let oldp = it._next.node;
//                     it._count = ((if it._min < c {
//                         c - it._min
//                     } else {
//                         it._min - c
//                     }) + 1) as u16;
//                     it._next.table = libc::malloc(
//                         it._count as usize * std::mem::size_of::<*mut GenericMtrie<T>>(),
//                     ) as *mut *mut GenericMtrie<T>;
//                     for i in 0..it._count {
//                         it._next.table[i] = 0;
//                     }
//                     it._min = cmp::min(it._min, c);
//                     it._next.table[oldc - it._min] = oldp;
//                 } else if it._min < c {
//                     let old_count = it._count;
//                     it._count = (c - it._min + 1) as u16;
//                     it._next.table = libc::realloc(
//                         it._next.table as *mut c_void,
//                         it._count as usize * std::mem::size_of::<*mut GenericMtrie<T>>(),
//                     ) as *mut *mut GenericMtrie<T>;
//                     for i in old_count..it._count {
//                         it._next.table[i] = null_mut();
//                     }
//                 } else {
//                     let old_count = it._count;
//                     it._count = (it._min + old_count - c) as u16;
//                     it._next.table = libc::realloc(
//                         it._next.table as *mut c_void,
//                         it._count as usize * std::mem::size_of::<*mut GenericMtrie<T>>(),
//                     ) as *mut *mut GenericMtrie<T>;
//                     libc::memmove(
//                         it._next.table.add((it._min - c) as usize) as *mut c_void,
//                         it._next.table as *const c_void,
//                         (old_count * std::mem::size_of::<*mut GenericMtrie<T>>()) as size_t,
//                     );
//                     for i in 0..(it._min - c) {
//                         it._next.table[i as usize] = null_mut();
//                     }
//                 }
//             }
//
//             if it._count == 1 {
//                 if it._next.node == null_mut() {
//                     it._next.node = Box::into_raw(Box::new(GenericMtrie::<T>::new()));
//                     it._live_nodes += 1;
//                 }
//                 it = &mut *it._next.node;
//             } else {
//                 if it._next.table[(c - it._min) as usize] == null_mut() {
//                     it._next.table[(c - it._min) as usize] =
//                         Box::into_raw(Box::new(GenericMtrie::<T>::new()));
//                     it._live_nodes += 1;
//                 }
//                 prefix_ = prefix_.add(1);
//                 size_ -= 1;
//                 it = &mut *it._next.table[(c - it._min) as usize];
//             }
//         }
//
//         let result = !self.pipes.is_empty();
//         if !it.pipes.is_empty() {
//             // it.pipes.insert(pipe);
//             self._num_prefixes.add(1);
//         }
//         return result;
//     }
// }
