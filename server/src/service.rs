use std::net::SocketAddr;

use shared_lib::command::PACKET_SIZE;
use state::State;
use tokio::{
    net::UdpSocket,
    select,
    sync::mpsc::{self},
};
use tracing::{span, Instrument, Level};

mod session;
mod state;

/// Cleanup interval in seconds.
/// This is used to remove inactive sessions.
const CLEANUP_INTERVAL: u64 = 5 * 60; // 5 minutes

/// Response to be sent to the client.
/// Contains the address of the client, session ID, ACK ID, and the message buffer.
#[derive(Clone)]
struct Response {
    addr: SocketAddr,
    session_id: u16,
    ack_id: u16,
    buf: Vec<u8>,
}

/// Start the server.
///
/// Returns a result indicating success or failure
///
/// **Arguments**
/// - `addr`: The address to bind the server to.
///
pub async fn start_server(addr: &str) -> std::io::Result<()> {
    // Try to open UDP socket
    let socket = UdpSocket::bind(addr).await?;
    log::info!("UDP server started on {}", addr);

    let (sender, mut receiver) = mpsc::channel::<Response>(10);

    // server state
    let mut state: State = State::new(sender);
    let mut buf = [0u8; PACKET_SIZE];

    // schedule cleanup task every 5 minutes
    let mut cleanup_interval =
        tokio::time::interval(std::time::Duration::from_secs(CLEANUP_INTERVAL));

    loop {
        let input_queue = socket.recv_from(&mut buf);
        let output_queue = receiver.recv();

        let cleanup_task = cleanup_interval.tick();

        // wait for either input or output
        select! {
            socket_received = input_queue => {
                match socket_received {
                    Ok((amt, addr)) => {
                        // randomly ignore it
                        if rand::random::<u8>() % 3 == 0 {
                            log::warn!("Ignoring message from {}", addr);
                            continue;
                        }
                        let socket_span = span!(Level::INFO, "udp_server", addr = addr.to_string());
                        // wait for the message to be processed (ignore errors)
                        let bytes = &buf[..amt];
                        let _ = state.process_received_message(bytes, addr).instrument(socket_span).await;
                    }
                    Err(err) => {
                        log::error!("Failed to receive message: {:?}", err);
                    }
                }
            },
            output_message = output_queue => {
                if let Some(output_message) = output_message {
                    let socket_span = span!(Level::INFO, "udp_server", addr = output_message.addr.to_string());
                    // wait for the message to be sent (ignore errors)
                    if rand::random::<u8>() % 3 == 0 {
                        log::warn!("Ignoring message to {}", output_message.addr);
                        continue;
                    }
                    let _ = send_response(&socket, output_message).instrument(socket_span).await;
                } else {
                    log::warn!("Response queue has been stopped");
                    // exit gracefully
                    break;
                }
            },
            _ = cleanup_task => {
                log::debug!("Run cleanup task");
                state.cleanup();
            },
            _ = tokio::signal::ctrl_c() => {
                log::info!("Received Ctrl-C, shutting down");
                break;
            }
        };
    }

    Ok(())
}

/// Send a response to the client.
///
/// Returns a result indicating success or failure
///
/// todo: check if it was delivered
/// **Arguments**
/// - `socket`: The socket to send the message to.
/// - `output_message`: The message to send.
#[tracing::instrument(skip(socket, output_message), fields(session_id=output_message.session_id, sequence_id=output_message.ack_id))]
async fn send_response(socket: &UdpSocket, output_message: Response) -> std::io::Result<()> {
    log::info!("Sending response to {}", output_message.addr);

    if let Err(error) = socket
        .send_to(&output_message.buf, output_message.addr)
        .await
    {
        // In case of error, log it and return it
        log::error!("Failed to send message: {:?}", error);
        Err(error)
    } else {
        Ok(())
    }
}
