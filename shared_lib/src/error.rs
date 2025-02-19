use core::fmt::Debug;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SerializeError {
    #[error("Message is too big")]
    TooBig,
    #[error("Cannot be parsed")]
    NotParsed,
    #[error("Message is too small")]
    NotEnough,
    #[error("Message is empty")]
    BufferEmpty,
    #[error("Unknown protocol")]
    UnknownProtocol,
    #[error("Unsupported protocol version")]
    UnsupportedVersion,
    #[error("Unsupported message type")]
    UnknownMessageType,
}
