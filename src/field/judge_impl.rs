use rand::random;
use crate::judges::Judge::*;
use super::*;

impl Field {
    pub(super) fn is_clear(&self, point: Coord) -> bool {
        match self.judge {
            Random => random::<f32>() > self.cell_risk(point),
            Kind => self.cell_risk(point) != 1.0,
            Strict => self.cell_risk(point) == 0.0,
            Global => {
                if self.risk_cache.is_empty() {
                    return true;
                }
                let best = *self.risk_cache.values().min_by(|&x, &y| x.partial_cmp(y).unwrap()).unwrap();
                let best = if best > self.density { self.density } else { best };
                self.cell_risk(point) == best
            },
            Kaboom => {
                let risk = self.cell_risk(point);
                if risk == 1.0 {
                    false
                } else if risk == 0.0 {
                    true
                } else {
                    self.risk_cache.contains_key(&point) && self.risk_cache.values().all(|&x| x != 0.0)
                }
            },
        }
    }

    pub fn definite_risk(&self, point: Coord) -> Option<bool> {
        match self.judge {
            Random => {
                let risk = self.cell_risk(point);
                if risk == 0.0 {
                    Some(false)
                } else if risk == 1.0 {
                    Some(true)
                } else {
                    None
                }
            },
            _ => Some(!self.is_clear(point)),
        }
    }
}
