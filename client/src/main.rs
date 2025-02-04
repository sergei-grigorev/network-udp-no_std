use std::{
    io::{self, Write},
    net::UdpSocket,
};

use shared_lib::{make_new_request, parse_ack};

const SERVER_ADDR: &str = "127.0.0.1:8080";

fn main() -> io::Result<()> {
    env_logger::init();

    // Bind to a random (ephemeral) port on localhost
    let local_addr = "127.0.0.1:0";
    let socket = UdpSocket::bind(local_addr)?;
    log::info!("Client bound to {}", socket.local_addr()?);

    // Connect this socket to the server address
    socket.connect(SERVER_ADDR)?;

    // make new request
    let device_id: u64 = 1234567890;
    let message_num: u64 = 1;

    let mut message_buf = [0u8; 1400];
    match make_new_request(&mut message_buf, device_id, message_num) {
        Ok(size) => {
            log::info!("Request size: {}", size);
            socket.send(&message_buf[..size])?;
        }
        Err(err) => log::error!("Error making new request: {:#?}", err),
    }

    // Receive the server's acknowledgement
    let n = socket.recv(&mut message_buf)?;

    match parse_ack(&message_buf[..n]) {
        Ok(ack) => {
            log::info!("Received ACK: {:#?}", ack);
        }
        Err(err) => log::error!("Error parsing ACK: {:#?}", err),
    }

    Ok(())
}
