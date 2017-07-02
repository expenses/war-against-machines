// A Map struct that combines Tiles and Units for convenience

use battle::units::{UnitSide, Units};
use battle::tiles::{Visibility, Tiles};

use std::fs::File;
use bincode;

const SIZE_LIMIT: bincode::Infinite = bincode::Infinite;
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

    pub fn load() -> Option<Map> {
        File::open(AUTOSAVE).ok()
            .and_then(|mut file| bincode::deserialize_from(&mut file, SIZE_LIMIT).ok())
    }

    pub fn save(&self) -> Option<()> {
        File::create(AUTOSAVE).ok()
            .and_then(|mut file| bincode::serialize_into(&mut file, self, SIZE_LIMIT).ok())
    }
}