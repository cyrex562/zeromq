// /* Some architectures, like sparc64 and some variants of aarch64, enforce pointer
//  * alignment and raise sigbus on violations. Make sure applications allocate
//  * zmq_msg_t on addresses aligned on a pointer-size boundary to avoid this issue.
//  */
pub struct ZmqRawMsg {
    // #if defined(_MSC_VER) && (defined(_M_X64) || defined(_M_ARM64))
    //     __declspec(align (8)) unsigned char _[64];
    // #elif defined(_MSC_VER)                                                        \
    //   && (defined(_M_IX86) || defined(_M_ARM_ARMV7VE) || defined(_M_ARM))
    //     __declspec(align (4)) unsigned char _[64];
    // #elif defined(__GNUC__) || defined(__INTEL_COMPILER)                           \
    //   || (defined(__SUNPRO_C) && __SUNPRO_C >= 0x590)                              \
    //   || (defined(__SUNPRO_CC) && __SUNPRO_CC >= 0x590)
    //     unsigned char _[64] __attribute__ ((aligned (sizeof (void *))));
    // #else
    //     unsigned char _[64];
    // #endif
    pub _x: [u8; 64],
}
