//! A `Map` struct that combines `Tiles` and `Units` for convenience

use battle::units::{UnitSide, Units};
use battle::tiles::{Visibility, Tiles};

/// The Map struct
pub struct Map {
    pub units: Units,
    pub tiles: Tiles
}

impl Map {
    /// Create a new map
    pub fn new() -> Map {
        Map {
            units: Units::new(),
            tiles: Tiles::new()
        }
    }

    /// Work out if a tile is taken or not
    pub fn taken(&self, x: usize, y: usize) -> bool {
        !self.tiles.at(x, y).walkable() ||
        self.units.at(x, y).is_some()
    }

    /// Work out how many units of a particular side are visible to the other side
    pub fn visible(&self, side: UnitSide) -> usize {
        self.units.iter()
            .filter(|&(_, unit)| unit.side == side && match side {
                UnitSide::Friendly => self.tiles.at(unit.x, unit.y).enemy_visibility,
                UnitSide::Enemy => self.tiles.at(unit.x, unit.y).unit_visibility
            } == Visibility::Visible)
            .count()
    }
}