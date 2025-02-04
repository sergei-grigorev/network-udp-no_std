use core::fmt::Debug;

#[derive(Debug)]
pub enum Error {
    TooBig,
    NotParsed,
    NotEnough,
    BufferEmpty,
}