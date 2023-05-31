use crate::domain::crew::CrewMember;
use crate::domain::modules::Module;
use crate::domain::modules::ModulePriority;
use crate::domain::resources::Resources;
use crate::domain::stats::Stats;
use crate::domain::status_effect::StatusEffect;
use core::cmp::max;
use core::cmp::min;

pub struct HoloDeck {
    energy_level: i32,
    name: &'static str,
    stats: Stats,
}

impl HoloDeck {
    pub fn new(name: &'static str) -> Self {
        Self {
            energy_level: 1,
            name,
            stats: Stats::zero(),
        }
    }

    pub fn set_stats(&mut self, stats: Stats) {
        self.stats = stats
    }
}

impl Module for HoloDeck {
    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> ModulePriority {
        ModulePriority::Low
    }

    fn set_energy_level(&mut self, level: i32) {
        self.energy_level = min(max(level, 0), 3)
    }

    fn consumption(&self) -> Resources {
        Resources::energy(self.energy_level * 2)
    }
    fn production(&self, _crew: Vec<&CrewMember>) -> Resources {
        Resources::zero()
    }
    fn status_effect(&self) -> Option<StatusEffect> {
        Some(StatusEffect::GainStat(self.stats.clone()))
    }

    fn finish_turn(&self) {}
}
