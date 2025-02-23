use std::net::UdpSocket;

use shared_lib::command::PACKET_SIZE;

const SERVER_ADDR: &str = "127.0.0.1:8080";

fn main() -> std::io::Result<()> {
    env_logger::init();

    // Bind to a random (ephemeral) port on localhost
    let local_addr = "127.0.0.1:0";
    let socket = UdpSocket::bind(local_addr)?;
    log::info!("Client bound to {}", socket.local_addr()?);

    // Connect this socket to the server address
    socket.connect(SERVER_ADDR)?;

    // run the client (no_std)
    let device_id: u32 = 1234567890;
    let mut client = client::Session::new(device_id);

    let handshake_init = client
        .initiate_handshake()
        .expect("Failed to initiate handshake");

    socket.send(handshake_init.as_slice())?;

    // wait for the server's response
    let mut buf = [0u8; PACKET_SIZE];
    let n = socket.recv(&mut buf)?;
    let (hrh, body) = client::parse_request(&buf[..n]).expect("Failed to parse ack");
    client
        .receive_handshake(hrh, &body)
        .expect("Failed to process received handshake");

    // prepare the temperature request
    log::info!("Sending encrypted temperature request");
    let temperature = client
        .temperature_message()
        .expect("Failed to create temperature message");

    socket.send(temperature.as_slice())?;

    // wait for acknowledgement
    let n = socket.recv(&mut buf)?;
    let (hrh, body) = client::parse_request(&buf[..n]).expect("Failed to parse ack");
    client
        .receive_ack(hrh, &body)
        .expect("Failed to process received ack");

    log::info!("Received ack, close connection");
    Ok(())
}
