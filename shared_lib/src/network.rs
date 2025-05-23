use byteorder::{ByteOrder, NetworkEndian};

use crate::error::SerializeError;

/// Network message types.
/// 1 - HandshakeRequest
/// 2 - HandshareResponse
/// 3 - EncryptedMessage
/// 4 - Ack
/// 5 - Timeout (session expired)
/// FF - Error
#[derive(PartialEq, Clone, Debug)]
pub enum MessageType {
    HandshakeRequest,
    HandshakeResponse,
    EncryptedMessage,
    Ack,
    Timeout,
    Error,
}

/// Network header structure.
/// Total size - 14 bytes for each packet.
/// Additional padding 2 bytes
#[derive(PartialEq, Debug)]
pub struct PackedHeader {
    // Protocol ID 2 bytes / 0-2
    protocol_id: u16,
    // Protocol version 1 byte / 2-3
    version: u8,
    // Message Type 1 byte / 3-4
    pub message_type: MessageType,
    // Unique device id 4 bytes / 4-8
    pub device_id: u32,
    // Unique session ID 2 bytes / 8-10
    pub session_id: u16,
    // Session message sequence 2 bytes / 10-12
    pub sequence: u16,
    // The most recent received sequence 2 bytes / 12-14
    pub ack: u16,
}

impl TryFrom<u8> for MessageType {
    type Error = SerializeError;

    fn try_from(value: u8) -> Result<Self, SerializeError> {
        match value {
            1 => Ok(Self::HandshakeRequest),
            2 => Ok(Self::HandshakeResponse),
            3 => Ok(Self::EncryptedMessage),
            4 => Ok(Self::Ack),
            5 => Ok(Self::Timeout),
            0xFF => Ok(Self::Error),
            _ => Err(SerializeError::UnknownMessageType),
        }
    }
}

impl From<MessageType> for u8 {
    fn from(val: MessageType) -> Self {
        match val {
            MessageType::HandshakeRequest => 1,
            MessageType::HandshakeResponse => 2,
            MessageType::EncryptedMessage => 3,
            MessageType::Ack => 4,
            MessageType::Timeout => 5,
            MessageType::Error => 0xFF,
        }
    }
}

impl PackedHeader {
    pub const SIZE: usize = 14;
    const PROTOCOL_ID: u16 = 0xDEFA;
    const VERSION: u8 = 1;

    pub fn new(
        message_type: MessageType,
        device_id: u32,
        session_id: u16,
        sequence: u16,
        ack: u16,
    ) -> Self {
        PackedHeader {
            protocol_id: Self::PROTOCOL_ID,
            version: 1,
            message_type,
            device_id,
            session_id,
            sequence,
            ack,
        }
    }

    /// Serialize the header into the buffer. Returns the number of bytes written.
    /// If the buffer is too small, function returns an error. Buffer recommended size is bigger or equal [PackedHeader::SIZE].
    pub fn serialize_info(&self, buf: &mut [u8]) -> Result<usize, SerializeError> {
        if buf.len() < PackedHeader::SIZE {
            return Err(SerializeError::NotEnough);
        }

        NetworkEndian::write_u16(&mut buf[0..2], self.protocol_id);
        buf[2] = self.version;
        buf[3] = self.message_type.clone().into();
        NetworkEndian::write_u32(&mut buf[4..8], self.device_id);
        NetworkEndian::write_u16(&mut buf[8..10], self.session_id);
        NetworkEndian::write_u16(&mut buf[10..12], self.sequence);
        NetworkEndian::write_u16(&mut buf[12..14], self.ack);

        Ok(PackedHeader::SIZE)
    }

    pub fn try_deserialize(buf: &[u8]) -> Result<Self, SerializeError> {
        if buf.len() >= PackedHeader::SIZE {
            let protocol_id: u16 = NetworkEndian::read_u16(&buf[0..2]);
            if protocol_id != Self::PROTOCOL_ID {
                return Err(SerializeError::UnknownProtocol);
            }

            let version: u8 = buf[2];
            if version != Self::VERSION {
                return Err(SerializeError::UnsupportedVersion);
            }

            let message_type: MessageType = MessageType::try_from(buf[3])?;
            let device_id: u32 = NetworkEndian::read_u32(&buf[4..8]);
            let session_id: u16 = NetworkEndian::read_u16(&buf[8..10]);
            let sequence: u16 = NetworkEndian::read_u16(&buf[10..12]);
            let ack: u16 = NetworkEndian::read_u16(&buf[12..14]);

            Ok(PackedHeader {
                protocol_id,
                version,
                message_type,
                device_id,
                session_id,
                sequence,
                ack,
            })
        } else {
            Err(SerializeError::NotEnough)
        }
    }

    /// Get nonce from session id and sequence.
    pub fn nonce(&self) -> u64 {
        ((self.session_id as u64) << 32) | (self.sequence as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_deserialize_header() {
        let mut buf = [0u8; PackedHeader::SIZE];

        let header = PackedHeader::new(MessageType::HandshakeRequest, 1234567890, 100, 200, 150);
        let _ = header
            .serialize_info(&mut buf)
            .expect("Failed to serialize header");

        // try to deserialize now
        let deserialized_header = PackedHeader::try_deserialize(&buf).unwrap();
        assert_eq!(deserialized_header, header);
    }
}
