use core::fmt::Debug;

use musli::{Decode, Encode};

pub const COMMAND_SIZE: usize = 1400;
pub type Buffer = [u8; COMMAND_SIZE];

pub const PACKET_SIZE: usize = 1500;

#[derive(Encode, Decode, Debug)]
pub enum Information {
    Temparature(f32),
    AirPressure(f32),
}

#[derive(Encode, Decode)]
pub struct EncodedCommand {
    pub size: usize,
    pub buf: Buffer,
}

impl Debug for EncodedCommand {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!(
            "NetworkCommand {{ size: {}, buf: <redundant> }}",
            self.size
        ))
    }
}
