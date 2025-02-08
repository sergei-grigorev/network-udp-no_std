use std::{io::Error, net::UdpSocket};

use shared_lib::{
    command::{NetworkCommand, Request},
    make_new_command,
    network::{MessageType, PackedHeader},
    parse_command, parse_payload,
};
use tracing::{event, info_span, span, Level};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

const SERVER_ADDR: &str = "127.0.0.1:8080";

fn main() -> Result<(), Error> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let socket = UdpSocket::bind(SERVER_ADDR)?;
    log::info!("Server listening on {}", SERVER_ADDR);

    let mut buf = [0u8; 1500];
    loop {
        // Block until we receive a message
        let (amt, src) = socket.recv_from(&mut buf)?;
        let socket_src = src.to_string();
        let span = span!(Level::INFO, "main", addr = socket_src);
        let _enter = span.enter();

        log::info!("Received message");

        // the first one should be handshake request
        let handshake_request =
            PackedHeader::try_deserialize(&buf).expect("Failed to deserialize handshake request");
        assert_eq!(
            handshake_request.message_type,
            MessageType::HandshakeRequest
        );
        assert_eq!(handshake_request.session_id, 0);
        assert_eq!(handshake_request.sequence, 1);

        let handshake_body = parse_command(&buf[PackedHeader::SIZE..amt])
            .expect("Failed to parse handshake request body");
        log::info!("Handshake body: {:?}", handshake_body);

        let session_id: u16 = 100;

        // reset span
        drop(_enter);
        drop(span);

        let span = span!(
            Level::INFO,
            "main",
            session_id = session_id,
            addr = socket_src
        );

        let _enter = span.enter();

        // generate key and write back
        let handshake_response_size = make_new_command(
            &NetworkCommand::HandshakeResponse,
            &mut buf,
            handshake_request.device_id,
            session_id,
            1,
            handshake_request.sequence,
        )
        .expect("Failed to generate handshake response");

        socket.send_to(&buf[..handshake_response_size], src)?;

        // todo: receive information about the temparature
        let (amt, src) = socket.recv_from(&mut buf)?;
        log::info!("Received new message from {}", src);

        // parse what that command is
        let header = PackedHeader::try_deserialize(&buf).expect("Failed to deserialize header");
        assert_eq!(header.device_id, handshake_request.device_id);
        assert_eq!(header.session_id, session_id);
        assert_eq!(header.sequence, 2);
        assert!(header.message_type == MessageType::EncryptedMessage);

        let command_body =
            parse_payload(&buf[PackedHeader::SIZE..amt]).expect("Failed to parse command body");
        if let Request::Temparature(temp) = command_body {
            log::info!("Received temperature: {}", temp);
        } else {
            log::info!("Received unknown command: {:?}", command_body);
        }
    }
}
