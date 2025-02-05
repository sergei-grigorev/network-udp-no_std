#![no_std]

use byteorder::ByteOrder;
use byteorder::NetworkEndian;
use command::Request;
use error::Error;
use libc_print::libc_eprintln as eprintln;
use libc_print::libc_println as println;
use musli::alloc::{ArrayBuffer, Slice};
use musli::{context, packed::Encoding};
use network::NetworkAck;
use network::NetworkRequest;
use network::NETWORK_PAYLOAD_OFFSET;

pub mod command;
pub mod error;
pub mod network;

const OPTIONS: musli::Options = musli::options::new().fixed().native_byte_order().build();
const ENCODING: Encoding<OPTIONS> = Encoding::new().with_options();
const U64_S: usize = size_of::<u64>();

/// Make a new request and serialize it into the buffer.
/// Buffer must be at least 24 bytes long to hold the device_id, message_num, and payload_size.
/// The payload_size is the size of the payload in bytes. The function returns the number of bytes written to the buffer.
/// If the buffer is too small, function panic. Buffer recommended size is 1400.
pub fn make_new_request(buf: &mut [u8], device_id: u64, message_num: u64) -> Result<usize, Error> {
    if let Ok(payload_size) = mk_new_request(&mut buf[NETWORK_PAYLOAD_OFFSET..]) {
        // serialize device_id and message_num
        NetworkEndian::write_u64(&mut buf[0..U64_S], device_id);
        NetworkEndian::write_u64(&mut buf[U64_S..U64_S * 2], message_num);
        // serialize payload_size
        NetworkEndian::write_u64(&mut buf[U64_S * 2..U64_S * 3], payload_size.into());
        Ok(U64_S * 3 + usize::from(payload_size))
    } else {
        Err(Error::NotParsed)
    }
}

/// Make a new ack and serialize it into the buffer.
/// Buffer must be at least 16 bytes long to hold the device_id and message_num.
/// The function returns the number of bytes written to the buffer.
pub fn make_ack(buf: &mut [u8], device_id: u64, message_num: u64) -> usize {
    NetworkEndian::write_u64(&mut buf[0..U64_S], device_id);
    NetworkEndian::write_u64(&mut buf[U64_S..U64_S * 2], message_num);
    U64_S * 2
}

/// Parse a request from the buffer. Buffer must be at least 24 bytes long to hold the device_id, message_num, and payload_size.
pub fn parse_request(buf: &[u8]) -> Result<NetworkRequest, Error> {
    let device_id: u64 = NetworkEndian::read_u64(&buf[0..U64_S]);
    let message_num: u64 = NetworkEndian::read_u64(&buf[U64_S..U64_S * 2]);
    let payload_size: u64 = NetworkEndian::read_u64(&buf[U64_S * 2..U64_S * 3]);

    // parse payload
    let mut alloc_buf = ArrayBuffer::<256>::with_size();
    let alloc = Slice::new(&mut alloc_buf);
    let cx = context::new_in(&alloc).with_trace();

    if let Ok(parsed) = ENCODING.from_slice_with(&cx, &buf[24..]) {
        Ok(NetworkRequest {
            device_id,
            message_number: message_num,
            message_size: payload_size,
            payload: parsed,
        })
    } else {
        for _error in cx.errors() {
            // report error
            eprintln!("Error: {}", _error);
        }

        Err(Error::NotParsed)
    }
}

/// Parse an ack from the buffer. Buffer must be at least 16 bytes long to hold the device_id and message_num.
pub fn parse_ack(buf: &[u8]) -> Result<NetworkAck, Error> {
    if buf.len() >= U64_S * 2 {
        let device_id = NetworkEndian::read_u64(&buf[0..U64_S]);
        let message_num = NetworkEndian::read_u64(&buf[U64_S..U64_S * 2]);

        Ok(NetworkAck {
            device_id,
            message_number: message_num,
        })
    } else {
        Err(Error::NotEnough)
    }
}

fn mk_new_request(buf: &mut [u8]) -> Result<u16, Error> {
    let request1 = Request::Temparature(21f32);
    println!("Request: {:?}", request1);

    let mut alloc_buf = ArrayBuffer::<256>::with_size();
    let alloc = Slice::new(&mut alloc_buf);
    let cx = context::new_in(&alloc).with_trace();

    let Ok(w) = ENCODING.to_slice_with(&cx, buf, &request1) else {
        for _error in cx.errors() {
            // report error
            eprintln!("Error: {}", _error);
        }

        return Err(Error::TooBig);
    };

    // safety: we don't expect longer messages (they would not fit in the array buffer anyway)
    Ok(w.try_into().unwrap())
}
