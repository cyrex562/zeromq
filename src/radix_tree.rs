/*
    Copyright (c) 2018 Contributors as noted in the AUTHORS file

    This file is part of libzmq, the ZeroMQ core engine in C+= 1.

    libzmq is free software; you can redistribute it and/or modify it under
    the terms of the GNU Lesser General Public License (LGPL) as published
    by the Free Software Foundation; either version 3 of the License, or
    (at your option) any later version.

    As a special exception, the Contributors give you permission to link
    this library with independent modules to produce an executable,
    regardless of the license terms of these independent modules, and to
    copy and distribute the resulting executable under terms of your choice,
    provided that you also meet, for each linked independent module, the
    terms and conditions of the license of that module. An independent
    module is a module which is not derived from or based on this library.
    If you modify this library, you must extend this exception to your
    version of the library.

    libzmq is distributed in the hope that it will be useful, but WITHOUT
    ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
    FITNESS FOR A PARTICULAR PURPOSE. See the GNU Lesser General Public
    License for more details.

    You should have received a copy of the GNU Lesser General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

// #include "precompiled.hpp"
// #include "macros.hpp"
// #include "err.hpp"
// #include "radix_tree.hpp"

// #include <stdlib.h>
// #include <string.h>
// #include <iterator>
// #include <vector>


// Wrapper type for a node's data layout.
//
// There are 3 32-bit unsigned integers that act as a header. These
// integers represent the following values in this order:
//
// (1) The reference count of the key held by the node. This is 0 if
// the node doesn't hold a key.
//
// (2) The number of characters in the node's prefix. The prefix is a
// part of one or more keys in the tree, e.g. the prefix of each node
// in a trie consists of a single character.
//
// (3) The number of outgoing edges from this node.
//
// The rest of the layout consists of 3 chunks in this order:
//
// (1) The node's prefix as a sequence of one or more bytes. The root
// node always has an empty prefix, unlike other nodes in the tree.
//
// (2) The first byte of the prefix of each of this node's children.
//
// (3) The pointer to each child node.
//
// The link to each child is looked up using its index, e.g. the child
// with index 0 will have its first byte and node pointer at the start
// of the chunk of first bytes and node pointers respectively.
struct node_t
{
    explicit node_t (unsigned char *data);

    bool operator== (node_t other_) const;
    bool operator!= (node_t other_) const;

    u32 refcount ();
    u32 prefix_length ();
    u32 edgecount ();
    unsigned char *prefix ();
    unsigned char *first_bytes ();
    unsigned char first_byte_at (index_: usize);
    unsigned char *node_pointers ();
    node_t node_at (index_: usize);
    void set_refcount (u32 value_);
    void set_prefix_length (u32 value_);
    void set_edgecount (u32 value_);
    void set_prefix (const unsigned char *bytes_);
    void set_first_bytes (const unsigned char *bytes_);
    void set_first_byte_at (index_: usize, unsigned char byte_);
    void set_node_pointers (const unsigned char *pointers_);
    void set_node_at (index_: usize, node_t node_);
    void set_edge_at (index_: usize, unsigned char first_byte_, node_t node_);
    void resize (prefix_length_: usize, edgecount_: usize);

    unsigned char *_data;
};

node_t make_node (refcount_: usize, prefix_length_: usize, edgecount_: usize);

struct match_result_t
{
    match_result_t (key_bytes_matched_: usize,
                    prefix_bytes_matched_: usize,
                    edge_index_: usize,
                    parent_edge_index_: usize,
                    node_t current_,
                    node_t parent_,
                    node_t grandparent);

    _key_bytes_matched: usize;
    _prefix_bytes_matched: usize;
    _edge_index: usize;
    _parent_edge_index: usize;
    node_t _current_node;
    node_t _parent_node;
    node_t _grandparent_node;
};

namespace zmq
{
pub struct radix_tree_t
{
//
    radix_tree_t ();
    ~radix_tree_t ();

    //  Add key to the tree. Returns true if this was a new key rather
    //  than a duplicate.
    bool add (const key_: &mut [u8], key_size_: usize);

    //  Remove key from the tree. Returns true if the item is actually
    //  removed from the tree.
    bool rm (const key_: &mut [u8], key_size_: usize);

    //  Check whether particular key is in the tree.
    bool check (const key_: &mut [u8], key_size_: usize);

    //  Apply the function supplied to each key in the tree.
    void apply (void (*func_) (data: &mut [u8], size: usize, arg: *mut c_void),
                arg_: &mut [u8]);

    //  Retrieve size of the radix tree. Note this is a multithread safe function.
    size_t size () const;

  //
    match_result_t
    match (const key_: &mut [u8], key_size_: usize, is_lookup_: bool) const;

    node_t _root;
    AtomicCounter _size;
};

node_t::node_t (unsigned char *data) : _data (data)
{
}

u32 node_t::refcount ()
{
    u32 u32;
    memcpy (&u32, _data, mem::size_of::<u32>());
    return u32;
}

void node_t::set_refcount (u32 value_)
{
    memcpy (_data, &value_, mem::size_of::<value_>());
}

u32 node_t::prefix_length ()
{
    u32 u32;
    memcpy (&u32, _data + mem::size_of::<u32>(), mem::size_of::<u32>());
    return u32;
}

void node_t::set_prefix_length (u32 value_)
{
    memcpy (_data + mem::size_of::<value_>(), &value_, mem::size_of::<value_>());
}

u32 node_t::edgecount ()
{
    u32 u32;
    memcpy (&u32, _data + 2 * mem::size_of::<u32>(), mem::size_of::<u32>());
    return u32;
}

void node_t::set_edgecount (u32 value_)
{
    memcpy (_data + 2 * mem::size_of::<value_>(), &value_, mem::size_of::<value_>());
}

unsigned char *node_t::prefix ()
{
    return _data + 3 * mem::size_of::<u32>();
}

void node_t::set_prefix (const unsigned char *bytes_)
{
    memcpy (prefix (), bytes_, prefix_length ());
}

unsigned char *node_t::first_bytes ()
{
    return prefix () + prefix_length ();
}

void node_t::set_first_bytes (const unsigned char *bytes_)
{
    memcpy (first_bytes (), bytes_, edgecount ());
}

unsigned char node_t::first_byte_at (index_: usize)
{
    // zmq_assert (index_ < edgecount ());
    return first_bytes ()[index_];
}

void node_t::set_first_byte_at (index_: usize, unsigned char byte_)
{
    // zmq_assert (index_ < edgecount ());
    first_bytes ()[index_] = byte_;
}

unsigned char *node_t::node_pointers ()
{
    return prefix () + prefix_length () + edgecount ();
}

void node_t::set_node_pointers (const unsigned char *pointers_)
{
    memcpy (node_pointers (), pointers_, edgecount () * sizeof (void *));
}

node_t node_t::node_at (index_: usize)
{
    // zmq_assert (index_ < edgecount ());

    unsigned char *data;
    memcpy (&data, node_pointers () + index_ * sizeof (void *), mem::size_of::<data>());
    return node_t (data);
}

void node_t::set_node_at (index_: usize, node_t node_)
{
    // zmq_assert (index_ < edgecount ());
    memcpy (node_pointers () + index_ * sizeof (void *), &node_._data,
            sizeof (node_._data));
}

void node_t::set_edge_at (index_: usize,
                          unsigned char first_byte_,
                          node_t node_)
{
    set_first_byte_at (index_, first_byte_);
    set_node_at (index_, node_);
}

bool node_t::operator== (node_t other_) const
{
    return _data == other_._data;
}

bool node_t::operator!= (node_t other_) const
{
    return !(*this == other_);
}

void node_t::resize (prefix_length_: usize, edgecount_: usize)
{
    const size_t node_size = 3 * mem::size_of::<u32>() + prefix_length_
                             + edgecount_ * (1 + sizeof (void *));
    unsigned char *new_data =
       (realloc (_data, node_size));
    // zmq_assert (new_data);
    _data = new_data;
    set_prefix_length ( (prefix_length_));
    set_edgecount ( (edgecount_));
}

node_t make_node (refcount_: usize, prefix_length_: usize, edgecount_: usize)
{
    const size_t node_size = 3 * mem::size_of::<u32>() + prefix_length_
                             + edgecount_ * (1 + sizeof (void *));

    unsigned char *data =  (malloc (node_size));
    // zmq_assert (data);

    node_t node (data);
    node.set_refcount ( (refcount_));
    node.set_prefix_length ( (prefix_length_));
    node.set_edgecount ( (edgecount_));
    return node;
}

// ----------------------------------------------------------------------

radix_tree_t::radix_tree_t () : _root (make_node (0, 0, 0)), _size (0)
{
}

static void free_nodes (node_t node_)
{
    for (size_t i = 0, count = node_.edgecount (); i < count; += 1i)
        free_nodes (node_.node_at (i));
    free (node_._data);
}

radix_tree_t::~radix_tree_t ()
{
    free_nodes (_root);
}

match_result_t::match_result_t (key_bytes_matched_: usize,
                                prefix_bytes_matched_: usize,
                                edge_index_: usize,
                                parent_edge_index_: usize,
                                node_t current_,
                                node_t parent_,
                                node_t grandparent_) :
    _key_bytes_matched (key_bytes_matched_),
    _prefix_bytes_matched (prefix_bytes_matched_),
    _edge_index (edge_index_),
    _parent_edge_index (parent_edge_index_),
    _current_node (current_),
    _parent_node (parent_),
    _grandparent_node (grandparent_)
{
}

match_result_t radix_tree_t::match (const key_: &mut [u8],
                                         key_size_: usize,
                                         bool is_lookup_ = false) const
{
    // zmq_assert (key_);

    // Node we're currently at in the traversal and its predecessors.
    node_t current_node = _root;
    node_t parent_node = current_node;
    node_t grandparent_node = current_node;
    // Index of the next byte to match in the key.
    size_t key_byte_index = 0;
    // Index of the next byte to match in the current node's prefix.
    size_t prefix_byte_index = 0;
    // Index of the edge from parent to current node.
    size_t edge_index = 0;
    // Index of the edge from grandparent to parent.
    size_t parent_edge_index = 0;

    while (current_node.prefix_length () > 0 || current_node.edgecount () > 0) {
        const unsigned char *const prefix = current_node.prefix ();
        const size_t prefix_length = current_node.prefix_length ();

        for (prefix_byte_index = 0;
             prefix_byte_index < prefix_length && key_byte_index < key_size_;
             += 1prefix_byte_index, += 1key_byte_index) {
            if (prefix[prefix_byte_index] != key_[key_byte_index])
                break;
        }

        // Even if a prefix of the key matches and we're doing a
        // lookup, this means we've found a matching subscription.
        if (is_lookup_ && prefix_byte_index == prefix_length
            && current_node.refcount () > 0) {
            key_byte_index = key_size_;
            break;
        }

        // There was a mismatch or we've matched the whole key, so
        // there's nothing more to do.
        if (prefix_byte_index != prefix_length || key_byte_index == key_size_)
            break;

        // We need to match the rest of the key. Check if there's an
        // outgoing edge from this node.
        node_t next_node = current_node;
        for (size_t i = 0, edgecount = current_node.edgecount (); i < edgecount;
             += 1i) {
            if (current_node.first_byte_at (i) == key_[key_byte_index]) {
                parent_edge_index = edge_index;
                edge_index = i;
                next_node = current_node.node_at (i);
                break;
            }
        }

        if (next_node == current_node)
            break; // No outgoing edge.
        grandparent_node = parent_node;
        parent_node = current_node;
        current_node = next_node;
    }

    return match_result_t (key_byte_index, prefix_byte_index, edge_index,
                           parent_edge_index, current_node, parent_node,
                           grandparent_node);
}

bool radix_tree_t::add (const key_: &mut [u8], key_size_: usize)
{
    const match_result_t match_result = match (key_, key_size_);
    const size_t key_bytes_matched = match_result._key_bytes_matched;
    const size_t prefix_bytes_matched = match_result._prefix_bytes_matched;
    const size_t edge_index = match_result._edge_index;
    node_t current_node = match_result._current_node;
    node_t parent_node = match_result._parent_node;

    if (key_bytes_matched != key_size_) {
        // Not all characters match, we might have to split the node.
        if (prefix_bytes_matched == current_node.prefix_length ()) {
            // The mismatch is at one of the outgoing edges, so we
            // create an edge from the current node to a new leaf node
            // that has the rest of the key as the prefix.
            node_t key_node = make_node (1, key_size_ - key_bytes_matched, 0);
            key_node.set_prefix (key_ + key_bytes_matched);

            // Reallocate for one more edge.
            current_node.resize (current_node.prefix_length (),
                                 current_node.edgecount () + 1);

            // Make room for the new edge. We need to shift the chunk
            // of node pointers one byte to the right. Since resize()
            // increments the edgecount by 1, node_pointers() tells us the
            // destination address. The chunk of node pointers starts
            // at one byte to the left of this destination.
            //
            // Since the regions can overlap, we use memmove.
            memmove (current_node.node_pointers (),
                     current_node.node_pointers () - 1,
                     (current_node.edgecount () - 1) * sizeof (void *));

            // Add an edge to the new node.
            current_node.set_edge_at (current_node.edgecount () - 1,
                                      key_[key_bytes_matched], key_node);

            // We need to update all pointers to the current node
            // after the call to resize().
            if (current_node.prefix_length () == 0)
                _root._data = current_node._data;
            else
                parent_node.set_node_at (edge_index, current_node);
            _size.add (1);
            return true;
        }

        // There was a mismatch, so we need to split this node.
        //
        // Create two nodes that will be reachable from the parent.
        // One node will have the rest of the characters from the key,
        // and the other node will have the rest of the characters
        // from the current node's prefix.
        node_t key_node = make_node (1, key_size_ - key_bytes_matched, 0);
        node_t split_node =
          make_node (current_node.refcount (),
                     current_node.prefix_length () - prefix_bytes_matched,
                     current_node.edgecount ());

        // Copy the prefix chunks to the new nodes.
        key_node.set_prefix (key_ + key_bytes_matched);
        split_node.set_prefix (current_node.prefix () + prefix_bytes_matched);

        // Copy the current node's edges to the new node.
        split_node.set_first_bytes (current_node.first_bytes ());
        split_node.set_node_pointers (current_node.node_pointers ());

        // Resize the current node to accommodate a prefix comprising
        // the matched characters and 2 outgoing edges to the above
        // nodes. Set the refcount to 0 since this node doesn't hold a
        // key.
        current_node.resize (prefix_bytes_matched, 2);
        current_node.set_refcount (0);

        // Add links to the new nodes. We don't need to copy the
        // prefix since resize() retains it in the current node.
        current_node.set_edge_at (0, key_node.prefix ()[0], key_node);
        current_node.set_edge_at (1, split_node.prefix ()[0], split_node);

        _size.add (1);
        parent_node.set_node_at (edge_index, current_node);
        return true;
    }

    // All characters in the key match, but we still might need to split.
    if (prefix_bytes_matched != current_node.prefix_length ()) {
        // All characters in the key match, but not all characters
        // from the current node's prefix match.

        // Create a node that contains the rest of the characters from
        // the current node's prefix and the outgoing edges from the
        // current node.
        node_t split_node =
          make_node (current_node.refcount (),
                     current_node.prefix_length () - prefix_bytes_matched,
                     current_node.edgecount ());
        split_node.set_prefix (current_node.prefix () + prefix_bytes_matched);
        split_node.set_first_bytes (current_node.first_bytes ());
        split_node.set_node_pointers (current_node.node_pointers ());

        // Resize the current node to hold only the matched characters
        // from its prefix and one edge to the new node.
        current_node.resize (prefix_bytes_matched, 1);

        // Add an edge to the split node and set the refcount to 1
        // since this key wasn't inserted earlier. We don't need to
        // set the prefix because the first `prefix_bytes_matched` bytes
        // in the prefix are preserved by resize().
        current_node.set_edge_at (0, split_node.prefix ()[0], split_node);
        current_node.set_refcount (1);

        _size.add (1);
        parent_node.set_node_at (edge_index, current_node);
        return true;
    }

    // zmq_assert (key_bytes_matched == key_size_);
    // zmq_assert (prefix_bytes_matched == current_node.prefix_length ());

    _size.add (1);
    current_node.set_refcount (current_node.refcount () + 1);
    return current_node.refcount () == 1;
}

bool radix_tree_t::rm (const key_: &mut [u8], key_size_: usize)
{
    const match_result_t match_result = match (key_, key_size_);
    const size_t key_bytes_matched = match_result._key_bytes_matched;
    const size_t prefix_bytes_matched = match_result._prefix_bytes_matched;
    const size_t edge_index = match_result._edge_index;
    const size_t parent_edge_index = match_result._parent_edge_index;
    node_t current_node = match_result._current_node;
    node_t parent_node = match_result._parent_node;
    node_t grandparent_node = match_result._grandparent_node;

    if (key_bytes_matched != key_size_
        || prefix_bytes_matched != current_node.prefix_length ()
        || current_node.refcount () == 0)
        return false;

    current_node.set_refcount (current_node.refcount () - 1);
    _size.sub (1);
    if (current_node.refcount () > 0)
        return false;

    // Don't delete the root node.
    if (current_node == _root)
        return true;

    const size_t outgoing_edges = current_node.edgecount ();
    if (outgoing_edges > 1)
        // This node can't be merged with any other node, so there's
        // nothing more to do.
        return true;

    if (outgoing_edges == 1) {
        // Merge this node with the single child node.
        node_t child = current_node.node_at (0);

        // Make room for the child node's prefix and edges. We need to
        // keep the old prefix length since resize() will overwrite
        // it.
        const u32 old_prefix_length = current_node.prefix_length ();
        current_node.resize (old_prefix_length + child.prefix_length (),
                             child.edgecount ());

        // Append the child node's prefix to the current node.
        memcpy (current_node.prefix () + old_prefix_length, child.prefix (),
                child.prefix_length ());

        // Copy the rest of child node's data to the current node.
        current_node.set_first_bytes (child.first_bytes ());
        current_node.set_node_pointers (child.node_pointers ());
        current_node.set_refcount (child.refcount ());

        free (child._data);
        parent_node.set_node_at (edge_index, current_node);
        return true;
    }

    if (parent_node.edgecount () == 2 && parent_node.refcount () == 0
        && parent_node != _root) {
        // Removing this node leaves the parent with one child.
        // If the parent doesn't hold a key or if it isn't the root,
        // we can merge it with its single child node.
        // zmq_assert (edge_index < 2);
        node_t other_child = parent_node.node_at (!edge_index);

        // Make room for the child node's prefix and edges. We need to
        // keep the old prefix length since resize() will overwrite
        // it.
        const u32 old_prefix_length = parent_node.prefix_length ();
        parent_node.resize (old_prefix_length + other_child.prefix_length (),
                            other_child.edgecount ());

        // Append the child node's prefix to the current node.
        memcpy (parent_node.prefix () + old_prefix_length,
                other_child.prefix (), other_child.prefix_length ());

        // Copy the rest of child node's data to the current node.
        parent_node.set_first_bytes (other_child.first_bytes ());
        parent_node.set_node_pointers (other_child.node_pointers ());
        parent_node.set_refcount (other_child.refcount ());

        free (current_node._data);
        free (other_child._data);
        grandparent_node.set_node_at (parent_edge_index, parent_node);
        return true;
    }

    // This is a leaf node that doesn't leave its parent with one
    // outgoing edge. Remove the outgoing edge to this node from the
    // parent.
    // zmq_assert (outgoing_edges == 0);

    // Replace the edge to the current node with the last edge. An
    // edge consists of a byte and a pointer to the next node. First
    // replace the byte.
    const size_t last_index = parent_node.edgecount () - 1;
    const unsigned char last_byte = parent_node.first_byte_at (last_index);
    const node_t last_node = parent_node.node_at (last_index);
    parent_node.set_edge_at (edge_index, last_byte, last_node);

    // Move the chunk of pointers one byte to the left, effectively
    // deleting the last byte in the region of first bytes by
    // overwriting it.
    memmove (parent_node.node_pointers () - 1, parent_node.node_pointers (),
             parent_node.edgecount () * sizeof (void *));

    // Shrink the parent node to the new size, which "deletes" the
    // last pointer in the chunk of node pointers.
    parent_node.resize (parent_node.prefix_length (),
                        parent_node.edgecount () - 1);

    // Nothing points to this node now, so we can reclaim it.
    free (current_node._data);

    if (parent_node.prefix_length () == 0)
        _root._data = parent_node._data;
    else
        grandparent_node.set_node_at (parent_edge_index, parent_node);
    return true;
}

bool radix_tree_t::check (const key_: &mut [u8], key_size_: usize)
{
    if (_root.refcount () > 0)
        return true;

    match_result_t match_result = match (key_, key_size_, true);
    return match_result._key_bytes_matched == key_size_
           && match_result._prefix_bytes_matched
                == match_result._current_node.prefix_length ()
           && match_result._current_node.refcount () > 0;
}

static void
visit_keys (node_t node_,
            std::vector<unsigned char> &buffer_,
            void (*func_) (data: &mut [u8], size: usize, arg_: &mut [u8]),
            arg_: &mut [u8])
{
    const size_t prefix_length = node_.prefix_length ();
    buffer_.reserve (buffer_.size () + prefix_length);
    std::copy (node_.prefix (), node_.prefix () + prefix_length,
               std::back_inserter (buffer_));

    if (node_.refcount () > 0) {
        // zmq_assert (!buffer_.empty ());
        func_ (&buffer_[0], buffer_.size (), arg_);
    }

    for (size_t i = 0, edgecount = node_.edgecount (); i < edgecount; += 1i) {
        visit_keys (node_.node_at (i), buffer_, func_, arg_);
    }
    buffer_.resize ( (buffer_.size () - prefix_length));
}

void radix_tree_t::apply (
  void (*func_) (data: &mut [u8], size: usize, arg_: &mut [u8]), arg_: &mut [u8])
{
    if (_root.refcount () > 0)
        func_ (null_mut(), 0, arg_); // Root node is always empty.

    std::vector<unsigned char> buffer;
    for (size_t i = 0; i < _root.edgecount (); += 1i)
        visit_keys (_root.node_at (i), buffer, func_, arg_);
}

size_t radix_tree_t::size () const
{
    return _size.get ();
}
