use battle::units::Units;
use battle::tiles::Tiles;

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
}