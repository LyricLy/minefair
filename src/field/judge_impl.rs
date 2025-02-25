use rand::random;
use crate::options::Judge::*;
use super::*;

impl Field {
    fn global_clear(&self, risk: f32) -> bool {
        risk <= self.density && self.risk_cache.values().all(|&v| risk <= v)
    }

    pub(super) fn is_clear(&self, point: Coord) -> bool {
        let risk = self.cell_risk(point);
        match self.judge {
            Random => random::<f32>() > risk,
            Kind => risk != 1.0,
            Strict => risk == 0.0,
            Local => {
                if !self.risk_cache.contains_key(&point) {
                    self.global_clear(risk)
                } else {
                    risk != 1.0 && self.group_from(point, false).into_iter().all(|c| risk <= *self.risk_cache.get(&c).unwrap())
                }
            },
            Global => self.global_clear(risk),
            Kaboom => {
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
