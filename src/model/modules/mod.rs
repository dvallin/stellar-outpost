use crate::model::resources::Resources;

use super::{crew::CrewMember, status_effect::StatusEffect};

pub struct ModuleEnergyLevelDescription<'a> {
    pub is_active: bool,
    pub consumption: Resources,
    pub production: Resources,
    pub assignment: Option<ModuleAssignmentDescription<'a>>,
}
pub struct ModuleAssignmentDescription<'a> {
    pub crew_name: &'a String,
    pub production_bonus: Resources,
}

#[typetag::serde(tag = "type")]
pub trait Module {
    fn name(&self) -> &String;

    fn priority(&self) -> ModulePriority;

    fn set_energy_level(&mut self, level: i32);
    fn increment_energy_level(&mut self);
    fn decrement_energy_level(&mut self);
    fn energy_levels<'a>(
        &self,
        crew: &Vec<&'a CrewMember>,
    ) -> Vec<ModuleEnergyLevelDescription<'a>>;

    fn consumption(&self) -> Resources;
    fn production(&self, crew: &Vec<&CrewMember>) -> Resources;
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
