use std::{io::Error, net::UdpSocket};

use shared_lib::{make_ack, parse_request};

const SERVER_ADDR: &str = "127.0.0.1:8080";

fn main() -> Result<(), Error> {
    env_logger::init();

    let socket = UdpSocket::bind(SERVER_ADDR)?;
    log::info!("Server listening on {}", SERVER_ADDR);

    let mut buf = [0u8; 1500];
    loop {
        // Block until we receive a message
        let (amt, src) = socket.recv_from(&mut buf)?;

        match parse_request(&buf[..amt]) {
            Ok(parsed) => {
                log::info!("Client sent: {:?}", parsed);
                let ack_size = make_ack(&mut buf, parsed.device_id, parsed.message_number);

                // Send an ACK back to the client
                socket.send_to(&buf[..ack_size], src)?;
            }
            Err(err) => log::error!("Error parsing request: {:#?}", err),
        }
    }
}
