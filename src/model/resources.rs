use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub struct Resources {
    pub energy: i32,
    pub living_space: i32,

    pub minerals: i32,
    pub food: i32,
    pub water: i32,
}

impl Resources {
    pub fn zero() -> Resources {
        Resources {
            energy: 0,
            living_space: 0,
            minerals: 0,
            food: 0,
            water: 0,
        }
    }

    pub fn energy(energy: i32) -> Resources {
        Resources {
            energy,
            living_space: 0,
            minerals: 0,
            food: 0,
            water: 0,
        }
    }

    pub fn living_space(living_space: i32) -> Resources {
        Resources {
            energy: 0,
            living_space,
            minerals: 0,
            food: 0,
            water: 0,
        }
    }

    pub fn minerals(minerals: i32) -> Resources {
        Resources {
            energy: 0,
            living_space: 0,
            minerals,
            food: 0,
            water: 0,
        }
    }

    pub fn food(food: i32) -> Resources {
        Resources {
            energy: 0,
            living_space: 0,
            minerals: 0,
            food,
            water: 0,
        }
    }

    pub fn water(water: i32) -> Resources {
        Resources {
            energy: 0,
            living_space: 0,
            minerals: 0,
            food: 0,
            water,
        }
    }
}

impl Add for Resources {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            energy: self.energy + other.energy,
            living_space: self.living_space + other.living_space,
            minerals: self.minerals + other.minerals,
            food: self.food + other.food,
            water: self.water + other.water,
        }
    }
}
impl Sub for Resources {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            energy: self.energy - other.energy,
            living_space: self.living_space - other.living_space,
            minerals: self.minerals - other.minerals,
            food: self.food - other.food,
            water: self.water - other.water,
        }
    }
}

impl AddAssign for Resources {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            energy: other.energy,
            living_space: other.living_space,
            minerals: self.minerals + other.minerals,
            food: self.food + other.food,
            water: self.water + other.water,
        };
    }
}

impl SubAssign for Resources {
    fn sub_assign(&mut self, other: Self) {
        *self = Self {
            energy: self.energy,
            living_space: self.living_space,
            minerals: std::cmp::max(self.minerals - other.minerals, 0),
            food: std::cmp::max(self.food - other.food, 0),
            water: std::cmp::max(self.water - other.water, 0),
        };
    }
}

impl Mul<i32> for Resources {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self {
        Self {
            energy: rhs * self.energy,
            living_space: rhs * self.living_space,
            minerals: rhs * self.minerals,
            food: rhs * self.food,
            water: rhs * self.water,
        }
    }
}
