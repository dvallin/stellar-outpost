use crate::model::crew::CrewMember;
use crate::model::modules::Module;
use crate::model::modules::ModulePriority;
use crate::model::resources::Resources;
use crate::model::stats::Stats;
use crate::model::status_effect::StatusEffect;
use core::cmp::max;
use core::cmp::min;
use serde::{Deserialize, Serialize};

use super::ModuleEnergyLevelDescription;

#[derive(Serialize, Deserialize)]
pub struct HoloDeck {
    energy_level: i32,
    name: String,
    stats: Stats,
}

impl HoloDeck {
    pub fn new(name: &str) -> Self {
        Self {
            energy_level: 1,
            name: name.to_string(),
            stats: Stats::zero(),
        }
    }

    pub fn set_stats(&mut self, stats: Stats) {
        self.stats = stats
    }
}

#[typetag::serde]
impl Module for HoloDeck {
    fn name(&self) -> &String {
        &self.name
    }

    fn priority(&self) -> ModulePriority {
        ModulePriority::Low
    }

    fn set_energy_level(&mut self, level: i32) {
        self.energy_level = min(max(level, 0), 3)
    }
    fn increment_energy_level(&mut self) {
        self.set_energy_level(self.energy_level + 1)
    }
    fn decrement_energy_level(&mut self) {
        self.set_energy_level(self.energy_level - 1)
    }
    fn energy_levels<'a>(
        &self,
        _crew: &Vec<&'a CrewMember>,
    ) -> Vec<ModuleEnergyLevelDescription<'a>> {
        let mut levels: Vec<ModuleEnergyLevelDescription> = vec![];
        for e in 1..4 {
            if e <= self.energy_level {
                levels.push(ModuleEnergyLevelDescription {
                    is_active: true,
                    consumption: Resources::energy(2),
                    production: Resources::zero(),
                    assignment: None,
                })
            } else {
                levels.push(ModuleEnergyLevelDescription {
                    is_active: false,
                    consumption: Resources::zero(),
                    production: Resources::zero(),
                    assignment: None,
                })
            }
        }
        levels
    }

    fn consumption(&self) -> Resources {
        Resources::energy(self.energy_level * 2)
    }
    fn production(&self, _crew: &Vec<&CrewMember>) -> Resources {
        Resources::zero()
    }
    fn status_effect(&self) -> Option<StatusEffect> {
        Some(StatusEffect::GainStat(self.stats.clone()))
    }

    fn finish_turn(&self) {}
}
