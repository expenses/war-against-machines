// The units in the game, and a struct to contain them

use rand;
use rand::Rng;

use std::fmt;
use std::collections::HashMap;
use std::collections::hash_map::{Iter, IterMut};

use battle::tiles::Tiles;
use items::{Item, ItemType};
use weapons::{Weapon, WeaponType};
use utils::distance_under;

// The sight range of units
pub const UNIT_SIGHT: f32 = 7.5;

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
    "Hawk",
    "Laura",
    "Bobby",
    "Jane"
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
    "Zhou",
    "Jensen",
    "Palmer",
    "Mason",
    "Johnson",
    "Briggs"
];

// Generate a new random squaddie name
fn generate_squaddie_name() -> String {
    let mut rng = rand::thread_rng();
    let first = rng.choose(FIRST_NAMES).unwrap();
    let last = rng.choose(LAST_NAMES).unwrap();

    format!("{} {}", first, last)
}

// Generate a new random machine name
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

impl fmt::Display for UnitType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            &UnitType::Squaddie => "Squaddie",
            &UnitType::Machine => "Machine"
        })
    }
}

// Which side the unit is on
#[derive(Eq, PartialEq)]
pub enum UnitSide {
    Player,
    // Neutral,
    AI
}

// A struct for a unit in the game
pub struct Unit {
    pub tag: UnitType,
    pub side: UnitSide,
    pub x: usize,
    pub y: usize,
    pub weapon: Weapon,
    pub image: &'static str,
    pub name: String,
    pub moves: usize,
    pub max_moves: usize,
    pub health: i16,
    pub max_health: i16,
    pub inventory: Vec<Item>
}

impl Unit {
    // Create a new unit based on unit type
    pub fn new(tag: UnitType, side: UnitSide, x: usize, y: usize) -> Unit {
        match tag {
            UnitType::Squaddie => {
                let weapon = Weapon::new(if rand::thread_rng().gen::<bool>() { WeaponType::Rifle } else { WeaponType::MachineGun });
                let moves = 30;
                let health = 100;

                Unit {
                    tag, side, x, y, moves, health, weapon,
                    image: "squaddie",
                    name: generate_squaddie_name(),
                    max_moves: moves,
                    max_health: health,
                    inventory: Vec::new()
                }
            },
            UnitType::Machine => {
                let moves = 25;
                let health = 150;

                Unit {
                    tag, side, x, y, moves, health,
                    image: "machine",
                    weapon: Weapon::new(WeaponType::PlasmaRifle),
                    name: generate_machine_name(),
                    max_moves: moves,
                    max_health: health,
                    inventory: Vec::new()
                }
            }
        }
    }

    // Move the unit to a location with a specific cost
    pub fn move_to(&mut self, x: usize, y: usize, cost: usize) {
        self.x = x;
        self.y = y;
        self.moves -= cost;
    }

    // Pick up an item
    pub fn _pick_up(&mut self, item: Item) {
        self.inventory.push(item)
    }
}

// A struct for containing all of the units
pub struct Units {
    index: usize,
    units: HashMap<usize, Unit>
}

impl Units {
    // Create a new Units struct
    pub fn new() -> Units {
        Units {
            index: 0,
            units: HashMap::new()
        }
    }

    // Push a unit
    pub fn push(&mut self, unit: Unit) {
        self.units.insert(self.index, unit);
        self.index += 1;
    }

    // Iterate over the units
    pub fn iter(&self) -> Iter<usize, Unit> {
        self.units.iter()
    }

    // Iterate mutably over the units
    pub fn iter_mut(&mut self) -> IterMut<usize, Unit> {
        self.units.iter_mut()
    }

    // Get a reference to a unit with a specific ID, if th unit exists
    pub fn get(&self, id: usize) -> Option<&Unit> {
        self.units.get(&id)
    }

    // Get a mutable reference to a unit with a specific ID, if the unit exists
    pub fn get_mut(&mut self, id: usize) -> Option<&mut Unit> {
        self.units.get_mut(&id)
    }

    // Return the ID and reference to a unit at (x, y)
    pub fn at(&self, x: usize, y: usize) -> Option<(usize, &Unit)> {
        self.iter()
            .find(|&(_, unit)| unit.x == x && unit.y == y)
            .map(|(i, unit)| (*i, unit))
    }

    // Check if any units on a particular side are alive
    pub fn any_alive(&self, side: UnitSide) -> bool {
        self.iter().any(|(_, unit)| unit.side == side)
    }

    // Calculate if (x, y) is visible to any units on a particular side
    pub fn visible(&self, x: usize, y: usize, side: UnitSide) -> bool {
        self.iter()
        .filter(|&(_, unit)| unit.side == side)
        .any(|(_, unit)| distance_under(unit.x, unit.y, x, y, UNIT_SIGHT))
    }

    // Kill a unit and drop a corpse
    pub fn kill(&mut self, tiles: &mut Tiles, id: usize) {
        let (x, y) = match self.get(id) {
            Some(unit) => (unit.x, unit.y),
            _ => return
        };

        let corpse = Item::new(match self.get(id).map(|unit| unit.tag) {
            Some(UnitType::Squaddie) => ItemType::SquaddieCorpse,
            Some(UnitType::Machine) => ItemType::MachineCorpse,
            _ => return
        });

        // Drop the units items
        tiles.drop_all(x, y, &mut self.get_mut(id).unwrap().inventory);
        // Drop the corpse
        tiles.drop(x, y, corpse);
        // Remove the unit
        self.units.remove(&id);
        // Update the visibility of the tiles
        tiles.update_visibility(&self);
    }
}