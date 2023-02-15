use crate::endpoint::ZmqEndpoint;
use crate::pipe::pipe_t;

#[derive(Default, Debug, Clone)]
pub struct PendingConnection {
    // ZmqEndpoint endpoint;
    pub endpoint: ZmqEndpoint,
    // pipe_t *connect_pipe;
    pub connect_pipe: pipe_t,
    // pipe_t *bind_pipe;
    pub bind_pipe: pipe_t,
}
