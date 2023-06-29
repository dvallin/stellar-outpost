use super::{modules::Module, resources::Resources, stats::Stats, Entity};

use nanoid::nanoid;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrewMember {
    id: String,
    pub stats: Stats,
    name: String,
    is_hungry: bool,
    is_thirsty: bool,
    is_tired: bool,
    health: i32,
    assigned_module: Option<String>,
    assigned_mission: Option<String>,
}

impl CrewMember {
    pub fn new(name: String) -> Self {
        Self {
            id: nanoid!(),
            stats: Stats::zero(),
            name,
            is_hungry: false,
            is_thirsty: false,
            is_tired: false,
            health: 5,
            assigned_module: None,
            assigned_mission: None,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn stats(&self) -> &Stats {
        &self.stats
    }
    pub fn health(&self) -> i32 {
        (self.health * 20) as i32
    }
    pub fn is_alive(&self) -> bool {
        self.health > 0
    }
    pub fn upkeep(&self) -> Resources {
        Resources {
            energy: 0,
            living_space: 1,
            minerals: 0,
            food: 1,
            water: 1,
        }
    }
    pub fn mood(&self) -> i32 {
        let mut m: i32 = 70;
        if self.is_hungry {
            m -= 30
        }
        if self.is_thirsty {
            m -= 70
        }
        if self.is_tired {
            m -= 30
        }
        2 * m.clamp(0, 100) - 100
    }
    pub fn apply_mood(&self, stat_bonus: f32) -> i32 {
        let mood_modifier = self.mood() as f32 / 50.0;
        (stat_bonus * (1.0 + mood_modifier)).ceil() as i32
    }

    pub fn finish_turn(&mut self) {
        if self.is_hungry {
            self.health -= 1;
        }
        if self.is_thirsty {
            self.health -= 1;
        }
        self.is_hungry = true;
        self.is_thirsty = true;
        self.is_tired = true;
    }
    pub fn eat(&mut self) {
        self.is_hungry = false;
    }
    pub fn drink(&mut self) {
        self.is_thirsty = false;
    }
    pub fn rest(&mut self) {
        self.is_tired = false;
    }
    pub fn assigned_module(&self) -> &Option<String> {
        &self.assigned_module
    }
    pub fn assign_to_module(&mut self, module_id: &String) {
        self.assigned_module = Some(module_id.clone())
    }
    pub fn assigned_mission(&self) -> &Option<String> {
        &self.assigned_mission
    }
    pub fn assign_to_mission(&mut self, mission_id: &String) {
        self.assigned_mission = Some(mission_id.clone())
    }
    pub fn is_assigned_to_module(&self, module: &Box<dyn Module>) -> bool {
        self.assigned_module
            .as_ref()
            .map(|a| a.eq(module.id()))
            .unwrap_or(false)
    }
}

impl Entity for CrewMember {
    fn id(&self) -> &String {
        &self.id
    }
}
