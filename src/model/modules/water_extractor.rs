use crate::model::crew::CrewMember;
use crate::model::modules::Module;
use crate::model::modules::ModulePriority;
use crate::model::resources::Resources;
use crate::model::status_effect::StatusEffect;
use core::cmp::max;
use core::cmp::min;
use serde::{Deserialize, Serialize};

use super::ModuleAssignmentDescription;
use super::ModuleEnergyLevelDescription;

#[derive(Serialize, Deserialize)]
pub struct WaterExtractor {
    energy_level: i32,
    name: String,
}

impl WaterExtractor {
    pub fn new(name: &str) -> Self {
        Self {
            energy_level: 1,
            name: name.to_string(),
        }
    }
}

pub fn production_bonus(crew: &CrewMember) -> i32 {
    ((crew.stats.chemistry as f32) / 3.0).ceil() as i32
}

#[typetag::serde]
impl Module for WaterExtractor {
    fn name(&self) -> &String {
        &self.name
    }

    fn priority(&self) -> ModulePriority {
        ModulePriority::Mid
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
        crew: &Vec<&'a CrewMember>,
    ) -> Vec<ModuleEnergyLevelDescription<'a>> {
        let mut levels: Vec<ModuleEnergyLevelDescription> = vec![];
        for e in 1..4 {
            if e <= self.energy_level {
                let assignment = crew.get(e as usize).map(|c| ModuleAssignmentDescription {
                    crew_name: c.name(),
                    production_bonus: Resources::water(production_bonus(c)),
                });

                levels.push(ModuleEnergyLevelDescription {
                    is_active: true,
                    consumption: Resources::energy(1),
                    production: Resources::water(1),
                    assignment,
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
        Resources::energy(self.energy_level)
    }

    fn production(&self, crew: &Vec<&CrewMember>) -> Resources {
        let mut crew_bonus = 0;
        for member in crew.iter().take(self.energy_level as usize) {
            crew_bonus += production_bonus(member)
        }
        Resources::water(self.energy_level + crew_bonus)
    }
    fn status_effect(&self) -> Option<StatusEffect> {
        None
    }

    fn finish_turn(&self) {}
}

#[cfg(test)]
mod tests {
    use crate::model::{crew::CrewMember, stats::Stats};

    use super::production_bonus;

    #[test]
    fn calculate_production_bonus() {
        let assert_bonus = |expected: i32, chemistry: i32| {
            let mut member = CrewMember::new("test");
            member.stats = Stats::chemistry(chemistry);

            assert_eq!(
                expected,
                production_bonus(&member),
                "{} chemistry should create production bonus {}",
                chemistry,
                expected
            );
        };
        assert_bonus(0, 0);
        assert_bonus(1, 1);
        assert_bonus(1, 2);
        assert_bonus(1, 3);
        assert_bonus(2, 4);
        assert_bonus(2, 5);
        assert_bonus(2, 6);
        assert_bonus(3, 7);
        assert_bonus(3, 8);
        assert_bonus(3, 9);
        assert_bonus(4, 10);
    }
}
