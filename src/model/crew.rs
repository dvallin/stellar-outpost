use super::{modules::Module, stats::Stats, status_effect::StatusEffect};

#[derive(Clone, PartialEq, Eq)]
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
    pub fn assign_to_module(&mut self, module_name: &String) {
        self.assigned_module = Some(module_name.clone())
    }
    pub fn is_assigned_to_module(&self, module: &Box<dyn Module>) -> bool {
        self.assigned_module
            .as_ref()
            .map(|a| a.eq(module.name()))
            .unwrap_or(false)
    }

    pub fn apply_status_effect(&mut self, e: &StatusEffect) {
        match e {
            StatusEffect::GainStat(s) => self.stats += s.clone(),
        }
    }
}
