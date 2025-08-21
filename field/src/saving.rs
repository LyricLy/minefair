pub mod legacy;

use savefile::prelude::{load, save, SavefileError};
use std::io::{Read, Write, Seek};

use crate::field::*;

const VERSION: u32 = 0;

impl Field {
    pub fn load(reader: &mut (impl Read + Seek)) -> Result<Self, SavefileError> {
        match load(reader, VERSION) {
            Err(e) => {
                reader.rewind()?;
                match legacy::Field::load(reader) {
                    Ok(x) => Ok(x.into()),
                    Err(_) => Err(e),
                }
            },
            x => x,
        }
    }

    pub fn save(&self, writer: &mut impl Write) -> Result<(), SavefileError> {
        save(writer, VERSION, self)
    }
}
