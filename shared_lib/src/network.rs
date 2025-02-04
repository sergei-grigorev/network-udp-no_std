use core::fmt;

use crate::command::Request;

pub const NETWORK_PAYLOAD_OFFSET: usize = size_of::<u64>() * 3;

#[derive(Debug)]
pub struct NetworkRequest {
    pub device_id: u64,
    pub message_number: u64,
    pub message_size: u64,
    pub payload: Request,
}

#[derive(Debug)]
pub struct NetworkAck {
    pub device_id: u64,
    pub message_number: u64,
}
