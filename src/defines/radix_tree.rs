// use std::ffi::c_void;
// use std::mem::size_of;

// use crate::defines::atomic_counter::ZmqAtomicCounter;

// pub struct ZmqNode {
//     pub _data: *mut u8,
// }

// impl ZmqNode {
//     pub fn new(data_: *mut u8) -> ZmqNode {
//         ZmqNode { _data: data_ }
//     }
//
//     pub unsafe fn refcount(&mut self) -> u32 {
//         let mut x = 0u32;
//         let mut x_bytes = x.to_le_bytes();
//         libc::memcpy(
//             x_bytes.as_mut_ptr() as *mut c_void,
//             self._data as *const c_void,
//             std::mem::size_of::<u32>(),
//         );
//         x = u32::from_le_bytes(x_bytes);
//         x
//     }
//
//     pub unsafe fn set_refcount(&mut self, x: u32) {
//         let mut x_bytes = x.to_le_bytes();
//         libc::memcpy(
//             self._data as *mut c_void,
//             x_bytes.as_mut_ptr() as *const c_void,
//             std::mem::size_of::<u32>(),
//         );
//     }
//
//     pub unsafe fn prefix_len(&mut self) -> u32 {
//         let mut x = 0u32;
//         let mut x_bytes = x.to_le_bytes();
//         libc::memcpy(
//             x_bytes.as_mut_ptr() as *mut c_void,
//             (self._data as *const c_void).offset(std::mem::size_of::<u32>() as isize),
//             std::mem::size_of::<u32>(),
//         );
//         x = u32::from_le_bytes(x_bytes);
//         x
//     }
//
//     pub unsafe fn set_prefix_len(&mut self, x: u32) {
//         let mut x_bytes = x.to_le_bytes();
//         libc::memcpy(
//             (self._data as *mut c_void).offset(std::mem::size_of::<u32>() as isize),
//             x_bytes.as_mut_ptr() as *const c_void,
//             std::mem::size_of::<u32>(),
//         );
//     }
//
//     pub unsafe fn edgecount(&mut self) -> u32 {
//         let mut x = 0u32;
//         let mut x_bytes = x.to_le_bytes();
//         libc::memcpy(
//             x_bytes.as_mut_ptr() as *mut c_void,
//             (self._data as *const c_void).offset(std::mem::size_of::<u32>() as isize * 2),
//             std::mem::size_of::<u32>(),
//         );
//         x = u32::from_le_bytes(x_bytes);
//         x
//     }
//
//     pub unsafe fn set_edgecount(&mut self, value_: u32) {
//         let mut x_bytes = value_.to_le_bytes();
//         libc::memcpy(
//             (self._data as *mut c_void).offset(std::mem::size_of::<u32>() as isize * 2),
//             x_bytes.as_mut_ptr() as *const c_void,
//             std::mem::size_of::<u32>(),
//         );
//     }
//
//     pub unsafe fn prefix(&mut self) -> *mut u8 {
//         (self._data as *mut c_void).offset(std::mem::size_of::<u32>() as isize * 3) as *mut u8
//     }
//
//     pub unsafe fn set_prefix(&mut self, value_: *const u8) {
//         libc::memcpy(
//             (self._data as *mut c_void).offset(std::mem::size_of::<u32>() as isize * 3),
//             value_.to_le_bytes().as_mut_ptr() as *const c_void,
//             std::mem::size_of::<*mut u8>(),
//         );
//     }
//
//     pub unsafe fn first_bytes(&mut self) -> *mut u8 {
//         (self._data as *mut c_void)
//             .offset(std::mem::size_of::<u32>() as isize * 3 + self.prefix_len() as isize)
//             as *mut u8
//     }
//
//     pub unsafe fn set_first_bytes(&mut self, value_: *mut u8) {
//         libc::memcpy(
//             (self._data as *mut c_void)
//                 .offset(std::mem::size_of::<u32>() as isize * 3 + self.prefix_len() as isize),
//             value_ as *const c_void,
//             std::mem::size_of::<*mut u8>(),
//         );
//     }
//
//     pub unsafe fn first_byte_at(&mut self, index_: usize) -> u8 {
//         let mut x = 0u8;
//         libc::memcpy(
//             x.to_le_bytes().as_mut_ptr() as *mut c_void,
//             (self._data as *const c_void).offset(
//                 std::mem::size_of::<u32>() as isize * 3
//                     + self.prefix_len() as isize
//                     + index_ as isize,
//             ),
//             std::mem::size_of::<u8>(),
//         );
//         x
//     }
//
//     pub unsafe fn set_first_byte_at(&mut self, index_: usize, byte_: u8) {
//         libc::memcpy(
//             (self._data as *mut c_void).offset(
//                 std::mem::size_of::<u32>() as isize * 3
//                     + self.prefix_len() as isize
//                     + index_ as isize,
//             ),
//             byte_.to_le_bytes().as_ptr() as *const c_void,
//             std::mem::size_of::<u8>(),
//         );
//     }
//
//     pub unsafe fn node_pointers(&mut self) -> *mut u8 {
//         self.prefix().offset(
//             self.prefix_len() as isize
//                 + self.edgecount() as isize * std::mem::size_of::<usize>() as isize,
//         ) as *mut u8
//     }
//
//     pub unsafe fn node_at(&mut self, index_: usize) -> ZmqNode {
//         ZmqNode::new(
//             self.node_pointers()
//                 .offset(index_ as isize * std::mem::size_of::<usize>() as isize)
//                 as *mut u8,
//         )
//     }
//
//     pub unsafe fn set_node_at(&mut self, index_: usize, node_: ZmqNode) {
//         libc::memcpy(
//             self.node_pointers()
//                 .offset(index_ as isize * std::mem::size_of::<usize>() as isize)
//                 as *mut c_void,
//             node_._data as *const c_void,
//             std::mem::size_of::<usize>(),
//         );
//     }
//
//     pub unsafe fn set_edge_at(&mut self, index_: usize, first_byte_: u8, node_: ZmqNode) {
//         self.set_first_byte_at(index_, first_byte_);
//         self.set_node_at(index_, node_);
//     }
//
//     pub unsafe fn resize(&mut self, prefix_length_: usize, edgecount_: usize) {
//         let mut node_size = 3 * 4 + prefix_length_ + edgecount_ * (1 + size_of::<*mut c_void>());
//
//         let new_data = libc::realloc(self._data as *mut c_void, node_size);
//         self._data = new_data as *mut u8;
//         self.set_prefix_len(prefix_length_ as u32);
//         self.set_edgecount(edgecount_ as u32);
//     }
// }

// pub unsafe fn make_node(refcount_: usize, prefix_length_: usize, edgecount_: usize) -> ZmqNode {
//     let mut node_size = 3 * 4 + prefix_length_ + edgecount_ * (1 + size_of::<*mut c_void>());
//     let mut data = libc::malloc(node_size);
//     libc::memset(data, 0, node_size);
//     let mut node = ZmqNode::new(data as *mut u8);
//     node.set_refcount(refcount_ as u32);
//     node.set_prefix_len(prefix_length_ as u32);
//     node.set_edgecount(edgecount_ as u32);
//     node
// }

// impl std::cmp::PartialEq for ZmqNode {
//     fn eq(&self, other: &Self) -> bool {
//         self._data == other._data
//     }
//
//     fn ne(&self, other: &Self) -> bool {
//         self._data != other._data
//     }
// }

// pub struct ZmqMatchResult {
//     pub _key_bytes_matched: usize,
//     pub _prefix_bytes_matched: usize,
//     pub _edge_index: usize,
//     pub _parent_edge_index: usize,
//     pub _current_node: ZmqNode,
//     pub _parent_node: ZmqNode,
//     pub _grandparent_node: ZmqNode,
// }

// impl ZmqMatchResult {
//     pub fn new(
//         key_bytes_matched: usize,
//         prefix_bytes_matched: usize,
//         edge_index: usize,
//         parent_edge_index: usize,
//         current_: ZmqNode,
//         parent_: ZmqNode,
//         grandparent_: ZmqNode,
//     ) -> Self {
//         ZmqMatchResult {
//             _key_bytes_matched: key_bytes_matched,
//             _prefix_bytes_matched: prefix_bytes_matched,
//             _edge_index: edge_index,
//             _parent_edge_index: parent_edge_index,
//             _current_node: current_,
//             _parent_node: parent_,
//             _grandparent_node: grandparent_,
//         }
//     }
// }

// pub struct ZmqRadixTree {
//     pub _root: ZmqNode,
//     pub _size: ZmqAtomicCounter,
// }

// impl ZmqRadixTree {
//     pub fn new() -> ZmqRadixTree {
//         ZmqRadixTree {
//             _root: unsafe { make_node(0, 0, 0) },
//             _size: ZmqAtomicCounter::new(0),
//         }
//     }
//
//     pub unsafe fn matches(
//         &mut self,
//         key_: *const u8,
//         key_size_: usize,
//         is_lookup_: bool,
//     ) -> ZmqMatchResult {
//         // Node we're currently at in the traversal and its predecessors.
//         let mut current_node = self._root.clone();
//         let mut parent_node = current_node;
//         let mut grandparent_node = current_node.clone();
//         // Index of the next byte to match in the key.
//         let mut key_byte_index = 0usize;
//         // Index of the next byte to match in the current node's prefix.
//         let mut prefix_byte_index = 0usize;
//         // Index of the edge from parent to current node.
//         let mut edge_index = 0usize;
//         // Index of the edge from grandparent to parent.
//         let mut parent_edge_index = 0usize;
//
//         while current_node.prefix_length() > 0 || current_node.edgecount() > 0 {
//             let mut prefix = current_node.prefix();
//             let mut prefix_length = current_node.prefix_length();
//
//             // for (prefix_byte_index = 0;
//             //      prefix_byte_index < prefix_length && key_byte_index < key_size_;
//             //      ++prefix_byte_index, ++key_byte_index)
//
//             prefix_byte_index = 0;
//             while prefix_byte_index < prefix_length && key_byte_index < key_size_ {
//                 if prefix[prefix_byte_index] != key_[key_byte_index] {
//                     break;
//                 }
//                 prefix_byte_index += 1;
//                 key_byte_index += 1;
//             }
//
//             // Even if a prefix of the key matches and we're doing a
//             // lookup, this means we've found a matching subscription.
//             if is_lookup_ && prefix_byte_index == prefix_length && current_node.refcount() > 0 {
//                 key_byte_index = key_size_;
//                 break;
//             }
//
//             // There was a mismatch or we've matched the whole key, so
//             // there's nothing more to do.
//             if prefix_byte_index != prefix_length || key_byte_index == key_size_ {
//                 break;
//             }
//
//             // We need to match the rest of the key. Check if there's an
//             // outgoing edge from this node.
//             let mut next_node = current_node;
//             // for (size_t i = 0, edgecount = current_node.edgecount (); i < edgecount;
//             //      ++i)
//             for i in 0..current_node.edgecount() {
//                 if current_node.first_byte_at(i) == key_[key_byte_index] {
//                     parent_edge_index = edge_index;
//                     edge_index = i;
//                     next_node = current_node.node_at(i);
//                     break;
//                 }
//             }
//
//             if next_node == current_node {
//                 break; // No outgoing edge.
//             }
//             grandparent_node = parent_node;
//             parent_node = current_node;
//             current_node = next_node;
//         }
//
//         return ZmqMatchResult::new(
//             key_byte_index,
//             prefix_byte_index,
//             edge_index,
//             parent_edge_index,
//             current_node,
//             parent_node,
//             grandparent_node,
//         );
//     }
//
//     pub unsafe fn add(&mut self, key_: *const u8, key_size_: usize) -> bool {
//         let match_result = self.matches(key_, key_size_, false);
//         let key_bytes_matched = match_result._key_bytes_matched;
//         let prefix_bytes_matched = match_result._prefix_bytes_matched;
//         let edge_index = match_result._edge_index;
//         let mut current_node = match_result._current_node;
//         let mut parent_node = match_result._parent_node;
//
//         if key_bytes_matched != key_size_ {
//             // Not all characters match, we might have to split the node.
//             if prefix_bytes_matched == current_node.prefix_length() {
//                 // The mismatch is at one of the outgoing edges, so we
//                 // create an edge from the current node to a new leaf node
//                 // that has the rest of the key as the prefix.
//                 let mut key_node = make_node(1, key_size_ - key_bytes_matched, 0);
//                 key_node.set_prefix(key_.add(key_bytes_matched));
//
//                 // Reallocate for one more edge.
//                 current_node.resize(
//                     current_node.prefix_length(),
//                     (current_node.edgecount() + 1) as usize,
//                 );
//
//                 // Make room for the new edge. We need to shift the chunk
//                 // of node pointers one byte to the right. Since resize()
//                 // increments the edgecount by 1, node_pointers() tells us the
//                 // destination address. The chunk of node pointers starts
//                 // at one byte to the left of this destination.
//                 //
//                 // Since the regions can overlap, we use memmove.
//                 libc::memmove(
//                     current_node.node_pointers() as *mut c_void,
//                     current_node.node_pointers().sub(1) as *mut c_void,
//                     ((current_node.edgecount() - 1) * size_of::<*mut c_void>()) as usize,
//                 );
//
//                 // Add an edge to the new node.
//                 current_node.set_edge_at(
//                     (current_node.edgecount() - 1) as usize,
//                     key_[key_bytes_matched],
//                     key_node,
//                 );
//
//                 // We need to update all pointers to the current node
//                 // after the call to resize().
//                 if current_node.prefix_length() == 0 {
//                     self._root._data = current_node._data;
//                 } else {
//                     parent_node.set_node_at(edge_index, current_node);
//                 }
//                 self._size.add(1);
//                 return true;
//             }
//
//             // There was a mismatch, so we need to split this node.
//             //
//             // Create two nodes that will be reachable from the parent.
//             // One node will have the rest of the characters from the key,
//             // and the other node will have the rest of the characters
//             // from the current node's prefix.
//             let mut key_node = make_node(1, key_size_ - key_bytes_matched, 0);
//             let mut split_node = make_node(
//                 current_node.refcount() as usize,
//                 current_node.prefix_length() - prefix_bytes_matched,
//                 current_node.edgecount() as usize,
//             );
//
//             // Copy the prefix chunks to the new nodes.
//             key_node.set_prefix(key_.add(key_bytes_matched));
//             split_node.set_prefix(current_node.prefix().add(prefix_bytes_matched));
//
//             // Copy the current node's edges to the new node.
//             split_node.set_first_bytes(current_node.first_bytes());
//             split_node.set_node_pointers(current_node.node_pointers());
//
//             // Resize the current node to accommodate a prefix comprising
//             // the matched characters and 2 outgoing edges to the above
//             // nodes. Set the refcount to 0 since this node doesn't hold a
//             // key.
//             current_node.resize(prefix_bytes_matched, 2);
//             current_node.set_refcount(0);
//
//             // Add links to the new nodes. We don't need to copy the
//             // prefix since resize() retains it in the current node.
//             current_node.set_edge_at(0, key_node.prefix()[0], key_node);
//             current_node.set_edge_at(1, split_node.prefix()[0], split_node);
//
//             self._size.add(1);
//             parent_node.set_node_at(edge_index, current_node);
//             return true;
//         }
//
//         // All characters in the key match, but we still might need to split.
//         if prefix_bytes_matched != current_node.prefix_length() {
//             // All characters in the key match, but not all characters
//             // from the current node's prefix match.
//
//             // Create a node that contains the rest of the characters from
//             // the current node's prefix and the outgoing edges from the
//             // current node.
//             let mut split_node = make_node(
//                 current_node.refcount() as usize,
//                 current_node.prefix_length() - prefix_bytes_matched,
//                 current_node.edgecount() as usize,
//             );
//             split_node.set_prefix(current_node.prefix().add(prefix_bytes_matched));
//             split_node.set_first_bytes(current_node.first_bytes());
//             split_node.set_node_pointers(current_node.node_pointers());
//
//             // Resize the current node to hold only the matched characters
//             // from its prefix and one edge to the new node.
//             current_node.resize(prefix_bytes_matched, 1);
//
//             // Add an edge to the split node and set the refcount to 1
//             // since this key wasn't inserted earlier. We don't need to
//             // set the prefix because the first `prefix_bytes_matched` bytes
//             // in the prefix are preserved by resize().
//             current_node.set_edge_at(0, split_node.prefix()[0], split_node);
//             current_node.set_refcount(1);
//
//             self._size.add(1);
//             parent_node.set_node_at(edge_index, current_node);
//             return true;
//         }
//
//         // zmq_assert (key_bytes_matched == key_size_);
//         // zmq_assert (prefix_bytes_matched == current_node.prefix_length ());
//
//         self._size.add(1);
//         current_node.set_refcount(current_node.refcount() + 1);
//         return current_node.refcount() == 1;
//     }
//
//     pub unsafe fn rm(&mut self, key_: *const u8, key_size_: usize) -> bool {
//         let match_result = self.matches(key_, key_size_, false);
//         let key_bytes_matched = match_result._key_bytes_matched;
//         let prefix_bytes_matched = match_result._prefix_bytes_matched;
//         let edge_index = match_result._edge_index;
//         let parent_edge_index = match_result._parent_edge_index;
//         let mut current_node = match_result._current_node;
//         let mut parent_node = match_result._parent_node;
//         let mut grandparent_node = match_result._grandparent_node;
//
//         if key_bytes_matched != key_size_
//             || prefix_bytes_matched != current_node.prefix_length()
//             || current_node.refcount() == 0
//         {
//             return false;
//         }
//
//         current_node.set_refcount(current_node.refcount() - 1);
//         self._size.sub(1);
//         if current_node.refcount() > 0 {
//             return false;
//         }
//
//         // Don't delete the root node.
//         if current_node == self._root {
//             return true;
//         }
//
//         let mut outgoing_edges = current_node.edgecount();
//         if outgoing_edges > 1 {
//             // This node can't be merged with any other node, so there's
//             // nothing more to do.
//             return true;
//         }
//
//         if outgoing_edges == 1 {
//             // Merge this node with the single child node.
//             let mut child = current_node.node_at(0);
//
//             // Make room for the child node's prefix and edges. We need to
//             // keep the old prefix length since resize() will overwrite
//             // it.
//             let old_prefix_length = current_node.prefix_length();
//             current_node.resize(
//                 old_prefix_length + child.prefix_length(),
//                 child.edgecount() as usize,
//             );
//
//             // Append the child node's prefix to the current node.
//             libc::memcpy(
//                 current_node.prefix().add(old_prefix_length) as *mut c_void,
//                 child.prefix() as *const c_void,
//                 child.prefix_length(),
//             );
//
//             // Copy the rest of child node's data to the current node.
//             current_node.set_first_bytes(child.first_bytes());
//             current_node.set_node_pointers(child.node_pointers());
//             current_node.set_refcount(child.refcount());
//
//             // free (child._data);
//             parent_node.set_node_at(edge_index, current_node);
//             return true;
//         }
//
//         if parent_node.edgecount() == 2
//             && parent_node.refcount() == 0
//             && parent_node != self._root
//         {
//             // Removing this node leaves the parent with one child.
//             // If the parent doesn't hold a key or if it isn't the root,
//             // we can merge it with its single child node.
//             // zmq_assert (edge_index < 2);
//             let mut other_child = parent_node.node_at(!edge_index);
//
//             // Make room for the child node's prefix and edges. We need to
//             // keep the old prefix length since resize() will overwrite
//             // it.
//             let old_prefix_length = parent_node.prefix_length();
//             parent_node.resize(
//                 old_prefix_length + other_child.prefix_length(),
//                 other_child.edgecount() as usize,
//             );
//
//             // Append the child node's prefix to the current node.
//             libc::memcpy(
//                 parent_node.prefix() + old_prefix_length,
//                 other_child.prefix() as *const c_void,
//                 other_child.prefix_length(),
//             );
//
//             // Copy the rest of child node's data to the current node.
//             parent_node.set_first_bytes(other_child.first_bytes());
//             parent_node.set_node_pointers(other_child.node_pointers());
//             parent_node.set_refcount(other_child.refcount());
//
//             // free (current_node._data);
//             // free (other_child._data);
//             grandparent_node.set_node_at(parent_edge_index, parent_node);
//             return true;
//         }
//
//         // This is a leaf node that doesn't leave its parent with one
//         // outgoing edge. Remove the outgoing edge to this node from the
//         // parent.
//         // zmq_assert (outgoing_edges == 0);
//
//         // Replace the edge to the current node with the last edge. An
//         // edge consists of a byte and a pointer to the next node. First
//         // replace the byte.
//         let last_index = parent_node.edgecount() - 1;
//         let last_byte = parent_node.first_byte_at(last_index as usize);
//         let last_node = parent_node.node_at(last_index as usize);
//         parent_node.set_edge_at(edge_index, last_byte, last_node);
//
//         // Move the chunk of pointers one byte to the left, effectively
//         // deleting the last byte in the region of first bytes by
//         // overwriting it.
//         libc::memmove(
//             parent_node.node_pointers().sub(1) as *mut c_void,
//             parent_node.node_pointers() as *const c_void,
//             (parent_node.edgecount() * size_of::<*mut c_void>()) as usize,
//         );
//
//         // Shrink the parent node to the new size, which "deletes" the
//         // last pointer in the chunk of node pointers.
//         parent_node.resize(
//             parent_node.prefix_length(),
//             (parent_node.edgecount() - 1) as usize,
//         );
//
//         // Nothing points to this node now, so we can reclaim it.
//         // free (current_node._data);
//
//         if parent_node.prefix_length() == 0 {
//             self._root._data = parent_node._data;
//         } else {
//             grandparent_node.set_node_at(parent_edge_index, parent_node);
//         }
//         return true;
//     }
//
//     pub unsafe fn check(&mut self, key_: *const u8, key_size_: usize) -> bool {
//         if self._root.refcount() > 0 {
//             return true;
//         }
//
//         let mut match_result = self.matches(key_, key_size_, true);
//         return match_result._key_bytes_matched == key_size_
//             && match_result._prefix_bytes_matched == match_result._current_node.prefix_length()
//             && match_result._current_node.refcount() > 0;
//     }
//
//     pub unsafe fn apply(&mut self, visitor: fn(*mut u8, usize, *mut c_void), arg_: *mut c_void) {
//         let mut buffer = Vec::new();
//         visit_keys(&mut self._root, &mut buffer, visitor, arg_);
//     }
// }

// pub unsafe fn visit_keys(
//     node_: &mut ZmqNode,
//     buffer_: &mut Vec<u8>,
//     visitor: fn(*mut u8, usize, *mut c_void),
//     arg: *mut c_void,
// ) {
//     let prefix_length = node_.prefix_length();
//     buffer_.reserve(buffer_.size() + prefix_length);
//     // std::copy (node_.prefix (), node_.prefix () + prefix_length,
//     //            std::back_inserter (buffer_));
//
//     if node_.refcount() > 0 {
//         // zmq_assert (!buffer_.empty ());
//         visitor(buffer_.as_mut_ptr(), buffer_.size(), arg);
//     }
//
//     // for (size_t i = 0, edgecount = node_.edgecount (); i < edgecount; ++i)
//     for i in 0..node_.edgecount() {
//         visit_keys(&mut node_.node_at(i as usize), buffer_, visitor, arg);
//     }
//     buffer_.resize((buffer_.size() - prefix_length), 0);
// }
