use crate::model::crew::CrewMember;
use crate::model::modules::Module;
use crate::model::modules::ModulePriority;
use crate::model::resources::Resources;
use serde::{Deserialize, Serialize};

use super::ModuleEnergyLevelDescription;

#[derive(Serialize, Deserialize)]
pub struct PowerGenerator {
    production: i32,
    name: String,
}

impl PowerGenerator {
    pub fn new(name: &str) -> Self {
        Self {
            production: 7,
            name: name.to_string(),
        }
    }
}

#[typetag::serde]
impl Module for PowerGenerator {
    fn name(&self) -> &String {
        &self.name
    }

    fn priority(&self) -> ModulePriority {
        ModulePriority::High
    }

    fn set_energy_level(&mut self, _level: i32) {}
    fn increment_energy_level(&mut self) {}
    fn decrement_energy_level(&mut self) {}
    fn energy_levels<'a>(
        &self,
        _crew: &Vec<&'a CrewMember>,
    ) -> Vec<ModuleEnergyLevelDescription<'a>> {
        vec![]
    }

    fn consumption(&self) -> Resources {
        Resources::zero()
    }
    fn production(&self, _crew: &Vec<&CrewMember>) -> Resources {
        Resources::energy(self.production)
    }

    fn finish_turn(&self) {}
}
