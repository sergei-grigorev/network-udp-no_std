use std::{collections::HashMap, net::SocketAddr};

use shared_lib::command::MESSAGE_SIZE;
use state::State;
use tokio::{
    net::UdpSocket,
    select,
    sync::mpsc::{self},
};
use tracing::{span, Instrument, Level};

mod session;
mod state;

// const ENC_PATTERN: &str = "Noise_NN_25519_ChaChaPoly_BLAKE2s";

struct Response {
    addr: SocketAddr,
    buf: Vec<u8>,
}

pub async fn start_server(addr: &str) -> std::io::Result<()> {
    let socket = UdpSocket::bind(addr).await?;

    let (sender, mut receiver) = mpsc::channel::<Response>(10);

    // server state
    let mut state: State = State::new(sender);
    let mut buf = [0u8; MESSAGE_SIZE];

    // todo: run some time to periodical cleanup
    // todo: add graceful shutdown
    let mut is_shutdown = false;
    while !is_shutdown {
        let input_queue = socket.recv_from(&mut buf);
        let output_queue = receiver.recv();

        select! {
            socket_received = input_queue => {
                match socket_received {
                    Ok((amt, addr)) => {
                        let socket_span = span!(Level::INFO, "udp_server", addr = addr.to_string());
                        let _ = state.process_received_message(&buf, amt, addr).instrument(socket_span).await;
                    }
                    Err(err) => {
                        log::error!("Failed to receive message: {:?}", err);
                    }
                }
            },
            output_message = output_queue => {
                if let Some(output_message) = output_message {
                    let socket_span = span!(Level::INFO, "udp_server", addr = output_message.addr.to_string());
                    let _ = send_response(&socket, output_message).instrument(socket_span).await;
                } else {
                    log::warn!("Response queue has been stopped");
                    is_shutdown = true;
                }
            }
        };
    }

    Ok(())
}

/// Send a response to a socket. Returns an error if the message could not be sent.
/// todo: check if it was delivered
async fn send_response(socket: &UdpSocket, output_message: Response) -> std::io::Result<()> {
    log::info!("Sending response to {}", output_message.addr);

    if let Err(error) = socket
        .send_to(&output_message.buf, output_message.addr)
        .await
    {
        log::error!("Failed to send message: {:?}", error);
        Err(error)
    } else {
        Ok(())
    }
}
