use std::{
    io::{self},
    net::UdpSocket,
};

use shared_lib::{
    command::{NetworkCommand, BUFFER_SIZE, MESSAGE_SIZE},
    make_new_command,
    network::{MessageType, PackedHeader},
    parse_command,
};

const SERVER_ADDR: &str = "127.0.0.1:8080";
const ENC_PATTERN: &str = "Noise_NN_25519_ChaChaPoly_BLAKE2s";

fn main() -> io::Result<()> {
    env_logger::init();

    // Bind to a random (ephemeral) port on localhost
    let local_addr = "127.0.0.1:0";
    let socket = UdpSocket::bind(local_addr)?;
    log::info!("Client bound to {}", socket.local_addr()?);

    // Connect this socket to the server address
    socket.connect(SERVER_ADDR)?;

    let mut initiator = snow::Builder::new(ENC_PATTERN.parse().unwrap())
        .build_initiator()
        .expect("Failed to build initiator");

    let mut handshake_buf = [0u8; BUFFER_SIZE];
    let handshake_buf_size = initiator
        .write_message(&[], &mut handshake_buf)
        .expect("Failed to write handshake message");

    // make new request
    let device_id: u32 = 1234567890;

    let mut buf = [0u8; MESSAGE_SIZE];
    let handshake_command = NetworkCommand::HandhakeInit {
        size: handshake_buf_size,
        buf: handshake_buf,
    };
    let handshake_size = make_new_command(&handshake_command, &mut buf, device_id, 0, 1, 0)
        .expect("Failed to make handshake command");
    socket
        .send(&buf[..handshake_size])
        .expect("Failed to send handshake command");

    // Receive the server's acknowledgement
    let n = socket.recv(&mut buf)?;

    // parse header
    let hrh = PackedHeader::try_deserialize(&buf[..n])
        .expect("Failed to deserialize handshake response header");
    log::info!("Handshake response header: {:?}", hrh);
    let server_body = parse_command(&buf[PackedHeader::SIZE..n])
        .expect("Failed to parse handshake response body");
    log::info!("Handshake response body: {:?}", server_body);
    // let session_id = hrh.session_id;

    assert_eq!(hrh.message_type, MessageType::HandshakeResponse);
    assert_eq!(hrh.device_id, device_id);
    assert_ne!(hrh.session_id, 0);
    assert_eq!(hrh.ack, 1);

    // send temperature information
    // let temp_command = NetworkCommand::EncryptedMessage(Request::Temparature(28f32));
    // let temp_size = make_new_command(
    //     &temp_command,
    //     &mut buf,
    //     device_id,
    //     session_id,
    //     2,
    //     hrh.sequence,
    // )
    // .expect("Failed to make temperature request");

    // socket
    //     .send(&buf[..temp_size])
    //     .expect("Failed to send temperature information");

    Ok(())
}
