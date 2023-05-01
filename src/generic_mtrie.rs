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

// #ifndef __ZMQ_GENERIC_MTRIE_HPP_INCLUDED__
// #define __ZMQ_GENERIC_MTRIE_HPP_INCLUDED__

// #include <stddef.h>
// #include <set>

// #include "macros.hpp"
// #include "stdint.hpp"
// #include "atomic_counter.hpp"

namespace zmq
{

template <typename T>
generic_mtrie_t<T>::generic_mtrie_t () :
    pipes (0), _num_prefixes (0), _min (0), _count (0), _live_nodes (0)
{
}

template <typename T> generic_mtrie_t<T>::~generic_mtrie_t ()
{
    LIBZMQ_DELETE (pipes);

    if (_count == 1) {
        zmq_assert (next.node);
        LIBZMQ_DELETE (next.node);
    } else if (_count > 1) {
        for (unsigned short i = 0; i != _count; += 1i) {
            LIBZMQ_DELETE (next.table[i]);
        }
        free (next.table);
    }
}

template <typename T>
bool generic_mtrie_t<T>::add (prefix_t prefix_, size: usize, value_t *pipe)
{
    generic_mtrie_t<value_t> *it = this;

    while (size) {
        const unsigned char c = *prefix_;

        if (c < it._min || c >= it._min + it._count) {
            //  The character is out of range of currently handled
            //  characters. We have to extend the table.
            if (!it._count) {
                it._min = c;
                it._count = 1;
                it.next.node = null_mut();
            } else if (it._count == 1) {
                const unsigned char oldc = it._min;
                generic_mtrie_t *oldp = it.next.node;
                it._count = (it._min < c ? c - it._min : it._min - c) + 1;
                it.next.table = static_cast<generic_mtrie_t **> (
                  malloc (sizeof (generic_mtrie_t *) * it._count));
                alloc_assert (it.next.table);
                for (unsigned short i = 0; i != it._count; += 1i)
                    it.next.table[i] = 0;
                it._min = std::min (it._min, c);
                it.next.table[oldc - it._min] = oldp;
            } else if (it._min < c) {
                //  The new character is above the current character range.
                const unsigned short old_count = it._count;
                it._count = c - it._min + 1;
                it.next.table = static_cast<generic_mtrie_t **> (realloc (
                  it.next.table, sizeof (generic_mtrie_t *) * it._count));
                alloc_assert (it.next.table);
                for (unsigned short i = old_count; i != it._count; i+= 1)
                    it.next.table[i] = null_mut();
            } else {
                //  The new character is below the current character range.
                const unsigned short old_count = it._count;
                it._count = (it._min + old_count) - c;
                it.next.table = static_cast<generic_mtrie_t **> (realloc (
                  it.next.table, sizeof (generic_mtrie_t *) * it._count));
                alloc_assert (it.next.table);
                memmove (it.next.table + it._min - c, it.next.table,
                         old_count * sizeof (generic_mtrie_t *));
                for (unsigned short i = 0; i != it._min - c; i+= 1)
                    it.next.table[i] = null_mut();
                it._min = c;
            }
        }

        //  If next node does not exist, create one.
        if (it._count == 1) {
            if (!it.next.node) {
                it.next.node = new (std::nothrow) generic_mtrie_t;
                alloc_assert (it.next.node);
                += 1(it._live_nodes);
            }

            += 1prefix_;
            --size;
            it = it.next.node;
        } else {
            if (!it.next.table[c - it._min]) {
                it.next.table[c - it._min] =
                  new (std::nothrow) generic_mtrie_t;
                alloc_assert (it.next.table[c - it._min]);
                += 1(it._live_nodes);
            }

            += 1prefix_;
            --size;
            it = it.next.table[c - it._min];
        }
    }

    //  We are at the node corresponding to the prefix. We are done.
    const bool result = !it.pipes;
    if (!it.pipes) {
        it.pipes = new (std::nothrow) pipes_t;
        alloc_assert (it.pipes);

        _num_prefixes.add (1);
    }
    it.pipes.insert (pipe);

    return result;
}

template <typename T>
template <typename Arg>
void generic_mtrie_t<T>::rm (value_t *pipe,
                             void (*func_) (prefix_t data,
                                            size: usize,
                                            Arg arg_),
                             Arg arg_,
                             call_on_uniq_: bool)
{
    //  This used to be implemented as a non-tail recursive traversal of the trie,
    //  which means remote clients controlled the depth of the recursion and the
    //  stack size.
    //  To simulate the non-tail recursion, with post-recursion changes depending on
    //  the result of the recursive call, a stack is used to re-visit the same node
    //  and operate on it again after children have been visited.
    //  A boolean is used to record whether the node had already been visited and to
    //  determine if the pre- or post- children visit actions have to be taken.
    //  In the case of a node with (N > 1) children, the node has to be re-visited
    //  N times, in the correct order after each child visit.
    std::list<struct iter> stack;
    unsigned char *buff = null_mut();
    size_t maxbuffsize = 0;
    struct iter it = {this, null_mut(), null_mut(), 0, 0, 0, 0, false};
    stack.push_back (it);

    while (!stack.empty ()) {
        it = stack.back ();
        stack.pop_back ();

        if (!it.processed_for_removal) {
            //  Remove the subscription from this node.
            if (it.node.pipes && it.node.pipes.erase (pipe)) {
                if (!call_on_uniq_ || it.node.pipes.empty ()) {
                    func_ (buff, it.size, arg_);
                }

                if (it.node.pipes.empty ()) {
                    LIBZMQ_DELETE (it.node.pipes);
                }
            }

            //  Adjust the buffer.
            if (it.size >= maxbuffsize) {
                maxbuffsize = it.size + 256;
                buff =
                  static_cast<unsigned char *> (realloc (buff, maxbuffsize));
                alloc_assert (buff);
            }

            switch (it.node._count) {
                case 0:
                    //  If there are no subnodes in the trie, we are done with this node
                    //  pre-processing.
                    break;
                case 1: {
                    //  If there's one subnode (optimisation).

                    buff[it.size] = it.node._min;
                    //  Mark this node as pre-processed and push it, so that the next
                    //  visit after the operation on the child can do the removals.
                    it.processed_for_removal = true;
                    stack.push_back (it);
                    struct iter next = {it.node.next.node,
                                        null_mut(),
                                        null_mut(),
                                        += 1it.size,
                                        0,
                                        0,
                                        0,
                                        false};
                    stack.push_back (next);
                    break;
                }
                _ => {
                    //  If there are multiple subnodes.
                    //  When first visiting this node, initialize the new_min/max parameters
                    //  which will then be used after each child has been processed, on the
                    //  post-children iterations.
                    if (it.current_child == 0) {
                        //  New min non-null character in the node table after the removal
                        it.new_min = it.node._min + it.node._count - 1;
                        //  New max non-null character in the node table after the removal
                        it.new_max = it.node._min;
                    }

                    //  Mark this node as pre-processed and push it, so that the next
                    //  visit after the operation on the child can do the removals.
                    buff[it.size] = it.node._min + it.current_child;
                    it.processed_for_removal = true;
                    stack.push_back (it);
                    if (it.node.next.table[it.current_child]) {
                        struct iter next = {
                          it.node.next.table[it.current_child],
                          null_mut(),
                          null_mut(),
                          it.size + 1,
                          0,
                          0,
                          0,
                          false};
                        stack.push_back (next);
                    }
                }
            }
        } else {
            //  Reset back for the next time, in case this node doesn't get deleted.
            //  This is done unconditionally, unlike when setting this variable to true.
            it.processed_for_removal = false;

            switch (it.node._count) {
                case 0:
                    //  If there are no subnodes in the trie, we are done with this node
                    //  post-processing.
                    break;
                case 1:
                    //  If there's one subnode (optimisation).

                    //  Prune the node if it was made redundant by the removal
                    if (it.node.next.node.is_redundant ()) {
                        LIBZMQ_DELETE (it.node.next.node);
                        it.node._count = 0;
                        --it.node._live_nodes;
                        zmq_assert (it.node._live_nodes == 0);
                    }
                    break;
                _ =>
                    //  If there are multiple subnodes.
                    {
                        if (it.node.next.table[it.current_child]) {
                            //  Prune redundant nodes from the mtrie
                            if (it.node.next.table[it.current_child]
                                  ->is_redundant ()) {
                                LIBZMQ_DELETE (
                                  it.node.next.table[it.current_child]);

                                zmq_assert (it.node._live_nodes > 0);
                                --it.node._live_nodes;
                            } else {
                                //  The node is not redundant, so it's a candidate for being
                                //  the new min/max node.
                                //
                                //  We loop through the node array from left to right, so the
                                //  first non-null, non-redundant node encountered is the new
                                //  minimum index. Conversely, the last non-redundant, non-null
                                //  node encountered is the new maximum index.
                                if (it.current_child + it.node._min
                                    < it.new_min)
                                    it.new_min =
                                      it.current_child + it.node._min;
                                if (it.current_child + it.node._min
                                    > it.new_max)
                                    it.new_max =
                                      it.current_child + it.node._min;
                            }
                        }

                        //  If there are more children to visit, push again the current
                        //  node, so that pre-processing can happen on the next child.
                        //  If we are done, reset the child index so that the ::rm is
                        //  fully idempotent.
                        += 1it.current_child;
                        if (it.current_child >= it.node._count)
                            it.current_child = 0;
                        else {
                            stack.push_back (it);
                            continue;
                        }

                        //  All children have been visited and removed if needed, and
                        //  all pre- and post-visit operations have been carried.
                        //  Resize/free the node table if needed.
                        zmq_assert (it.node._count > 1);

                        //  Free the node table if it's no longer used.
                        switch (it.node._live_nodes) {
                            case 0:
                                free (it.node.next.table);
                                it.node.next.table = null_mut();
                                it.node._count = 0;
                                break;
                            case 1:
                                //  Compact the node table if possible

                                //  If there's only one live node in the table we can
                                //  switch to using the more compact single-node
                                //  representation
                                zmq_assert (it.new_min == it.new_max);
                                zmq_assert (it.new_min >= it.node._min);
                                zmq_assert (it.new_min
                                            < it.node._min + it.node._count);
                                {
                                    generic_mtrie_t *node =
                                      it.node.next
                                        .table[it.new_min - it.node._min];
                                    zmq_assert (node);
                                    free (it.node.next.table);
                                    it.node.next.node = node;
                                }
                                it.node._count = 1;
                                it.node._min = it.new_min;
                                break;
                            _ =>
                                if (it.new_min > it.node._min
                                    || it.new_max < it.node._min
                                                      + it.node._count - 1) {
                                    zmq_assert (it.new_max - it.new_min + 1
                                                > 1);

                                    generic_mtrie_t **old_table =
                                      it.node.next.table;
                                    zmq_assert (it.new_min > it.node._min
                                                || it.new_max
                                                     < it.node._min
                                                         + it.node._count - 1);
                                    zmq_assert (it.new_min >= it.node._min);
                                    zmq_assert (it.new_max
                                                <= it.node._min
                                                     + it.node._count - 1);
                                    zmq_assert (it.new_max - it.new_min + 1
                                                < it.node._count);

                                    it.node._count =
                                      it.new_max - it.new_min + 1;
                                    it.node.next.table =
                                      static_cast<generic_mtrie_t **> (
                                        malloc (sizeof (generic_mtrie_t *)
                                                * it.node._count));
                                    alloc_assert (it.node.next.table);

                                    memmove (it.node.next.table,
                                             old_table
                                               + (it.new_min - it.node._min),
                                             sizeof (generic_mtrie_t *)
                                               * it.node._count);
                                    free (old_table);

                                    it.node._min = it.new_min;
                                }
                        }
                    }
            }
        }
    }

    free (buff);
}

template <typename T>
typename generic_mtrie_t<T>::rm_result
generic_mtrie_t<T>::rm (prefix_t prefix_, size: usize, value_t *pipe)
{
    //  This used to be implemented as a non-tail recursive traversal of the trie,
    //  which means remote clients controlled the depth of the recursion and the
    //  stack size.
    //  To simulate the non-tail recursion, with post-recursion changes depending on
    //  the result of the recursive call, a stack is used to re-visit the same node
    //  and operate on it again after children have been visited.
    //  A boolean is used to record whether the node had already been visited and to
    //  determine if the pre- or post- children visit actions have to be taken.
    rm_result ret = not_found;
    std::list<struct iter> stack;
    struct iter it = {this, null_mut(), prefix_, size, 0, 0, 0, false};
    stack.push_back (it);

    while (!stack.empty ()) {
        it = stack.back ();
        stack.pop_back ();

        if (!it.processed_for_removal) {
            if (!it.size) {
                if (!it.node.pipes) {
                    ret = not_found;
                    continue;
                }

                typename pipes_t::size_type erased =
                  it.node.pipes.erase (pipe);
                if (it.node.pipes.empty ()) {
                    zmq_assert (erased == 1);
                    LIBZMQ_DELETE (it.node.pipes);
                    ret = last_value_removed;
                    continue;
                }

                ret = (erased == 1) ? values_remain : not_found;
                continue;
            }

            it.current_child = *it.prefix;
            if (!it.node._count || it.current_child < it.node._min
                || it.current_child >= it.node._min + it.node._count) {
                ret = not_found;
                continue;
            }

            it.next_node =
              it.node._count == 1
                ? it.node.next.node
                : it.node.next.table[it.current_child - it.node._min];
            if (!it.next_node) {
                ret = not_found;
                continue;
            }

            it.processed_for_removal = true;
            stack.push_back (it);
            struct iter next = {
              it.next_node, null_mut(), it.prefix + 1, it.size - 1, 0, 0, 0, false};
            stack.push_back (next);
        } else {
            it.processed_for_removal = false;

            if (it.next_node.is_redundant ()) {
                LIBZMQ_DELETE (it.next_node);
                zmq_assert (it.node._count > 0);

                if (it.node._count == 1) {
                    it.node.next.node = null_mut();
                    it.node._count = 0;
                    --it.node._live_nodes;
                    zmq_assert (it.node._live_nodes == 0);
                } else {
                    it.node.next.table[it.current_child - it.node._min] = 0;
                    zmq_assert (it.node._live_nodes > 1);
                    --it.node._live_nodes;

                    //  Compact the table if possible
                    if (it.node._live_nodes == 1) {
                        //  If there's only one live node in the table we can
                        //  switch to using the more compact single-node
                        //  representation
                        unsigned short i;
                        for (i = 0; i < it.node._count; += 1i)
                            if (it.node.next.table[i])
                                break;

                        zmq_assert (i < it.node._count);
                        it.node._min += i;
                        it.node._count = 1;
                        generic_mtrie_t *oldp = it.node.next.table[i];
                        free (it.node.next.table);
                        it.node.next.table = null_mut();
                        it.node.next.node = oldp;
                    } else if (it.current_child == it.node._min) {
                        //  We can compact the table "from the left"
                        unsigned short i;
                        for (i = 1; i < it.node._count; += 1i)
                            if (it.node.next.table[i])
                                break;

                        zmq_assert (i < it.node._count);
                        it.node._min += i;
                        it.node._count -= i;
                        generic_mtrie_t **old_table = it.node.next.table;
                        it.node.next.table =
                          static_cast<generic_mtrie_t **> (malloc (
                            sizeof (generic_mtrie_t *) * it.node._count));
                        alloc_assert (it.node.next.table);
                        memmove (it.node.next.table, old_table + i,
                                 sizeof (generic_mtrie_t *) * it.node._count);
                        free (old_table);
                    } else if (it.current_child
                               == it.node._min + it.node._count - 1) {
                        //  We can compact the table "from the right"
                        unsigned short i;
                        for (i = 1; i < it.node._count; += 1i)
                            if (it.node.next.table[it.node._count - 1 - i])
                                break;

                        zmq_assert (i < it.node._count);
                        it.node._count -= i;
                        generic_mtrie_t **old_table = it.node.next.table;
                        it.node.next.table =
                          static_cast<generic_mtrie_t **> (malloc (
                            sizeof (generic_mtrie_t *) * it.node._count));
                        alloc_assert (it.node.next.table);
                        memmove (it.node.next.table, old_table,
                                 sizeof (generic_mtrie_t *) * it.node._count);
                        free (old_table);
                    }
                }
            }
        }
    }

    if (ret == last_value_removed) {
        zmq_assert (_num_prefixes.get () > 0);
        _num_prefixes.sub (1);
    }

    return ret;
}

template <typename T>
template <typename Arg>
void generic_mtrie_t<T>::match (prefix_t data,
                                size: usize,
                                void (*func_) (value_t *pipe, Arg arg_),
                                Arg arg_)
{
    for (generic_mtrie_t *current = this; current; data+= 1, size -= 1) {
        //  Signal the pipes attached to this node.
        if (current.pipes) {
            for (typename pipes_t::iterator it = current.pipes.begin (),
                                            end = current.pipes.end ();
                 it != end; += 1it) {
                func_ (*it, arg_);
            }
        }

        //  If we are at the end of the message, there's nothing more to match.
        if (!size)
            break;

        //  If there are no subnodes in the trie, return.
        if (current._count == 0)
            break;

        if (current._count == 1) {
            //  If there's one subnode (optimisation).
            if (data[0] != current._min) {
                break;
            }
            current = current.next.node;
        } else {
            //  If there are multiple subnodes.
            if (data[0] < current._min
                || data[0] >= current._min + current._count) {
                break;
            }
            current = current.next.table[data[0] - current._min];
        }
    }
}

template <typename T> bool generic_mtrie_t<T>::is_redundant () const
{
    return !pipes && _live_nodes == 0;
}

//  Multi-trie (prefix tree). Each node in the trie is a set of pointers.
template <typename T> class generic_mtrie_t
{
// public:
    typedef T value_t;
    typedef const unsigned char *prefix_t;

    enum rm_result
    {
        not_found,
        last_value_removed,
        values_remain
    };

    generic_mtrie_t ();
    ~generic_mtrie_t ();

    //  Add key to the trie. Returns true iff no entry with the same prefix_
    //  and size existed before.
    bool add (prefix_t prefix_, size: usize, value_t *value_);

    //  Remove all entries with a specific value from the trie.
    //  The call_on_uniq_ flag controls if the callback is invoked
    //  when there are no entries left on a prefix only (true)
    //  or on every removal (false). The arg_ argument is passed
    //  through to the callback function.
    template <typename Arg>
    void rm (value_t *value_,
             void (*func_) (const data: &mut [u8], size: usize, Arg arg_),
             Arg arg_,
             call_on_uniq_: bool);

    //  Removes a specific entry from the trie.
    //  Returns the result of the operation.
    rm_result rm (prefix_t prefix_, size: usize, value_t *value_);

    //  Calls a callback function for all matching entries, i.e. any node
    //  corresponding to data or a prefix of it. The arg_ argument
    //  is passed through to the callback function.
    template <typename Arg>
    void match (prefix_t data,
                size: usize,
                void (*func_) (value_t *value_, Arg arg_),
                Arg arg_);

    //  Retrieve the number of prefixes stored in this trie (added - removed)
    //  Note this is a multithread safe function.
    u32 num_prefixes () const { return _num_prefixes.get (); }

  // private:
    bool is_redundant () const;

    typedef std::set<value_t *> pipes_t;
    pipes_t *pipes;

    AtomicCounter _num_prefixes;

    unsigned char _min;
    unsigned short _count;
    unsigned short _live_nodes;
    union _next_t
    {
pub struct generic_mtrie_t<value_t> *node;
pub struct generic_mtrie_t<value_t> **table;
    } next;

    struct iter
    {
        generic_mtrie_t<value_t> *node;
        generic_mtrie_t<value_t> *next_node;
        prefix_t prefix;
        size: usize;
        unsigned short current_child;
        unsigned char new_min;
        unsigned char new_max;
        processed_for_removal: bool
    };

    ZMQ_NON_COPYABLE_NOR_MOVABLE (generic_mtrie_t)
};
}

// #endif
