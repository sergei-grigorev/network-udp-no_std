use crate::command::EncodedCommand;
use crate::command::Information;
use crate::error::SerializeError;
use crate::network::MessageType;
use crate::network::PackedHeader;
use byteorder::ByteOrder;
use byteorder::NetworkEndian;
use libc_print::libc_eprintln as eprintln;
use musli::alloc::{ArrayBuffer, Slice};
use musli::{context, packed::Encoding};

const BUF_SIZE: usize = size_of::<u16>();

const OPTIONS: musli::Options = musli::options::new().fixed().native_byte_order().build();
const ENCODING: Encoding<OPTIONS> = Encoding::new().with_options();

/// Make a new message and serialize it into the buffer.
/// Buffer must be enough to fit at least header_size + message_size + payload_size.
/// The payload_size is the size of the payload in bytes. The function returns the number of bytes written to the buffer.
/// If the buffer is too small, function panic. Buffer recommended size is 1400.
pub fn make_new_command(
    message_type: MessageType,
    command: &EncodedCommand,
    buf: &mut [u8],
    device_id: u32,
    session_id: u16,
    sequence: u16,
    ack: u16,
) -> Result<usize, SerializeError> {
    if let Ok(payload_size) = write(command, &mut buf[PackedHeader::SIZE..]) {
        // serialize header
        let header = PackedHeader::new(message_type, device_id, session_id, sequence, ack);
        // add header
        header.serialize_info(&mut buf[0..PackedHeader::SIZE])?;
        Ok(PackedHeader::SIZE + usize::from(payload_size))
    } else {
        Err(SerializeError::NotParsed)
    }
}

/// Parse a command from the buffer. Buffer must start with u64 representing the payload size.
pub fn parse_command(buf: &[u8]) -> Result<EncodedCommand, SerializeError> {
    let payload_size: u16 = NetworkEndian::read_u16(buf);
    if buf.len() >= usize::from(payload_size) + BUF_SIZE {
        // parse payload
        let mut alloc_buf = ArrayBuffer::<256>::with_size();
        let alloc = Slice::new(&mut alloc_buf);
        let cx = context::new_in(&alloc);

        ENCODING
            .from_slice_with(&cx, &buf[BUF_SIZE..])
            .map_err(|err| {
                eprintln!("Error: {}", err);
                SerializeError::NotParsed
            })
    } else {
        Err(SerializeError::NotEnough)
    }
}

pub fn parse_non_encrypted(buf: &[u8]) -> Result<Information, SerializeError> {
    let payload_size: u16 = NetworkEndian::read_u16(buf);
    if buf.len() >= usize::from(payload_size) + BUF_SIZE {
        // parse payload
        let mut alloc_buf = ArrayBuffer::<256>::with_size();
        let alloc = Slice::new(&mut alloc_buf);
        let cx = context::new_in(&alloc);

        ENCODING
            .from_slice_with(&cx, &buf[BUF_SIZE..])
            .map_err(|err| {
                eprintln!("Error: {}", err);
                SerializeError::NotParsed
            })
    } else {
        Err(SerializeError::NotEnough)
    }
}

pub fn write_non_encrypted(
    informatiin: &Information,
    buf: &mut [u8],
) -> Result<u16, SerializeError> {
    let mut alloc_buf = ArrayBuffer::<256>::with_size();
    let alloc = Slice::new(&mut alloc_buf);
    let cx = context::new_in(&alloc);

    match ENCODING.to_slice_with(&cx, &mut buf[BUF_SIZE..], informatiin) {
        Ok(w) => {
            NetworkEndian::write_u16(&mut buf[0..BUF_SIZE], w.try_into().unwrap());
            Ok(w + BUF_SIZE).map(|w| w.try_into().unwrap())
        }
        Err(error) => {
            // report error
            eprintln!("Error: {}", error);
            Err(SerializeError::TooBig)
        }
    }
}

fn write(payload: &EncodedCommand, buf: &mut [u8]) -> Result<u16, SerializeError> {
    let mut alloc_buf = ArrayBuffer::<256>::with_size();
    let alloc = Slice::new(&mut alloc_buf);
    let cx = context::new_in(&alloc);

    match ENCODING.to_slice_with(&cx, &mut buf[BUF_SIZE..], payload) {
        Ok(w) => {
            NetworkEndian::write_u16(&mut buf[0..BUF_SIZE], w.try_into().unwrap());
            Ok(w + BUF_SIZE).map(|w| w.try_into().unwrap())
        }
        Err(error) => {
            // report error
            eprintln!("Error: {}", error);
            Err(SerializeError::TooBig)
        }
    }
}
