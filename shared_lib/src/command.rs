use musli::{Decode, Encode};

#[derive(Encode, Decode, Debug)]
pub enum Request {
    Temparature(f32),
}
