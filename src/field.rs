mod solver;
mod judge_impl;

use rand::random;
use itertools::Itertools;
use bincode::{Decode, Encode};
use std::collections::HashMap;

use crate::judges::Judge;
use crate::Args;

#[derive(Clone, Copy, Encode, Decode)]
struct CellData {
    /* bit-packed representation:
       x   x   x   x   x   x   x   x
       -------------           |   -
       mine count              |   revealed?
                               |
                               flagged by player?
    */
    data: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Hidden(bool),
    Revealed(u8),
}

impl Cell {
    fn new() -> Self {
        Cell::Hidden(false)
    }

    fn is_revealed(self) -> bool {
        match self {
            Cell::Revealed(_) => true,
            Cell::Hidden(_) => false,
        }
    }

    fn to_data(self) -> CellData {
        CellData {
            data: match self {
                Cell::Hidden(p) => (p as u8) << 1,
                Cell::Revealed(n) => n << 4 | 1,
            },
        }
    }
}

impl CellData {
    fn to_cell(self) -> Cell {
        if self.data & 1 == 1 {
            Cell::Revealed(self.data >> 4)
        } else {
            Cell::Hidden(self.data >> 1 & 1 == 1)
        }
    }
}

type Coord = (isize, isize);
const CHUNK_SIZE: isize = 64;
const CHUNK_AREA: usize = (CHUNK_SIZE * CHUNK_SIZE) as usize;

pub fn adjacents((x, y): Coord) -> impl Iterator<Item=Coord> {
    [(x-1, y-1), (x, y-1), (x+1, y-1), (x-1, y), (x+1, y), (x-1, y+1), (x, y+1), (x+1, y+1)].into_iter()
}

fn chunk_point((x, y): Coord) -> (Coord, usize) {
    let chunk = (x.div_euclid(CHUNK_SIZE), y.div_euclid(CHUNK_SIZE));
    let point = y.rem_euclid(CHUNK_SIZE)*CHUNK_SIZE + x.rem_euclid(CHUNK_SIZE);
    (chunk, point as usize)
}

#[derive(Encode, Decode)]
pub struct Field {
    chunks: HashMap<Coord, [CellData; CHUNK_AREA]>,
    risk_cache: HashMap<Coord, f32>,
    density: f32,
    judge: Judge,
    solvable: bool,
}

impl Field {
    pub fn new(args: Args) -> Self {
        Field { chunks: HashMap::new(), risk_cache: HashMap::new(), density: args.density, judge: args.judge, solvable: args.solvable }
    }

    pub fn get(&self, point: Coord) -> Cell {
        let (chunk_coord, idx) = chunk_point(point);
        match self.chunks.get(&chunk_coord) {
            Some(chunk) => chunk[idx].to_cell(),
            None => Cell::new(),
        }
    }

    fn set(&mut self, point: Coord, cell: Cell) {
        let (chunk_coord, idx) = chunk_point(point);
        let chunk = self.chunks.entry(chunk_coord).or_insert_with(|| [Cell::new().to_data(); CHUNK_AREA]);
        chunk[idx] = cell.to_data();
    }

    pub fn toggle_flag(&mut self, point: Coord) {
        if let Cell::Hidden(p) = self.get(point) {
            self.set(point, Cell::Hidden(!p));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_conversion() {
        for cell in [Cell::Hidden(true), Cell::Revealed(3)] {
            assert_eq!(cell.to_data().to_cell(), cell);
        }
    }

    #[test]
    fn field_permanence() {
        let mut field = Field::new();
        let cell = Cell::Revealed(3);
        let point = (0, 2);
        field.set(point, cell);
        assert_eq!(field.get(point), cell);
    }

    #[test]
    fn uninitialized() {
        let field = Field::new();
        let point = (0, 2);
        assert_eq!(field.get(point), Cell::new());
    }

    #[test]
    fn negative_index() {
        let mut field = Field::new();
        let cell = Cell::Hidden(false);
        let point = (-2, 0);
        field.set(point, cell);
        assert_eq!(field.get(point), cell);
    }

    #[test]
    fn barrage() {
        let mut field = Field::new();
        for x in -128..=128 {
            for y in -128..=128 {
                field.set((x, y), Cell::Revealed(0));
            }
        }
        for x in -128..=128 {
            for y in -128..=128 {
                assert_eq!(field.get((x, y)), Cell::Revealed(0));
            }
        }
    }
}
