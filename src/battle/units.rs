// The units in the game, and a struct to contain them

use rand;
use rand::{Rng, ThreadRng};

use std::fmt;
use std::collections::hash_map::*;
use std::iter::*;

use super::paths::PathPoint;
use super::map::*;
use items::Item;
use weapons::{Weapon, WeaponType};
use utils::{chance_to_hit, distance_under};
use resources::Image;
use context::*;

// The cost for a unit to pick up / drop / use / throw an item
pub const ITEM_COST: u16 = 5;

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
fn generate_squaddie_name(rng: &mut ThreadRng) -> String {
    let first = rng.choose(FIRST_NAMES).unwrap();
    let last = rng.choose(LAST_NAMES).unwrap();
    format!("{} {}", first, last)
}

// Generate a new random machine name
fn generate_machine_name(rng: &mut ThreadRng) -> String {
    format!("SK{:05}", rng.gen_range(0, 100_000))
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Side {
    PlayerA,
    PlayerB
}

impl Side {
    pub fn enemies(self) -> Side {
        match self {
            Side::PlayerA => Side::PlayerB,
            Side::PlayerB => Side::PlayerA
        }
    }

    pub fn vs_ai_string(self) -> &'static str {
        match self {
            Side::PlayerA => "Player",
            Side::PlayerB => "AI"
        }
    }

    pub fn multiplayer_string(self) -> &'static str {
        match self {
            Side::PlayerA => "Player A",
            Side::PlayerB => "Player B"
        }
    }
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.multiplayer_string())
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum UnitFacing {
    Bottom,
    BottomLeft,
    Left,
    TopLeft,
    Top,
    TopRight,
    Right,
    BottomRight
}

impl UnitFacing {
    pub fn from_points(x: usize, y: usize, target_x: usize, target_y: usize) -> Self {
        let x = target_x as f32 - x as f32;
        let y = target_y as f32 - y as f32;
        let rotation = (y.atan2(x).to_degrees() + 360.0 / 16.0 * 15.0) % 360.0;

        if rotation < 45.0 {
            UnitFacing::Bottom
        } else if rotation < 90.0 {
            UnitFacing::BottomLeft
        } else if rotation < 135.0 {
            UnitFacing::Left
        } else if rotation < 180.0 {
            UnitFacing::TopLeft
        } else if rotation < 225.0 {
            UnitFacing::Top
        } else if rotation < 270.0 {
            UnitFacing::TopRight
        } else if rotation < 315.0 {
            UnitFacing::Right
        } else {
            UnitFacing::BottomRight
        }
    }
    
    // Which tiles should be visible relative to the center depending on the facing
    // You can use
    // http://www.wolframalpha.com/input/?+-abs(y)+%3E=+x&i=graph+y+%3E%3D+0+%26%26+abs(x)+%3C%3D+y
    // to check that there are correct
    fn visible(self, x: isize, y: isize) -> bool {
        match self {
            UnitFacing::Bottom      => x >= 0 && y >= 0,
            UnitFacing::BottomLeft  => y >= 0 && x.abs() <= y,
            UnitFacing::Left        => x <= 0 && y >= 0,
            UnitFacing::TopLeft     => x <= 0 && -y.abs() >= x,
            UnitFacing::Top         => x <= 0 && y <= 0,
            UnitFacing::TopRight    => y <= 0 && -x.abs() >= y,
            UnitFacing::Right       => x >= 0 && y <= 0,
            UnitFacing::BottomRight => x >= 0 && y.abs() <= x
        }
    }

    pub fn can_see(self, x: usize, y: usize, target_x: usize, target_y: usize, sight: f32) -> bool {
        self.visible(
            target_x as isize - x as isize,
            target_y as isize - y as isize
        ) && distance_under(x, y, target_x, target_y, sight)
    }

    pub fn rotate_cw(self) -> Self {
        match self {
            UnitFacing::Bottom      => UnitFacing::BottomLeft,
            UnitFacing::BottomLeft  => UnitFacing::Left,
            UnitFacing::Left        => UnitFacing::TopLeft,
            UnitFacing::TopLeft     => UnitFacing::Top,
            UnitFacing::Top         => UnitFacing::TopRight,
            UnitFacing::TopRight    => UnitFacing::Right,
            UnitFacing::Right       => UnitFacing::BottomRight,
            UnitFacing::BottomRight => UnitFacing::Bottom,
        }
    }

    pub fn rotate_ccw(self) -> Self {
        match self {
            UnitFacing::Bottom      => UnitFacing::BottomRight,
            UnitFacing::BottomRight => UnitFacing::Right,
            UnitFacing::Right       => UnitFacing::TopRight,
            UnitFacing::TopRight    => UnitFacing::Top,
            UnitFacing::Top         => UnitFacing::TopLeft,
            UnitFacing::TopLeft     => UnitFacing::Left,
            UnitFacing::Left        => UnitFacing::BottomLeft,
            UnitFacing::BottomLeft  => UnitFacing::Bottom,
        }
    }

    pub fn rotation_cost_and_direction(self, target: Self) -> (u16, bool) {
        let mut facing = self;
        let mut cw_turns = 0;
        while facing != target {
            facing = facing.rotate_cw();
            cw_turns += 1;
        }

        let mut facing = self;
        let mut ccw_turns = 0;
        while facing != target {
            facing = facing.rotate_ccw();
            ccw_turns += 1;
        }

        if ccw_turns < cw_turns {
            (ccw_turns, false)
        } else {
            (cw_turns, true)
        }
    }
}

// The type of a unit
#[derive(PartialEq, Copy, Clone, Serialize, Deserialize, Debug)]
pub enum UnitType {
    Squaddie,
    Machine
}

impl UnitType {
    pub fn moves(self) -> u16 {
        match self {
            UnitType::Squaddie => 30,
            UnitType::Machine => 25
        }
    }

    pub fn health(self) -> i16 {
        match self {
            UnitType::Squaddie => 100,
            UnitType::Machine => 150
        }
    }

    pub fn capacity(self) -> f32 {
        match self {
            UnitType::Squaddie => 25.0,
            UnitType::Machine => 75.0
        }
    }

    pub fn sight(self) -> f32 {
        Unit::SIGHT
    }

    // How far the unit can throw
    pub fn throw_distance(self) -> f32 {
        self.sight() * 1.5
    }

    fn front_image(self) -> Image {
        match self {
            UnitType::Squaddie => Image::SquaddieFront,
            UnitType::Machine => Image::MachineFront
        }
    }

    fn left_image(self) -> Image {
        match self {
            UnitType::Squaddie => Image::SquaddieLeft,
            UnitType::Machine => Image::MachineFront
        }
    }

    fn right_image(self) -> Image {
        match self {
            UnitType::Squaddie => Image::SquaddieRight,
            UnitType::Machine => Image::MachineFront
        }
    }

    fn back_image(self) -> Image {
        match self {
            UnitType::Squaddie => Image::SquaddieBack,
            UnitType::Machine => Image::MachineBack
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


// A struct for a unit in the game
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Unit {
    pub id: u8,
    pub x: usize,
    pub y: usize,
    pub tag: UnitType,
    pub side: Side,
    pub weapon: Weapon,
    pub facing: UnitFacing,
    pub moves: u16,
    health: i16,
    name: String,
    inventory: Vec<Item>
}

impl Unit {
    // The cost for a unit to walk laterally
    pub const WALK_LATERAL_COST: u16 = 2;
    // The cost for a unit to walk diagonally
    pub const WALK_DIAGONAL_COST: u16 = 3;
    // How far a unit can see
    pub const SIGHT: f32 = 7.5;

    // Create a new unit based on unit type
    pub fn new(tag: UnitType, side: Side, x: usize, y: usize, facing: UnitFacing, id: u8) -> Unit {
        let mut rng = rand::thread_rng();

        match tag {
            UnitType::Squaddie => {   
                // Randomly choose a weapon
                let weapon_type = *rng.choose(&[WeaponType::Rifle, WeaponType::MachineGun]).unwrap();
                let capacity = weapon_type.capacity();

                Unit {
                    tag, side, x, y, facing, id,
                    weapon: Weapon::new(weapon_type, capacity),
                    name: generate_squaddie_name(&mut rng),
                    moves: tag.moves(),
                    health: tag.health(),
                    inventory: if let WeaponType::Rifle = weapon_type {
                        vec![Item::RifleClip(capacity), Item::RifleClip(capacity), Item::Bandages, Item::Grenade(false)]
                    } else {
                        vec![Item::MachineGunClip(capacity), Item::MachineGunClip(capacity), Item::Bandages, Item::Grenade(false)]
                    }
                }
            },
            UnitType::Machine => {
                Unit {
                    tag, side, x, y, facing, id,
                    weapon: Weapon::new(WeaponType::PlasmaRifle, WeaponType::PlasmaRifle.capacity()),
                    name: generate_machine_name(&mut rng),
                    moves: tag.moves(),
                    health: tag.health(),
                    inventory: Vec::new(),
                }
            }
        }
    }

    pub fn inventory(&self) -> &[Item] {
        &self.inventory
    }

    // The weight of the items that the units is carrying
    pub fn carrying(&self) -> f32 {
        self.inventory.iter().fold(
            self.weapon.tag.weight(),
            |total, item| total + item.weight()
        )
    }

    pub fn damage(&mut self, damage: i16) -> bool {
        self.health -= damage;
        self.health <= 0
    }

    // Move the unit to a location with a specific cost
    pub fn move_to(&mut self, point: &PathPoint) {
        self.x = point.x;
        self.y = point.y;
        self.facing = point.facing;
        self.moves -= point.cost;
    }

    pub fn inventory_remove(&mut self, item: usize) -> Option<Item> {
        if item < self.inventory.len() {
            Some(self.inventory.remove(item))
        } else {
            None
        }
    }

    // Get the chance-to-hit of a tile from the unit
    pub fn chance_to_hit(&self, target_x: usize, target_y: usize) -> f32 {
        chance_to_hit(self.x, self.y, target_x, target_y)
    }

    pub fn can_heal_from(&self, item: Item) -> bool {
        let amount = item.heal(self.tag);
        amount > 0 && self.moves >= ITEM_COST && self.tag.health() - self.health >= amount
    }

    pub fn can_reload_from(&self, item: Item) -> bool {
        let ammo = item.ammo(self.weapon.tag);
        ammo > 0 && self.weapon.can_reload(ammo)
    }

    // Drop an item from the unit's inventory
    pub fn drop_item(&mut self, tiles: &mut Tiles, index: usize) {
        if self.moves < ITEM_COST {
            return;
        }

        if let Some(item) = self.inventory_remove(index) {
            tiles.drop(self.x, self.y, item);
            self.moves -= ITEM_COST;
            return;
        }
    }

    pub fn pickup_item(&mut self, tiles: &mut Tiles, index: usize) {
        if self.moves < ITEM_COST {
            return;
        }
        
        let tile = tiles.at_mut(self.x, self.y);

        if let Some(item) = tile.items_remove(index) {
            if self.carrying() + item.weight() <= self.tag.capacity() {                        
                self.inventory.push(item);
                self.moves -= ITEM_COST;
                return;
            } 
        }
    }

    pub fn use_item(&mut self, index: usize) {
        let mut item_consumed = false;
        let mut new_item = None;

        // Return if the unit doesn't have the moves to use the item
        if self.moves < ITEM_COST {
            return;
        }

        if let Some(item) = self.inventory.get(index) {
            item_consumed = match (*item, self.weapon.tag) {
                // Reload the corresponding weapon
                (Item::RifleClip(ammo), WeaponType::Rifle) |
                (Item::MachineGunClip(ammo), WeaponType::MachineGun) |
                (Item::PlasmaClip(ammo), WeaponType::PlasmaRifle) => self.weapon.reload(ammo),
                // Switch weapons
                (Item::Rifle(ammo), _) => {
                    new_item = Some(self.weapon.to_item());
                    self.weapon = Weapon::new(WeaponType::Rifle, ammo);
                    true
                },
                (Item::MachineGun(ammo), _) => {
                    new_item = Some(self.weapon.to_item());
                    self.weapon = Weapon::new(WeaponType::MachineGun, ammo);
                    true
                },
                (Item::PlasmaRifle(ammo), _) => {
                    new_item = Some(self.weapon.to_item());
                    self.weapon = Weapon::new(WeaponType::PlasmaRifle, ammo);
                    true
                },
                // Use other items
                (Item::Bandages, _) if self.can_heal_from(*item) => {
                    self.health += item.heal(self.tag);
                    true
                },
                (Item::Grenade(primed), _) if !primed => {
                    new_item = Some(Item::Grenade(true));
                    true
                }
                _ => false
            }
        }

        // If the item was consumed, remove it from the inventory
        if item_consumed {
            self.inventory.remove(index);
        }

        // If a new item was created, add it to the inventory
        if let Some(item) = new_item {
            self.inventory.push(item);
        }

        if item_consumed || new_item.is_some() {
            self.moves -= ITEM_COST;
        }
    }

    pub fn fire_weapon(&mut self) -> bool {
        let can_fire = self.moves >= self.weapon.tag.cost() && self.weapon.can_fire();

        if can_fire {
            self.moves -= self.weapon.tag.cost();
            self.weapon.ammo -= 1;
        }

        can_fire
    }

    pub fn info(&self) -> String {
        format!("Name: {}, Moves: {}, Health: {}, Weapon: {}", self.name, self.moves, self.health, self.weapon)
    }

    pub fn carrying_info(&self) -> String {
        format!(
            "{}\n{} - {} kg\nCarry Capacity: {}/{} kg",
            self.name, self.weapon, self.weapon.tag.weight(), self.carrying(), self.tag.capacity()
        )
    }

    pub fn render(&self, ctx: &mut Context, dest: [f32; 2], zoom: f32, overlay: [f32; 4]) {
        let image = match self.facing {
            UnitFacing::Right | UnitFacing::BottomRight => self.tag.right_image(),
            UnitFacing::Bottom | UnitFacing::BottomLeft => self.tag.front_image(),
            UnitFacing::Left | UnitFacing::TopLeft => self.tag.left_image(),
            _ => self.tag.back_image()
        };

        ctx.render_with_overlay(image, dest, zoom, overlay);
    }
}

// A struct for containing all of the units
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Units {
    pub max_player_a_units: u8,
    pub max_player_b_units: u8,
    index: u8,
    units: HashMap<u8, Unit>
}

impl Units {
    // Create a new Units struct
    pub fn new() -> Units {
        Units {
            index: 0,
            max_player_a_units: 0,
            max_player_b_units: 0,
            units: HashMap::new()
        }
    }

    // Add a unit to the struct
    pub fn add(&mut self, tag: UnitType, side: Side, x: usize, y: usize, facing: UnitFacing) {
        match side {
            Side::PlayerA => self.max_player_a_units += 1,
            Side::PlayerB => self.max_player_b_units += 1,
        }

        self.units.insert(self.index, Unit::new(tag, side, x, y, facing, self.index));
        self.index += 1;
    }

    // Iterate over the units
    pub fn iter(&self) -> Values<u8, Unit> {
        self.units.values()
    }

    // Iterate mutably over the units
    pub fn iter_mut(&mut self) -> ValuesMut<u8, Unit> {
        self.units.values_mut()
    }

    // Get a reference to a unit with a specific ID, if the unit exists
    pub fn get(&self, id: u8) -> Option<&Unit> {
        self.units.get(&id)
    }

    // Get a mutable reference to a unit with a specific ID, if the unit exists
    pub fn get_mut(&mut self, id: u8) -> Option<&mut Unit> {
        self.units.get_mut(&id)
    }

    // Return a reference to a unit at (x, y)
    pub fn at(&self, x: usize, y: usize) -> Option<&Unit> {
        self.iter().find(|unit| unit.x == x && unit.y == y)
    }

    // Return a mutable reference to a unit at (x, y)
    pub fn at_mut(&mut self, x: usize, y: usize) -> Option<&mut Unit> {
        self.iter_mut().find(|unit| unit.x == x && unit.y == y)
    }

    // Count the number of units on a particular side
    pub fn count(&self, side: Side) -> u8 {
        self.iter().filter(|unit| unit.side == side).count() as u8
    }

    // Is a unit on a particular side at (x, y)?
    pub fn on_side(&self, x: usize, y: usize, side: Side) -> bool {
        self.at(x, y).map(|unit| unit.side == side).unwrap_or(false)
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
        self.units.remove(&id);
        // Update the visibility of the tiles
        tiles.update_visibility(self);
    }
}

impl FromIterator<Unit> for Units {
    fn from_iter<I: IntoIterator<Item=Unit>>(iterator: I) -> Self {
        let mut max_player_a_units = 0;
        let mut max_player_b_units = 0;

        let units = iterator.into_iter().inspect(|unit| {
                match unit.side {
                    Side::PlayerA => max_player_a_units += 1,
                    Side::PlayerB => max_player_b_units += 1
                }
            })
            .map(|unit| (unit.id, unit))
            .collect();

        Self {
            max_player_a_units, max_player_b_units, units,
            index: 0
        }
    }
}

#[test]
fn unit_actions() {
    let mut units = Units::new();
    let mut tiles = Tiles::new(30, 30);

    let rifle = WeaponType::Rifle;
    let plasma_rifle = WeaponType::PlasmaRifle;

    // After adding 10 units, there should be 10 ai units into total

    for i in 0 .. 10 {
        units.add(UnitType::Machine, Side::PlayerB, i, i, UnitFacing::Bottom);
    }

    assert_eq!(units.count(Side::PlayerB), 10);

    // Iterating over the units should work as expected

    {
        // A unit should be carrying the weight of a plasma rifle

        let unit = units.get_mut(0).unwrap();

        assert_eq!(unit.carrying(), plasma_rifle.weight());

        // Test picking up a rifle

        tiles.at_mut(0, 0).items.push(Item::Rifle(rifle.capacity()));

        assert_eq!(unit.inventory, Vec::new());
        unit.pickup_item(&mut tiles, 0);
        assert_eq!(unit.inventory, vec![Item::Rifle(rifle.capacity())]);
        assert_eq!(tiles.at(0, 0).items, Vec::new());

        assert_eq!(unit.inventory[0], Item::Rifle(rifle.capacity()));

        assert_eq!(unit.carrying(), plasma_rifle.weight() + rifle.weight());

        assert_eq!(unit.moves, unit.tag.moves() - ITEM_COST);

        // Test equpping a rifle

        unit.use_item(0);

        assert_eq!(unit.inventory[0], Item::PlasmaRifle(plasma_rifle.capacity()));

        assert_eq!(unit.moves, unit.tag.moves() - ITEM_COST * 2);

        // The unit is at full health and shouldn't be able to heal

        assert!(!unit.can_heal_from(Item::Bandages));

        // Or reload

        assert!(!unit.can_reload_from(Item::RifleClip(rifle.capacity())));

        // Test firing the weapon

        assert!(unit.fire_weapon());

        assert_eq!(unit.moves, unit.tag.moves() - ITEM_COST * 2 - rifle.cost());
    }

    // After killing a unit there should be 9 left

    units.kill(&mut tiles, 0);

    assert_eq!(units.count(Side::PlayerB), 9);

    // And the tile under the unit should have items on it

    assert_ne!(tiles.at(0, 0).items, Vec::new());
}

#[test]
fn test_turning() {
    let (cost, direction) = UnitFacing::Bottom.rotation_cost_and_direction(UnitFacing::Top);
    assert_eq!((cost, direction), (4, true));

    let (cost, direction) = UnitFacing::BottomRight.rotation_cost_and_direction(UnitFacing::Top);
    assert_eq!((cost, direction), (3, false));
}
