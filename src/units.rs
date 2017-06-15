use rand;
use rand::Rng;

use weapons::{Weapon, Bullet};
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
    pub image: String,
    pub name: String,
    pub moves: usize,
    pub max_moves: usize,
    pub health: i16,
    pub max_health: i16
}

impl Unit {
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

    pub fn update(&mut self) {
        if self.health <= 0 {
            self.image = match self.tag {
                UnitType::Squaddie => "dead_friendly".into(),
                UnitType::Robot => "dead_enemy".into()
            }
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

    pub fn fire_at(&mut self, target: &mut Unit, bullets: &mut Vec<Bullet>) {
        if self.moves < self.weapon.cost || target.health <= 0 {
            return;
        }

        self.moves -= self.weapon.cost;

        let hit_chance = chance_to_hit(self, target);
        let random = rand::random::<f32>();

        let will_hit = hit_chance > random;

        if will_hit {
            target.health -= self.weapon.damage;
        }

        bullets.push(Bullet::new(self, target, will_hit));
    }
}

pub fn chance_to_hit(from: &Unit, target: &Unit) -> f32 {
    let distance = (from.x as f32 - target.x as f32).hypot(from.y as f32 - target.y as f32);

    1.0 / (1.0 + 0.02 * 4.0_f32.powf(distance / 3.0))
}