// A Map struct that combines Tiles and Units for convenience
// This struct contains all the stuff that is saved/loaded


use settings::*;
use error::*;

use std::fs::*;
use std::path::*;

mod grid;
mod iter_2d;
mod tiles;
mod vision;
mod walls;

use super::units::*;
use super::animations::*;
use super::messages::*;
use super::commands::*;

pub use self::walls::*;
pub use self::tiles::*;

use bincode;

const EXTENSION: &str = ".sav";

// The Map struct
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Map {
    pub units: Units,
    pub tiles: Tiles,
    pub light: f32,
    pub side: Side,
    turn: u8
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
    pub fn load(path: &Path) -> Result<Map> {
        File::open(path)
            .map_err(|error| error.into())
            .and_then(|mut file| bincode::deserialize_from(&mut file).map_err(|err| err.into()))
    }

    // Save the skirmish
    pub fn save(&self, mut filename: String, settings: &Settings) -> ServerAnimations {
        filename.push_str(EXTENSION);
        let directory = Path::new(&settings.savegames);
        let mut animations = ServerAnimations::new();

        if filename.starts_with('.') {
            animations.push_message(format!("Error: '{}' is an invalid name for a savegame", filename));
            return animations;
        } else if !directory.exists() {
            if let Err(error) = create_dir_all(&directory) {
                animations.push_message(format!("Error '{}' recieved while attemping to create directory '{}", error, directory.display()));
                return animations;
            }
        }

        let savegame = directory.join(&filename);

        let result = File::create(&savegame)
            .map_err(|error| Box::new(bincode::ErrorKind::Io(error)))
            .and_then(|mut file| bincode::serialize_into(&mut file, self));

        let message = match result {
            Ok(()) => format!("Game saved to '{}'", savegame.display()),
            Err(error) => format!("Error recieved while trying to save to '{}': {}", savegame.display(), error)
        };

        animations.push_message(message);
        animations
    }

    pub fn handle_message(&mut self, message: ClientMessage, settings: &Settings) -> (Vec<Animation>, Vec<Animation>) {
        match message {
            ClientMessage::EndTurn => self.end_turn(),
            ClientMessage::SaveGame(filename) => self.save(filename, settings),
            ClientMessage::Command {unit, command} => self.perform_command(unit, command)
        }.split()
    }

    pub fn perform_command(&mut self, id: u8, command: Command) -> ServerAnimations {
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

        animations
    }

    pub fn end_turn(&mut self) -> ServerAnimations {
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
        animations
    }

    pub fn clone_visible(&mut self, side: Side) -> Self {
        // Update visibility first
        self.tiles.update_visibility(&self.units);

        Self {
            light: self.light,
            turn: self.turn,
            side: self.side,
            units: self.tiles.visible_units(&self.units, side).cloned().collect(),
            tiles: self.tiles.clone_visible(side)
        }
    }

    pub fn update_from(&mut self, mut new: Map, side: Side) {
        self.tiles.update_from(new.tiles, side);
        new.tiles = self.tiles.clone();
        *self = new;
    }

    pub fn info(&self) -> String {
        format!("Turn {} - {}", self.turn, self.side)
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

    let (player_a_animations, player_b_animations) = map.save("test".into(), &settings).split();

    assert_eq!(player_a_animations, player_b_animations);
    assert_eq!(player_a_animations, vec![Animation::Message(format!("Game saved to '{}'", output.display()))]);
    Map::load(&output).unwrap();
}