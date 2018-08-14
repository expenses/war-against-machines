// A Map struct that combines Tiles and Units for convenience
// This struct contains all the stuff that is saved/loaded

use super::units::*;
use super::animations::*;
use settings::*;

use std::fs::*;
use std::path::*;

mod grid;
mod iter_2d;
mod tiles;
mod vision;
mod walls;

use super::messages::*;
use super::commands::*;

pub use self::walls::*;
pub use self::tiles::*;

use bincode;

const EXTENSION: &str = ".sav";

// The Map struct
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Map {
    pub units: Units,
    pub tiles: Tiles,
    pub turn: u8,
    pub light: f32,
    pub side: Side
}

impl Map {
    // Create a new map
    pub fn new(cols: usize, rows: usize, light: f32) -> Map {
        Map {
            light, 
            units: Units::new(),
            tiles: Tiles::new(cols, rows),
            turn: 1,
            side: Side::PlayerA
        }
    }

    pub fn new_from_settings(settings: SkirmishSettings) -> Self {
        let mut map = Self::new(settings.cols, settings.rows, settings.light);

        // Add player units
        for x in 0 .. settings.player_units {
            map.units.add(settings.player_unit_type, Side::PlayerA, x, 0, UnitFacing::Bottom);
        }

        // Add ai units
        for y in settings.cols - settings.ai_units .. settings.cols {
            map.units.add(settings.ai_unit_type, Side::PlayerB, y, settings.rows - 1, UnitFacing::Top);
        }
        
        // Generate tiles
        map.tiles.generate(&map.units);

        map
    }
    
    // Work out if a tile is taken or not
    pub fn taken(&self, x: usize, y: usize) -> bool {
        !self.tiles.at(x, y).obstacle.is_empty() || self.units.at(x, y).is_some()
    }

    // Work out how many units of a particular side are visible to the other side
    pub fn visible(&self, side: Side) -> impl Iterator<Item=&Unit> {
        self.units.iter()
            .filter(move |unit| unit.side == side && self.tiles.visibility_at(unit.x, unit.y, side.enemies()).is_visible())
    }

    // Load a skirmish if possible
    pub fn load(path: &Path) -> Option<Map> {
        File::open(path).ok()
            .and_then(|mut file| bincode::deserialize_from(&mut file).ok())
    }

    // Save the skirmish
    pub fn save(&self, mut filename: String, settings: &Settings) -> Option<PathBuf> {
        filename.push_str(EXTENSION);
        
        let directory = Path::new(&settings.savegames);

        // Don't save invisible files and return if the directory fails to be created
        if filename.starts_with('.') || (!directory.exists() && create_dir_all(&directory).is_err()) {
            return None;
        }

        // Save the game and return the path

        let save = directory.join(filename);

        File::create(&save).ok()
            .and_then(|mut file| bincode::serialize_into(&mut file, self).ok())
            .map(|_| save)
    }

    pub fn perform_command(&mut self, id: u8, command: Command) -> (Vec<Animation>, Vec<Animation>) {
        let mut animations = ServerAnimations::new();

        match command {
            Command::Walk(path) => move_command(self, id, path, &mut animations),
            Command::Turn(facing) => turn_command(self, id, facing, &mut animations),
            Command::UseItem(item) => use_item_command(self, id, item, &mut animations),
            Command::PickupItem(item) => pickup_item_command(self, id, item, &mut animations),
            Command::DropItem(item) => drop_item_command(self, id, item, &mut animations),
            Command::ThrowItem {item, x, y} => throw_item_command(self, id, item, x, y, &mut animations),
            Command::Fire {x, y} => fire_command(self, id, x, y, &mut animations),
        }

        animations.split()
    }

    pub fn end_turn(&mut self) -> (Vec<Animation>, Vec<Animation>) {
        for unit in self.units.iter_mut() {
            unit.moves = unit.tag.moves();
        }

        match self.side {
            Side::PlayerA => self.side = Side::PlayerB,
            Side::PlayerB => {
                self.turn += 1;
                self.side = Side::PlayerA;
            }
        }

        let mut animations = ServerAnimations::new();
        animations.push_state(self);
        animations.split()
    }

    pub fn clone_visible(&mut self, side: Side) -> Self {
        // Update visibility first
        self.tiles.update_visibility(&self.units);

        Self {
            light: self.light,
            turn: self.turn,
            side: self.side,
            units: self.tiles.visible_units(&self.units, side).cloned().collect(),
            // todo: should only clone the info of visible tiles and not clone the enemy vision
            tiles: self.tiles.clone()

        }
    }
}

#[test]
fn load_save() {
    use super::units::*;

    // Test saving and loading a map

    let settings = Settings::default();
    let mut output = PathBuf::from(&settings.savegames);
    output.push("test.sav");

    let mut map = Map::new(20, 20, 0.5);
    map.units.add(UnitType::Squaddie, Side::PlayerA, 0, 0, UnitFacing::Bottom);
    map.tiles.update_visibility(&map.units);

    assert_eq!(map.save("test".into(), &settings), Some(output.clone()));
    Map::load(&output).unwrap();
}