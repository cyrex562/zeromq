
pub struct socket_base_t
{
    pub own: own_t,
    pub poll_events: i_poll_events,
    pub pipe_events: i_pipe_events,
}