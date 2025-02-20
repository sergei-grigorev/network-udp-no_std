use std::{
    io::{self},
    net::UdpSocket,
};

use shared_lib::{
    command::{EncodedCommand, Information, COMMAND_SIZE, PACKET_SIZE},
    make_new_command,
    network::{MessageType, PackedHeader},
    parse_command, serialize,
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

    let mut handshake_buf = [0u8; COMMAND_SIZE];
    let handshake_buf_size = initiator
        .write_message(&[], &mut handshake_buf)
        .expect("Failed to write handshake message");

    // make new request
    let device_id: u32 = 1234567890;

    let mut buf = [0u8; PACKET_SIZE];
    let handshake_command = EncodedCommand {
        size: handshake_buf_size,
        buf: handshake_buf,
    };
    let handshake_size = make_new_command(
        MessageType::HandshakeRequest,
        &handshake_command,
        &mut buf,
        device_id,
        0,
        1,
        0,
    )
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

    let mut read_buf = [0u8; COMMAND_SIZE];
    initiator
        .read_message(&server_body.buf[..server_body.size], &mut read_buf)
        .expect("Failed to read handshake message");

    let mut noise = initiator
        .into_transport_mode()
        .expect("Failed to enter transport mode");

    // prepare temperature information
    let mut tmp_buf = [0u8; COMMAND_SIZE];
    let information = Information::Temparature(25f32);
    let inf_size = usize::from(
        serialize::write_non_encrypted(&information, &mut tmp_buf)
            .expect("Failed to serialize temperature information"),
    );

    // encrypt message
    let mut enc_buf = [0u8; COMMAND_SIZE];
    let enc_size = noise
        .write_message(&tmp_buf[..inf_size], &mut enc_buf)
        .expect("Failed to write message");

    // send temperature information
    let temp_command = EncodedCommand {
        size: enc_size,
        buf: enc_buf,
    };
    let temp_size = make_new_command(
        MessageType::EncryptedMessage,
        &temp_command,
        &mut buf,
        hrh.device_id,
        hrh.session_id,
        2,
        hrh.sequence,
    )
    .expect("Failed to make temperature request");

    socket
        .send(&buf[..temp_size])
        .expect("Failed to send temperature information");

    // Receive the server's acknowledgement
    let n = socket.recv(&mut buf)?;

    // parse header
    let hrh = PackedHeader::try_deserialize(&buf[..n])
        .expect("Failed to deserialize ack response header");
    log::info!("Ack response header: {:?}", hrh);
    let server_body =
        parse_command(&buf[PackedHeader::SIZE..n]).expect("Failed to parse ack response body");
    log::info!("Ack response body: {:?}", server_body);
    // let session_id = hrh.session_id;

    assert_eq!(hrh.message_type, MessageType::Ack);
    assert_eq!(hrh.device_id, device_id);
    assert_ne!(hrh.session_id, 0);
    assert_eq!(hrh.ack, 2);

    Ok(())
}
