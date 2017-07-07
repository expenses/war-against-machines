// The units in the game, and a struct to contain them

use rand;
use rand::Rng;

use std::fmt;
use std::slice::{Iter, IterMut};

use battle::tiles::Tiles;
use items::{Item, ItemType};
use weapons::{Weapon, WeaponType};
use utils::{distance_under, chance_to_hit};
use resources::SetImage;

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
#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum UnitType {
    Squaddie,
    Machine
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
    pub id: usize,
    pub tag: UnitType,
    pub side: UnitSide,
    pub x: usize,
    pub y: usize,
    pub weapon: Weapon,
    pub image: SetImage,
    pub name: String,
    pub moves: usize,
    pub max_moves: usize,
    pub health: i16,
    pub max_health: i16,
    pub inventory: Vec<Item>
}

impl Unit {
    // Create a new unit based on unit type
    fn new(tag: UnitType, side: UnitSide, x: usize, y: usize, id: usize) -> Unit {
        match tag {
            UnitType::Squaddie => {                
                let moves = 30;
                let health = 100;
                let mut rng = rand::thread_rng();
                let weapons = [WeaponType::Rifle, WeaponType::MachineGun];

                Unit {
                    tag, side, x, y, moves, health, id,
                    image: SetImage::Squaddie,
                    weapon: Weapon::new(*rng.choose(&weapons).unwrap()),
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
                    tag, side, x, y, moves, health, id,
                    image: SetImage::Machine,
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

    pub fn chance_to_hit(&self, target_x: usize, target_y: usize) -> f32 {
        chance_to_hit(self.x, self.y, target_x, target_y)
    }
}

// A struct for containing all of the units
#[derive(Serialize, Deserialize)]
pub struct Units {
    index: usize,
    units: Vec<Unit>
}

impl Units {
    // Create a new Units struct
    pub fn new() -> Units {
        Units {
            index: 0,
            units: Vec::new()
        }
    }

    pub fn add(&mut self, tag: UnitType, side: UnitSide, x: usize, y: usize) {
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

    // Get a reference to a unit with a specific ID, if th unit exists
    pub fn get(&self, id: usize) -> Option<&Unit> {
        self.units.iter().find(|unit| unit.id == id)
    }

    // Get a mutable reference to a unit with a specific ID, if the unit exists
    pub fn get_mut(&mut self, id: usize) -> Option<&mut Unit> {
        self.units.iter_mut().find(|unit| unit.id == id)
    }

    // Return the ID and reference to a unit at (x, y)
    pub fn at(&self, x: usize, y: usize) -> Option<&Unit> {
        self.iter().find(|unit| unit.x == x && unit.y == y)
    }

    // Check if any units on a particular side are alive
    pub fn any_alive(&self, side: UnitSide) -> bool {
        self.iter().any(|unit| unit.side == side)
    }

    // Calculate if (x, y) is visible to any units on a particular side
    pub fn visible(&self, x: usize, y: usize, side: UnitSide) -> bool {
        self.iter()
            .filter(|unit| unit.side == side)
            .any(|unit| distance_under(unit.x, unit.y, x, y, UNIT_SIGHT))
    }

    fn id_to_index(&self, id: usize) -> Option<usize> {
        self.iter().enumerate().find(|&(_, unit)| unit.id == id).map(|(i, _)| i)
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
        let to_remove = self.id_to_index(id).unwrap();
        self.units.remove(to_remove);
        // Update the visibility of the tiles
        tiles.update_visibility(self);
    }
}