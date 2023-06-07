use serde::{Deserialize, Serialize};
use std::ops::Add;
use std::ops::AddAssign;

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub struct Stats {
    pub biology: i32,
    pub chemistry: i32,
    pub engineering: i32,
    pub geology: i32,
    pub astrophysics: i32,
    pub military: i32,
}

impl Stats {
    pub fn zero() -> Stats {
        Stats {
            biology: 0,
            chemistry: 0,
            engineering: 0,
            geology: 0,
            astrophysics: 0,
            military: 0,
        }
    }
    pub fn biology(biology: i32) -> Stats {
        Stats {
            biology,
            chemistry: 0,
            engineering: 0,
            geology: 0,
            astrophysics: 0,
            military: 0,
        }
    }
    pub fn chemistry(chemistry: i32) -> Stats {
        Stats {
            biology: 0,
            chemistry,
            engineering: 0,
            geology: 0,
            astrophysics: 0,
            military: 0,
        }
    }
    pub fn engineering(engineering: i32) -> Stats {
        Stats {
            biology: 0,
            chemistry: 0,
            engineering,
            geology: 0,
            astrophysics: 0,
            military: 0,
        }
    }
    pub fn geology(geology: i32) -> Stats {
        Stats {
            biology: 0,
            chemistry: 0,
            engineering: 0,
            geology,
            astrophysics: 0,
            military: 0,
        }
    }
    pub fn astrophysics(astrophysics: i32) -> Stats {
        Stats {
            biology: 0,
            chemistry: 0,
            engineering: 0,
            geology: 0,
            astrophysics,
            military: 0,
        }
    }
    pub fn military(military: i32) -> Stats {
        Stats {
            biology: 0,
            chemistry: 0,
            engineering: 0,
            geology: 0,
            astrophysics: 0,
            military,
        }
    }
}

impl Add for Stats {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            biology: self.biology + other.biology,
            chemistry: self.chemistry + other.chemistry,
            engineering: self.engineering + other.engineering,
            geology: self.geology + other.geology,
            astrophysics: self.astrophysics + other.astrophysics,
            military: self.military + other.military,
        }
    }
}

impl AddAssign for Stats {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            biology: self.biology + other.biology,
            chemistry: self.chemistry + other.chemistry,
            engineering: self.engineering + other.engineering,
            geology: self.geology + other.geology,
            astrophysics: self.astrophysics + other.astrophysics,
            military: self.military + other.military,
        };
    }
}
