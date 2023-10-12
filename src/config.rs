#![allow(non_upper_case_globals)]

pub const command_pipe_granularity: usize = 16;
pub const message_pipe_granularity: usize = 256;
pub const inbound_poll_rate: u32 = 100;
pub const max_wm_delta: u32 = 1024;
pub const max_io_events: u32 = 256;
pub const proxy_burst_size: u32 = 1000;
pub const max_command_delay: u32 = 3000000;
pub const clock_precision: u32 = 1000000;
pub const signaler_port: u32 = 0;
