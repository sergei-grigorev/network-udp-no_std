use std::{net::UdpSocket, time::Duration};

use heapless::Vec;
use shared_lib::{command::PACKET_SIZE, network::PackedHeader};

/// Send a message and wait for a response.
pub fn send_and_wait(
    socket: &UdpSocket,
    write_buf: &[u8],
    seq_num: u16,
    read_buf: &mut Vec<u8, PACKET_SIZE>,
    timeout: Duration,
    max_retries: usize,
) -> std::io::Result<()> {
    let mut attempt = 0;

    loop {
        // Send the message
        socket.send(write_buf)?;
        socket.set_read_timeout(Some(timeout))?;

        // Wait for a response
        read_buf.clear();
        let _ = read_buf.resize_default(PACKET_SIZE);

        match socket.recv(read_buf.as_mut_slice()) {
            Ok(n) => {
                if let Ok(header) = PackedHeader::try_deserialize(&read_buf[..n]) {
                    if header.ack == seq_num {
                        read_buf.truncate(n);
                        return Ok(());
                    } else {
                        log::warn!("Received unexpected ack: {}", header.ack);
                    }
                } else {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Failed to parse header",
                    ));
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                log::warn!("Attempt {} timed out", attempt + 1);
            }
            Err(e) => {
                return Err(e);
            }
        }

        attempt += 1;

        if attempt >= max_retries {
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "Max retries reached",
            ));
        }
    }
}
