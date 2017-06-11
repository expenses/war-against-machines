use rand;
use rand::Rng;

use images;
use weapons::Weapon;
use weapons::WeaponType::{Rifle, MachineGun, PlasmaRifle};

const FIRST_NAMES: &[&str; 9] = &[
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

const LAST_NAMES: &[&str; 7] = &[
    "Cooper",
    "Yang",
    "Smith",
    "Denton",
    "Simons",
    "Rivers",
    "Savage"
];

pub enum UnitType {
    Squaddie,
    Robot
}

pub enum UnitSide {
    Friendly,
    // Neutral,
    Enemy
}

pub struct Unit {
    pub tag: UnitType,
    pub side: UnitSide,
    pub x: usize,
    pub y: usize,
    pub weapon: Weapon,
    pub image: usize,
    pub dead_image: usize,
    pub name: String,
    pub moves: usize,
    pub max_moves: usize,
    pub health: u8,
    pub max_health: u8
}

impl Unit {
    pub fn new(tag: UnitType, side: UnitSide, x: usize, y: usize) -> Unit {
        let (weapon, image, dead_image, name, max_moves, max_health) = match tag {
            UnitType::Squaddie => {
                // Generate a random name
                let mut rng = rand::thread_rng();
                let first = rng.choose(FIRST_NAMES).unwrap();
                let last = rng.choose(LAST_NAMES).unwrap();

                let weapon_type = if rng.gen::<bool>() { Rifle } else { MachineGun };

                (
                    Weapon::new(weapon_type),
                    images::FRIENDLY, images::DEAD_FRIENDLY,
                    format!("{} {}", first, last), 30, 75
                )
            },
            UnitType::Robot => {
                (
                    Weapon::new(PlasmaRifle),
                    images::ENEMY, images::DEAD_ENEMY,
                    format!("ROBOT"), 25, 150
                )
            }
        };

        Unit {
            tag, side, x, y, weapon, image, dead_image, name, max_moves, max_health,
            moves: max_moves,
            health: max_health
        }
    }

    pub fn alive(&self) -> bool {
        self.health > 0
    }

    pub fn image(&self) -> usize {
        if self.alive() {
            self.image
        } else {
            self.dead_image
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

    pub fn fire_at(&mut self, target: &mut Unit) {
        if self.moves < self.weapon.cost || !target.alive() {
            return;
        }

        self.moves -= self.weapon.cost;

        let distance = (self.x as f32 - target.x as f32).hypot(self.y as f32 - target.y as f32);
        let hit_chance = chance_to_hit(distance);
        let random = rand::random::<f32>();

        if hit_chance > random {
            target.health = if target.health < self.weapon.damage {
                0
            } else {
                target.health - self.weapon.damage
            };
        }
    }
}

fn chance_to_hit(distance: f32) -> f32 {
    1.0 / (1.0 + 0.02 * 4.0_f32.powf(distance / 3.0))
}