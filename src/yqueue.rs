use std::collections::VecDeque;
// use std::ffi::c_void;
// use std::ptr::{null, null_mut};
//
// #[derive(Default, Debug, Clone)]
// pub struct Chunk<'a, T, const N: usize> {
//     pub values: [T; N],
//     pub prev: &'a mut Chunk<'a, T, N>, // really chunk_t
//     pub next: &'a mut Chunk<'a, T, N>, // really chunk_t
// }
//
// pub struct YQueue<'a, T: Clone + PartialEq, const N: usize> {
//     pub _begin_chunk: Chunk<'a, T, N>,
//     pub _begin_pos: usize,
//     pub _back_chunk: Chunk<'a, T, N>,
//     pub _back_pos: usize,
//     pub _end_chunk: Chunk<'a, T, N>,
//     pub _end_pos: usize,
//     pub _spare_chunk: Chunk<'a, T, N>,
// }
//
// impl<T: Clone + PartialEq, const N: usize> YQueue<T, N> {
//     pub unsafe fn new() -> Self {
//         let mut out = Self {
//             _begin_chunk: Chunk::default(),
//             _begin_pos: 0,
//             _back_chunk: Chunk::default(),
//             _back_pos: 0,
//             _end_chunk: Chunk::default(),
//             _end_pos: 0,
//             _spare_chunk: Chunk::default(),
//         };
//         // out._end_chunk = out._begin_chunk;
//         out
//     }
//
//     pub unsafe fn front(&mut self) -> &T {
//         &self._begin_chunk.values[self._begin_pos]
//     }
//
//     pub unsafe fn front_mut(&mut self) -> &mut T {
//         &mut self._begin_chunk.values[self._begin_pos]
//     }
//
//     pub unsafe fn back(&mut self) -> &T {
//         &self._back_chunk.values[self._back_pos]
//     }
//
//     pub unsafe fn back_mut(&mut self) -> &mut T {
//         &mut self._back_chunk.values[self._back_pos]
//     }
//
//     pub fn set_back(&mut self, value_: &mut T) {
//         self._back_chunk.values[self._back_pos] = value_.clone();
//     }
//
//     pub unsafe fn push(&mut self) {
//         self._back_chunk = self._end_chunk;
//         self._back_pos = self._end_pos;
//
//         self._end_pos += 1;
//         if self._end_pos != N {
//             return;
//         }
//
//         let sc: *mut Chunk<T, N> = self._spare_chunk;
//         if sc != null_mut() {
//             (*self._end_chunk).next = sc as *mut c_void;
//             (*sc).prev = self._end_chunk as *mut c_void;
//         } else {
//             (*self._end_chunk).next = Self::allocate_chunk() as *mut c_void;
//             (*((*self._end_chunk).next as *mut Chunk<T, N>)).prev = self._end_chunk as *mut c_void;
//         }
//
//         (*self)._end_chunk = (*self._end_chunk).next as *mut Chunk<T, N>;
//         self._end_pos = 0;
//     }
//
//     pub unsafe fn unpush(&mut self) {
//         if self._back_pos != 0 {
//             self._back_pos -= 1;
//         } else {
//             self._back_pos = N - 1;
//             self._back_chunk = (*self._back_chunk).prev as *mut Chunk<T, N>;
//         }
//
//         if self._end_pos != 0usize {
//             self._end_pos -= 1;
//         } else {
//             self._end_pos = N - 1;
//             self._end_chunk = (*self._end_chunk).prev as *mut Chunk<T, N>;
//             // TODO:
//             libc::free((*self._end_chunk).next);
//             (*self._end_chunk).next = null_mut();
//         }
//     }
//
//     pub unsafe fn pop(&mut self) {
//         self._begin_pos += 1;
//         if self._begin_pos == N {
//             let mut o: *mut Chunk<T, N> = self._begin_chunk;
//             self._begin_chunk = (*self._begin_chunk).next as *mut Chunk<T, N>;
//             (*self._begin_chunk).prev = null_mut();
//             self._begin_pos = 0;
//             let cs: *mut Chunk<T, N> = self._spare_chunk;
//             libc::free(cs as *mut c_void);
//         }
//     }
//
//     pub unsafe fn allocate_chunk() -> *mut Chunk<T, N> {
//         let out: *mut Chunk<T, N> =
//             libc::malloc(std::mem::size_of::<Chunk<T, N>>()) as *mut Chunk<T, N>;
//         (*out).prev = null_mut();
//         (*out).next = null_mut();
//         out
//     }
// }

pub type YQueue<T> = VecDeque<T>;
