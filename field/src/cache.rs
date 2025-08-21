use savefile::prelude::Savefile;
use std::cmp::Ordering;
use std::collections::{HashMap, BTreeSet};
use crate::field::Coord;

#[derive(Savefile, Clone, Copy, PartialEq)]
#[repr(C)]
struct ByRisk(Coord, f32);

impl Eq for ByRisk {}

impl PartialOrd for ByRisk {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// cheat because risks are never NaN. panics if we're wrong
impl Ord for ByRisk {
    fn cmp(&self, other: &Self) -> Ordering {
        self.1.partial_cmp(&other.1).unwrap().then_with(|| self.0.cmp(&other.0))
    }
}

#[derive(Savefile, Clone, Default)]
pub struct RiskCache {
    contents: HashMap<Coord, f32>,
    by_risk: BTreeSet<ByRisk>,
}

impl RiskCache {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get(&self, point: Coord) -> Option<f32> {
        self.contents.get(&point).copied()
    }

    pub fn contains_key(&self, point: Coord) -> bool {
        self.contents.contains_key(&point)
    }

    pub(crate) fn insert(&mut self, point: Coord, risk: f32) {
        if let Some(old_risk) = self.contents.insert(point, risk) {
            if risk == old_risk { return }
            assert!(self.by_risk.remove(&ByRisk(point, old_risk)));
        }
        self.by_risk.insert(ByRisk(point, risk));
    }

    pub(crate) fn remove(&mut self, point: Coord) {
        if let Some(old_risk) = self.contents.remove(&point) {
            assert!(self.by_risk.remove(&ByRisk(point, old_risk)));
        }
    }

    pub(crate) fn clear(&mut self) {
        self.contents.clear();
        self.by_risk.clear();
    }

    pub fn global_best(&self) -> f32 {
        self.by_risk.first().map_or(1.0, |&x| x.1)
    }

    pub fn is_empty(&self) -> bool {
        self.contents.is_empty()
    }

    pub fn len(&self) -> usize {
        self.contents.len()
    }

    pub fn iter(&self) -> impl Iterator<Item=(Coord, f32)> {
        self.contents.iter().map(|(&x, &y)| (x, y))
    }

    pub fn keys(&self) -> impl Iterator<Item=Coord> {
        self.contents.keys().copied()
    }

    pub fn values(&self) -> impl Iterator<Item=f32> {
        self.contents.values().copied()
    }
}

impl From<HashMap<Coord, f32>> for RiskCache {
    fn from(contents: HashMap<Coord, f32>) -> Self {
        Self { by_risk: contents.iter().map(|(&point, &risk)| ByRisk(point, risk)).collect(), contents }
    }
}
