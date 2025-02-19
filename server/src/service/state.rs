use std::{collections::HashMap, net::SocketAddr, time::Instant};

use shared_lib::{error::SerializeError, network::PackedHeader};
use thiserror::Error;
use tokio::sync::mpsc::Sender;

const SESSIONS_MAX_COUNT: usize = 100;

use super::{
    session::{self, Session},
    Response,
};

/// Server state. Contains the last session ID and a map of sessions.
pub struct State {
    pub sender: Sender<Response>,
    last_session_id: u16,
    sessions: HashMap<u16, Session>,
}

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Failed to deserialize header or body")]
    DeserializeFailed(#[from] SerializeError),
    #[error("Opened too many sessions: {current} / {limit}")]
    TooManySessions { current: usize, limit: usize },
    #[error("Sessions is not found or closed: {0}")]
    SessionsNotFound(u16),
    #[error("Session has been closed: {0}")]
    SessionClosed(u16),
}

impl State {
    pub fn new(sender: Sender<Response>) -> Self {
        Self {
            sender,
            last_session_id: 0,
            sessions: HashMap::new(),
        }
    }

    pub async fn process_received_message(
        &mut self,
        buf: &[u8],
        amt: usize,
        socket_addr: SocketAddr,
    ) -> Result<(), ProcessingError> {
        // try to parse if the message has the correct format
        log::info!("Received new message size: {}", amt);
        let (header, body) = parse_request(&buf, amt)?;

        if let Err(error) = self.process_message(socket_addr, header, body).await {
            log::error!("Failed to process message: {:?}", error);
        }
        Ok(())
    }
    async fn process_message(
        &mut self,
        addr: SocketAddr,
        header: PackedHeader,
        body: Vec<u8>,
    ) -> Result<(), ProcessingError> {
        let session_id = if header.session_id == 0 {
            // todo: restart if sessions are MAX_SIZE
            self.last_session_id = self.last_session_id + 1;

            let session_id = self.last_session_id;
            log::info!("Assign new session id: {}", session_id);

            let queue = Session::spawn_new(header.device_id, session_id, self.sender.clone());
            let new_session = Session {
                last_sequence_id: 0,
                last_timestamp: Instant::now(),
                channel: queue,
            };

            // prevent infinite grow, that is something unexpected
            if self.sessions.len() >= SESSIONS_MAX_COUNT {
                log::error!("Too many sessions opened: {}", self.sessions.len());
                return Err(ProcessingError::TooManySessions {
                    current: self.sessions.len(),
                    limit: SESSIONS_MAX_COUNT,
                });
            }

            self.sessions.insert(session_id, new_session);
            session_id
        } else {
            log::info!(
                "Received message from session [{} / {}]",
                header.session_id,
                header.sequence
            );
            header.session_id
        };

        // get the session from the map and update it
        if let Some(session) = self.sessions.get_mut(&session_id) {
            if session.last_sequence_id < header.sequence {
                // update session state
                session.last_sequence_id = header.sequence;
                session.last_timestamp = Instant::now();

                // send message to processing
                if let Err(error) = session
                    .channel
                    .send(session::ChannelMessage {
                        addr,
                        header: header,
                        body: body,
                    })
                    .await
                {
                    log::error!(
                        "Failed to send message to session [{}]: {}",
                        session_id,
                        error
                    );

                    self.sessions.remove(&session_id);
                    Err(ProcessingError::SessionClosed(session_id))
                } else {
                    Ok(())
                }
            } else {
                // ignore as duplicate
                log::warn!(
                    "Duplicate message received: {} (sequence: {})",
                    session_id,
                    session.last_sequence_id
                );

                // nothing to do, just ignore
                Ok(())
            }
        } else {
            Err(ProcessingError::SessionsNotFound(session_id))
        }
    }
}

/// Parse a buffer to header and command. Buffer expected size should be at least PackedHeader::SIZE. Returns header and command.
/// In case of error, returns short Error description.
fn parse_request(buf: &[u8], amt: usize) -> Result<(PackedHeader, Vec<u8>), SerializeError> {
    let header: PackedHeader = PackedHeader::try_deserialize(buf)?;
    let mut buffer = Vec::with_capacity(amt - PackedHeader::SIZE);
    buffer.extend_from_slice(&buf[PackedHeader::SIZE..amt]);
    Ok((header, buffer))
}
