use bincode::{Encode, Decode, error::{DecodeError, EncodeError}};
use std::collections::HashMap;
use std::io::{Write, Read};

use crate::field::*;

const CHUNK_AREA: usize = 64 * 64;

#[derive(Encode, Decode)]
pub enum Judge {
    Random,
    Strict,
    Kind,
    Local,
    Global,
    KaboomGlobal,
    KaboomLocal,
}

#[derive(Clone, Copy, Encode, Decode)]
pub struct CellData {
    pub data: u8,
}

#[derive(Encode, Decode)]
pub struct Field {
    pub chunks: HashMap<Coord, [CellData; CHUNK_AREA]>,
    pub risk_cache: HashMap<Coord, f32>,
    pub density: f32,
    pub judge: Judge,
    pub solvable: bool,
}

impl Field {
    pub fn load(reader: &mut impl Read) -> Result<Self, DecodeError> {
        bincode::decode_from_std_read(reader, bincode::config::standard())
    }

    pub fn save(&self, writer: &mut impl Write) -> Result<(), EncodeError> {
        bincode::encode_into_std_write(self, writer, bincode::config::standard())?;
        Ok(())
    }
}
