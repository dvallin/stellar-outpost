use crate::model::crew::CrewMember;
use crate::model::modules::living_quarters::LivingQuarters;
use crate::model::modules::power_generator::PowerGenerator;
use crate::model::modules::Module;
use crate::model::resources::Resources;
use core::cmp::min;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Outpost {
    pub resources: Resources,
    pub modules: Vec<Box<dyn Module>>,
    pub crew: Vec<CrewMember>,
}

impl Outpost {
    pub fn new() -> Self {
        let mut s = Self {
            crew: vec![],
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

        s.add_crew_member(CrewMember::new("a"));
        s.add_crew_member(CrewMember::new("b"));
        s.add_crew_member(CrewMember::new("c"));
        s.add_crew_member(CrewMember::new("d"));

        s
    }

    pub fn add_crew_member(&mut self, crew_member: CrewMember) {
        self.crew.push(crew_member)
    }
    pub fn mut_crew_member(&mut self, crew_name: &String) -> Option<&mut CrewMember> {
        self.crew.iter_mut().find(|m| crew_name.eq(m.name()))
    }
    pub fn crew_member(&self, crew_name: &String) -> Option<&CrewMember> {
        self.crew.iter().find(|m| crew_name.eq(m.name()))
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
    pub fn mut_module(&mut self, module_name: &String) -> Option<&mut Box<dyn Module>> {
        self.modules.iter_mut().find(|m| module_name.eq(m.name()))
    }
    pub fn module(&self, module_name: &String) -> Option<&Box<dyn Module>> {
        self.modules.iter().find(|m| module_name.eq(m.name()))
    }

    pub fn finish_turn(&mut self) {
        self.store_production();

        for c in self.crew.iter_mut() {
            c.finish_turn();
        }
        self.support_crew();
        self.support_modules();
        self.apply_status_effects();
    }

    pub fn assign_crew_member_to_module(&mut self, crew_name: &String, module_name: &String) {
        let crew = self.mut_crew_member(crew_name);
        crew.map(|c| c.assign_to_module(module_name));
    }

    pub fn consumption(&self) -> Resources {
        self.modules
            .iter()
            .map(|m| m.consumption())
            .reduce(|a, b| a + b)
            .unwrap_or_else(Resources::zero)
    }

    pub fn production(&self) -> Resources {
        self.modules
            .iter()
            .map(|m| {
                m.production(
                    self.crew
                        .iter()
                        .filter(|c| c.is_assigned_to_module(m))
                        .collect(),
                )
            })
            .reduce(|a, b| a + b)
            .unwrap_or_else(Resources::zero)
    }

    fn store_production(&mut self) {
        self.resources += self.production();
    }

    fn support_crew(&mut self) {
        let mut available_space = self.resources.living_space;
        for c in self.crew.iter_mut() {
            if self.resources.food > 0 {
                self.resources.food -= 1;
                c.eat();
            }
            if self.resources.water > 0 {
                self.resources.water -= 1;
                c.drink();
            }
            if available_space > 0 {
                available_space -= 1;
                c.rest();
            }
        }
    }

    fn support_modules(&mut self) {
        let missing_energy = self.consumption().energy - self.resources.energy;
        if missing_energy > 0 {
            self.cut_energy(missing_energy);
        }
    }

    fn cut_energy(&mut self, mut missing_energy: i32) {
        // sorting the modules by priority
        self.sort_modules_asc_by_priority();

        // now cut greedily as much energy as necessary
        for m in self.modules.iter_mut() {
            let module_consumption = m.consumption();
            let energy_cut = min(module_consumption.energy, missing_energy);

            missing_energy -= energy_cut;
            m.set_energy_level(module_consumption.energy - energy_cut);

            if missing_energy <= 0 {
                break;
            }
        }
    }

    fn apply_status_effects(&mut self) {
        for m in self.modules.iter() {
            m.status_effect().map(|e| {
                for c in self.crew.iter_mut() {
                    if c.is_assigned_to_module(m) {
                        c.apply_status_effect(&e)
                    }
                }
            });
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
