use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GameState {
    pub logs: Vec<String>,
    pub current_turn: u32,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            logs: vec![],
            current_turn: 0,
        }
    }

    pub fn finish_turn(&mut self) {
        self.current_turn += 1;
    }

    fn log<'a>(&mut self, message: &'a str) {
        self.logs.push(String::from(message))
    }
}
