use std::{net::UdpSocket, time::Duration};

use heapless::Vec;
use shared_lib::command::PACKET_SIZE;

const SERVER_ADDR: &str = "127.0.0.1:8080";

mod channel;

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

    // send and wait for the server's response
    let mut read_buf: Vec<u8, PACKET_SIZE> = Vec::new();
    channel::send_and_wait(
        &socket,
        &handshake_init,
        1,
        &mut read_buf,
        Duration::from_secs(1),
        5,
    )?;

    let (hrh, body) = client::parse_request(&read_buf).expect("Failed to parse ack");
    client
        .receive_handshake(hrh, &body)
        .expect("Failed to process received handshake");

    // prepare the temperature request
    log::info!("Sending encrypted temperature request");
    let temperature = client
        .temperature_message()
        .expect("Failed to create temperature message");

    // send and wait for the server's response
    channel::send_and_wait(
        &socket,
        temperature.as_slice(),
        2,
        &mut read_buf,
        Duration::from_secs(1),
        5,
    )?;

    // wait for acknowledgement
    let (hrh, body) = client::parse_request(&read_buf).expect("Failed to parse ack");
    client
        .receive_ack(hrh, &body)
        .expect("Failed to process received ack");

    log::info!("Received ack, close connection");
    Ok(())
}
