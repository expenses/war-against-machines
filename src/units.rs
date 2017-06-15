use rand;
use rand::Rng;

use weapons::{Weapon, Bullet};
use weapons::WeaponType::{Rifle, MachineGun, PlasmaRifle};

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

                (
                    Weapon::new(weapon_type),
                    "friendly".into(),
                    format!("{} {}", first, last), 30, 75
                )
            },
            UnitType::Robot => {
                (
                    Weapon::new(PlasmaRifle),
                    "enemy".into(),
                    format!("ROBOT"), 25, 150
                )
            }
        };

        Unit {
            tag, side, x, y, weapon, image, name, max_moves, max_health,
            moves: max_moves,
            health: max_health
        }
    }

    // Update the image of the unit
    pub fn update(&mut self) {
        if self.health <= 0 {
            self.image = match self.tag {
                UnitType::Squaddie => "dead_friendly".into(),
                UnitType::Robot => "dead_enemy".into()
            }
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
        if self.moves < self.weapon.cost || target.health <= 0 {
            return;
        }

        self.moves -= self.weapon.cost;

        // Get the chance to hit and compare it to a random number

        let hit_chance = chance_to_hit(self, target);
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

// A change to hit function based on a fairly simple sigmoid curve.
pub fn chance_to_hit(from: &Unit, target: &Unit) -> f32 {
    let distance = (from.x as f32 - target.x as f32).hypot(from.y as f32 - target.y as f32);

    1.0 / (1.0 + 0.02 * 4.0_f32.powf(distance / 3.0))
}