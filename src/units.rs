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
    pub max_moves: usize,
    pub health: u8
}

impl Squaddie {
   pub fn new(x: usize, y: usize) -> Squaddie {
        // Generate a random name
        let mut rng = rand::thread_rng();
        let first_name = FIRST_NAMES[rng.gen_range(0, FIRST_NAMES.len())];
        let last_name = LAST_NAMES[rng.gen_range(0, LAST_NAMES.len())];

        Squaddie {
            x,
            y,
            image: images::FRIENDLY,
            name: format!("{} {}", first_name, last_name),
            moves: MOVES,
            max_moves: MOVES,
            health: 100
        }
    }

    pub fn fire_at(&self, enemy: &Enemy) {
        println!("{}", (self.x as f32 - enemy.x as f32).hypot(self.y as f32 - enemy.y as f32));
    }
}

pub struct Enemy {
    pub x: usize,
    pub y: usize,
    pub image: usize,
    pub name: String,
    pub health: u8
}

impl Enemy {
    pub fn new(x: usize, y: usize) -> Enemy {
        Enemy {
            x,
            y,
            image: images::ENEMY,
            name: String::from("S122"),
            health: 100
        }
    }
}