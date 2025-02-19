use std::time::Instant;
use tokio::sync::mpsc::{self, Sender};

use std::net::SocketAddr;

use shared_lib::{
    command::{NetworkCommand, MESSAGE_SIZE},
    make_new_command,
    network::PackedHeader,
};
use tracing::{info_span, Instrument};

use super::Response;

mod handler;

pub struct Session {
    pub last_sequence_id: u16,
    pub last_timestamp: Instant,
    pub channel: Sender<ChannelMessage>,
}

impl Session {
    /// Create a new session with the given session ID.
    /// It starts a new async task to handle messages for this session.
    pub fn spawn_new(
        device_id: u32,
        session_id: u16,
        response_queue: Sender<Response>,
    ) -> Sender<ChannelMessage> {
        let (sender, mut receiver) = mpsc::channel::<ChannelMessage>(10);

        // start new async task to handle messages
        tokio::spawn(async move {
            let mut last_sequence_id: u16 = 0;
            loop {
                if let Some(ChannelMessage { addr, header, body }) = receiver.recv().await {
                    // increase sequence id for the future response
                    last_sequence_id = last_sequence_id + 1;

                    // handle the message
                    let socket_src = addr.to_string();
                    let span = info_span!("handle_message", remote = socket_src);

                    let request_sequence_id = header.sequence;

                    let result = handler::process(session_id, header, &body)
                        .instrument(span)
                        .await;

                    // if message is processed successfully, send response back
                    // otherwise, send an error
                    let response: NetworkCommand = match result {
                        Ok(result) => result,
                        Err(error) => {
                            log::error!("Failed to process message: {error}");
                            NetworkCommand::Error
                        }
                    };

                    // try to serialize
                    let mut buf = [0u8; MESSAGE_SIZE];
                    match make_new_command(
                        &response,
                        &mut buf,
                        device_id,
                        session_id,
                        last_sequence_id,
                        request_sequence_id,
                    ) {
                        Ok(content_size) => {
                            let mut content: Vec<u8> = Vec::with_capacity(content_size);
                            content.extend_from_slice(&buf[..content_size]);

                            // send response back to the client
                            if let Err(err) =
                                response_queue.send(Response { addr, buf: content }).await
                            {
                                log::error!(
                                    "Failed to send response, server might be stopped: {err}"
                                );
                                // close the session
                                break;
                            }
                        }
                        Err(error) => {
                            // response will not be send
                            log::error!("Failed to serialize response: {:?}", error);
                            // close the session
                            break;
                        }
                    }
                } else {
                    // handle timeout
                    log::info!("Session timed out");
                    break;
                }
            }
        });

        sender
    }
}

/// Session channel message types.
pub struct ChannelMessage {
    pub addr: SocketAddr,
    pub header: PackedHeader,
    pub body: Vec<u8>,
}
