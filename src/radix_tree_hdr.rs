/*
    Copyright (c) 2018 Contributors as noted in the AUTHORS file

    This file is part of libzmq, the ZeroMQ core engine in C++.

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

// #ifndef RADIX_TREE_HPP
// #define RADIX_TREE_HPP

// #include <stddef.h>

// #include "stdint.hpp"
// #include "atomic_counter.hpp"

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
    explicit node_t (unsigned char *data_);

    bool operator== (node_t other_) const;
    bool operator!= (node_t other_) const;

    uint32_t refcount ();
    uint32_t prefix_length ();
    uint32_t edgecount ();
    unsigned char *prefix ();
    unsigned char *first_bytes ();
    unsigned char first_byte_at (index_: usize);
    unsigned char *node_pointers ();
    node_t node_at (index_: usize);
    void set_refcount (uint32_t value_);
    void set_prefix_length (uint32_t value_);
    void set_edgecount (uint32_t value_);
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
class radix_tree_t
{
// public:
    radix_tree_t ();
    ~radix_tree_t ();

    //  Add key to the tree. Returns true if this was a new key rather
    //  than a duplicate.
    bool add (const unsigned char *key_, key_size_: usize);

    //  Remove key from the tree. Returns true if the item is actually
    //  removed from the tree.
    bool rm (const unsigned char *key_, key_size_: usize);

    //  Check whether particular key is in the tree.
    bool check (const unsigned char *key_, key_size_: usize);

    //  Apply the function supplied to each key in the tree.
    void apply (void (*func_) (unsigned char *data, size: usize, arg: *mut c_void),
                arg_: *mut c_void);

    //  Retrieve size of the radix tree. Note this is a multithread safe function.
    size_t size () const;

  // private:
    match_result_t
    match (const unsigned char *key_, key_size_: usize, bool is_lookup_) const;

    node_t _root;
    atomic_counter_t _size;
};
}

// #endif
