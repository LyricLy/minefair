use rand::prelude::*;
use rand::distr::weighted::WeightedIndex;
use crate::field::*;

/// A finite section of a Field, in which each revealed cell stores the number of mines and unknowns neighbouring it.
struct SmallWorld {
    marsh: Vec<Option<(i8, i8)>>,
    ox: isize,
    oy: isize,
    width: usize,
    height: usize,
}

impl SmallWorld {
    fn new(field: &Field, (ox, oy): Coord, (width, height): (usize, usize)) -> Self {
        let mut marsh = Vec::with_capacity(width*height);

        for y in oy..oy+height as isize {
            for x in ox..ox+width as isize {
                marsh.push(match field.get((x, y)) {
                    Some(Cell::Revealed(n)) => Some((n as i8, 0)),
                    _ => None,
                });
            }
        }

        Self { marsh, ox, oy, width, height }
    }

    fn get_mut(&mut self, point: usize) -> &mut Option<(i8, i8)> {
        &mut self.marsh[point]
    }

    fn index_of(&self, (x, y): Coord) -> Option<usize> {
        let x = x.wrapping_sub(self.ox) as usize;
        let y = y.wrapping_sub(self.oy) as usize;
        (x < self.width && y < self.height).then(|| y*self.width + x)
    }

    fn index_of_unchecked(&self, (x, y): Coord) -> usize {
        let x = (x - self.ox) as usize;
        let y = (y - self.oy) as usize;
        y*self.width + x
    }

    fn position_of(&self, pos: usize) -> Coord {
        ((pos % self.width) as isize + self.ox, (pos / self.width) as isize + self.oy)
    }

    #[inline(always)]
    fn adjacents(&self, point: usize) -> impl Iterator<Item=usize> + 'static {
        let width = self.width;
        [point - width - 1, point - width, point - width + 1, point - 1, point + 1, point + width - 1, point + width, point + width + 1].into_iter()
    }
}

impl Field {
    pub(super) fn group_from(&self, mut stack: Vec<Coord>, cut_on_safe: bool) -> Vec<Coord> {
        let mut group = Vec::new();

        while let Some(p) = stack.pop() {
            let risk = self.risk_cache.get(p);
            if group.contains(&p) || risk == Some(1.0) || cut_on_safe && risk == Some(0.0) || self.get(p).is_none_or(|x| x.is_revealed()) {
                continue;
            }
            for adj in adjacents(p) {
                if self.get(adj).is_some_and(|x| x.is_revealed() && x != Cell::Revealed(0)) {
                    for their_adj in adjacents(adj) {
                        if their_adj != p {
                            stack.push(their_adj);
                        }
                    }
                }
            }
            group.push(p);
        }

        group
    }

    pub fn is_one_group(&self) -> bool {
        let group_candidates: Vec<_> = self.risk_cache.iter().filter_map(|(c, r)| (r != 0.0 && r != 1.0).then_some(c)).collect();
        if group_candidates.is_empty() {
            return true;
        }
        self.group_from(vec![group_candidates[0]], true).len() == group_candidates.len()
    }

    fn solve_from(&mut self, point: Coord, first_zero: bool) -> u8 {
        let mut stack: Vec<Coord> = adjacents(point).collect();
        stack.push(point);
        let group = self.group_from(stack, true);

        let mut lx = isize::MAX;
        let mut hx = isize::MIN;
        let mut ly = isize::MAX;
        let mut hy = isize::MIN;
        for &(x, y) in &group {
            lx = lx.min(x - 1);
            hx = hx.max(x + 2);
            ly = ly.min(y - 1);
            hy = hy.max(y + 2);
        }

        let mut world = SmallWorld::new(self, (lx, ly), ((hx-lx) as usize, (hy-ly) as usize));

        // The cell being clicked ("target cell") is in a quasi-state where its number is not yet known (since this function's job is to decide it),
        // so treat it like a number that starts with -1 expected mines. In the solving loop, we decrement from the expected mine count whenever
        // we add a neighbouring mine. If it is 0 before decrementing, we consider the world "invalid" and backtrack. Initializing the mine count
        // to -1 prevents this underflow from ever occurring. We can later invert the value again to retrieve the number of mines surrounding the
        // target cell. (That is, the number it should display in that world.)
        let point_index = world.index_of_unchecked(point);
        *world.get_mut(point_index) = Some((!0, 0));

        // subtract already-known mines from each number
        for y in ly-1 .. hy+1 {
            for x in lx-1 .. hx+1 {
                if self.risk_cache.get((x, y)) == Some(1.0) {
                    for adj in adjacents((x, y)) {
                        if let Some(x) = world.index_of(adj).and_then(|i| world.marsh[i].as_mut()) {
                            x.0 -= 1;
                        }
                    }
                }
            }
        }

        // collect the relevant unknowns we need to solve for
        let mut unknowns = Vec::new();
        let mut unconstrained = Vec::new();
        for &pos in &group[1..] {
            // cell is not touching any numbers and is "unconstrained". put these in a separate bucket and do not solve them
            if !self.risk_cache.contains_key(pos) {
                unconstrained.push(pos);
                continue;
            }

            let i = world.index_of_unchecked(pos);
            unknowns.push((i, [0.0; 9], false));

            // each number needs to know how many unknowns are adjacent to it
            for adj in world.adjacents(i) {
                if let Some(x) = world.get_mut(adj) {
                    x.1 += 1;
                }
            }
        }

        // unconst_num_probs[n] is the probability of exactly n of the unconstrained cells being mines
        // this just computes (unconstained.len() choose i) * self.density^i * (1 - self.density)^(uncontained.len() - i)
        // AKA the binomial distribution, and there are less stupid/more accurate ways to do this, but whatever
        let mut unconst_num_probs = [0.0; 9];
        for n in 0..(1u32 << unconstrained.len()) {
            let num = n.count_ones() as i32;
            unconst_num_probs[num as usize] += self.density.powi(num) * (1.0 - self.density).powi(unconstrained.len() as i32 - num);
        }

        // proportion of valid placements by the number (not including unconstrained mines) they show on the target cell
        let mut valid_by_num = [0.0; 9];

        let mut i = 0;
        let mut stack = Vec::new();
        let mut ok = true;

        // main solving loop
        loop {
            if ok {
                if i == unknowns.len() {
                    let mine_count = unknowns.iter().filter(|&&(_, _, v)| v).count() as i32;
                    let num = !(world.get_mut(point_index).unwrap().0 as usize);

                    // chance of this particular placement happening, taking into account the density
                    let weight = self.density.powi(mine_count) * (1.0 - self.density).powi(unknowns.len() as i32 - mine_count);

                    valid_by_num[num] += weight;
                    for &mut (_, ref mut counts, b) in &mut unknowns {
                        if b {
                            counts[num] += weight;
                        }
                    }
                } else {
                    stack.push((false, false));
                    stack.push((true, false));
                }
            }
            ok = true;

            let Some(&mut (action, ref mut done)) = stack.last_mut() else { break };

            if *done {
                stack.pop();
                i -= 1;
                unknowns[i].2 = false;

                for j in world.adjacents(unknowns[i].0) {
                    if let Some((n, u)) = world.get_mut(j) {
                        if action {
                            *n += 1;
                        }
                        *u += 1;
                    }
                }

                ok = false;
            } else {
                *done = true;
                unknowns[i].2 = action;

                for j in world.adjacents(unknowns[i].0) {
                    if let Some((n, u)) = world.get_mut(j) {
                        if action { 
                            if *n == 0 {
                                ok = false;
                            }
                            *n -= 1;
                        }
                        if u == n {
                            ok = false;
                        }
                        *u -= 1;
                    }
                }

                i += 1;
            }
        }

        // chance of each number appearing when the target cell is revealed
        let mut num_probs = [0.0; 9];
        // like valid_by_num, but for unconstrained mines
        let mut unconst_by_num = [0.0; 9];

        for (i, x) in valid_by_num.into_iter().enumerate() {
            for (j, y) in unconst_num_probs[0..9-i].iter().enumerate() {
                let weight = x * y;
                num_probs[i+j] += weight;
                unconst_by_num[i+j] += j as f32 / unconstrained.len() as f32 * weight;
            }
        }

        let weights = if self.solvable && self.risk_cache.global_best() > 0.0
        // prefer a possibility with safe cells if one exists, since there are none left
        && let safe_havers = std::array::from_fn::<_, 9, _>(|num| {
            unknowns.iter().any(|&(_, counts, _)| {
                for (i, x) in counts[0..=num].iter().enumerate() {
                    if x * unconst_num_probs[num-i] != 0.0 {
                        return false;
                    }
                }
                true
            }) || !unconstrained.is_empty() && unconst_by_num[num] == 0.0
        }) && safe_havers.iter().any(|&x| x) {
            let mut new_probs = num_probs;
            for (i, x) in safe_havers.into_iter().enumerate() {
                if !x {
                    new_probs[i] = 0.0;
                }
            }
            new_probs
        } else {
            num_probs
        };

        let num = if first_zero && self.risk_cache.is_empty() && num_probs[0] != 0.0 {
            // first click always gives you a 0
            0
        } else {
            WeightedIndex::new(weights).unwrap().sample(&mut rand::rng())
        };

        // finally just plug in risks
        for (i, counts, _) in unknowns {
            // `counts` does not take into account possible unconstrained mines yet, so fix that with the same logic as for `valid_by_num`
            let mut final_count = 0.0;
            for (i, x) in counts[0..=num].iter().enumerate() {
                final_count += x * unconst_num_probs[num-i];
            }

            let weight = final_count / num_probs[num];
            self.risk_cache.insert(world.position_of(i), weight);
        }

        let unconst_weight = unconst_by_num[num] / num_probs[num];
        for point in unconstrained {
            self.risk_cache.insert(point, unconst_weight);
        }

        num as u8
    }

    pub fn cell_risk(&self, point: Coord) -> f32 {
        if let Some(p) = self.risk_cache.get(point) {
            // frontier
            p
        } else if self.get(point).is_none_or(|x| x.is_revealed()) || self.risk_cache.is_empty() && self.density < 1.0 {
            // already revealed or first click
            0.0
        } else {
            // no info
            self.density
        }
    }

    fn reveal_cell_internal(&mut self, point: Coord, first_zero: bool) -> Option<u8> {
        if !self.is_clear(point) {
            return None;
        }

        self.risk_cache.remove(point);

        let num = self.solve_from(point, first_zero);
        self.set(point, Cell::Revealed(num));

        Some(num)
    }

    pub fn reveal_cell(&mut self, point: Coord) -> Option<u8> {
        self.reveal_cell_internal(point, false)
    }

    pub fn reveal_cell_first_zero(&mut self, point: Coord) -> Option<u8> {
        self.reveal_cell_internal(point, true)
    }
}

#[cfg(test)]
mod tests {
    use rand::prelude::*;
    use std::collections::HashMap;
    use super::*;

    #[test]
    fn sanity() {
        let mut rng = rand::rng();
        let mut field = Field::default();
        let _ = field.reveal_cell((0, 0));
        for _ in 0..1000 {
            let point = *field.safe_frontier().choose(&mut rng).unwrap();
            let _ = field.reveal_cell(point);
        }

        for risk in field.risk_cache.values() {
            assert!(risk.is_finite() && risk >= 0.0 && risk <= 1.0, "risk {:?} is not sane", risk);
        }

        let mut surrounding_info: HashMap<Coord, (u8, u8, u8)> = HashMap::new();

        for (point, risk) in field.risk_cache.iter() {
            for neighbour in adjacents(point) {
                if field.get(neighbour).unwrap().is_revealed() {
                    let e = surrounding_info.entry(neighbour).or_default();
                    if risk == 0.0 {
                        e.0 += 1;
                    } else if risk == 1.0 {
                        e.1 += 1;
                    } else {
                        e.2 += 1;
                    }
                }
            }
        }

        for (point, (conf_safes, conf_mines, others)) in surrounding_info {
            let Some(Cell::Revealed(num)) = field.get(point) else { unreachable!() };
            assert!(conf_mines <= num);
            assert!(conf_mines+others >= num);
            assert!(conf_safes <= 8 - num);
        }
    }
}
