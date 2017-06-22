use rand;
use rand::Rng;

use std::collections::HashMap;
use std::collections::hash_map::{Iter, IterMut};

use weapons::Weapon;
use weapons::WeaponType::{Rifle, MachineGun, PlasmaRifle};

// A list of first names to pick from
const FIRST_NAMES: &[&str] = &[
    "David",
    "Dale",
    "Robert",
    "Lucy",
    "Ashley",
    "Mia",
    "JC",
    "Paul",
    "Heisenberg",
    "John",
    "Kyle",
    "Sarah",
    "Dylan",
    "Connor",
    "Hawk"
];

// A list of last names to pick from
const LAST_NAMES: &[&str] = &[
    "Cooper",
    "Yang",
    "Smith",
    "Denton",
    "Simons",
    "Rivers",
    "Savage",
    "Connor",
    "Reese",
    "Rhodes",
    "Zhou"
];

fn generate_squaddie_name() -> String {
    let mut rng = rand::thread_rng();
    let first = rng.choose(FIRST_NAMES).unwrap();
    let last = rng.choose(LAST_NAMES).unwrap();

    format!("{} {}", first, last)
}

fn generate_machine_name() -> String {
    let mut serial = format!("{}", rand::thread_rng().gen_range(0, 100_000));

    while serial.len() < 5 {
        serial.insert(0, '0');
    }

    format!("SK{}", serial)
}

// The type of a unit
#[derive(Copy, Clone)]
pub enum UnitType {
    Squaddie,
    Machine
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
                let weapon = Weapon::new(if rand::thread_rng().gen::<bool>() { Rifle } else { MachineGun });
                let image = "squaddie".into();
                let moves = 30;
                let health = 100;

                Unit {
                    tag, side, x, y, moves, health, image, weapon,
                    name: generate_squaddie_name(),
                    max_moves: moves,
                    max_health: health
                }
            },
            UnitType::Machine => {
                let image = "machine".into();
                let moves = 25;
                let health = 150;

                Unit {
                    tag, side, x, y, moves, health, image,
                    weapon: Weapon::new(PlasmaRifle),
                    name: generate_machine_name(),
                    max_moves: moves,
                    max_health: health
                }
            }
        }
    }

    pub fn move_to(&mut self, x: usize, y: usize, cost: usize) {
        self.x = x;
        self.y = y;
        self.moves -= cost;
    }
}

pub struct Units {
    index: usize,
    units: HashMap<usize, Unit>
}

impl Units {
    pub fn new() -> Units {
        Units {
            index: 0,
            units: HashMap::new()
        }
    }

    pub fn push(&mut self, unit: Unit) {
        self.units.insert(self.index, unit);
        self.index += 1;
    }

    pub fn iter(&self) -> Iter<usize, Unit> {
        self.units.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<usize, Unit> {
        self.units.iter_mut()
    }

    pub fn get(&self, id: usize) -> Option<&Unit> {
        self.units.get(&id)
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut Unit> {
        self.units.get_mut(&id)
    }

    pub fn at(&self, x: usize, y: usize) -> Option<(usize, &Unit)> {
        self.iter()
            .find(|&(_, unit)| unit.x == x && unit.y == y)
            .map(|(i, unit)| (*i, unit))
    }

    pub fn at_i(&self, x: usize, y: usize) -> Option<usize> {
        self.at(x, y).map(|(i, _)| i)
    }

    pub fn any_alive(&self, side: UnitSide) -> bool {
        self.iter().any(|(_, unit)| unit.side == side)
    }

    pub fn kill(&mut self, id: usize) {
        self.units.remove(&id);
    }
}