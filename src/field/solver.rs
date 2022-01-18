use super::*;

impl Field {
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
        let (lx, ly, hx, hy) = (lx-2, ly-2, hx+3, hy+3);  // half-open
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
                macro_rules! will_pass {
                    ($pattern:pat) => {
                        matches!(small_world.get(j.wrapping_sub(1+width)), Some(&Some($pattern)))
                        || (j+1)%width == 0 && matches!(small_world.get(j.wrapping_sub(width)), Some(&Some($pattern)))
                        || j > width*(height-1) && matches!(small_world[j-1], Some($pattern))
                    };
                }

                for k in small_adjacents(j, width) {
                    if let Some(Some(0)) = small_world.get(k) {
                        if will_pass!(1..) {
                            return false;
                        } else {
                            continue 'outer;
                        }
                    }
                }
                if will_pass!(2..) {
                    return false;
                }
                if will_pass!(1) {
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
