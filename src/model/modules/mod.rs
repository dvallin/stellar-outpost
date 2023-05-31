use crate::model::resources::Resources;

use super::{crew::CrewMember, status_effect::StatusEffect};
use std::slice::Iter;
use std::slice::IterMut;

pub trait Module {
    fn name(&self) -> &String;

    fn priority(&self) -> ModulePriority;

    fn set_energy_level(&mut self, level: i32);

    fn consumption(&self) -> Resources;
    fn production(&self, crew: Vec<&CrewMember>) -> Resources;
    fn status_effect(&self) -> Option<StatusEffect>;

    fn finish_turn(&self);
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ModulePriority {
    High,
    Mid,
    Low,
}

pub mod farm;
pub mod holo_deck;
pub mod living_quarters;
pub mod mine;
pub mod power_generator;
pub mod water_extractor;

pub struct ModuleRepository {
    modules: Vec<Box<dyn Module>>,
}

impl ModuleRepository {
    pub fn new() -> Self {
        Self { modules: vec![] }
    }

    pub fn modules(&self) -> Iter<Box<dyn Module>> {
        self.modules.iter()
    }
    pub fn modules_mut(&mut self) -> IterMut<Box<dyn Module>> {
        self.modules.iter_mut()
    }

    pub fn add(&mut self, crew: Box<dyn Module>) {
        self.modules.push(crew);
        self.sort_by_priority();
    }
    pub fn clear(&mut self) {
        self.modules.clear()
    }

    fn sort_by_priority(&mut self) {
        self.modules.sort_by(|a, b| {
            let priority_cmp = a.priority().cmp(&b.priority());
            match priority_cmp {
                std::cmp::Ordering::Equal => a.consumption().cmp(&b.consumption()),
                std::cmp::Ordering::Less => std::cmp::Ordering::Greater,
                std::cmp::Ordering::Greater => std::cmp::Ordering::Less,
            }
        });
    }
}
