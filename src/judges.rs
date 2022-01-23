use bincode::{Decode, Encode};

#[derive(Clone, Decode, Encode)]
pub enum Judge {
    Random,
    Strict,
    Kind,
    Global,
    Kaboom,
}
