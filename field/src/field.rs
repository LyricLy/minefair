use bincode::{Decode, Encode};
use std::collections::HashMap;

use crate::judges::Judge;

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
    pub(crate) fn new() -> Self {
        Self::Hidden(false)
    }

    pub(crate) fn is_revealed(self) -> bool {
        match self {
            Self::Revealed(_) => true,
            Self::Hidden(_) => false,
        }
    }

    fn to_data(self) -> CellData {
        CellData {
            data: match self {
                Self::Hidden(p) => (p as u8) << 1,
                Self::Revealed(n) => n << 4 | 1,
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

pub(crate) type Coord = (isize, isize);
const CHUNK_SIZE: isize = 64;
const CHUNK_AREA: usize = (CHUNK_SIZE * CHUNK_SIZE) as usize;

pub fn adjacents((x, y): Coord) -> impl Iterator<Item=Coord> {
    [(x, y-1), (x+1, y-1), (x+1, y), (x+1, y+1), (x, y+1), (x-1, y+1), (x-1, y), (x-1, y-1)].into_iter()
}

fn chunk_point((x, y): Coord) -> (Coord, usize) {
    let chunk = (x.div_euclid(CHUNK_SIZE), y.div_euclid(CHUNK_SIZE));
    let point = y.rem_euclid(CHUNK_SIZE)*CHUNK_SIZE + x.rem_euclid(CHUNK_SIZE);
    (chunk, point as usize)
}

#[derive(Encode, Decode, Clone)]
pub struct Field {
    chunks: HashMap<Coord, [CellData; CHUNK_AREA]>,
    pub(crate) risk_cache: HashMap<Coord, f32>,
    pub(crate) density: f32,
    pub(crate) judge: Judge,
    pub(crate) solvable: bool,
}

impl Field {
    pub fn new(density: f32, judge: Judge, solvable: bool) -> Self {
        Self { chunks: HashMap::new(), risk_cache: HashMap::new(), density, judge, solvable }
    }

    pub fn get(&self, point: Coord) -> Cell {
        let (chunk_coord, idx) = chunk_point(point);
        match self.chunks.get(&chunk_coord) {
            Some(chunk) => chunk[idx].to_cell(),
            None => Cell::new(),
        }
    }

    pub fn clear(&mut self) {
        self.chunks.clear();
        self.risk_cache.clear();
    }

    pub(crate) fn set(&mut self, point: Coord, cell: Cell) {
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

impl Default for Field {
    fn default() -> Self {
        Self::new(0.3, Judge::Kind, false)
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
        let mut field = Field::default();
        let cell = Cell::Revealed(3);
        let point = (0, 2);
        field.set(point, cell);
        assert_eq!(field.get(point), cell);
    }

    #[test]
    fn uninitialized() {
        let field = Field::default();
        let point = (0, 2);
        assert_eq!(field.get(point), Cell::new());
    }

    #[test]
    fn negative_index() {
        let mut field = Field::default();
        let cell = Cell::Hidden(false);
        let point = (-2, 0);
        field.set(point, cell);
        assert_eq!(field.get(point), cell);
    }

    #[test]
    fn barrage() {
        let mut field = Field::default();
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
