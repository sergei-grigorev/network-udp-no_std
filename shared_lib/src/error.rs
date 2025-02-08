use core::fmt::Debug;

#[derive(Debug)]
pub enum SerializeError {
    TooBig,
    NotParsed,
    NotEnough,
    BufferEmpty,
    UnknownProtocol,
    UnsupportedVersion,
    UnknownMessageType
}
