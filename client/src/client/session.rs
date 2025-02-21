use heapless::Vec;
use shared_lib::{
    command::{EncodedCommand, Information, COMMAND_SIZE, PACKET_SIZE},
    make_new_command,
    network::{MessageType, PackedHeader},
    parse_command, serialize,
};
use thiserror::Error;

const ENC_PATTERN: &str = "Noise_NN_25519_ChaChaPoly_BLAKE2s";

type OutputVec = Vec<u8, PACKET_SIZE>;
pub type Result<T> = core::result::Result<T, Error>;

pub struct Session {
    device_id: u32,
    session_id: u16,
    last_server_message_id: u16,
    sequence_id: u16,
    snow_state: Noise,
}

/// The current state of the session.
#[allow(clippy::large_enum_variant)]
enum Noise {
    None,
    HandshakeState(snow::HandshakeState),
    TransportState(snow::TransportState),
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Encryption error: {0}")]
    Encryption(#[from] snow::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] shared_lib::error::SerializeError),
    #[error("Incorrect state")]
    IncorrectState,
}

impl Session {
    pub fn new(device_id: u32) -> Self {
        Session {
            device_id,
            session_id: 0,
            last_server_message_id: 0,
            sequence_id: 0,
            snow_state: Noise::None,
        }
    }

    pub fn initiate_handshake(&mut self) -> Result<OutputVec> {
        self.sequence_id += 1;

        let mut initiator = snow::Builder::new(ENC_PATTERN.parse().unwrap()).build_initiator()?;

        // create new handshake
        let mut handshake_buf = [0u8; COMMAND_SIZE];
        let handshake_buf_size = initiator.write_message(&[], &mut handshake_buf)?;

        let handshake_command = EncodedCommand {
            size: handshake_buf_size,
            buf: handshake_buf,
        };

        // serialize that into message
        let mut output_vec = OutputVec::new();
        let _ = output_vec.resize_default(PACKET_SIZE);
        let handshake_size = make_new_command(
            MessageType::HandshakeRequest,
            &handshake_command,
            output_vec.as_mut_slice(),
            self.device_id,
            0,
            1,
            0,
        )?;

        self.snow_state = Noise::HandshakeState(initiator);

        output_vec.truncate(handshake_size);
        Ok(output_vec)
    }

    pub fn receive_handshake(&mut self, buf: &[u8]) -> Result<()> {
        // parse header
        let hrh = PackedHeader::try_deserialize(buf)?;

        log::info!("Handshake response header: {:?}", hrh);
        let server_body = parse_command(&buf[PackedHeader::SIZE..])?;
        log::info!("Handshake response body: {:?}", server_body);

        self.session_id = hrh.session_id;
        self.last_server_message_id = hrh.sequence;

        assert_eq!(hrh.message_type, MessageType::HandshakeResponse);
        assert_eq!(hrh.device_id, self.device_id);
        assert_ne!(hrh.session_id, 0);
        assert_eq!(hrh.ack, 1);

        let mut read_buf = [0u8; COMMAND_SIZE];
        let noise_state = self.snow_state.take();
        // read handshake message and finish handshake
        if let Noise::HandshakeState(mut initiator) = noise_state {
            initiator.read_message(&server_body.buf[..server_body.size], &mut read_buf)?;
            self.snow_state = Noise::TransportState(initiator.into_transport_mode()?);
            Ok(())
        } else {
            Err(Error::IncorrectState)
        }
    }

    pub fn temperature_message(&mut self) -> Result<OutputVec> {
        if let Noise::TransportState(ref mut noise) = self.snow_state {
            self.sequence_id += 1;
            // prepare temperature information
            let mut tmp_buf = [0u8; COMMAND_SIZE];
            let information = Information::Temparature(25f32);
            let inf_size = usize::from(serialize::write_non_encrypted(&information, &mut tmp_buf)?);

            // encrypt message
            let mut enc_buf = [0u8; COMMAND_SIZE];
            let enc_size = noise.write_message(&tmp_buf[..inf_size], &mut enc_buf)?;

            // send temperature information
            let temp_command = EncodedCommand {
                size: enc_size,
                buf: enc_buf,
            };

            let mut output_vec = OutputVec::new();
            let _ = output_vec.resize_default(PACKET_SIZE);
            let temperature_size = make_new_command(
                MessageType::EncryptedMessage,
                &temp_command,
                output_vec.as_mut_slice(),
                self.device_id,
                self.session_id,
                self.sequence_id,
                0,
            )?;

            output_vec.truncate(temperature_size);
            Ok(output_vec)
        } else {
            Err(Error::IncorrectState)
        }
    }

    pub fn receive_ack(&mut self, buf: &[u8]) -> Result<()> {
        // parse header
        let hrh = PackedHeader::try_deserialize(buf)?;

        assert_eq!(hrh.message_type, MessageType::Ack);
        assert_eq!(hrh.device_id, self.device_id);
        assert_eq!(hrh.session_id, self.session_id);
        assert_eq!(hrh.ack, self.sequence_id);

        self.last_server_message_id = hrh.sequence;
        Ok(())
    }
}

impl Noise {
    fn take(&mut self) -> Self {
        core::mem::replace(self, Noise::None)
    }
}
