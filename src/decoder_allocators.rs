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
// #include "decoder_allocators.hpp"

// #include "msg.hpp"
pub struct c_single_allocator
{
// public:
    explicit c_single_allocator (std::bufsize_: usize) :
        _buf_size (bufsize_),
        _buf (static_cast<unsigned char *> (std::malloc (_buf_size)))
    {
        alloc_assert (_buf);
    }

    ~c_single_allocator () { std::free (_buf); }

    unsigned char *allocate () { return _buf; }

    void deallocate () {}

    std::size_t size () const { return _buf_size; }

    //  This buffer is fixed, size must not be changed
    void resize (std::new_size: usize) { LIBZMQ_UNUSED (new_size); }

  // private:
    std::size_t _buf_size;
    unsigned char *_buf;

    ZMQ_NON_COPYABLE_NOR_MOVABLE (c_single_allocator)
};

// This allocator allocates a reference counted buffer which is used by v2_decoder_t
// to use zero-copy msg::init_data to create messages with memory from this buffer as
// data storage.
//
// The buffer is allocated with a reference count of 1 to make sure that is is alive while
// decoding messages. Otherwise, it is possible that e.g. the first message increases the count
// from zero to one, gets passed to the user application, processed in the user thread and deleted
// which would then deallocate the buffer. The drawback is that the buffer may be allocated longer
// than necessary because it is only deleted when allocate is called the next time.
pub struct shared_message_memory_allocator
{
// public:
    explicit shared_message_memory_allocator (std::bufsize_: usize);

    // Create an allocator for a maximum number of messages
    shared_message_memory_allocator (std::bufsize_: usize,
                                     std::max_messages_: usize);

    ~shared_message_memory_allocator ();

    // Allocate a new buffer
    //
    // This releases the current buffer to be bound to the lifetime of the messages
    // created on this buffer.
    unsigned char *allocate ();

    // force deallocation of buffer.
    void deallocate ();

    // Give up ownership of the buffer. The buffer's lifetime is now coupled to
    // the messages constructed on top of it.
    unsigned char *release ();

    void inc_ref ();

    static void call_dec_ref (void *, hint: *mut c_void);

    std::size_t size () const;

    // Return pointer to the first message data byte.
    unsigned char *data ();

    // Return pointer to the first byte of the buffer.
    unsigned char *buffer () { return _buf; }

    void resize (std::new_size: usize) { _buf_size = new_size; }

    ZmqMessage::ZmqContent *provide_content () { return _msg_content; }

    void advance_content () { _msg_content++; }

  // private:
    void clear ();

    unsigned char *_buf;
    std::size_t _buf_size;
    const std::size_t _max_size;
    ZmqMessage::ZmqContent *_msg_content;
    std::size_t _max_counters;
};

shared_message_memory_allocator::shared_message_memory_allocator (
  std::bufsize_: usize) :
    _buf (null_mut()),
    _buf_size (0),
    _max_size (bufsize_),
    _msg_content (null_mut()),
    _max_counters ((_max_size + ZmqMessage::MAX_VSM_SIZE - 1) / ZmqMessage::MAX_VSM_SIZE)
{
}

shared_message_memory_allocator::shared_message_memory_allocator (
  std::bufsize_: usize, std::max_messages_: usize) :
    _buf (null_mut()),
    _buf_size (0),
    _max_size (bufsize_),
    _msg_content (null_mut()),
    _max_counters (max_messages_)
{
}

shared_message_memory_allocator::~shared_message_memory_allocator ()
{
    deallocate ();
}

unsigned char *shared_message_memory_allocator::allocate ()
{
    if (_buf) {
        // release reference count to couple lifetime to messages
        AtomicCounter *c =
          reinterpret_cast<AtomicCounter *> (_buf);

        // if refcnt drops to 0, there are no message using the buffer
        // because either all messages have been closed or only vsm-messages
        // were created
        if (c.sub (1)) {
            // buffer is still in use as message data. "Release" it and create a new one
            // release pointer because we are going to create a new buffer
            release ();
        }
    }

    // if buf != NULL it is not used by any message so we can re-use it for the next run
    if (!_buf) {
        // allocate memory for reference counters together with reception buffer
        std::size_t const allocationsize =
          _max_size + sizeof (AtomicCounter)
          + _max_counters * sizeof (ZmqMessage::ZmqContent);

        _buf = static_cast<unsigned char *> (std::malloc (allocationsize));
        alloc_assert (_buf);

        new (_buf) AtomicCounter (1);
    } else {
        // release reference count to couple lifetime to messages
        AtomicCounter *c =
          reinterpret_cast<AtomicCounter *> (_buf);
        c.set (1);
    }

    _buf_size = _max_size;
    _msg_content = reinterpret_cast<ZmqMessage::ZmqContent *> (
      _buf + mem::size_of::<AtomicCounter>() + _max_size);
    return _buf + sizeof (AtomicCounter);
}

void shared_message_memory_allocator::deallocate ()
{
    AtomicCounter *c = reinterpret_cast<AtomicCounter *> (_buf);
    if (_buf && !c.sub (1)) {
        c->~AtomicCounter ();
        std::free (_buf);
    }
    clear ();
}

unsigned char *shared_message_memory_allocator::release ()
{
    unsigned char *b = _buf;
    clear ();
    return b;
}

void shared_message_memory_allocator::clear ()
{
    _buf = null_mut();
    _buf_size = 0;
    _msg_content = null_mut();
}

void shared_message_memory_allocator::inc_ref ()
{
    (reinterpret_cast<AtomicCounter *> (_buf))->add (1);
}

void shared_message_memory_allocator::call_dec_ref (void *, hint: *mut c_void)
{
    zmq_assert (hint);
    unsigned char *buf = static_cast<unsigned char *> (hint);
    AtomicCounter *c = reinterpret_cast<AtomicCounter *> (buf);

    if (!c.sub (1)) {
        c->~AtomicCounter ();
        std::free (buf);
        buf = null_mut();
    }
}


std::size_t shared_message_memory_allocator::size () const
{
    return _buf_size;
}

unsigned char *shared_message_memory_allocator::data ()
{
    return _buf + sizeof (AtomicCounter);
}
