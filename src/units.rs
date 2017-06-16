use rand;
use rand::Rng;

use weapons::{Weapon, Bullet};
use weapons::WeaponType::{Rifle, MachineGun, PlasmaRifle};
use utils::chance_to_hit;

// A list of first names to pick from
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

// A list of last names to pick from
const LAST_NAMES: &[&str; 7] = &[
    "Cooper",
    "Yang",
    "Smith",
    "Denton",
    "Simons",
    "Rivers",
    "Savage"
];

// The type of a unit
pub enum UnitType {
    Squaddie,
    Robot
}

// The side of a unit
pub enum UnitSide {
    Friendly,
    // Neutral,
    Enemy
}

// A struct for a unit in the game
pub struct Unit {
    pub tag: UnitType,
    pub side: UnitSide,
    pub x: usize,
    pub y: usize,
    pub weapon: Weapon,
    pub image: String,
    pub name: String,
    pub moves: usize,
    pub max_moves: usize,
    pub health: i16,
    pub max_health: i16
}

impl Unit {
    // Create a new unit based on unit type
    pub fn new(tag: UnitType, side: UnitSide, x: usize, y: usize) -> Unit {
        let (weapon, image, name, max_moves, max_health) = match tag {
            UnitType::Squaddie => {
                // Generate a random name
                let mut rng = rand::thread_rng();
                let first = rng.choose(FIRST_NAMES).unwrap();
                let last = rng.choose(LAST_NAMES).unwrap();

                let weapon_type = if rng.gen::<bool>() { Rifle } else { MachineGun };

                let image = match side {
                    UnitSide::Friendly => "friendly_squaddie",
                    UnitSide::Enemy => "enemy_squaddie"
                };

                (Weapon::new(weapon_type), image, format!("{} {}", first, last), 30, 75)
            },
            UnitType::Robot => {
                let image = match side {
                    UnitSide::Friendly => "friendly_robot",
                    UnitSide::Enemy => "enemy_robot"
                };

                (Weapon::new(PlasmaRifle), image, format!("ROBOT"), 25, 150)
            }
        };

        Unit {
            tag, side, x, y, weapon, name, max_moves, max_health,
            image: image.into(),
            moves: max_moves,
            health: max_health
        }
    }

    pub fn alive(&self) -> bool {
        self.health > 0
    }

    // Update the image of the unit
    pub fn update(&mut self) {
        if !self.alive() {
            self.image = match self.tag {
                UnitType::Squaddie => match self.side {
                    UnitSide::Friendly => "dead_friendly_squaddie",
                    UnitSide::Enemy => "dead_enemy_squaddie"
                },
                UnitType::Robot => match self.side {
                    UnitSide::Friendly => "dead_friendly_robot",
                    UnitSide::Enemy => "dead_enemy_robot"
                }
            }.into();
        }
    }

    // Move the unit to a location
    pub fn move_to(&mut self, x: usize, y: usize, cost: usize) -> bool {
        if self.moves >= cost {
            self.x = x;
            self.y = y;
            self.moves -= cost;
            return true;
        }

        false
    }

    // Fire the units weapon at another unit
    pub fn fire_at(&mut self, target: &mut Unit, bullets: &mut Vec<Bullet>) {
        // return if the unit cannot fire or the unit is already dead
        if self.moves < self.weapon.cost || !target.alive() {
            return;
        }

        self.moves -= self.weapon.cost;

        // Get the chance to hit and compare it to a random number

        let hit_chance = chance_to_hit(self.x, self.y, target.x, target.y);
        let random = rand::random::<f32>();

        let will_hit = hit_chance > random;

        // Lower the targets health
        if will_hit {
            target.health -= self.weapon.damage;
        }

        // Add a bullet to the array for drawing
        bullets.push(Bullet::new(self, target, will_hit));
    }
}