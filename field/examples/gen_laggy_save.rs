use minefair_field::Field;
use rand::prelude::*;
use std::time::{Instant, Duration};
use std::fs::File;

pub fn main() {
    let mut rng = rand::rng();
    'top: loop {
        let mut field = Field::default();
        let _ = field.reveal_cell((0, 0));
        for _ in 0..100 {
            let to_reveal = *field.safe_frontier().choose(&mut rng).unwrap();
            let before = field.clone();
            let start = Instant::now();
            let _ = field.reveal_cell(to_reveal);
            if dbg!(start.elapsed()) > Duration::from_secs(10) {
                dbg!(to_reveal);
                let mut file = File::create("thesave.minefair").unwrap();
                bincode::encode_into_std_write(&before, &mut file, bincode::config::standard()).unwrap();
                break 'top;
            }
        }
    }
}
