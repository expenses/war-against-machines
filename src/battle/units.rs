// The units in the game, and a struct to contain them

use rand;
use rand::Rng;

use std::fmt;
use std::slice::{Iter, IterMut};

use super::tiles::Tiles;
use items::Item;
use weapons::{Weapon, WeaponType};
use utils::{distance_under, chance_to_hit};
use resources::Image;

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
    let mut serial = rand::thread_rng().gen_range(0, 100_000).to_string();

    while serial.len() < 5 {
        serial.insert(0, '0');
    }

    format!("SK{}", serial)
}

// The type of a unit
#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum UnitType {
    Squaddie,
    Machine
}

impl UnitType {
    pub fn moves(&self) -> u16 {
        match *self {
            UnitType::Squaddie => 30,
            UnitType::Machine => 25
        }
    }

    pub fn health(&self) -> i16 {
        match *self {
            UnitType::Squaddie => 100,
            UnitType::Machine => 150
        }
    }

    pub fn image(&self) -> Image {
        match *self {
            UnitType::Squaddie => Image::Squaddie,
            UnitType::Machine => Image::Machine
        }
    }

    pub fn capacity(&self) -> f32 {
        match *self {
            UnitType::Squaddie => 25.0,
            UnitType::Machine => 75.0
        }
    }
}

impl fmt::Display for UnitType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            UnitType::Squaddie => "Squaddie",
            UnitType::Machine => "Machine"
        })
    }
}

// Which side the unit is on
#[derive(Eq, PartialEq, Serialize, Deserialize)]
pub enum UnitSide {
    Player,
    // Neutral,
    AI
}

// A struct for a unit in the game
#[derive(Serialize, Deserialize)]
pub struct Unit {
    pub id: u8,
    pub tag: UnitType,
    pub side: UnitSide,
    pub x: usize,
    pub y: usize,
    pub weapon: Weapon,
    pub name: String,
    pub moves: u16,
    pub health: i16,
    pub inventory: Vec<Item>
}

impl Unit {
    // Create a new unit based on unit type
    fn new(tag: UnitType, side: UnitSide, x: usize, y: usize, id: u8) -> Unit {
        match tag {
            UnitType::Squaddie => {   
                // Randomly choose a weapon
                let mut rng = rand::thread_rng();
                let weapons = [WeaponType::Rifle, WeaponType::MachineGun];
                let weapon_type = *rng.choose(&weapons).unwrap();
                let capacity = weapon_type.capacity();

                Unit {
                    tag, side, x, y, id,
                    weapon: Weapon::new(weapon_type, capacity),
                    name: generate_squaddie_name(),
                    moves: tag.moves(),
                    health: tag.health(),
                    inventory: if let WeaponType::Rifle = weapon_type {
                        vec![Item::RifleClip(capacity), Item::RifleClip(capacity), Item::Bandages]
                    } else {
                        vec![Item::MachineGunClip(capacity), Item::MachineGunClip(capacity), Item::Bandages]
                    }
                }
            },
            UnitType::Machine => {
                Unit {
                    tag, side, x, y, id,
                    weapon: Weapon::new(WeaponType::PlasmaRifle, WeaponType::PlasmaRifle.capacity()),
                    name: generate_machine_name(),
                    moves: tag.moves(),
                    health: tag.health(),
                    inventory: Vec::new()
                }
            }
        }
    }

    // Move the unit to a location with a specific cost
    pub fn move_to(&mut self, x: usize, y: usize, cost: u16) {
        self.x = x;
        self.y = y;
        self.moves -= cost;
    }

    // Get the chance-to-hit of a tile from the unit
    pub fn chance_to_hit(&self, target_x: usize, target_y: usize) -> f32 {
        chance_to_hit(self.x, self.y, target_x, target_y)
    }
}

// A struct for containing all of the units
#[derive(Serialize, Deserialize)]
pub struct Units {
    pub max_player_units: u8,
    pub max_ai_units: u8,
    index: u8,
    units: Vec<Unit>
}

impl Units {
    // Create a new Units struct
    pub fn new() -> Units {
        Units {
            index: 0,
            max_player_units: 0,
            max_ai_units: 0,
            units: Vec::new()
        }
    }

    // Add a unit to the struct
    pub fn add(&mut self, tag: UnitType, side: UnitSide, x: usize, y: usize) {
        match side {
            UnitSide::Player => self.max_player_units += 1,
            UnitSide::AI => self.max_ai_units += 1
        };

        self.units.push(Unit::new(tag, side, x, y, self.index));
        self.index += 1;
    }

    // Iterate over the units
    pub fn iter(&self) -> Iter<Unit> {
        self.units.iter()
    }

    // Iterate mutably over the units
    pub fn iter_mut(&mut self) -> IterMut<Unit> {
        self.units.iter_mut()
    }

    // Get a reference to a unit with a specific ID, if the unit exists
    pub fn get(&self, id: u8) -> Option<&Unit> {
        self.units
            .binary_search_by_key(&id, |unit| unit.id).ok()
            .and_then(move |id| self.units.get(id))
    }

    // Get a mutable reference to a unit with a specific ID, if the unit exists
    pub fn get_mut(&mut self, id: u8) -> Option<&mut Unit> {
        self.units
            .binary_search_by_key(&id, |unit| unit.id).ok()
            .and_then(move |id| self.units.get_mut(id))
    }

    // Return the ID and reference to a unit at (x, y)
    pub fn at(&self, x: usize, y: usize) -> Option<&Unit> {
        self.iter().find(|unit| unit.x == x && unit.y == y)
    }

    // Count the number of units on a particular side
    pub fn count(&self, side: UnitSide) -> u8 {
        self.iter().filter(|unit| unit.side == side).count() as u8
    }

    // Calculate if (x, y) is visible to any units on a particular side
    pub fn visible(&self, x: usize, y: usize, side: UnitSide) -> bool {
        self.iter()
            .filter(|unit| unit.side == side)
            .any(|unit| distance_under(unit.x, unit.y, x, y, UNIT_SIGHT))
    }

    // Convert a unit ID to that unit's index in the vec
    fn id_to_index(&self, id: u8) -> Option<usize> {
        self.iter()
            .enumerate()
            .find(|&(_, unit)| unit.id == id)
            .map(|(i, _)| i)
    }

    // Kill a unit and drop a corpse
    pub fn kill(&mut self, tiles: &mut Tiles, id: u8) {
        if let Some(unit) = self.get_mut(id) {
            let corpse = match unit.tag {
                UnitType::Squaddie => Item::SquaddieCorpse,
                UnitType::Machine => Item::MachineCorpse,
            };

            // Drop the unit's items
            tiles.drop_all(unit.x, unit.y, &mut unit.inventory);
            // Drop the unit's weapon
            tiles.drop(unit.x, unit.y, unit.weapon.to_item());
            // Drop the unit's corpse
            tiles.drop(unit.x, unit.y, corpse);
        } else {
            return;
        }
        // Remove the unit
        let to_remove = self.id_to_index(id).unwrap();
        self.units.remove(to_remove);
        // Update the visibility of the tiles
        tiles.update_visibility(self);
    }
}