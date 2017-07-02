// A Map struct that combines Tiles and Units for convenience

use battle::units::{UnitSide, Units};
use battle::tiles::{Visibility, Tiles};

use std::fs::{File, create_dir_all};
use std::path::Path;

use bincode;

const SIZE_LIMIT: bincode::Infinite = bincode::Infinite;

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
        !self.tiles.at(x, y).walkable() ||
        self.units.at(x, y).is_some()
    }

    // Work out how many units of a particular side are visible to the other side
    pub fn visible(&self, side: UnitSide) -> usize {
        self.units.iter()
            .filter(|&(_, unit)| unit.side == side && match side {
                UnitSide::Player => self.tiles.at(unit.x, unit.y).ai_visibility,
                UnitSide::AI => self.tiles.at(unit.x, unit.y).player_visibility
            } == Visibility::Visible)
            .count()
    }

    pub fn load_skirmish(filename: &str) -> Option<Map> {
        let path = Path::new("savegames/skirmishes").join(filename);

        File::open(path).ok()
            .and_then(|mut file| bincode::deserialize_from(&mut file, SIZE_LIMIT).ok())
    }

    pub fn save_skrimish(&self, filename: &str) -> Option<()> {
        let directory = Path::new("savegames/skirmishes");

        if !directory.exists() && create_dir_all(&directory).is_err() {
            return None;
        }

        File::create(directory.join(filename)).ok()
            .and_then(|mut file| bincode::serialize_into(&mut file, self, SIZE_LIMIT).ok())
    }
}