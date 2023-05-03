
// ****************************************************************************
//   0MQ Internal Use
// ****************************************************************************

// #define LIBZMQ_UNUSED(object) (void) object
// #define LIBZMQ_DELETE(p_object)                                                \
    {                                                                          \
        delete p_object;                                                       \
        p_object = 0;                                                          \
    }

// ****************************************************************************

// #if !defined ZMQ_NOEXCEPT
// #if defined ZMQ_HAVE_NOEXCEPT
// #define ZMQ_NOEXCEPT noexcept
// #else
// #define ZMQ_NOEXCEPT
// #endif
// #endif

// #if !defined
// #if defined ZMQ_HAVE_NOEXCEPT
// #define  override
// #else
// #define
// #endif
// #endif

// #if !defined
// #if defined ZMQ_HAVE_NOEXCEPT
// #define  final
// #else
// #define
// #endif
// #endif

// #if !defined ZMQ_DEFAULT
// #if defined ZMQ_HAVE_NOEXCEPT
// #define ZMQ_DEFAULT = default;
// #else
// #define ZMQ_DEFAULT                                                            \
    {                                                                          \
    }
// #endif
// #endif

// #if !defined // ZMQ_NON_COPYABLE_NOR_MOVABLE
// #if defined ZMQ_HAVE_NOEXCEPT
// #define // ZMQ_NON_COPYABLE_NOR_MOVABLE(classname)                                \
//                                                                       \
pub structname (const classname &) = delete;                                    \
pub structname &operator= (const classname &) = delete;                         \
pub structname (classname &&) = delete;                                         \
pub structname &operator= (classname &&) = delete;
// #else
// #define // ZMQ_NON_COPYABLE_NOR_MOVABLE(classname)                                \
  //                                                                      \
pub structname (const classname &);                                             \
pub structname &operator= (const classname &);
// #endif
// #endif
