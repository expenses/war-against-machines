// A Map struct that combines Tiles and Units for convenience
// This struct contains all the stuff that is saved/loaded

use super::units::{UnitSide, Units};
use super::tiles::Tiles;
use super::drawer::Camera;

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
    pub camera: Camera,
    pub turn: u8
}

impl Map {
    // Create a new map
    pub fn new(cols: usize, rows: usize) -> Map {
        Map {
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
        
        let directory = Path::new(SAVES);

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
    let mut map = Map::new(20, 20);
    map.units.add(UnitType::Squaddie, UnitSide::Player, 0, 0);
    map.tiles.update_visibility(&map.units);

    assert_eq!(map.save(Some("test".into())), Some(PathBuf::from("savegames/skirmishes/test.sav")));
    Map::load("test.sav").unwrap();
}