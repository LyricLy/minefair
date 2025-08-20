use bincode::{Decode, Encode};
use rand::random;
use crate::field::*;

#[derive(Clone, Decode, Encode)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum Judge {
    Random,
    Strict,
    Kind,
    Local,
    Global,
    #[cfg_attr(feature = "clap", clap(alias = "kaboom"))]
    KaboomGlobal,
    KaboomLocal,
}
use Judge::*;

impl Field {
    fn global_clear(&self, risk: f32) -> bool {
        risk < 1.0 && risk <= self.density && risk <= self.risk_cache.global_best()
    }

    pub(crate) fn is_clear(&self, point: Coord) -> bool {
        let risk = self.cell_risk(point);
        match self.judge {
            Random => random::<f32>() > risk,
            Kind => risk != 1.0,
            Strict => risk == 0.0,
            Local => {
                if !self.risk_cache.contains_key(point) {
                    self.global_clear(risk)
                } else {
                    risk != 1.0 && self.group_from(vec![point], false).into_iter().all(|c| risk <= self.risk_cache.get(c).unwrap())
                }
            },
            Global => self.global_clear(risk),
            KaboomGlobal => {
                if risk == 1.0 {
                    false
                } else if risk == 0.0 {
                    true
                } else {
                    self.risk_cache.contains_key(point) && self.risk_cache.global_best() > 0.0
                }
            },
            KaboomLocal => {
                if risk == 1.0 {
                    false
                } else if risk == 0.0 {
                    true
                } else {
                    self.risk_cache.contains_key(point) && self.group_from(vec![point], false).into_iter().all(|c| self.risk_cache.get(c).unwrap() != 0.0)
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

    pub fn has_safe(&self) -> bool {
        match self.judge {
            Random | Strict => self.risk_cache.global_best() == 0.0,
            _ => self.risk_cache.global_best() < 1.0,
        }
    }

    pub fn safe_frontier(&self) -> Vec<Coord> {
        self.risk_cache.keys().filter(|&p| self.definite_risk(p) == Some(false)).collect()
    }
}
