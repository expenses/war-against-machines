use std::fs::File;
use std::io::{Read, Write};

use battle::units::UnitType;

use toml;

const FILENAME: &str = "settings.toml";
const MIN_MAP_SIZE: usize = 10;
const MAX_MAP_SIZE: usize = 60;

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub volume: u8,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            volume: 100
        }
    }
}

impl Settings {
    pub fn load() -> Settings {
        let mut string = String::new();

        File::open(FILENAME).ok()
            .and_then(|mut file| file.read_to_string(&mut string).ok())
            .and_then(|_| toml::from_str(&string).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let mut file = File::create(FILENAME).unwrap();
        let buffer = toml::to_vec(self).unwrap();
        file.write_all(&buffer).unwrap();
    }

    pub fn clamp(&mut self) {
        self.volume = clamp!(self.volume, 0, 100);
    }
}

// A struct for holding the initialization settings for a skirmish
pub struct SkirmishSettings {
    pub cols: usize,
    pub rows: usize,
    pub player_units: usize,
    pub ai_units: usize,
    pub player_unit_type: UnitType,
    pub ai_unit_type: UnitType
}

impl Default for SkirmishSettings {
    fn default() -> SkirmishSettings {
        SkirmishSettings {
            cols: 30,
            rows: 30,
            player_units: 6,
            ai_units: 4,
            player_unit_type: UnitType::Squaddie,
            ai_unit_type: UnitType::Machine
        }
    }
}

impl SkirmishSettings {
    // Ensure that the settings are between their min and max values
    pub fn clamp(&mut self) {
        self.cols = clamp!(self.cols, MIN_MAP_SIZE, MAX_MAP_SIZE);
        self.rows = clamp!(self.rows, MIN_MAP_SIZE, MAX_MAP_SIZE);
        self.player_units = clamp!(self.player_units, 1, self.cols);
        self.ai_units = clamp!(self.ai_units, 1, self.cols);
    }

    // Switch the player unit type
    pub fn change_player_unit_type(&mut self) {
        self.player_unit_type = match self.player_unit_type {
            UnitType::Squaddie => UnitType::Machine,
            UnitType::Machine => UnitType::Squaddie
        }
    }

    // Switch the ai unit type
    pub fn change_ai_unit_type(&mut self) {
        self.ai_unit_type = match self.ai_unit_type {
            UnitType::Squaddie => UnitType::Machine,
            UnitType::Machine => UnitType::Squaddie
        }
    }
}
