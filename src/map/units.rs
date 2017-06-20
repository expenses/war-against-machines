use rand;
use rand::Rng;

use std::slice::{Iter, IterMut};

use weapons::Weapon;
use weapons::WeaponType::{Rifle, MachineGun, PlasmaRifle};
use utils::chance_to_hit;
use map::animations::{Bullet, Walk, AnimationQueue};
use map::paths::PathPoint;

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
    _Robot
}

// The which side the unit is on
#[derive(Eq, PartialEq)]
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
        match tag {
            UnitType::Squaddie => {
                // Generate a random name
                let mut rng = rand::thread_rng();
                let first = rng.choose(FIRST_NAMES).unwrap();
                let last = rng.choose(LAST_NAMES).unwrap();

                let weapon_type = if rng.gen::<bool>() { Rifle } else { MachineGun };

                let image = match side {
                    UnitSide::Friendly => "friendly_squaddie",
                    UnitSide::Enemy => "enemy_squaddie"
                }.into();

                let moves = 30;
                let health = 100;

                Unit {
                    tag, side, x, y, moves, health, image,
                    weapon: Weapon::new(weapon_type),
                    name: format!("{} {}", first, last),
                    max_moves: moves,
                    max_health: health
                }
            },
            UnitType::_Robot => {
                let image = match side {
                    UnitSide::Friendly => "friendly_robot",
                    UnitSide::Enemy => "enemy_robot"
                }.into();

                let moves = 25;
                let health = 150;

                Unit {
                    tag, side, x, y, moves, health, image,
                    weapon: Weapon::new(PlasmaRifle),
                    name: format!("ROBOT"),
                    max_moves: moves,
                    max_health: health
                }
            }
        }
    }

    // Is the unit alive
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
                UnitType::_Robot => match self.side {
                    UnitSide::Friendly => "dead_friendly_robot",
                    UnitSide::Enemy => "dead_enemy_robot"
                }
            }.into();
        }
    }

    // Add the movement of the unit along a path to the animation queue
    pub fn move_to(&mut self, unit_id: usize, path: Vec<PathPoint>, cost: usize, animation_queue: &mut AnimationQueue) {
        if self.moves >= cost {
            animation_queue.add_walk(Walk::new(unit_id, path)); 
        }
    }

    // Fire the units weapon at another unit
    pub fn fire_at(&mut self, target_id: usize, target: &mut Unit, animation_queue: &mut AnimationQueue) {
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

        let lethal = !target.alive();

        // Add a bullet to the array for drawing
        animation_queue.add_bullet(Bullet::new(self, target_id, target, lethal, will_hit));
    }
}

pub struct Units {
    units: Vec<Unit>
}

impl Units {
    pub fn new() -> Units {
        Units {
            units: Vec::new()
        }
    }

    pub fn push(&mut self, unit: Unit) {
        self.units.push(unit);
    }

    pub fn iter(&self) -> Iter<Unit> {
        self.units.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<Unit> {
        self.units.iter_mut()
    }

    pub fn get(&self, id: usize) -> &Unit {
        &self.units[id]
    }

    pub fn get_mut(&mut self, id: usize) -> &mut Unit {
        &mut self.units[id]
    }

    pub fn get_two_mut(&mut self, a: usize, b: usize) -> (&mut Unit, &mut Unit) {
        if a < b {
            let (slice_1, slice_2) = self.units.split_at_mut(b);
            (&mut slice_1[a], &mut slice_2[0])
        } else {
            let (slice_1, slice_2) = self.units.split_at_mut(a);
            (&mut slice_2[0], &mut slice_1[b])
        }
    }

    pub fn at(&self, x: usize, y: usize) -> Option<(usize, &Unit)> {
        self.units.iter().enumerate().find(|&(_, unit)| unit.x == x && unit.y == y)
    }

    pub fn at_i(&self, x: usize, y: usize) -> Option<usize> {
        self.at(x, y).map(|(i, _)| i)
    }

    pub fn len(&self) -> usize {
        self.units.len()
    }

    pub fn any_alive(&self, side: UnitSide) -> bool {
        self.iter().any(|unit| unit.side == side && unit.alive())
    }
}