use shared_lib::{
    command::{NetworkCommand, BUFFER_SIZE},
    error::SerializeError,
    network::{MessageType, PackedHeader},
    parse_command,
};
use thiserror::Error;
use tracing::instrument;

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Unexpected message type: {0:?}")]
    NotExpectedMessage(MessageType),
    #[error("Not yet supported message type: {0:?}")]
    NotImplemented(MessageType),
    #[error("Unexpected session starting message session_id [{session_id}] (seq: {seq})")]
    IncorrectHandshake { session_id: u16, seq: u16 },
    #[error("Message is corrupted and cannot be parsed: {0}")]
    MessageCorrupted(#[from] SerializeError),
}

#[instrument(skip(header, body), fields(seq=header.sequence, device=header.device_id))]
pub async fn process(
    session_id: u16,
    header: PackedHeader,
    body: &[u8],
) -> Result<NetworkCommand, ProcessingError> {
    log::info!("Received message: {:?}", header);

    match header.message_type {
        MessageType::HandshakeRequest => {
            if header.session_id != 0 || header.sequence != 1 {
                return Err(ProcessingError::IncorrectHandshake {
                    session_id: header.session_id,
                    seq: header.sequence,
                });
            }

            let handshake_body = parse_command(&&body)?;
            log::info!("Handshake body: {:?}", handshake_body);

            // generate key and write back
            Ok(NetworkCommand::HandshakeResponse {
                size: 0,
                buf: [0u8; BUFFER_SIZE],
            })
        }
        MessageType::HandshakeResponse => {
            Err(ProcessingError::NotExpectedMessage(header.message_type))
        }
        MessageType::EncryptedMessage => Err(ProcessingError::NotImplemented(header.message_type)),
        MessageType::Ack => Err(ProcessingError::NotImplemented(header.message_type)),
        MessageType::Error => Err(ProcessingError::NotImplemented(header.message_type)),
    }
}

// loop {
//     // let mut responder = snow::Builder::new(ENC_PATTERN.parse().unwrap())
//     //     .build_responder()
//     //     .expect("Failed to build responder");

//     // the first one should be handshake request

//     // let NetworkCommand::HandhakeInit {
//     //     size: key_size,
//     //     buf: key,
//     // } = handshake_body
//     // else {
//     //     return todo!();
//     // };

//     // responder
//     //     .read_message(&key[..key_size], &mut read_buf)
//     //     .await?;

//     // parse the handshake request body
//     let handshake_body = parse_command(&buf[PackedHeader::SIZE..amt])
//         .expect("Failed to parse handshake request body");
//     span.in_scope(|| {
//         log::info!("Handshake body: {:?}", handshake_body);
//     });

//     // responder.read_message(handshake_body)

//     let session_id: u16 = 100;

//     let span = span!(
//         Level::INFO,
//         "main",
//         session_id = session_id,
//         addr = socket_src
//     );

//     // generate key and write back
//     let handshake_response_size = make_new_command(
//         &NetworkCommand::HandshakeResponse {
//             size: 0,
//             buf: [0u8; 1024],
//         },
//         &mut buf,
//         handshake_request.device_id,
//         session_id,
//         1,
//         handshake_request.sequence,
//     )
//     .expect("Failed to generate handshake response");

//     socket.send_to(&buf[..handshake_response_size], src).await?;

//     // todo: receive information about the temparature
//     let (amt, src) = socket.recv_from(&mut buf).await?;
//     span.in_scope(|| {
//         log::info!("Received new message from {}", src);
//     });

//     // parse what that command is
//     let header = PackedHeader::try_deserialize(&buf).expect("Failed to deserialize header");
//     assert_eq!(header.device_id, handshake_request.device_id);
//     assert_eq!(header.session_id, session_id);
//     assert_eq!(header.sequence, 2);
//     assert!(header.message_type == MessageType::EncryptedMessage);

//     let command_body =
//         parse_command(&buf[PackedHeader::SIZE..amt]).expect("Failed to parse command body");

//     span.in_scope(|| {
//         if let NetworkCommand::DeviceRequest(Request::Temparature(temp)) = command_body {
//             log::info!("Received temperature: {}", temp);
//         } else {
//             log::info!("Received unknown command: {:?}", command_body);
//         }
//     });
