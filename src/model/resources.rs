use std::ops::Add;
use std::ops::AddAssign;

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
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
