use crate::pipe::ZmqPipe;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct out_pipe_t {
    // ZmqPipe *pipe;
    pub pipe: ZmqPipe,
    // active: bool
    pub active: bool,
}
