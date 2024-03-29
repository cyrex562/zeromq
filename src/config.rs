//  Compile-time settings.

//  Number of new messages in message pipe needed to trigger new memory
//  allocation. Setting this parameter to 256 decreases the impact of
//  memory allocation by approximately 99.6%
pub const MESSAGE_PIPE_GRANULARITY: i32 = 256;

//  Commands in pipe per allocation event.
pub const COMMAND_PIPE_GRANULARITY: i32 = 16;

//  Determines how often does socket poll for new commands when it
//  still has unprocessed messages to handle. Thus, if it is set to 100,
//  socket will process 100 inbound messages before doing the poll.
//  If there are no unprocessed messages available, poll is Done
//  immediately. Decreasing the value trades overall latency for more
//  real-time behaviour (less latency peaks).
pub const INBOUND_POLL_RATE: i32 = 100;

//  Maximal delta between high and low watermark.
pub const MAX_WM_DELTA: i32 = 1024;

//  Maximum number of events the I/O thread can process in one go.
pub const MAX_IO_EVENTS: i32 = 256;

//  Maximal batch size of packets forwarded by a ZMQ proxy.
//  Increasing this value improves throughput at the expense of
//  latency and fairness.
pub const PROXY_BURST_SIZE: i32 = 1000;

//  Maximal delay to process command in API thread (in CPU ticks).
//  3,000,000 ticks equals to 1 - 2 milliseconds on current CPUs.
//  Note that delay is only applied when there is continuous stream of
//  messages to process. If not so, commands are processed immediately.
pub const MAX_COMMAND_DELAY: i32 = 3000000;

//  Low-precision clock precision in CPU ticks. 1ms. Value of 1000000
//  should be OK for CPU frequencies above 1GHz. If should work
//  reasonably well for CPU frequencies above 500MHz. For lower CPU
//  frequencies you may consider lowering this value to get best
//  possible latencies.
pub const CLOCK_PRECISION: i32 = 1000000;

//  On some OSes the signaler has to be emulated using a TCP
//  connection. In such cases following port is used.
//  If 0, it lets the OS choose a free port without requiring use of a
//  global mutex. The original implementation of a Windows signaler
//  socket used port 5905 instead of letting the OS choose a free port.
//  https://github.com/zeromq/libzmq/issues/1542
pub const SIGNALER_PORT: i32 = 0;

// #define crypto_box_SECRETKEYBYTES 32
pub const CRYPTO_BOX_SECRETKEYBYTES: usize = 32;
// #define crypto_box_BOXZEROBYTES 16
pub const CRYPTO_BOX_BOXZEROBYTES: usize = 16;
// #define crypto_box_NONCEBYTES 24
pub const CRYPTO_BOX_NONCEBYTES: usize = 24;
// #define crypto_box_ZEROBYTES 32
pub const CRYPTO_BOX_ZEROBYTES: usize = 32;
// #define crypto_box_PUBLICKEYBYTES 32
pub const CRYPTO_BOX_PUBLICKEYBYTES: usize = 32;
// #define crypto_box_BEFORENMBYTES 32
pub const CRYPTO_BOX_BEFORENMBYTES: usize = 32;
// #define crypto_secretbox_KEYBYTES 32
pub const CRYPTO_SECRETBOX_KEYBYTES: usize = 32;
// #define crypto_secretbox_NONCEBYTES 24
pub const CRYPTO_SECRETBOX_NONCEBYTES: usize = 24;
// #define crypto_secretbox_ZEROBYTES 32
pub const CRYPTO_SECRETBOX_ZEROBYTES: usize = 32;
// #define crypto_secretbox_BOXZEROBYTES 16
pub const CRYPTO_SECRETBOX_BOXZEROBYTES: usize = 16;
