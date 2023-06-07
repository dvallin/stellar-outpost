use crate::model::crew::CrewMember;
use crate::model::modules::Module;
use crate::model::modules::ModulePriority;
use crate::model::resources::Resources;
use crate::model::status_effect::StatusEffect;
use serde::{Deserialize, Serialize};

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

    fn consumption(&self) -> Resources {
        Resources::zero()
    }
    fn production(&self, _crew: Vec<&CrewMember>) -> Resources {
        Resources::energy(self.production)
    }
    fn status_effect(&self) -> Option<StatusEffect> {
        None
    }

    fn finish_turn(&self) {}
}
