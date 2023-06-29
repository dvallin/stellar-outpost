use nanoid::nanoid;
use rand::Rng;
use serde::de::Error as _;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::crew::CrewMember;
use super::game_state::GameState;
use super::resources::Resources;
use super::{AxialHexCoordinates, Entity, Storage};

#[derive(Hash, PartialEq, Eq)]
pub struct Coordinates {
    pub x: i32,
    pub y: i32,
}

impl Coordinates {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn hex_distance_to(self, other: Self) -> i32 {
        let hex: AxialHexCoordinates = self.into();
        hex.distance_to(other.into())
    }

    pub fn hex_length(self) -> i32 {
        let hex: AxialHexCoordinates = self.into();
        hex.length()
    }
}

impl<'de> Deserialize<'de> for Coordinates {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let data = <&str>::deserialize(deserializer)?;
        let mut parts = data.splitn(2, ',');
        let x: i32 = parts.next().unwrap().parse().map_err(D::Error::custom)?;
        let y: i32 = parts.next().unwrap().parse().map_err(D::Error::custom)?;
        Ok(Coordinates { x, y })
    }
}
impl Serialize for Coordinates {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{},{}", self.x, self.y))
    }
}

#[derive(Serialize, Deserialize)]
pub struct Sector {
    sub_sectors: Storage<SubSector>,
    sub_sectors_map: HashMap<Coordinates, String>,

    missions: Storage<Mission>,

    active_mission: Option<ActiveMission>,
}

impl Sector {
    pub fn new() -> Self {
        Self {
            sub_sectors: Storage::new(),
            sub_sectors_map: HashMap::new(),
            missions: Storage::new(),
            active_mission: None,
        }
    }

    pub fn add_subsector(&mut self, x: i32, y: i32, sub_sector: SubSector) {
        self.sub_sectors_map
            .insert(Coordinates::new(x, y), sub_sector.id().clone());
        self.sub_sectors.add(sub_sector);
    }

    pub fn finish_turn(&mut self, state: &mut GameState) {
        self.active_mission.as_mut().map(|a| {
            let mission = &self.missions[&a.mission_id];
            a.finish_turn(state, mission);
        });
    }

    pub fn bounds_at_y(&self, y: i32) -> (i32, i32) {
        let mut result = (0, 0);
        for (coordinates, _) in &self.sub_sectors_map {
            if coordinates.y == y {
                result.0 = std::cmp::min(coordinates.x, result.0);
                result.1 = std::cmp::max(coordinates.x, result.1);
            }
        }
        result
    }

    pub fn bounds_at_x(&self, x: i32) -> (i32, i32) {
        let mut result = (0, 0);
        for (coordinates, _) in &self.sub_sectors_map {
            if coordinates.x == x {
                result.0 = std::cmp::min(coordinates.y, result.0);
                result.1 = std::cmp::max(coordinates.y, result.1);
            }
        }
        result
    }

    pub fn missions_at(&self, x: i32, y: i32) -> Vec<&Mission> {
        let sub_sector = &self.sub_sectors[&self.sub_sectors_map[&Coordinates::new(x, y)]];
        self.missions
            .iter()
            .filter(|m| m.sub_sector_id == *sub_sector.id())
            .collect()
    }

    pub fn sub_sectors_map(&self) -> Vec<(&Coordinates, &SubSector)> {
        self.sub_sectors_map
            .iter()
            .map(|(c, s)| (c, &self.sub_sectors[s]))
            .collect()
    }

    pub fn add_mission(&mut self, x: i32, y: i32, mission_type: MissionType) {
        let sub_sector = &self.sub_sectors[&self.sub_sectors_map[&Coordinates::new(x, y)]];
        self.missions
            .add(Mission::new(sub_sector.id().clone(), mission_type))
    }

    pub fn set_active_mission(&mut self, active_mission: ActiveMission) {
        self.active_mission = Some(active_mission)
    }
}

#[derive(Serialize, Deserialize)]
pub struct SubSector {
    pub sector_type: SectorType,
    id: String,
}

#[derive(Serialize, Deserialize)]
pub enum SectorType {
    EmptySpace,
    SolarSystem,
    GasCloud,
    StellarRift,
}

impl SubSector {
    pub fn new(sector_type: SectorType) -> Self {
        Self {
            id: nanoid!(),
            sector_type,
        }
    }
}

impl Entity for SubSector {
    fn id(&self) -> &String {
        &self.id
    }
}

#[derive(Serialize, Deserialize)]
pub struct Mission {
    pub id: String,
    pub sub_sector_id: String,
    pub mission_type: MissionType,
}

impl Mission {
    pub fn new(sub_sector_id: String, mission_type: MissionType) -> Self {
        Self {
            id: nanoid!(),
            sub_sector_id,
            mission_type,
        }
    }
}

impl Entity for Mission {
    fn id(&self) -> &String {
        &self.id
    }
}

#[derive(Serialize, Deserialize)]
pub enum MissionType {
    Mining(u16, u16),
}

#[derive(Serialize, Deserialize)]
pub struct ActiveMission {
    pub mission_id: String,
    pub resources: Resources,
    pub distance: u16,
    pub state: ActiveMissionState,
    crew: Storage<CrewMember>,
}

impl ActiveMission {
    pub fn new(mission_id: &String, resources: Resources, crew: Vec<CrewMember>) -> Self {
        Self {
            mission_id: mission_id.clone(),
            resources,
            distance: 0,
            state: ActiveMissionState::OutwardTrip(0),
            crew: Storage::from(crew),
        }
    }

    pub fn finish_turn(&mut self, state: &mut GameState, mission: &Mission) {
        use ActiveMissionState::*;
        match self.state {
            AtDestination(turn) => {
                match mission.mission_type {
                    MissionType::Mining(min, max) => {
                        self.resources += Resources::minerals(state.rng.gen_range(min..max).into())
                    }
                }
                self.state = AtDestination(turn + 1);
            }
            OutwardTrip(turn) => {
                if self.distance >= turn {
                    self.state = AtDestination(0)
                } else {
                    self.state = OutwardTrip(turn + 1);
                }
            }
            ReturnTrip(turn) => {
                if self.distance >= turn {
                    self.state = Returned
                } else {
                    self.state = ReturnTrip(turn + 1);
                }
            }
            Returned => (),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum ActiveMissionState {
    AtDestination(u16),
    OutwardTrip(u16),
    ReturnTrip(u16),
    Returned,
}
