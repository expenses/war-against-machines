use battle::units::{UnitSide, Units};
use battle::tiles::{Visibility, Tiles};

pub struct Map {
    pub units: Units,
    pub tiles: Tiles
}

impl Map {
    pub fn new() -> Map {
        Map {
            units: Units::new(),
            tiles: Tiles::new()
        }
    }

    pub fn taken(&self, x: usize, y: usize) -> bool {
        !self.tiles.at(x, y).walkable() ||
        self.units.at(x, y).is_some()
    }

    pub fn visible(&self, side: UnitSide) -> usize {
        self.units.iter()
            .filter(|&(_, unit)| unit.side == side && match side {
                UnitSide::Friendly => self.tiles.at(unit.x, unit.y).enemy_visibility,
                UnitSide::Enemy => self.tiles.at(unit.x, unit.y).unit_visibility
            } == Visibility::Visible)
            .count()
    }
}