/*
    Copyright (c) 2007-2016 Contributors as noted in the AUTHORS file

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

// #include "precompiled.hpp"
// #include "macros.hpp"
// #include "err.hpp"
// #include "trie.hpp"

// #include <stdlib.h>

// #include <new>
// #include <algorithm>
pub struct trie_t
{
// public:
    trie_t ();
    ~trie_t ();

    //  Add key to the trie. Returns true if this is a new item in the trie
    //  rather than a duplicate.
    bool add (unsigned char *prefix_, size: usize);

    //  Remove key from the trie. Returns true if the item is actually
    //  removed from the trie.
    bool rm (unsigned char *prefix_, size: usize);

    //  Check whether particular key is in the trie.
    bool check (const unsigned char *data, size: usize) const;

    //  Apply the function supplied to each subscription in the trie.
    void apply (void (*func_) (unsigned char *data, size: usize, arg_: *mut c_void),
                arg_: *mut c_void);

  // private:
    void apply_helper (unsigned char **buff_,
                       buffsize_: usize,
                       maxbuffsize_: usize,
                       void (*func_) (unsigned char *data,
                                      size: usize,
                                      arg_: *mut c_void),
                       arg_: *mut c_void) const;
    bool is_redundant () const;

    uint32_t _refcnt;
    unsigned char _min;
    unsigned short _count;
    unsigned short _live_nodes;
    union
    {
pub struct trie_t *node;
pub struct trie_t **table;
    } _next;

    ZMQ_NON_COPYABLE_NOR_MOVABLE (trie_t)
};


// lightweight wrapper around trie_t adding tracking of total number of prefixes
pub struct trie_with_size_t
{
// public:
    trie_with_size_t () {}
    ~trie_with_size_t () {}

    bool add (unsigned char *prefix_, size: usize)
    {
        if (_trie.add (prefix_, size)) {
            _num_prefixes.add (1);
            return true;
        } else
            return false;
    }

    bool rm (unsigned char *prefix_, size: usize)
    {
        if (_trie.rm (prefix_, size)) {
            _num_prefixes.sub (1);
            return true;
        } else
            return false;
    }

    bool check (const unsigned char *data, size: usize) const
    {
        return _trie.check (data, size);
    }

    void apply (void (*func_) (unsigned char *data, size: usize, arg_: *mut c_void),
                arg_: *mut c_void)
    {
        _trie.apply (func_, arg_);
    }

    //  Retrieve the number of prefixes stored in this trie (added - removed)
    //  Note this is a multithread safe function.
    uint32_t num_prefixes () const { return _num_prefixes.get (); }

  // private:
    AtomicCounter _num_prefixes;
    trie_t _trie;
};

trie_t::trie_t () : _refcnt (0), _min (0), _count (0), _live_nodes (0)
{
}

trie_t::~trie_t ()
{
    if (_count == 1) {
        zmq_assert (_next.node);
        LIBZMQ_DELETE (_next.node);
    } else if (_count > 1) {
        for (unsigned short i = 0; i != _count; ++i) {
            LIBZMQ_DELETE (_next.table[i]);
        }
        free (_next.table);
    }
}

bool trie_t::add (unsigned char *prefix_, size: usize)
{
    //  We are at the node corresponding to the prefix. We are done.
    if (!size) {
        ++_refcnt;
        return _refcnt == 1;
    }

    const unsigned char c = *prefix_;
    if (c < _min || c >= _min + _count) {
        //  The character is out of range of currently handled
        //  characters. We have to extend the table.
        if (!_count) {
            _min = c;
            _count = 1;
            _next.node = NULL;
        } else if (_count == 1) {
            const unsigned char oldc = _min;
            trie_t *oldp = _next.node;
            _count = (_min < c ? c - _min : _min - c) + 1;
            _next.table =
              static_cast<trie_t **> (malloc (sizeof (trie_t *) * _count));
            alloc_assert (_next.table);
            for (unsigned short i = 0; i != _count; ++i)
                _next.table[i] = 0;
            _min = std::min (_min, c);
            _next.table[oldc - _min] = oldp;
        } else if (_min < c) {
            //  The new character is above the current character range.
            const unsigned short old_count = _count;
            _count = c - _min + 1;
            _next.table = static_cast<trie_t **> (
              realloc (_next.table, sizeof (trie_t *) * _count));
            zmq_assert (_next.table);
            for (unsigned short i = old_count; i != _count; i++)
                _next.table[i] = NULL;
        } else {
            //  The new character is below the current character range.
            const unsigned short old_count = _count;
            _count = (_min + old_count) - c;
            _next.table = static_cast<trie_t **> (
              realloc (_next.table, sizeof (trie_t *) * _count));
            zmq_assert (_next.table);
            memmove (_next.table + _min - c, _next.table,
                     old_count * sizeof (trie_t *));
            for (unsigned short i = 0; i != _min - c; i++)
                _next.table[i] = NULL;
            _min = c;
        }
    }

    //  If next node does not exist, create one.
    if (_count == 1) {
        if (!_next.node) {
            _next.node = new (std::nothrow) trie_t;
            alloc_assert (_next.node);
            ++_live_nodes;
            zmq_assert (_live_nodes == 1);
        }
        return _next.node->add (prefix_ + 1, size - 1);
    }
    if (!_next.table[c - _min]) {
        _next.table[c - _min] = new (std::nothrow) trie_t;
        alloc_assert (_next.table[c - _min]);
        ++_live_nodes;
        zmq_assert (_live_nodes > 1);
    }
    return _next.table[c - _min]->add (prefix_ + 1, size - 1);
}

bool trie_t::rm (unsigned char *prefix_, size: usize)
{
    //  TODO: Shouldn't an error be reported if the key does not exist?
    if (!size) {
        if (!_refcnt)
            return false;
        _refcnt--;
        return _refcnt == 0;
    }
    const unsigned char c = *prefix_;
    if (!_count || c < _min || c >= _min + _count)
        return false;

    trie_t *next_node = _count == 1 ? _next.node : _next.table[c - _min];

    if (!next_node)
        return false;

    const bool ret = next_node->rm (prefix_ + 1, size - 1);

    //  Prune redundant nodes
    if (next_node->is_redundant ()) {
        LIBZMQ_DELETE (next_node);
        zmq_assert (_count > 0);

        if (_count == 1) {
            //  The just pruned node is was the only live node
            _next.node = 0;
            _count = 0;
            --_live_nodes;
            zmq_assert (_live_nodes == 0);
        } else {
            _next.table[c - _min] = 0;
            zmq_assert (_live_nodes > 1);
            --_live_nodes;

            //  Compact the table if possible
            if (_live_nodes == 1) {
                //  We can switch to using the more compact single-node
                //  representation since the table only contains one live node
                trie_t *node = 0;
                //  Since we always compact the table the pruned node must
                //  either be the left-most or right-most ptr in the node
                //  table
                if (c == _min) {
                    //  The pruned node is the left-most node ptr in the
                    //  node table => keep the right-most node
                    node = _next.table[_count - 1];
                    _min += _count - 1;
                } else if (c == _min + _count - 1) {
                    //  The pruned node is the right-most node ptr in the
                    //  node table => keep the left-most node
                    node = _next.table[0];
                }
                zmq_assert (node);
                free (_next.table);
                _next.node = node;
                _count = 1;
            } else if (c == _min) {
                //  We can compact the table "from the left".
                //  Find the left-most non-null node ptr, which we'll use as
                //  our new min
                unsigned char new_min = _min;
                for (unsigned short i = 1; i < _count; ++i) {
                    if (_next.table[i]) {
                        new_min = i + _min;
                        break;
                    }
                }
                zmq_assert (new_min != _min);

                trie_t **old_table = _next.table;
                zmq_assert (new_min > _min);
                zmq_assert (_count > new_min - _min);

                _count = _count - (new_min - _min);
                _next.table =
                  static_cast<trie_t **> (malloc (sizeof (trie_t *) * _count));
                alloc_assert (_next.table);

                memmove (_next.table, old_table + (new_min - _min),
                         sizeof (trie_t *) * _count);
                free (old_table);

                _min = new_min;
            } else if (c == _min + _count - 1) {
                //  We can compact the table "from the right".
                //  Find the right-most non-null node ptr, which we'll use to
                //  determine the new table size
                unsigned short new_count = _count;
                for (unsigned short i = 1; i < _count; ++i) {
                    if (_next.table[_count - 1 - i]) {
                        new_count = _count - i;
                        break;
                    }
                }
                zmq_assert (new_count != _count);
                _count = new_count;

                trie_t **old_table = _next.table;
                _next.table =
                  static_cast<trie_t **> (malloc (sizeof (trie_t *) * _count));
                alloc_assert (_next.table);

                memmove (_next.table, old_table, sizeof (trie_t *) * _count);
                free (old_table);
            }
        }
    }
    return ret;
}

bool trie_t::check (const unsigned char *data, size: usize) const
{
    //  This function is on critical path. It deliberately doesn't use
    //  recursion to get a bit better performance.
    const trie_t *current = this;
    while (true) {
        //  We've found a corresponding subscription!
        if (current->_refcnt)
            return true;

        //  We've checked all the data and haven't found matching subscription.
        if (!size)
            return false;

        //  If there's no corresponding slot for the first character
        //  of the prefix, the message does not match.
        const unsigned char c = *data;
        if (c < current->_min || c >= current->_min + current->_count)
            return false;

        //  Move to the next character.
        if (current->_count == 1)
            current = current->_next.node;
        else {
            current = current->_next.table[c - current->_min];
            if (!current)
                return false;
        }
        data++;
        size--;
    }
}

void trie_t::apply (
  void (*func_) (unsigned char *data, size: usize, arg_: *mut c_void), arg_: *mut c_void)
{
    unsigned char *buff = NULL;
    apply_helper (&buff, 0, 0, func_, arg_);
    free (buff);
}

void trie_t::apply_helper (unsigned char **buff_,
                                buffsize_: usize,
                                maxbuffsize_: usize,
                                void (*func_) (unsigned char *data,
                                               size: usize,
                                               arg_: *mut c_void),
                                arg_: *mut c_void) const
{
    //  If this node is a subscription, apply the function.
    if (_refcnt)
        func_ (*buff_, buffsize_, arg_);

    //  Adjust the buffer.
    if (buffsize_ >= maxbuffsize_) {
        maxbuffsize_ = buffsize_ + 256;
        *buff_ = static_cast<unsigned char *> (realloc (*buff_, maxbuffsize_));
        zmq_assert (*buff_);
    }

    //  If there are no subnodes in the trie, return.
    if (_count == 0)
        return;

    //  If there's one subnode (optimisation).
    if (_count == 1) {
        (*buff_)[buffsize_] = _min;
        buffsize_++;
        _next.node->apply_helper (buff_, buffsize_, maxbuffsize_, func_, arg_);
        return;
    }

    //  If there are multiple subnodes.
    for (unsigned short c = 0; c != _count; c++) {
        (*buff_)[buffsize_] = _min + c;
        if (_next.table[c])
            _next.table[c]->apply_helper (buff_, buffsize_ + 1, maxbuffsize_,
                                          func_, arg_);
    }
}

bool trie_t::is_redundant () const
{
    return _refcnt == 0 && _live_nodes == 0;
}
