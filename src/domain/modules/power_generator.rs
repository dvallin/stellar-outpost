use crate::domain::crew::CrewMember;
use crate::domain::modules::Module;
use crate::domain::modules::ModulePriority;
use crate::domain::resources::Resources;
use crate::domain::status_effect::StatusEffect;

#[derive(Clone, PartialEq, Eq)]
pub struct PowerGenerator {
    production: i32,
    name: &'static str,
}

impl PowerGenerator {
    pub fn new(name: &'static str) -> Self {
        Self {
            production: 7,
            name,
        }
    }
}

impl Module for PowerGenerator {
    fn name(&self) -> &str {
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
