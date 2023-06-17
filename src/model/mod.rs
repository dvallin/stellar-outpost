use self::{game_state::GameState, outpost::Outpost};
use serde::{Deserialize, Serialize};

pub mod crew;
pub mod game_state;
pub mod modules;
pub mod outpost;
pub mod resources;
pub mod stats;

#[derive(Serialize, Deserialize)]
pub struct Game {
    pub state: GameState,
    pub outpost: Outpost,
}

impl Game {
    pub fn new() -> Self {
        Self {
            state: GameState::new(),
            outpost: Outpost::new(),
        }
    }

    pub fn finish_turn(&mut self) {
        self.outpost.finish_turn(&mut self.state);
        self.state.finish_turn();
    }
}
