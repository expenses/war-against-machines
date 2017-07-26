// A Map struct that combines Tiles and Units for convenience
// This struct contains all the stuff that is saved/loaded

use super::units::{UnitSide, Units};
use super::tiles::{Visibility, Tiles};
use items::Item;
use weapons::{Weapon, WeaponType};

use std::fs::{File, create_dir_all};
use std::path::{Path, PathBuf};

use bincode;

const SIZE_LIMIT: bincode::Infinite = bincode::Infinite;
const EXTENSION: &str = ".sav";
const SAVES: &str = "savegames/skirmishes";
const AUTOSAVE: &str = "autosave.sav";

// The Map struct
#[derive(Serialize, Deserialize)]
pub struct Map {
    pub units: Units,
    pub tiles: Tiles,
    pub turn: u8
}

impl Map {
    // Create a new map
    pub fn new() -> Map {
        Map {
            units: Units::new(),
            tiles: Tiles::new(),
            turn: 1
        }
    }
    
    // Work out if a tile is taken or not
    pub fn taken(&self, x: usize, y: usize) -> bool {
        !self.tiles.at(x, y).walkable() || self.units.at(x, y).is_some()
    }

    // Work out how many units of a particular side are visible to the other side
    pub fn visible(&self, side: UnitSide) -> usize {
        self.units.iter()
            .filter(|unit| unit.side == side && match side {
                UnitSide::Player => self.tiles.at(unit.x, unit.y).ai_visibility,
                UnitSide::AI => self.tiles.at(unit.x, unit.y).player_visibility
            } == Visibility::Visible)
            .count()
    }

    // Get a unit to drop an item
    pub fn drop_item(&mut self, unit: u8, index: usize) {
        if let Some(unit) = self.units.get_mut(unit) {
            if let Some(item) = unit.inventory.get(index).cloned() {
                self.tiles.drop(unit.x, unit.y, item);
                unit.inventory.remove(index);
            }
        }
    }

    // Get a unit to pick up am item
    pub fn pick_up_item(&mut self, unit: u8, index: usize) {
        if let Some(unit) = self.units.get_mut(unit) {
            let tile = self.tiles.at_mut(unit.x, unit.y);
        
            if let Some(item) = tile.items.get(index).cloned() {
                unit.inventory.push(item);
                tile.items.remove(index);
            }
        }
    }

    // Get a unit to use an item in its inventory
    pub fn use_item(&mut self, unit: u8, index: usize) -> bool {
        let mut item_consumed = false;
        let mut new_item = None;

        if let Some(unit) = self.units.get_mut(unit) {
            if let Some(item) = unit.inventory.get(index) {
                match (*item, unit.weapon.tag) {
                    // Reload the corresponding weapon
                    (Item::RifleClip(ammo), WeaponType::Rifle) => if unit.weapon.can_reload(ammo) {
                        unit.weapon.ammo += ammo;
                        item_consumed = true;
                    },
                    (Item::MachineGunClip(ammo), WeaponType::MachineGun) => if unit.weapon.can_reload(ammo) {
                        unit.weapon.ammo += ammo;
                        item_consumed = true;
                    },
                    (Item::PlasmaClip(ammo), WeaponType::PlasmaRifle) => if unit.weapon.can_reload(ammo) {
                        unit.weapon.ammo += ammo;
                        item_consumed = true;
                    },
                    // Switch weapons
                    (Item::Rifle(ammo), _) => {
                        new_item = Some(unit.weapon.to_item());
                        unit.weapon = Weapon::new(WeaponType::Rifle, ammo);
                        item_consumed = true;
                    },
                    (Item::MachineGun(ammo), _) => {
                        new_item = Some(unit.weapon.to_item());
                        unit.weapon = Weapon::new(WeaponType::MachineGun, ammo);
                        item_consumed = true;
                    },
                    (Item::PlasmaRifle(ammo), _) => {
                        new_item = Some(unit.weapon.to_item());
                        unit.weapon = Weapon::new(WeaponType::PlasmaRifle, ammo);
                        item_consumed = true;
                    },
                    _ => {}
                }
            }

            // If the item was consumed, remove it from the inventory
            if item_consumed {
                unit.inventory.remove(index);
            }

            // If a new item was created, add it to the inventory
            if let Some(item) = new_item {
                unit.inventory.push(item);
            }
        }

        // return true if an item was consumed and no item took its place
        item_consumed && new_item.is_none()
    }

    // Load a skirmish if possible
    pub fn load(filename: &str) -> Option<Map> {
        let path = Path::new(SAVES).join(filename);

        File::open(path).ok()
            .and_then(|mut file| bincode::deserialize_from(&mut file, SIZE_LIMIT).ok())
    }

    // Save the skirmish
    pub fn save(&self, filename: Option<String>) -> Option<PathBuf> {
        // Push the extension onto the filename if it is given or use the default filename
        let filename = filename.map(|mut filename| {
            filename.push_str(EXTENSION);
            filename
        }).unwrap_or_else(|| AUTOSAVE.into());
        
        // Don't save invisible files
        if filename.starts_with('.') {
            return None;
        }

        // Create the directory

        let directory = Path::new(SAVES);

        if !directory.exists() && create_dir_all(&directory).is_err() {
            return None;
        }

        // Save the game and return the path

        let save = directory.join(filename);

        File::create(&save).ok()
            .and_then(|mut file| bincode::serialize_into(&mut file, self, SIZE_LIMIT).ok())
            .map(|_| save)
    }
}