use musli::{Decode, Encode};

use crate::network::MessageType;

#[derive(Encode, Decode, Debug)]
pub enum Request {
    Temparature(f32),
    AirPressure(f32)
}

#[derive(Encode, Decode, Debug)]
pub enum NetworkCommand {
    HandhakeInit,
    HandshakeResponse,
}

impl Into<MessageType> for &NetworkCommand {
    fn into(self) -> MessageType {
        match self {
            NetworkCommand::HandhakeInit => MessageType::HandshakeRequest,
            NetworkCommand::HandshakeResponse => MessageType::HandshakeResponse,
        }
    }
}
