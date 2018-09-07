// A Map struct that combines Tiles and Units for convenience
// This struct contains all the stuff that is (de)serialized

use bincode;

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
use super::responses::*;
use super::messages::*;
use super::commands::*;

pub use self::walls::*;
pub use self::tiles::*;

const EXTENSION: &str = ".sav";

// The Map struct
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Map {
    pub units: Units,
    pub tiles: Tiles,
    pub light: f32,
    pub side: Side,
    turn: u16
}

impl Map {
    // Create a new map
    pub fn new(width: usize, height: usize, light: f32) -> Map {
        Map {
            light, 
            units: Units::new(),
            tiles: Tiles::new(width, height),
            turn: 1,
            side: Side::PlayerA
        }
    }

    pub fn new_or_load(settings: &SkirmishSettings) -> Result<Self> {
        match settings.save_game {
            None => Ok(Self::new_from_settings(settings)),
            Some(ref path) => Self::load(path)
        }
    }

    pub fn new_from_settings(settings: &SkirmishSettings) -> Self {
        let mut map = Self::new(settings.width, settings.height, f32::from(settings.light) / 10.0);

        // Add player units
        for x in 0 .. settings.player_a_units {
            map.units.add(settings.player_a_unit_type, Side::PlayerA, x, 0, UnitFacing::Bottom);
        }

        // Add ai units
        for y in settings.width - settings.player_b_units .. settings.width {
            map.units.add(settings.player_b_unit_type, Side::PlayerB, y, settings.height - 1, UnitFacing::Top);
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
    pub fn save(&self, mut filename: String, settings: &Settings) -> ServerResponses {
        filename.push_str(EXTENSION);
        let directory = Path::new(&settings.savegames);
        let mut responses = ServerResponses::new();

        if filename.starts_with('.') {
            responses.push_message(format!("Error: '{}' is an invalid name for a savegame", filename));
            return responses;
        } else if !directory.exists() {
            if let Err(error) = create_dir_all(&directory) {
                responses.push_message(format!("Error '{}' recieved while attemping to create directory '{}", error, directory.display()));
                return responses;
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

        responses.push_message(message);
        responses
    }

    pub fn handle_message(&mut self, message: ClientMessage, settings: &Settings, side: Side) -> (Vec<Response>, Vec<Response>) {
        match message {
            ClientMessage::EndTurn => self.end_turn(side),
            ClientMessage::SaveGame(filename) => self.save(filename, settings),
            ClientMessage::Command {unit, command} => self.perform_command(unit, command, side)
        }.split()
    }

    pub fn perform_command(&mut self, id: u8, command: Command, side: Side) -> ServerResponses {
        let mut responses = ServerResponses::new();

        // Checks that:
        // A) The side is correct
        // B) The unit exists
        // C) The unit is on that side
        if !(side == self.side && self.units.get(id).map(|unit| unit.side == side).unwrap_or(false)) {
            return responses;
        }

        let moves = self.units.get(id).unwrap().moves;

        match command {
            Command::Walk(path)             => move_command(self, id, path, &mut responses),
            Command::Turn(facing)           => turn_command(self, id, facing, &mut responses),
            Command::UseItem(item)          => use_item_command(self, id, item, &mut responses),
            Command::PickupItem(item)       => pickup_item_command(self, id, item, &mut responses),
            Command::DropItem(item)         => drop_item_command(self, id, item, &mut responses),
            Command::ThrowItem {item, x, y} => throw_item_command(self, id, item, x, y, &mut responses),
            Command::Fire {x, y}            => fire_command(self, id, x, y, &mut responses),
        }

        // All commands should have a cost, so if one doesn't, it failed
        if self.units.get(id).unwrap().moves == moves {
            responses.push(side, Response::InvalidCommand);
        }

        let player_a_units = self.units.count(Side::PlayerA);
        let player_b_units = self.units.count(Side::PlayerB);

        // Push the gameover response
        if player_a_units == 0 || player_b_units == 0 {
            let player_a_units_lost = self.units.max_player_a_units - player_a_units;
            let player_b_units_lost = self.units.max_player_b_units - player_b_units;

            responses.push(Side::PlayerA, Response::GameOver(GameStats {
                won: player_a_units != 0,
                units_lost: player_a_units_lost,
                units_killed: player_b_units_lost
            }));

            responses.push(Side::PlayerB, Response::GameOver(GameStats {
                won: player_b_units != 0,
                units_lost: player_b_units_lost,
                units_killed: player_a_units_lost
            }));
        }

        responses
    }

    pub fn end_turn(&mut self, side: Side) -> ServerResponses {
        let mut responses = ServerResponses::new();

        if side != self.side {
            return responses;
        }

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

        responses.push_and_update_state(self);
        responses
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

    pub fn turn(&self) -> u16 {
        self.turn
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

    let (player_a_responses, player_b_responses) = map.save("test".into(), &settings).split();

    assert_eq!(player_a_responses, player_b_responses);
    assert_eq!(player_a_responses, vec![Response::Message(format!("Game saved to '{}'", output.display()))]);
    Map::load(&output).unwrap();
}