// A Map struct that combines Tiles and Units for convenience
// This struct contains all the stuff that is saved/loaded

use super::units::{UnitSide, Units};
use super::tiles::Tiles;
use super::drawer::Camera;
use settings::Settings;

use std::fs::{File, create_dir_all};
use std::path::{Path, PathBuf};

use bincode;

const SIZE_LIMIT: bincode::Infinite = bincode::Infinite;
const EXTENSION: &str = ".sav";

// The Map struct
#[derive(Serialize, Deserialize)]
pub struct Map {
    pub units: Units,
    pub tiles: Tiles,
    pub camera: Camera,
    pub turn: u8,
    pub light: f32
}

impl Map {
    // Create a new map
    pub fn new(cols: usize, rows: usize, light: f32) -> Map {
        Map {
            light, 
            units: Units::new(),
            tiles: Tiles::new(cols, rows),
            camera: Camera::new(),
            turn: 1
        }
    }
    
    // Work out if a tile is taken or not
    pub fn taken(&self, x: usize, y: usize) -> bool {
        !self.tiles.at(x, y).obstacle.is_empty() || self.units.at(x, y).is_some()
    }

    // Work out how many units of a particular side are visible to the other side
    pub fn visible(&self, side: UnitSide) -> usize {
        self.units.iter()
            .filter(|unit| unit.side == side && match side {
                UnitSide::Player => self.tiles.at(unit.x, unit.y).ai_visibility,
                UnitSide::AI => self.tiles.at(unit.x, unit.y).player_visibility
            }.is_visible())
            .count()
    }

    // Load a skirmish if possible
    pub fn load(filename: &str, settings: &Settings) -> Option<Map> {
        let path = Path::new(&settings.savegames).join(filename);

        File::open(path).ok()
            .and_then(|mut file| bincode::deserialize_from(&mut file, SIZE_LIMIT).ok())
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
            .and_then(|mut file| bincode::serialize_into(&mut file, self, SIZE_LIMIT).ok())
            .map(|_| save)
    }
}

#[test]
fn load_save() {
    use super::units::UnitType;

    // Test saving and loading a map

    let settings = Settings::default();
    let mut output = PathBuf::from(&settings.savegames);
    output.push("test.sav");

    let mut map = Map::new(20, 20, 0.5);
    map.units.add(UnitType::Squaddie, UnitSide::Player, 0, 0);
    map.tiles.update_visibility(&map.units);

    assert_eq!(map.save("test".into(), &settings), Some(output));
    Map::load("test.sav", &settings).unwrap();
}