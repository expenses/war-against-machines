use rand;
use rand::Rng;

use images;

const MOVES: usize = 30;

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
    pub image: usize,
    pub name: String,
    pub moves: usize,
    pub max_moves: usize
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
            image: images::FRIENDLY,
            name: format!("{} {}", first_name, last_name),
            moves: MOVES,
            max_moves: MOVES
        }
    }
}

pub struct Enemy {
    pub x: usize,
    pub y: usize,
    pub image: usize,
    pub name: String
}

impl Enemy {
    pub fn new(x: usize, y: usize) -> Enemy {
        Enemy {
            x: x,
            y: y,
            image: images::ENEMY,
            name: String::from("S122")
        }
    }
}