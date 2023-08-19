use crate::model::crew::CrewMember;
use crate::model::modules::Module;
use crate::model::modules::ModulePriority;
use crate::model::resources::Resources;
use crate::model::Entity;
use core::cmp::max;
use core::cmp::min;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};

use super::ModuleAssignmentDescription;
use super::ModuleEnergyLevelDescription;

#[derive(Serialize, Deserialize)]
pub struct PowerGenerator {
    id: String,
    energy_level: i32,
    name: String,
}

impl PowerGenerator {
    pub fn new(name: &str) -> Self {
        Self {
            id: nanoid!(),
            energy_level: 1,
            name: name.to_string(),
        }
    }
}

impl Entity for PowerGenerator {
    fn id(&self) -> &String {
        &self.id
    }
}

pub fn production_bonus(crew: &CrewMember) -> i32 {
    crew.apply_mood((1.0 + (crew.stats.engineering as f32 / 10.0)) / 2.0)
}

#[typetag::serde]
impl Module for PowerGenerator {
    fn name(&self) -> &String {
        &self.name
    }

    fn priority(&self) -> ModulePriority {
        ModulePriority::High
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
                    consumption: Resources::minerals(1),
                    production: Resources::energy(2),
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
        Resources::minerals(self.energy_level)
    }
    fn production(&self, crew: &Vec<&CrewMember>) -> Resources {
        if crew.len() == 0 {
            return Resources::zero();
        }

        let mut crew_bonus = 0;
        for member in crew.iter().take(self.energy_level as usize) {
            crew_bonus += production_bonus(member)
        }
        Resources::energy(self.energy_level * 2 + crew_bonus)
    }
    fn production_bonus(&self, crew: &CrewMember) -> Resources {
        Resources::energy(production_bonus(crew))
    }

    fn finish_turn(&self) {}
}
