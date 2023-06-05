use crate::model::resources::Resources;

use super::{crew::CrewMember, status_effect::StatusEffect};

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
