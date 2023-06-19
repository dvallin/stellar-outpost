use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Region {
    pub sectors: HashMap<(i32, i32), Sector>,
}

impl Region {
    pub fn new() -> Self {
        use SectorType::*;
        let mut sectors = HashMap::new();
        sectors.insert((-1, -1), Sector::new(StellarRift));
        sectors.insert((0, -1), Sector::new(SolarSystem));
        sectors.insert((1, -1), Sector::new(StellarRift));
        sectors.insert((-1, 0), Sector::new(EmptySpace));
        sectors.insert((0, 0), Sector::new(SolarSystem));
        sectors.insert((1, 0), Sector::new(EmptySpace));
        sectors.insert((-1, 1), Sector::new(StellarRift));
        sectors.insert((0, 1), Sector::new(SolarSystem));
        sectors.insert((1, 1), Sector::new(EmptySpace));
        Self { sectors }
    }

    pub fn finish_turn(&mut self) {}
}

#[derive(Serialize, Deserialize)]
pub struct Sector {
    pub sector_type: SectorType,
}

#[derive(Serialize, Deserialize)]
pub enum SectorType {
    EmptySpace,
    SolarSystem,
    GasCloud,
    StellarRift,
}

impl Sector {
    pub fn new(sector_type: SectorType) -> Self {
        Self { sector_type }
    }
}
