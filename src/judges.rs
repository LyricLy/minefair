use clap::ArgEnum;
use bincode::{Decode, Encode};

#[derive(Clone, ArgEnum, Decode, Encode)]
pub enum Judge {
    Random,
    Strict,
    Kind,
    Local,
    Global,
    Kaboom,
}
