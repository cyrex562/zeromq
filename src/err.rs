use std::error::Error;

#[derive(thiserror::Error)]
pub enum ZmqError {
    #[error("Invalid context")]
    InvalidContext(&'static str),
    #[error("Mailbox Error")]
    MailboxError(),
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
}
