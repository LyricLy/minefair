use rand::random;
use super::*;

pub trait Judge {
    fn is_clear(&self, field: &mut Field, point: Coord) -> bool;
}

pub struct Random;
impl Judge for Random {
    fn is_clear(&self, field: &mut Field, point: Coord) -> bool {
        random::<f32>() > field.cell_risk(point)
    }
}

pub struct Threshold(pub f32);
impl Judge for Threshold {
    fn is_clear(&self, field: &mut Field, point: Coord) -> bool {
        field.cell_risk(point) <= self.0
    }
}

pub struct LocalBest;
impl Judge for LocalBest {
    fn is_clear(&self, field: &mut Field, point: Coord) -> bool {
        if let Some(group) = field.group_from(point) {
            let best = group.into_iter().map(|x| field.cell_risk(x)).min_by(|&x, &y| x.partial_cmp(&y).unwrap()).unwrap();
            field.cell_risk(point) == best
        } else {
            true
        }
    }
}

pub struct GlobalBest;
impl Judge for GlobalBest {
    fn is_clear(&self, field: &mut Field, point: Coord) -> bool {
        let best = *field.risk_cache.values().min_by(|&x, &y| x.partial_cmp(&y).unwrap()).unwrap();
        let best = if best > field.density { field.density } else { best };
        field.cell_risk(point) == best
    }
}

pub struct Kaboom;
impl Judge for Kaboom {
    fn is_clear(&self, field: &mut Field, point: Coord) -> bool {
        let risk = field.cell_risk(point);
        if risk == 1.0 {
            false
        } else if risk == 0.0 {
            true
        } else {
            field.risk_cache.values().all(|&x| x != 1.0)
        }
    }
}
