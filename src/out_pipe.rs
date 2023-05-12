use crate::pipe::ZmqPipe;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct ZmqOutPipe {
    // ZmqPipe *pipe;
    pub pipe: ZmqPipe,
    // active: bool
    pub active: bool,
}
