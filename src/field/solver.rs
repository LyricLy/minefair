use super::*;

impl Field {
    pub(super) fn group_from(&self, point: Coord, cut_on_safe: bool) -> Vec<Coord> {
        let mut group = Vec::new();
        let mut stack = vec![point];

        while let Some(p) = stack.pop() {
            let risk = self.risk_cache.get(&p);
            if group.contains(&p) || risk == Some(&1.0) || cut_on_safe && risk == Some(&0.0) || self.get(p).is_revealed() {
                continue;
            }
            for adj in adjacents(p) {
                if self.get(adj).is_revealed() {
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

    fn solve_group(&mut self, group: Vec<Coord>) -> bool {
        #[inline(always)]
        fn small_adjacents(point: usize, width: usize) -> impl Iterator<Item=usize> {
            [point.wrapping_sub(1+width), point.wrapping_sub(width), (point+1).wrapping_sub(width), point.wrapping_sub(1), point+1, point-1+width, point+width, point+1+width].into_iter()
        }

        let (lx, hx) = match group.iter().map(|&(x, _)| x).minmax().into_option() {
            Some(x) => x,
            None => return true,
        };
        let (ly, hy) = match group.iter().map(|&(_, y)| y).minmax().into_option() {
            Some(y) => y,
            None => return true,
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
                        let mut u: u8 = 0;
                        for adj in adjacents((x, y)) {
                            if self.risk_cache.get(&adj) == Some(&1.0) {
                                n -= 1;
                            } else if group.contains(&adj) {
                                u += 1;
                            }
                        }
                        Some((n, u))
                    }
                };
                small_world.push(c);
            }
        }

        let mut paths = 0;
        let mut i = 0;
        let mut stack = vec![(false, false), (true, false)];

        while let Some(&mut (action, ref mut done)) = stack.last_mut() {
            if *done {
                stack.pop();
                i -= 1;
                unknowns[i].3 = false;
                for j in small_adjacents(unknowns[i].0, width) {
                    if let Some(Some((ref mut n, ref mut u))) = small_world.get_mut(j) {
                        if action {
                            *n = n.wrapping_add(1);
                        }
                        *u = u.wrapping_add(1);
                    }
                }
            } else {
                *done = true;
                unknowns[i].3 = action;
                let mut ok = true;
                for j in small_adjacents(unknowns[i].0, width) {
                    if let Some(Some((ref mut n, ref mut u))) = small_world.get_mut(j) {
                        if action { 
                            if *n == 0 {
                                ok = false;
                            }
                            *n = n.wrapping_sub(1);
                        }
                        if u == n {
                            ok = false;
                        }
                        *u = u.wrapping_sub(1);
                    }
                }
                i += 1;
                if ok {
                    if i < unknowns.len() {
                        stack.push((false, false));
                        stack.push((true, false));
                    } else {
                        paths += 1;
                        for &mut (_, ref mut count, _, b) in &mut unknowns {
                            if b {
                                *count += 1;
                            }
                        }
                    }
                }
            }
        }

        if paths == 0 {
            false
        } else {
            let paths = paths as f32;
            for (_, count, orig, _) in unknowns {
                self.risk_cache.insert(orig, count as f32 / paths);
            }
            true
        }
    }

    pub fn cell_risk(&self, point: Coord) -> f32 {
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

    pub fn reveal_cell(&mut self, point: (isize, isize)) -> Result<u8, ()> {
        if !self.is_clear(point) {
            return Err(());
        }

        self.risk_cache.remove(&point);

        let mut num;
        'outer: loop {
            num = 0;
            for adj in adjacents(point) {
                if random::<f32>() < self.cell_risk(adj) {
                    num += 1;
                }
            }
            self.set(point, Cell::Revealed(num));

            for adj in adjacents(point) {
                let group = self.group_from(adj, true);
                if group.is_empty() {
                    continue;
                }
                if self.solve_group(group) && (!self.solvable || self.risk_cache.values().any(|&x| x == 0.0)) {
                    break 'outer;
                } else {
                    continue 'outer;
                }
            }

            break;
        }

        Ok(num)
    }
}
