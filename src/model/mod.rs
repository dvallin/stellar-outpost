use self::{game_state::GameState, outpost::Outpost, region::Region};
use serde::{Deserialize, Serialize};

pub mod crew;
pub mod game_state;
pub mod modules;
pub mod outpost;
pub mod region;
pub mod resources;
pub mod stats;

#[derive(Serialize, Deserialize)]
pub struct Game {
    pub state: GameState,
    pub outpost: Outpost,
    pub region: Region,
}

impl Game {
    pub fn new() -> Self {
        Self {
            state: GameState::new(),
            outpost: Outpost::new(),
            region: Region::new(),
        }
    }

    pub fn finish_turn(&mut self) {
        self.outpost.finish_turn(&mut self.state);
        self.state.finish_turn();
        self.region.finish_turn();
    }
}
