use std::time::Instant;
use tokio::sync::mpsc::{self, Sender};

use std::net::SocketAddr;

use shared_lib::{
    command::{EncodedCommand, COMMAND_SIZE, PACKET_SIZE},
    network::{MessageType, PackedHeader},
    write_command,
};
use tracing::{info_span, Instrument};

use super::Response;

mod handler;

const ENC_PATTERN: &str = "Noise_NN_25519_ChaChaPoly_BLAKE2s";
const QUEUE_SIZE: usize = 10;

pub struct Session {
    pub last_sequence_id: u16,
    pub last_timestamp: Instant,
    pub channel: Sender<ChannelMessage>,
}

struct SessionState {
    device_id: u32,
    session_id: u16,
    sequnce_id: u16,
    last_ack_id: u16,
    receiver: mpsc::Receiver<ChannelMessage>,
    response_queue: Sender<Response>,
    snow_state: SnowState,
    last_response: Option<Response>,
}

enum SnowState {
    // no state
    None,
    // waiting for handshake
    Handshake(Box<snow::HandshakeState>),
    // ready to send and receive messages
    Transport(snow::StatelessTransportState),
}

impl Session {
    /// Create a new session with the given session ID.
    /// It starts a new async task to handle messages for this session.
    pub fn spawn_new(
        device_id: u32,
        session_id: u16,
        response_queue: Sender<Response>,
    ) -> Sender<ChannelMessage> {
        let (sender, receiver) = mpsc::channel::<ChannelMessage>(QUEUE_SIZE);

        // start new async task to handle messages
        tokio::spawn(async move {
            let mut session_state = SessionState {
                device_id,
                session_id,
                sequnce_id: 0,
                last_ack_id: 0,
                receiver,
                response_queue,
                snow_state: SnowState::Handshake(Box::new(
                    snow::Builder::new(ENC_PATTERN.parse().unwrap())
                        .build_responder()
                        .expect("Failed to build initiator"),
                )),
                last_response: None,
            };

            session_state.run_loop().await;
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

impl SessionState {
    async fn run_loop(&mut self) {
        loop {
            if let Some(ChannelMessage { addr, header, body }) = self.receiver.recv().await {
                if self.last_ack_id < header.sequence {
                    // increase sequence id for the future response
                    self.sequnce_id += 1;
                    self.last_ack_id = header.sequence;

                    // handle the message
                    let socket_src = addr.to_string();
                    let span = info_span!("handle_message", remote = socket_src);

                    let request_sequence_id = header.sequence;

                    let result = handler::process(self, header, &body)
                        .instrument(span.clone())
                        .await;

                    // if message is processed successfully, send response back
                    // otherwise, send an error
                    let (response_type, response) = match result {
                        Ok(success) => (success.message_type, success.command),
                        Err(error) => {
                            span.in_scope(|| {
                                log::error!("Failed to process message: {:?}", error);
                            });
                            (
                                MessageType::Error,
                                EncodedCommand {
                                    size: 0,
                                    buf: [0u8; COMMAND_SIZE],
                                },
                            )
                        }
                    };

                    // try to serialize
                    let header = PackedHeader::new(
                        response_type,
                        self.device_id,
                        self.session_id,
                        self.last_ack_id,
                        request_sequence_id,
                    );
                    let mut buf = [0u8; PACKET_SIZE];
                    match write_command(&header, &response, &mut buf) {
                        Ok(content_size) => {
                            let mut content: Vec<u8> = Vec::with_capacity(content_size);
                            content.extend_from_slice(&buf[..content_size]);

                            // copy response for the future resend
                            self.last_response = Some(Response {
                                addr,
                                buf: content.clone(),
                                session_id: self.session_id,
                                ack_id: header.sequence,
                            });

                            // send response back to the client
                            if let Err(err) = self
                                .response_queue
                                .send(Response {
                                    addr,
                                    buf: content,
                                    session_id: self.session_id,
                                    ack_id: header.sequence,
                                })
                                .await
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
                } else if let Some(last_response) = self.last_response.as_ref() {
                    // check if the previous addr is the same
                    if last_response.addr == addr {
                        if last_response.ack_id == header.sequence {
                            // resend the last message
                            if let Err(err) = self.response_queue.send(last_response.clone()).await
                            {
                                log::error!(
                                    "Failed to send response, server might be stopped: {err}"
                                );
                                // close the session
                                break;
                            }
                        } else {
                            // nothing to do, just ignore
                            log::info!(
                                "Received old message, ignored: {} (sequence: {})",
                                self.session_id,
                                header.sequence
                            );
                        }
                    } else {
                        log::warn!("Security issue: requested duplicate the last message from another addr [orig: {}, new: {}]", self.last_response.as_ref().unwrap().addr, addr);
                        break;
                    }
                } else {
                    log::warn!("Security issue: requested duplicate the last message, but it is not available");
                    break;
                }
            } else {
                // handle timeout
                log::info!(
                    "Session [{}], device [{}] timed out",
                    self.session_id,
                    self.device_id
                );
                break;
            }
        }
    }

    fn make_transport_mode(&mut self) -> Result<bool, snow::Error> {
        match self.snow_state.take() {
            SnowState::Handshake(handshake) => {
                let transport_state = handshake.into_stateless_transport_mode()?;
                self.snow_state = SnowState::Transport(transport_state);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

impl SnowState {
    /// Take the current state and replace it with `None`.
    fn take(&mut self) -> SnowState {
        std::mem::replace(self, SnowState::None)
    }
}
