use clap::ArgEnum;
use bincode::{Decode, Encode};

#[derive(Clone, ArgEnum, Decode, Encode)]
pub enum Judge {
    Random,
    Strict,
    Kind,
    Global,
    Kaboom,
}
