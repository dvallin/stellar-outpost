use self::{
    game_state::GameState,
    outpost::Outpost,
    sector::{MissionType, Sector, SectorType},
};
use crate::model::modules::Module;
use crate::model::{
    crew::CrewMember,
    modules::{
        farm::Farm, living_quarters::LivingQuarters, power_generator::PowerGenerator,
        water_extractor::WaterExtractor,
    },
    sector::SubSector,
};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::ValuesMut;
use std::collections::HashMap;
use std::ops::{Index, IndexMut};
use std::slice::IterMut;

pub mod crew;
pub mod game_state;
pub mod modules;
pub mod outpost;
pub mod resources;
pub mod sector;
pub mod stats;

#[derive(Serialize, Deserialize)]
pub struct Game {
    pub state: GameState,
    pub outpost: Outpost,
    pub sector: Sector,
}

impl Game {
    pub fn new() -> Self {
        let state = GameState::new();
        let mut outpost = Outpost::new();

        let power_generator = Box::new(PowerGenerator::new("power"));
        outpost.add_module(power_generator);

        let mut quarters = Box::new(LivingQuarters::new("quarters"));
        quarters.set_energy_level(2);
        outpost.add_module(quarters);

        let mut water = Box::new(WaterExtractor::new("water"));
        let water_id = water.id().clone();
        water.set_energy_level(1);
        outpost.add_module(water);

        let mut farm = Box::new(Farm::new("farm"));
        let farm_id = farm.id().clone();
        farm.set_energy_level(1);
        outpost.add_module(farm);

        let a = CrewMember::new("a");
        let a_id = a.id().clone();
        outpost.add_crew_member(a);

        let b = CrewMember::new("b");
        let b_id = b.id().clone();
        outpost.add_crew_member(b);

        outpost.add_crew_member(CrewMember::new("c"));
        outpost.add_crew_member(CrewMember::new("d"));

        outpost.assign_crew_member_to_module(&a_id, &water_id);
        outpost.assign_crew_member_to_module(&b_id, &farm_id);

        let mut sector = Sector::new();

        use SectorType::*;
        sector.add_subsector(-1, -1, SubSector::new(StellarRift));
        sector.add_subsector(1, -1, SubSector::new(StellarRift));
        sector.add_subsector(0, 0, SubSector::new(SolarSystem));
        sector.add_subsector(1, 0, SubSector::new(EmptySpace));
        sector.add_subsector(-1, 1, SubSector::new(StellarRift));
        sector.add_subsector(0, 1, SubSector::new(SolarSystem));
        sector.add_subsector(1, 1, SubSector::new(EmptySpace));
        sector.add_subsector(0, -1, SubSector::new(SolarSystem));

        use MissionType::*;
        sector.add_mission(0, -1, Mining);

        Self {
            state,
            outpost,
            sector,
        }
    }

    pub fn finish_turn(&mut self) {
        self.outpost.finish_turn(&mut self.state);
        self.state.finish_turn();
        self.sector.finish_turn();
    }

    pub fn increment_energy_level(&mut self, module_index: usize) {
        let id = self.outpost.module_id_by_index(module_index);
        self.outpost.increment_energy_level(&id)
    }
    pub fn decrement_energy_level(&mut self, module_index: usize) {
        let id = self.outpost.module_id_by_index(module_index);
        self.outpost.decrement_energy_level(&id)
    }
    pub fn assign_crew_member_to_module(&mut self, crew_member_index: usize, module_index: usize) {
        let crew_member_id = self.outpost.crew_member_id_by_index(crew_member_index);
        let module_id = self.outpost.module_id_by_index(module_index);
        self.outpost
            .assign_crew_member_to_module(&crew_member_id, &module_id)
    }
}

pub trait Entity {
    fn id(&self) -> &String;
}

#[derive(Serialize, Deserialize)]
pub struct Storage<T>
where
    T: Entity,
{
    data: HashMap<String, T>,
}

impl<T: Entity> Storage<T> {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn add(&mut self, entity: T) {
        self.data.insert(entity.id().clone(), entity);
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.values()
    }

    pub fn iter_mut(&mut self) -> ValuesMut<String, T> {
        self.data.values_mut()
    }

    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.data.retain(|_id, value| f(value))
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn id_by_index(&self, index: usize) -> Option<&String> {
        self.iter().nth(index).map(|e| e.id())
    }
}

impl<T: Entity> Index<&String> for Storage<T> {
    type Output = T;

    fn index(&self, id: &String) -> &Self::Output {
        &self.data[id]
    }
}

impl<T: Entity> IndexMut<&String> for Storage<T> {
    fn index_mut(&mut self, id: &String) -> &mut Self::Output {
        self.data.get_mut(id).unwrap()
    }
}

#[derive(Serialize, Deserialize)]
pub struct SortableStorage<T>
where
    T: Entity,
{
    data: Vec<T>,
}

impl<T: Entity> SortableStorage<T> {
    fn new() -> Self {
        Self { data: vec![] }
    }

    pub fn add(&mut self, entity: T) {
        self.data.push(entity)
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        self.data.iter_mut()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn id_by_index(&self, index: usize) -> Option<&String> {
        self.data.get(index).map(|e| e.id())
    }

    pub fn sort_by<F>(&mut self, compare: F)
    where
        F: FnMut(&T, &T) -> std::cmp::Ordering,
    {
        self.data.sort_by(compare)
    }
}

impl<T: Entity> Index<&String> for SortableStorage<T> {
    type Output = T;

    fn index(&self, id: &String) -> &Self::Output {
        self.data.iter().find(|v| v.id() == id).unwrap()
    }
}

impl<T: Entity> IndexMut<&String> for SortableStorage<T> {
    fn index_mut(&mut self, id: &String) -> &mut Self::Output {
        self.data.iter_mut().find(|v| v.id() == id).unwrap()
    }
}
