use super::{modules::Module, resources::Resources, stats::Stats};

use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrewMember {
    pub stats: Stats,
    name: String,
    is_hungry: bool,
    is_thirsty: bool,
    is_tired: bool,
    assigned_module: Option<String>,
}

impl CrewMember {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            is_hungry: false,
            is_thirsty: false,
            is_tired: false,
            assigned_module: None,
            stats: Stats::zero(),
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn stats(&self) -> &Stats {
        &self.stats
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
        let mut m: i32 = 50;
        if self.is_hungry {
            m -= 30
        }
        if self.is_thirsty {
            m -= 30
        }
        if self.is_tired {
            m -= 30
        }
        m.clamp(0, 100)
    }
    pub fn apply_mood(&self, stat_bonus: f32) -> i32 {
        let mood_modifier = (self.mood() - 50) as f32 / 25.0;
        (stat_bonus * (1.0 + mood_modifier)).ceil() as i32
    }

    pub fn finish_turn(&mut self) {
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
    pub fn assign_to_module(&mut self, module_name: &String) {
        self.assigned_module = Some(module_name.clone())
    }
    pub fn is_assigned_to_module(&self, module: &Box<dyn Module>) -> bool {
        self.assigned_module
            .as_ref()
            .map(|a| a.eq(module.name()))
            .unwrap_or(false)
    }
}
