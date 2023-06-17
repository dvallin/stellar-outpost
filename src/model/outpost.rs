use crate::model::crew::CrewMember;
use crate::model::modules::living_quarters::LivingQuarters;
use crate::model::modules::power_generator::PowerGenerator;
use crate::model::modules::Module;
use crate::model::resources::Resources;
use serde::{Deserialize, Serialize};

use super::{
    game_state::GameState,
    modules::{
        farm::Farm, water_extractor::WaterExtractor, ModuleEnergyLevelDescription, ModulePriority,
    },
    stats::Stats,
};

#[derive(Serialize, Deserialize)]
pub struct Outpost {
    pub resources: Resources,
    pub modules: Vec<Box<dyn Module>>,
    pub crew: Vec<CrewMember>,
    pub cemetery: Vec<CrewMember>,
}

pub struct CrewDescription<'a> {
    pub name: &'a String,
    pub mood: i32,
    pub stats: &'a Stats,
    pub upkeep: Resources,
    pub assignment: Option<CrewAssignmentDescription<'a>>,
}
impl<'a> CrewDescription<'a> {
    pub fn assigned_module_name(&self) -> String {
        match &self.assignment {
            Some(a) => a.module_name.to_string(),
            None => String::from("not assigned"),
        }
    }
    pub fn flow(&self) -> Resources {
        let zero = Resources::zero();
        let bonus = match &self.assignment {
            Some(a) => &a.production_bonus,
            None => &zero,
        };
        bonus.clone()
    }
}

pub struct CrewAssignmentDescription<'a> {
    pub module_name: &'a String,
    pub production_bonus: Resources,
}

pub struct ModuleDescription<'a> {
    pub name: &'a String,
    pub priority: ModulePriority,
    pub production: Resources,
    pub consumption: Resources,
    pub energy_levels: Vec<ModuleEnergyLevelDescription<'a>>,
}

impl Outpost {
    pub fn new() -> Self {
        let mut s = Self {
            crew: vec![],
            cemetery: vec![],
            modules: vec![],

            resources: Resources {
                energy: 0,
                living_space: 0,
                minerals: 10,
                food: 10,
                water: 10,
            },
        };

        let power_generator = Box::new(PowerGenerator::new("power"));
        s.add_module(power_generator);

        let mut quarters = Box::new(LivingQuarters::new("quarters"));
        quarters.set_energy_level(2);
        s.add_module(quarters);

        let mut water = Box::new(WaterExtractor::new("water"));
        water.set_energy_level(1);
        s.add_module(water);

        let mut farm = Box::new(Farm::new("farm"));
        farm.set_energy_level(1);
        s.add_module(farm);

        s.add_crew_member(CrewMember::new("a"));
        s.add_crew_member(CrewMember::new("b"));
        s.add_crew_member(CrewMember::new("c"));
        s.add_crew_member(CrewMember::new("d"));

        s.assign_crew_member_to_module(0, 2);
        s.assign_crew_member_to_module(1, 3);

        s
    }

    pub fn add_crew_member(&mut self, crew_member: CrewMember) {
        self.crew.push(crew_member)
    }
    pub fn describe_crew_member<'a>(&'a self, crew: &'a CrewMember) -> CrewDescription<'a> {
        CrewDescription {
            name: crew.name(),
            mood: crew.mood(),
            stats: crew.stats(),
            upkeep: crew.upkeep(),
            assignment: crew
                .assigned_module()
                .as_ref()
                .and_then(|a| self.module(&a))
                .map(|m: &Box<dyn Module>| CrewAssignmentDescription {
                    module_name: m.name(),
                    production_bonus: m.production_bonus(crew),
                }),
        }
    }

    pub fn add_module(&mut self, module: Box<dyn Module>) {
        self.modules.push(module);
    }
    fn sort_modules_asc_by_priority(&mut self) {
        self.modules.sort_by(|a, b| {
            let priority_cmp = a.priority().cmp(&b.priority());
            match priority_cmp {
                std::cmp::Ordering::Equal => a.consumption().cmp(&b.consumption()),
                std::cmp::Ordering::Less => std::cmp::Ordering::Greater,
                std::cmp::Ordering::Greater => std::cmp::Ordering::Less,
            }
        });
    }
    pub fn module(&self, module_name: &String) -> Option<&Box<dyn Module>> {
        self.modules.iter().find(|m| module_name.eq(m.name()))
    }
    pub fn describe_module<'a>(&'a self, module: &'a Box<dyn Module>) -> ModuleDescription<'a> {
        let crew = self.crew_of_module(module);
        ModuleDescription {
            name: module.name(),
            priority: module.priority(),
            production: module.production(&crew),
            consumption: module.consumption(),
            energy_levels: module.energy_levels(&crew),
        }
    }

    pub fn finish_turn(&mut self, _state: &mut GameState) {
        self.store_production();

        for c in self.crew.iter_mut() {
            c.finish_turn();
            if !c.is_alive() {
                self.cemetery.push(c.clone());
            }
        }
        self.crew.retain(|c| c.is_alive());

        self.support_modules();
        self.support_crew();
    }

    pub fn assign_crew_member_to_module(&mut self, crew_index: usize, module_index: usize) {
        let module = &self.modules[module_index];
        let crew = &mut self.crew[crew_index];
        crew.assign_to_module(module.name());
    }

    pub fn consumption(&self) -> Resources {
        self.modules
            .iter()
            .map(|m| m.consumption())
            .reduce(|a, b| a + b)
            .unwrap_or_else(Resources::zero)
    }

    pub fn crew_of_module(&self, m: &Box<dyn Module>) -> Vec<&CrewMember> {
        self.crew
            .iter()
            .filter(|c| c.is_assigned_to_module(m))
            .collect()
    }

    pub fn production(&self) -> Resources {
        self.modules
            .iter()
            .map(|m| m.production(&self.crew_of_module(m)))
            .reduce(|a, b| a + b)
            .unwrap_or_else(Resources::zero)
    }

    fn store_production(&mut self) {
        self.resources += self.production();
    }

    pub fn crew_upkeep(&self) -> Resources {
        let len = self.crew.len() as i32;
        Resources {
            energy: 0,
            living_space: len,
            minerals: 0,
            food: len,
            water: len,
        }
    }

    fn support_crew(&mut self) {
        let mut available_space = self.resources.living_space;
        let mut available_energy = self.resources.energy;
        for c in self.crew.iter_mut() {
            let upkeep = c.upkeep();

            if upkeep.food == 0 && upkeep.minerals == 0 {
                c.eat()
            } else if self.resources.food >= upkeep.food
                && self.resources.minerals >= upkeep.minerals
            {
                self.resources.food -= upkeep.food;
                self.resources.minerals -= upkeep.minerals;
                c.eat();
            }

            if upkeep.water == 0 && upkeep.energy == 0 {
                c.drink()
            } else if self.resources.water >= upkeep.water && available_energy >= upkeep.energy {
                self.resources.water -= upkeep.water;
                available_energy -= upkeep.energy;
                c.drink();
            }

            if upkeep.living_space == 0 {
                c.rest()
            } else if available_space >= upkeep.living_space {
                available_space -= upkeep.living_space;
                c.rest();
            }
        }
    }

    fn support_modules(&mut self) {
        loop {
            let consumption = self.consumption();
            let can_self_sustain = self.resources.energy >= consumption.energy
                && self.resources.living_space >= consumption.living_space
                && self.resources.minerals >= consumption.minerals
                && self.resources.food >= consumption.food
                && self.resources.water >= consumption.water;
            if can_self_sustain {
                break;
            }
            self.cut_energy(consumption);
        }
        self.resources -= self.consumption();
    }

    fn cut_energy(&mut self, consumption: Resources) {
        // run over all modules starting with lowest priority
        self.sort_modules_asc_by_priority();
        for m in self.modules.iter_mut() {
            // find out if this module is a relevant consumer
            let delta = consumption.clone() - self.resources.clone();
            let consumption = m.consumption();
            let module_is_relevant = (delta.energy > 0 && consumption.energy > 0)
                || (delta.living_space > 0 && consumption.living_space > 0)
                || (delta.minerals > 0 && consumption.minerals > 0)
                || (delta.food > 0 && consumption.food > 0)
                || (delta.water > 0 && consumption.water > 0);
            if module_is_relevant {
                m.decrement_energy_level();
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::modules::farm::Farm;
    use crate::model::modules::mine::Mine;
    use crate::model::modules::water_extractor::WaterExtractor;
    use crate::model::modules::Module;
    use crate::model::outpost::Outpost;

    #[test]
    fn finish_turn_stores_production() {
        let mut o = Outpost::new();
        assert_eq!(10, o.resources.minerals);
        assert_eq!(10, o.resources.food);
        assert_eq!(10, o.resources.water);

        o.crew.clear();
        o.modules.push(Box::new(Mine::new("mine1")));
        o.modules.push(Box::new(Mine::new("mine2")));
        o.modules.push(Box::new(Farm::new("farm1")));
        o.modules
            .push(Box::new(WaterExtractor::new("water_extractor1")));

        o.finish_turn();

        assert_eq!(12, o.resources.minerals);
        assert_eq!(11, o.resources.food);
        assert_eq!(11, o.resources.water);
    }

    #[test]
    fn finish_turn_consumes_crew_upkeep() {
        let mut o = Outpost::new();
        assert_eq!(10, o.resources.food);
        assert_eq!(10, o.resources.water);

        o.finish_turn();

        assert_eq!(6, o.resources.food);
        assert_eq!(6, o.resources.water);
    }

    #[test]
    fn finish_turn_cuts_energy_levels() {
        let mut o = Outpost::new();

        let mut mine1 = Mine::new("mine1");
        mine1.set_energy_level(3);
        o.modules.push(Box::new(mine1));

        let mut mine2 = Mine::new("mine2");
        mine2.set_energy_level(3);
        o.modules.push(Box::new(mine2));

        let mut farm1 = Farm::new("farm1");
        farm1.set_energy_level(3);
        o.modules.push(Box::new(farm1));

        let mut water = WaterExtractor::new("water_extractor1");
        water.set_energy_level(3);
        o.modules.push(Box::new(water));

        o.finish_turn();

        let assert_consumption = |expected: i32, name: &str| {
            assert_eq!(
                expected,
                o.module(&name.to_string()).unwrap().consumption().energy,
                "{} should have energy {}",
                name,
                expected
            );
        };

        assert_consumption(0, "mine1");
        assert_consumption(0, "mine2");
        assert_consumption(3, "farm1");
        assert_consumption(2, "water_extractor1");
    }
}
