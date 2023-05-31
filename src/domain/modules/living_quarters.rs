use crate::domain::crew::CrewMember;
use crate::domain::modules::Module;
use crate::domain::modules::ModulePriority;
use crate::domain::resources::Resources;
use crate::domain::status_effect::StatusEffect;
use core::cmp::max;
use core::cmp::min;

pub struct LivingQuarters {
    energy_level: i32,
    name: &'static str,
}

impl LivingQuarters {
    pub fn new(name: &'static str) -> Self {
        Self {
            energy_level: 1,
            name,
        }
    }
}

impl Module for LivingQuarters {
    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> ModulePriority {
        ModulePriority::High
    }

    fn set_energy_level(&mut self, level: i32) {
        self.energy_level = min(max(level, 0), 3)
    }

    fn consumption(&self) -> Resources {
        Resources::energy(self.energy_level)
    }
    fn production(&self, _crew: Vec<&CrewMember>) -> Resources {
        Resources::living_space(self.energy_level * 2)
    }
    fn status_effect(&self) -> Option<StatusEffect> {
        None
    }

    fn finish_turn(&self) {}
}
