use crate::{zmq_content::ZmqContent, message::MAX_VSM_SIZE};
use std::rc::Rc;

#[derive(Default,Debug,Clone)]
pub struct c_single_allocator
{
// public:
  // private:
    // std::size_t buf_size;
    pub buf_size: usize,
    // unsigned char *buf;
    pub buf: Vec<u8>,

    // ZMQ_NON_COPYABLE_NOR_MOVABLE (c_single_allocator)
}

impl c_single_allocator {
//     explicit c_single_allocator (std::bufsize_: usize) :
//     buf_size (bufsize_),
//     buf (static_cast<unsigned char *> (std::malloc (buf_size)))
// {
//     alloc_assert (buf);
// }
pub fn new(buf_size: usize) -> Self {
    let mut out = Self::default();
    out.buf_size = buf_size;
    out.buf = vec![0; buf_size];
    out
}

// ~c_single_allocator () { std::free (buf); }

// unsigned char *allocate () { return buf; }
pub fn allocate(&mut self) -> *mut u8 {
    self.buf.as_mut_ptr()
}

// void deallocate () {}

// std::size_t size () const { return buf_size; }
pub fn size(&self) -> usize {
    self.buf_size
}

//  This buffer is fixed, size must not be changed
// void resize (std::new_size: usize) { LIBZMQ_UNUSED (new_size); }
}

// This allocator allocates a reference counted buffer which is used by v2_decoder_t
// to use zero-copy msg::init_data to create messages with memory from this buffer as
// data storage.
//
// The buffer is allocated with a reference count of 1 to make sure that is is alive while
// decoding messages. Otherwise, it is possible that e.g. the first message increases the count
// from zero to one, gets passed to the user application, processed in the user thread and deleted
// which would then deallocate the buffer. The drawback is that the buffer may be allocated longer
// than necessary because it is only deleted when allocate is called the next time.
#[derive(Default,Debug,Clone)]
pub struct shared_message_memory_allocator
{
    // unsigned char *buf;
    pub buf: Rc<Vec<u8>>,
    // std::size_t buf_size;
    pub buf_size: usize,
    // const std::size_t max_size;
    pub max_size: usize,
    // ZmqMessage::ZmqContent *msg_content;
    pub msg_content: Option<ZmqContent>,
    // std::size_t max_counters;
    pub max_counters: usize,
}

//
impl shared_message_memory_allocator {
    // public:
    // explicit shared_message_memory_allocator (std::bufsize_: usize);
    // shared_message_memory_allocator::shared_message_memory_allocator (
    //     std::bufsize_: usize) :
    //       buf (null_mut()),
    //       buf_size (0),
    //       max_size (bufsize_),
    //       msg_content (null_mut()),
    //       max_counters ((max_size + ZmqMessage::MAX_VSM_SIZE - 1) / ZmqMessage::MAX_VSM_SIZE)
    //   {
    //   }
    pub fn with_size(in_size: usize) -> Self {
        Self {
            buf: Rc::new(vec![]),
            buf_size: 0,
            max_size: in_size,
            msg_content: None,
            max_counters: in_size + MAX_VSM_SIZE - 1,
        }
    }

    // Create an allocator for a maximum number of messages
    // shared_message_memory_allocator (std::bufsize_: usize,
                                    //  std::max_messages_: usize);
    // shared_message_memory_allocator::shared_message_memory_allocator (
    //     std::bufsize_: usize, std::max_messages_: usize) :
    //         buf (null_mut()),
    //         buf_size (0),
    //         max_size (bufsize_),
    //         msg_content (null_mut()),
    //         max_counters (max_messages_)
    //     {
    //     }
    pub fn with_size_and_msg_cnt(in_size: usize, max_messages: usize) {
        Self {
            buf: Rc::new(vec![]),
            buf_size: 0,
            max_size: in_size,
            msg_content: None,
            max_counters: max_messages
        }
    }

    // ~shared_message_memory_allocator ();
    // shared_message_memory_allocator::~shared_message_memory_allocator ()
    // {
    //     deallocate ();
    // }

    // Allocate a new buffer
    //
    // This releases the current buffer to be bound to the lifetime of the messages
    // created on this buffer.
    // unsigned char *allocate ();
    pub fn allocate (&mut self) -> &mut Vec<u8>
    {
        // release reference count to couple lifetime to messages
            // AtomicCounter *c =
            // reinterpret_cast<AtomicCounter *> (buf);
            // if refcnt drops to 0, there are no message using the buffer
            // because either all messages have been closed or only vsm-messages
            // were created
        if self.buf.len() > 0 {
            if Rc::strong_count(&self.buf) > 0 {
                // buffer is still in use as message data. "Release" it and create a new one
                // release pointer because we are going to create a new buffer
                self.release();
            }
        }

        // if buf != NULL it is not used by any message so we can re-use it for the next run
        if (self.buf.is_empty()) {
            // allocate memory for reference counters together with reception buffer
            // std::size_t const allocationsize =
            let allocationSize: usize = self.max_size + max_counters * std::mem::<ZmqContent>() + std::mem::size_of::<AtomicCounter>();
            // max_size + sizeof (AtomicCounter)
            // + max_counters * sizeof (ZmqMessage::ZmqContent);
            // buf = static_cast<unsigned char *> (std::malloc (allocationsize));
            // alloc_assert (buf);
            self.buf.clear();
            self.buf.reserve(allocationSize);

            // new (buf) AtomicCounter (1);
        } else {
            // release reference count to couple lifetime to messages
            // AtomicCounter *c =
            // reinterpret_cast<AtomicCounter *> (buf);
            // c.set (1);
            // TODO
        }

        self.buf_size = self.max_size;
        // msg_content = reinterpret_cast<ZmqMessage::ZmqContent *> (
        // buf + mem::size_of::<AtomicCounter>() + max_size);
        self.msg_content = Some(bincode::deserialize(self.buf.as_slice()));
        return &mut self.buf
    }

    // force deallocation of buffer.
    // void deallocate ();
    pub fn deallocate (&mut self)
    {
        // AtomicCounter *c = reinterpret_cast<AtomicCounter *> (buf);
        // if (buf && !c.sub (1)) {
        //     c->~AtomicCounter ();
        //     std::free (buf);
        // }
        self.clear ();
    }

    // Give up ownership of the buffer. The buffer's lifetime is now coupled to
    // the messages constructed on top of it.
    // unsigned char *release ();
    pub fn release (&mut self) -> &mut Vec<u8>
    {
        // unsigned char *b = buf;
        self.clear ();
        // return b;
        &mut self.buf
    }

    // void inc_ref ();
    pub fn inc_ref (&mut self)
    {
        // (reinterpret_cast<AtomicCounter *> (buf))->add (1);
    }

    // static void call_dec_ref (void *, hint: *mut c_void);
    pub fn call_dec_ref (&mut self, hint: *mut c_void)
    {
        // zmq_assert (hint);
        // unsigned char *buf = static_cast<unsigned char *> (hint);
        // AtomicCounter *c = reinterpret_cast<AtomicCounter *> (buf);

        // if (!c.sub (1)) {
        //     c->~AtomicCounter ();
        //     std::free (buf);
        //     buf = null_mut();
        // }
        // TODO
    }

    // std::size_t size () const;
    // std::size_t shared_message_memory_allocator::size () const
    pub fn size(&self) -> usize
    {
        // return buf_size;
        self.buf.len()
    }


    // Return pointer to the first message data byte.
    // unsigned char *data ();
    // unsigned char *shared_message_memory_allocator::data ()
    // {
    //     return buf + sizeof (AtomicCounter);
    // }
    pub fn data(&mut self) -> &mut Vec<u8> {
        &mut self.buf
    }

    // Return pointer to the first byte of the buffer.
    // unsigned char *buffer () { return buf; }
    pub fn buffer(&mut self) -> &mut Vec<u8> {
        &mut self.buf
    }

    // void resize (std::new_size: usize) { buf_size = new_size; }
    pub fn resize(&mut self, new_size: usize) {
        self.buf.resize(new_size, 0);
    }

    // ZmqMessage::ZmqContent *provide_content () { return msg_content; }
    pub fn provide_content(&mut self) -> &mut Option<ZmqContent> {
        &mut self.msg_content
    }

    // void advance_content () { msg_content++; }
    pub fn advance_content(&mut self) {
        todo!()
    }

  // private:
    // void clear ();
    pub fn clear (&mut self)
    {
        // buf = null_mut();
        // buf_size = 0;
        // msg_content = null_mut();
        self.buf.clear();
        self.buf_size = 0;
        self.msg_content = None;
    }
}
