use crate::model::crew::CrewMember;
use crate::model::modules::Module;
use crate::model::resources::Resources;
use serde::{Deserialize, Serialize};

use super::{
    game_state::GameState,
    modules::{ModuleEnergyLevelDescription, ModulePriority},
    sector::ActiveMission,
    stats::Stats,
    Entity, SortableStorage, Storage,
};

#[derive(Serialize, Deserialize)]
pub struct Outpost {
    resources: Resources,
    modules: SortableStorage<ModuleBox>,
    crew: Storage<CrewMember>,
    cemetery: Vec<CrewMember>,
    mission_preparation: Option<MissionPreparation>,
}

#[derive(Serialize, Deserialize)]
pub struct MissionPreparation {
    pub crew_ids: Vec<String>,
    pub mission_id: String,
    pub turns: u16,
}

pub struct MissionPreparationIssue {}

#[derive(Serialize, Deserialize)]
pub struct ModuleBox {
    module: Box<dyn Module>,
}

impl ModuleBox {
    pub fn unwrap(&self) -> &Box<dyn Module> {
        &self.module
    }
    pub fn unwrap_mut(&mut self) -> &mut Box<dyn Module> {
        &mut self.module
    }
    pub fn new(module: Box<dyn Module>) -> Self {
        Self { module }
    }
}

impl Entity for ModuleBox {
    fn id(&self) -> &String {
        self.module.id()
    }
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
        Self {
            crew: Storage::new(),
            cemetery: vec![],
            modules: SortableStorage::new(),

            resources: Resources {
                energy: 0,
                living_space: 0,
                minerals: 10,
                food: 10,
                water: 10,
            },

            mission_preparation: None,
        }
    }

    /** Modules */
    pub fn add_module(&mut self, module: Box<dyn Module>) {
        self.modules.add(ModuleBox::new(module));
    }
    pub fn module_id_by_index(&self, module_index: usize) -> String {
        self.modules.id_by_index(module_index).unwrap().clone()
    }
    pub fn get_module(&self, module_id: &String) -> &Box<dyn Module> {
        self.modules[module_id].unwrap()
    }
    pub fn modules_len(&self) -> usize {
        self.modules.len()
    }
    pub fn modules(&self) -> Vec<&Box<dyn Module>> {
        self.modules.iter().map(|a| a.unwrap()).collect()
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
    pub fn increment_energy_level(&mut self, module_id: &String) {
        self.modules[module_id]
            .unwrap_mut()
            .increment_energy_level()
    }
    pub fn decrement_energy_level(&mut self, module_id: &String) {
        self.modules[module_id]
            .unwrap_mut()
            .decrement_energy_level()
    }

    /** Crew */
    pub fn add_crew_member(&mut self, crew_member: CrewMember) {
        self.crew.add(crew_member)
    }
    pub fn crew_member_id_by_index(&self, crew_member_index: usize) -> String {
        self.crew.id_by_index(crew_member_index).unwrap().clone()
    }
    pub fn get_crew_member(&self, crew_id: &String) -> &CrewMember {
        &self.crew[crew_id]
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
                .map(|a| self.get_module(&a))
                .map(|m: &Box<dyn Module>| CrewAssignmentDescription {
                    module_name: m.name(),
                    production_bonus: m.production_bonus(crew),
                }),
        }
    }
    pub fn crew(&self) -> Vec<&CrewMember> {
        self.crew.iter().collect()
    }
    pub fn crew_len(&self) -> usize {
        self.crew.len()
    }
    pub fn assign_crew_member_to_module(&mut self, crew_member_id: &String, module_id: &String) {
        let crew_member = &mut self.crew[crew_member_id];
        crew_member.assign_to_module(module_id);
    }
    pub fn crew_of_module(&self, m: &Box<dyn Module>) -> Vec<&CrewMember> {
        self.crew
            .iter()
            .filter(|c| c.is_assigned_to_module(m))
            .collect()
    }

    /** Mission */
    pub fn prepare_mission(&mut self, mission_id: &String, turns: u16) {
        self.mission_preparation = Some(MissionPreparation {
            mission_id: mission_id.clone(),
            crew_ids: vec![],
            turns,
        });
    }
    pub fn prepare_crew_member_for_mission(&mut self, crew_member_id: &String) {
        if let Some(preparation) = &mut self.mission_preparation {
            preparation.crew_ids.push(crew_member_id.clone());
        }
    }
    pub fn set_prepare_for_turns(&mut self, turns: u16) {
        if let Some(preparation) = &mut self.mission_preparation {
            preparation.turns = turns;
        }
    }
    pub fn start_mission(&mut self) -> ActiveMission {
        let mut crew = vec![];
        let preparation = self.mission_preparation.as_ref().unwrap();
        for crew_member_id in &preparation.crew_ids {
            let mut crew_member = self.crew.remove(crew_member_id).unwrap();
            crew_member.assign_to_mission(&preparation.mission_id);
            crew.push(crew_member)
        }

        let upkeep_for_turns =
            Resources::food(preparation.turns.into()) + Resources::water(preparation.turns.into());
        if upkeep_for_turns < self.resources {
            // subAssign takes ownership of upkee_for_turns so clone it
            self.resources -= upkeep_for_turns.clone();
        }

        let mission = ActiveMission::new(&preparation.mission_id, upkeep_for_turns, crew);
        self.mission_preparation = None;
        mission
    }

    /** Resources */
    pub fn resources(&self) -> &Resources {
        &self.resources
    }
    pub fn production(&self) -> Resources {
        self.modules
            .iter()
            .map(|m| m.unwrap())
            .map(|m| m.production(&self.crew_of_module(m)))
            .reduce(|a, b| a + b)
            .unwrap_or_else(Resources::zero)
    }
    pub fn consumption(&self) -> Resources {
        self.modules
            .iter()
            .map(|m| m.unwrap())
            .map(|m| m.consumption())
            .reduce(|a, b| a + b)
            .unwrap_or_else(Resources::zero)
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

    /** Finish turn */
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

    fn sort_modules_asc_by_priority(&mut self) {
        self.modules.sort_by(|a, b| {
            let left = a.unwrap();
            let right = b.unwrap();
            let priority_cmp = left.priority().cmp(&right.priority());
            match priority_cmp {
                std::cmp::Ordering::Equal => left.consumption().cmp(&right.consumption()),
                std::cmp::Ordering::Less => std::cmp::Ordering::Greater,
                std::cmp::Ordering::Greater => std::cmp::Ordering::Less,
            }
        });
    }

    fn store_production(&mut self) {
        self.resources += self.production();
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
            let module = m.unwrap_mut();
            let delta = consumption.clone() - self.resources.clone();
            let consumption = module.consumption();
            let module_is_relevant = (delta.energy > 0 && consumption.energy > 0)
                || (delta.living_space > 0 && consumption.living_space > 0)
                || (delta.minerals > 0 && consumption.minerals > 0)
                || (delta.food > 0 && consumption.food > 0)
                || (delta.water > 0 && consumption.water > 0);
            if module_is_relevant {
                module.decrement_energy_level();
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
