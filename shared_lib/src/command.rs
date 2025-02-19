use core::fmt::Debug;

use musli::{Decode, Encode};

use crate::network::MessageType;

pub const BUFFER_SIZE: usize = 1400;
pub type Buffer = [u8; BUFFER_SIZE];

pub const MESSAGE_SIZE: usize = 1500;

#[derive(Encode, Decode, Debug)]
pub enum Request {
    Temparature(f32),
    AirPressure(f32),
}

#[derive(Encode, Decode)]
pub enum NetworkCommand {
    HandhakeInit { size: usize, buf: Buffer },
    HandshakeResponse { size: usize, buf: Buffer },
    EncryptedMessage { size: usize, buf: Buffer },
    Error,
}

impl Into<MessageType> for &NetworkCommand {
    fn into(self) -> MessageType {
        match self {
            NetworkCommand::HandhakeInit { size: _, buf: _ } => MessageType::HandshakeRequest,
            NetworkCommand::HandshakeResponse { size: _, buf: _ } => MessageType::HandshakeResponse,
            NetworkCommand::EncryptedMessage { size: _, buf: _ } => MessageType::EncryptedMessage,
            NetworkCommand::Error => MessageType::Error,
        }
    }
}

impl Debug for NetworkCommand {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            NetworkCommand::HandhakeInit { size, buf: _ } => f.write_fmt(format_args!(
                "HandshakeInit {{ size: {}, buf: <redundant> }}",
                size
            )),
            NetworkCommand::HandshakeResponse { size, buf: _ } => f.write_fmt(format_args!(
                "HandshakeResponse {{ size: {}, buf: <redundant> }}",
                size
            )),
            NetworkCommand::EncryptedMessage { size, buf: _ } => f.write_fmt(format_args!(
                "DeviceRequest {{ size: {}, buf: <redundant> }}",
                size
            )),
            NetworkCommand::Error => f.write_str("Error()"),
        }
    }
}
