use rustc_hash::FxHashMap;
use rand::random;
use itertools::Itertools;

#[derive(Clone, Copy)]
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

fn adjacents((x, y): Coord) -> impl Iterator<Item=Coord> {
    [(x-1, y-1), (x, y-1), (x+1, y-1), (x-1, y), (x+1, y), (x-1, y+1), (x, y+1), (x+1, y+1)].into_iter()
}

fn chunk_point((x, y): Coord) -> (Coord, usize) {
    let chunk = (x.div_euclid(CHUNK_SIZE), y.div_euclid(CHUNK_SIZE));
    let point = y.rem_euclid(CHUNK_SIZE)*CHUNK_SIZE + x.rem_euclid(CHUNK_SIZE);
    (chunk, point as usize)
}

pub struct Field {
    chunks: FxHashMap<Coord, [CellData; CHUNK_AREA]>,
    risk_cache: FxHashMap<Coord, f32>,
    density: f32,
}

impl Field {
    pub fn new() -> Self {
        Field { chunks: FxHashMap::default(), risk_cache: FxHashMap::default(), density: 0.25 }
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

    fn group_from(&self, point: Coord) -> Option<Vec<Coord>> {
        let risk = self.risk_cache.get(&point);
        if risk == Some(&1.0) || risk == Some(&0.0) || self.get(point).is_revealed() {
            return None;
        }

        let mut group = Vec::new();
        let mut stack = vec![point];

        while !stack.is_empty() {
            let p = stack.pop().unwrap();
            if group.contains(&p) {
                continue;
            }
            for adj in adjacents(p) {
                if let Cell::Revealed(_) = self.get(adj) {
                    for their_adj in adjacents(adj) {
                        let risk = self.risk_cache.get(&their_adj);
                        if their_adj != p && matches!(self.get(their_adj), Cell::Hidden(_)) && risk != Some(&1.0) && risk != Some(&0.0) {
                            stack.push(their_adj);
                        }
                    }
                }
            }
            group.push(p);
        }

        Some(group)
    }

    fn solve_group(&mut self, group: Vec<Coord>) {
        //dbg!(&group);

        #[inline(always)]
        fn small_adjacents(point: usize, width: usize) -> impl Iterator<Item=usize> {
            [point.wrapping_sub(1+width), point.wrapping_sub(width), (point+1).wrapping_sub(width), point.wrapping_sub(1), point+1, point-1+width, point+width, point+1+width].into_iter()
        }

        let (lx, hx) = match group.iter().map(|&(x, _)| x).minmax().into_option() {
            Some(x) => x,
            None => return,
        };
        let (ly, hy) = match group.iter().map(|&(_, y)| y).minmax().into_option() {
            Some(y) => y,
            None => return,
        };
        let (lx, ly, hx, hy) = (lx-1, ly-1, hx+2, hy+2);  // half-open
        let (width, height) = ((hx-lx) as usize, (hy-ly) as usize);
        let mut small_world = Vec::with_capacity(width*height);
        let mut unknowns = Vec::new();

        for y in ly..hy {
            for x in lx..hx {
                let c = match self.get((x, y)) {
                    Cell::Hidden(_) => {
                        if group.contains(&(x, y)) {
                            unknowns.push((small_world.len(), 0, (x, y), false));
                        }
                        None
                    },
                    Cell::Revealed(mut n) => {
                        for adj in adjacents((x, y)) {
                            if self.risk_cache.get(&adj) == Some(&1.0) {
                                n -= 1;
                            }
                        }
                        Some(n)
                    }
                };
                small_world.push(c);
            }
        }

        fn find_bombs(stack: &mut Vec<(usize, bool)>, small_world: &[Option<u8>], unknowns: &[(usize, u32, (isize, isize), bool)], i: usize, width: usize, height: usize) -> bool {
            'outer: for (idx, &(j, _, _, _)) in (i..).zip(unknowns[i..].iter()) {
                for k in small_adjacents(j, width) {
                    if let Some(Some(0)) = small_world.get(k) {
                        if matches!(small_world.get(j.wrapping_sub(1+width)), Some(&Some(1..)))
                        || (j+1)%width == 0 && matches!(small_world.get(j.wrapping_sub(width)), Some(&Some(1..)))
                        || j > width*(height-1) && matches!(small_world[j-1], Some(1..)) {
                            return false;
                        } else {
                            continue 'outer;
                        }
                    }
                }
                if matches!(small_world.get(j.wrapping_sub(1+width)), Some(&Some(2..)))
                || (j+1)%width == 0 && matches!(small_world.get(j.wrapping_sub(width)), Some(&Some(2..)))
                || j > width*(height-1) && matches!(small_world[j-1], Some(2..)) {
                    return false;
                }
                if matches!(small_world.get(j.wrapping_sub(1+width)), Some(&Some(1)))
                || (j+1)%width == 0 && matches!(small_world.get(j.wrapping_sub(width)), Some(&Some(1)))
                || j > width*(height-1) && matches!(small_world[j-1], Some(1)) {
                    stack.push((idx, false));
                    return false;
                }
                stack.push((idx, false));
            }
            true
        }

        let mut paths = 0;
        let mut stack = Vec::new();
        find_bombs(&mut stack, &small_world, &unknowns, 0, width, height);

        while !stack.is_empty() {
            //dbg!(&stack, &small_world);
            let &mut (i, ref mut done) = stack.last_mut().unwrap();
            if *done {
                stack.pop();
                unknowns[i].3 = false;
                for j in small_adjacents(unknowns[i].0, width) {
                    if let Some(Some(ref mut n)) = small_world.get_mut(j) {
                        *n += 1;
                    }
                }
            } else {
                *done = true;
                unknowns[i].3 = true;
                for j in small_adjacents(unknowns[i].0, width) {
                    if let Some(Some(ref mut n)) = small_world.get_mut(j) {
                        *n -= 1;
                    }
                }
                if !find_bombs(&mut stack, &small_world, &unknowns, i+1, width, height) {
                    continue;
                }
                //eprintln!("worked!");
                paths += 1;
                for &mut (_, ref mut count, _, b) in &mut unknowns {
                    if b {
                        *count += 1;
                    }
                }
            }
        }

        if paths == 0 {
            for (_, _, orig, _) in unknowns {
                self.risk_cache.insert(orig, 0.0);
            }
        } else {
            let paths = paths as f32;
            for (_, count, orig, _) in unknowns {
                //dbg!(orig, count, paths);
                self.risk_cache.insert(orig, count as f32 / paths);
            }
        }
    }

    fn cell_risk(&self, point: Coord) -> f32 {
        if let Some(p) = self.risk_cache.get(&point) {
            // frontier
            *p
        } else if self.get(point).is_revealed() || self.risk_cache.is_empty() {
            // already revealed or first click
            0.0
        } else {
            // no info
            self.density
        }
    }

    pub fn reveal_cell(&mut self, point: (isize, isize)) -> Result<(), ()> {
        if self.get(point).is_revealed() {
            return Ok(());
        }

        

        let mut num = 0;
        for adj in adjacents(point) {
            if random::<f32>() < self.cell_risk(adj) {
                num += 1;
            }
        }
        //dbg!(num);
        self.set(point, Cell::Revealed(num));
        self.risk_cache.remove(&point);

        for adj in adjacents(point) {
            if let Some(group) = self.group_from(adj) {
                self.solve_group(group);
                break;
            }
        }

        if num == 0 {
            for adj in adjacents(point) {
                self.reveal_cell(adj)?;
            }
        }
        Ok(())
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
