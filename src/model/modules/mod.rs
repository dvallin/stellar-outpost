use crate::model::resources::Resources;

use super::{crew::CrewMember, Entity};

pub struct ModuleEnergyLevelDescription<'a> {
    pub is_active: bool,
    pub consumption: Resources,
    pub production: Resources,
    pub assignment: Option<ModuleAssignmentDescription<'a>>,
}
impl<'a> ModuleEnergyLevelDescription<'a> {
    pub fn assigned_crew_name(&self) -> String {
        match &self.assignment {
            Some(a) => a.crew_name.to_string(),
            None => String::from("empty"),
        }
    }
    pub fn flow(&self) -> Resources {
        let zero = Resources::zero();
        let bonus = match &self.assignment {
            Some(a) => &a.production_bonus,
            None => &zero,
        };
        self.production.clone() - self.consumption.clone() + bonus.clone()
    }
}

pub struct ModuleAssignmentDescription<'a> {
    pub crew_name: &'a String,
    pub production_bonus: Resources,
}

#[typetag::serde(tag = "type")]
pub trait Module: Entity {
    fn name(&self) -> &String;

    fn priority(&self) -> ModulePriority;

    fn set_energy_level(&mut self, level: i32);
    fn increment_energy_level(&mut self);
    fn decrement_energy_level(&mut self);
    fn energy_levels<'a>(
        &self,
        crew: &Vec<&'a CrewMember>,
    ) -> Vec<ModuleEnergyLevelDescription<'a>>;
    fn available_slots<'a>(&self, crew: &Vec<&'a CrewMember>) -> usize;

    fn consumption(&self) -> Resources;
    fn production(&self, crew: &Vec<&CrewMember>) -> Resources;
    fn production_bonus(&self, crew: &CrewMember) -> Resources;

    fn finish_turn(&self);
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ModulePriority {
    High,
    Mid,
    Low,
}

pub mod farm;
pub mod living_quarters;
pub mod power_generator;
pub mod water_extractor;
