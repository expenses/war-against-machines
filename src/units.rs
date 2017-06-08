use rand;
use rand::Rng;

use images;
use weapons::{Weapon, WeaponType};

const MOVES: usize = 30;

const FIRST_NAMES: [&str; 9] = [
    "David",
    "Dale",
    "Robert",
    "Lucy",
    "Ashley",
    "Mia",
    "JC",
    "Paul",
    "Heisenberg"
];

const LAST_NAMES: [&str; 7] = [
    "Cooper",
    "Yang",
    "Smith",
    "Denton",
    "Simons",
    "Rivers",
    "Savage"
];

pub struct Squaddie {
    pub x: usize,
    pub y: usize,
    pub weapon: Weapon,
    pub image: usize,
    pub name: String,
    pub moves: usize,
    pub max_moves: usize,
    pub health: i16
}

impl Squaddie {
   pub fn new(x: usize, y: usize) -> Squaddie {
        // Generate a random name
        let mut rng = rand::thread_rng();
        let first = &FIRST_NAMES;
        let last = &LAST_NAMES;

        let first_name = rng.choose(first).unwrap();
        let last_name = rng.choose(last).unwrap();

        Squaddie {
            x, y,
            weapon: Weapon::new(WeaponType::Rifle),
            image: images::FRIENDLY,
            name: format!("{} {}", first_name, last_name),
            moves: MOVES,
            max_moves: MOVES,
            health: 100
        }
    }

    pub fn move_to(&mut self, x: usize, y: usize, cost: usize) -> bool {
        if self.moves >= cost {
            self.x = x;
            self.y = y;
            self.moves -= cost;
            return true;
        }

        false
    }

    pub fn fire_at(&mut self, enemy: &mut Enemy) {
        let cost = 5;
        let damage = 25;

        if self.moves < cost || !enemy.alive() {
            return;
        }

        self.moves -= cost;

        let distance = (self.x as f32 - enemy.x as f32).hypot(self.y as f32 - enemy.y as f32);
        let hit_chance = chance_to_hit(distance);
        let random = rand::random::<f32>();

        if hit_chance > random {
            enemy.health -= damage;
        }
    }

    pub fn alive(&self) -> bool {
        self.health > 0
    }
}

fn chance_to_hit(distance: f32) -> f32 {
    1.0 / (1.0 + 0.02 * 4.0_f32.powf(distance / 3.0))
}

pub struct Enemy {
    pub x: usize,
    pub y: usize,
    pub image: usize,
    pub name: String,
    pub health: i16
}

impl Enemy {
    pub fn new(x: usize, y: usize) -> Enemy {
        Enemy {
            x, y,
            image: images::ENEMY,
            name: String::from("S122"),
            health: 100
        }
    }

    pub fn alive(&self) -> bool {
        self.health > 0
    }
}