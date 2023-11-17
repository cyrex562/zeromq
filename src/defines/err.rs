#[derive(thiserror::Error, Debug)]
pub enum ZmqError {
    #[error("Invalid context")]
    InvalidContext(&'static str),
    #[error("Mailbox Error")]
    MailboxError(&'static str),
    #[error("SocketError")]
    SocketError(&'static str),
    #[error("Parsing Error")]
    ParseError(&'static str),
    #[error("failed to parse int")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("invalid property")]
    InvalidProperty(&'static str),
    #[error("poller error")]
    PollerError(&'static str),
    #[error("timer error")]
    TimerError(&'static str),
    #[error("pipe error")]
    PipeError(&'static str),
    #[error("message error")]
    MessageError(&'static str),
    #[error("engine error")]
    EngineError(&'static str),
    #[error("mechanism error")]
    MechanismError(&'static str),
    #[error("session error")]
    SessionError(&'static str),
    #[error("zap error")]
    ZapError(&'static str),
    #[error("DECODER error")]
    DecoderError(&'static str),
    #[error("platform error")]
    PlatformError(&'static str),
    #[error("proxy error")]
    ProxyError(&'static str),
    #[error("options error")]
    OptionsError(&'static str),
    #[error("context error")]
    ContextError(&'static str),
}
