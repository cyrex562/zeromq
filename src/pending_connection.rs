use crate::endpoint::ZmqEndpoint;
use crate::pipe::ZmqPipe;

#[derive(Default, Debug, Clone)]
pub struct PendingConnection<'a> {
    // ZmqEndpoint endpoint;
    pub endpoint: ZmqEndpoint<'a>,
    // ZmqPipe *connect_pipe;
    pub connect_pipe: ZmqPipe,
    // ZmqPipe *bind_pipe;
    pub bind_pipe: ZmqPipe,
}
