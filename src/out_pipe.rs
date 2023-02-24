use serde::{Deserialize, Serialize};
use crate::pipe::pipe_t;

#[derive(Default,Debug,Clone,Deserialize,Serialize)]
pub struct out_pipe_t
{
    // pipe_t *pipe;
    pub pipe: pipe_t,
    // active: bool
    pub active: bool,
}
