extern crate rand;

use rand::Rng;
use images;

const FIRST_NAMES: [&str; 6] = [
    "David",
    "Dale",
    "Robert",
    "Lucy",
    "Ashley",
    "Mia",
];

const LAST_NAMES: [&str; 3] = [
    "Cooper",
    "Yang",
    "Smith"
];

pub struct Squaddie {
    pub x: usize,
    pub y: usize,
    pub sprite: usize,
    pub name: String,
}

impl Squaddie {
   pub fn new(x: usize, y: usize) -> Squaddie {
        // Generate a random name
        let mut rng = rand::thread_rng();
        let first_name = FIRST_NAMES[rng.gen_range(0, FIRST_NAMES.len())];
        let last_name = LAST_NAMES[rng.gen_range(0, LAST_NAMES.len())];

        Squaddie {
            x: x,
            y: y,
            sprite: images::FRIENDLY,
            name: format!("{} {}", first_name, last_name),
        }
    }
}

pub struct Enemy {
    pub x: usize,
    pub y: usize,
    pub sprite: usize
}

impl Enemy {
    pub fn new(x: usize, y: usize) -> Enemy {
        Enemy {
            x: x,
            y: y,
            sprite: images::ENEMY
        }
    }
}