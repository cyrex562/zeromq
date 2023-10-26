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
}
