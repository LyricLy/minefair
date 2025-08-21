use savefile::prelude::Savefile;
use std::collections::HashMap;
use std::time::Duration;

use crate::judges::Judge;
use crate::cache::RiskCache;
use crate::saving::legacy;

#[derive(Clone, Copy, Savefile)]
#[repr(C)]
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

impl From<legacy::CellData> for CellData {
    #[inline(always)]
    fn from(value: legacy::CellData) -> Self {
        Self { data: value.data }
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

#[derive(Savefile, Clone)]
pub struct Field {
    chunks: HashMap<Coord, [CellData; CHUNK_AREA]>,
    pub(crate) risk_cache: RiskCache,
    pub(crate) density: f32,
    pub judge: Judge,
    pub(crate) solvable: bool,
    size: Option<(usize, usize)>,
    cells_revealed: usize,
    time_elapsed: Duration,
}

impl From<legacy::Field> for Field {
    fn from(old: legacy::Field) -> Self {
        Self {
            cells_revealed: old.chunks.values().map(|c| c.iter().filter(|&&c| CellData::from(c).to_cell().is_revealed()).count()).sum(),
            chunks: old.chunks.into_iter().map(|(p, c)| (p, c.map(From::from))).collect(),
            risk_cache: old.risk_cache.into(),
            density: old.density,
            judge: old.judge.into(),
            solvable: old.solvable,
            size: None,
            time_elapsed: Duration::ZERO,
        }
    }
}

impl Field {
    pub fn new(density: f32, judge: Judge, solvable: bool, size: Option<(usize, usize)>) -> Self {
        Self {
            chunks: HashMap::new(),
            risk_cache: RiskCache::new(),
            density, judge, solvable, size,
            cells_revealed: 0,
            time_elapsed: Duration::ZERO,
        }
    }

    fn in_bounds(&self, point: Coord) -> bool {
        self.size.is_none_or(|(width, height)| {
            let (width, height) = (width as isize, height as isize);
            (-width).div_euclid(2) <= point.0 && point.0 < width / 2 && (-height).div_euclid(2) <= point.1 && point.1 < height / 2
        })
    }

    pub fn get(&self, point: Coord) -> Option<Cell> {
        if !self.in_bounds(point) { return None; }
        let (chunk_coord, idx) = chunk_point(point);
        Some(match self.chunks.get(&chunk_coord) {
            Some(chunk) => chunk[idx].to_cell(),
            None => Cell::new(),
        })
    }

    pub fn clear(&mut self) {
        self.chunks.clear();
        self.risk_cache.clear();
        self.cells_revealed = 0;
    }

    pub(crate) fn set(&mut self, point: Coord, cell: Cell) {
        let (chunk_coord, idx) = chunk_point(point);
        let chunk = self.chunks.entry(chunk_coord).or_insert_with(|| [Cell::new().to_data(); CHUNK_AREA]);
        self.cells_revealed -= chunk[idx].to_cell().is_revealed() as usize;
        self.cells_revealed += cell.is_revealed() as usize;
        chunk[idx] = cell.to_data();
    }

    pub fn toggle_flag(&mut self, point: Coord) {
        if let Some(Cell::Hidden(p)) = self.get(point) {
            self.set(point, Cell::Hidden(!p));
        }
    }

    pub fn density(&self) -> f32 {
        self.density
    }

    pub fn risks(&self) -> &RiskCache {
        &self.risk_cache
    }

    pub fn pass_time(&mut self, time: Duration) {
        self.time_elapsed += time;
    }

    pub fn cells_revealed(&self) -> usize {
        self.cells_revealed
    }

    pub fn time_elapsed(&self) -> Duration {
        self.time_elapsed
    }

    pub fn is_won(&self) -> bool {
        self.size.is_some_and(|(width, height)| width*height == self.cells_revealed + self.risk_cache.len() && !self.has_safe())
    }
}

impl Default for Field {
    fn default() -> Self {
        Self::new(0.3, Judge::Kind, false, None)
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
        assert_eq!(field.get(point), Some(cell));
    }

    #[test]
    fn uninitialized() {
        let field = Field::default();
        let point = (0, 2);
        assert_eq!(field.get(point), Some(Cell::new()));
    }

    #[test]
    fn negative_index() {
        let mut field = Field::default();
        let cell = Cell::Hidden(false);
        let point = (-2, 0);
        field.set(point, cell);
        assert_eq!(field.get(point), Some(cell));
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
                assert_eq!(field.get((x, y)), Some(Cell::Revealed(0)));
            }
        }
    }
}
