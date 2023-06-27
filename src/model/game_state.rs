use base64::{engine::general_purpose, Engine as _};
use rand_core::SeedableRng;
use rand_pcg::Pcg64;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GameState {
    pub seed: String,
    pub logs: Vec<String>,
    pub current_turn: u32,
    pub rng: Pcg64,
}

impl GameState {
    pub fn new(seed: u64) -> Self {
        let data: [u8; 8] = seed.to_be_bytes();
        let encoded_seed = general_purpose::STANDARD.encode(&data);
        Self {
            seed: encoded_seed,
            logs: vec![],
            current_turn: 0,
            rng: SeedableRng::seed_from_u64(seed),
        }
    }

    pub fn finish_turn(&mut self) {
        self.current_turn += 1;
    }

    fn log<'a>(&mut self, message: &'a str) {
        self.logs.push(String::from(message))
    }
}
