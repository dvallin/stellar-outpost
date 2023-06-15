use crate::model::crew::CrewMember;
use crate::model::modules::Module;
use crate::model::modules::ModulePriority;
use crate::model::resources::Resources;
use core::cmp::max;
use core::cmp::min;
use serde::{Deserialize, Serialize};

use super::ModuleAssignmentDescription;
use super::ModuleEnergyLevelDescription;

#[derive(Serialize, Deserialize)]
pub struct Mine {
    energy_level: i32,
    name: String,
}

impl Mine {
    pub fn new(name: &str) -> Self {
        Self {
            energy_level: 1,
            name: name.to_string(),
        }
    }
}

pub fn production_bonus(crew: &CrewMember) -> i32 {
    crew.apply_mood((1.0 + (crew.stats.geology as f32) / 10.0) / 2.0)
}

#[typetag::serde]
impl Module for Mine {
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
        crew: &Vec<&'a CrewMember>,
    ) -> Vec<ModuleEnergyLevelDescription<'a>> {
        let mut levels: Vec<ModuleEnergyLevelDescription> = vec![];
        for e in 1..4 {
            if e <= self.energy_level {
                let assignment = crew
                    .get((e - 1) as usize)
                    .map(|c| ModuleAssignmentDescription {
                        crew_name: c.name(),
                        production_bonus: self.production_bonus(c),
                    });

                levels.push(ModuleEnergyLevelDescription {
                    is_active: true,
                    consumption: Resources::energy(1),
                    production: Resources::minerals(1),
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
    fn available_slots<'a>(&self, crew: &Vec<&'a CrewMember>) -> usize {
        std::cmp::max(self.energy_level - crew.len() as i32, 0) as usize
    }

    fn consumption(&self) -> Resources {
        Resources::energy(self.energy_level)
    }
    fn production(&self, crew: &Vec<&CrewMember>) -> Resources {
        let mut crew_bonus = 0;
        for member in crew.iter().take(self.energy_level as usize) {
            crew_bonus += production_bonus(member)
        }
        Resources::minerals(self.energy_level + crew_bonus)
    }
    fn production_bonus(&self, crew: &CrewMember) -> Resources {
        Resources::minerals(production_bonus(crew))
    }

    fn finish_turn(&self) {}
}

#[cfg(test)]
mod tests {
    use crate::model::{crew::CrewMember, stats::Stats};

    use super::production_bonus;

    #[test]
    fn calculate_production_bonus() {
        let assert_bonus = |expected: i32, geology: i32| {
            let mut member = CrewMember::new("test");
            member.stats = Stats::geology(geology);

            assert_eq!(
                expected,
                production_bonus(&member),
                "{} geology should create production bonus {}",
                geology,
                expected
            );
        };
        assert_bonus(1, 0);
        assert_bonus(1, 10);
        assert_bonus(2, 20);
        assert_bonus(2, 30);
        assert_bonus(3, 40);
        assert_bonus(3, 50);
        assert_bonus(4, 60);
        assert_bonus(4, 70);
        assert_bonus(5, 80);
        assert_bonus(5, 90);
        assert_bonus(6, 100);
    }
}
