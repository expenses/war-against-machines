use std::fs::File;
use std::io::Read;

use battle::units::UnitType;
use networking::*;
use utils::clamp;

use toml;
use toml::Value;

use std::path::PathBuf;

type Table = toml::map::Map<String, Value>;

// Extract the table out of a toml value
fn to_table(value: Value) -> Option<Table> {
    if let Value::Table(table) = value {
        Some(table)
    } else {
        None
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    pub volume: u8,
    pub window_width: u32,
    pub window_height: u32,
    pub fullscreen: bool,
    pub savegames: String,
}

// The default settings
impl Default for Settings {
    fn default() -> Settings {
        Settings {
            volume: Self::DEFAULT_VOLUME,
            window_width: 960,
            window_height: 540,
            fullscreen: false,
            savegames: "savegames".into(),
        }
    }
}

impl Settings {
    const DEFAULT_VOLUME: u8 = 100;
    const FILENAME: &'static str = "settings.toml";

    // Load the settings or use the defaults
    pub fn load() -> Settings {
        let mut buffer = String::new();
        let mut settings = Settings::default();

        // If the settings were able to be loaded and parsed into a toml table
        if let Some(loaded) = File::open(Self::FILENAME)
            .ok()
            .and_then(|mut file| file.read_to_string(&mut buffer).ok())
            .and_then(|_| buffer.parse::<Value>().ok())
            .and_then(to_table)
        {
            let mut settings_table = settings.to_table();

            // Change the default settings to the new keys
            for (key, value) in loaded {
                match settings_table.entry(key) {
                    toml::map::Entry::Occupied(mut entry) => {
                        entry.insert(value);
                    }
                    toml::map::Entry::Vacant(entry) => error!(
                        "Warning: '{}' key '{}' does not exist.",
                        Self::FILENAME,
                        entry.key()
                    ),
                }
            }

            // Deserialize the settings again
            if let Ok(settings_struct) = Value::Table(settings_table).try_into() {
                settings = settings_struct;
            }
        }

        settings.clamp();
        settings
    }

    // Save the settings
    #[cfg(test)]
    pub fn save(&self) {
        use std::io::Write;

        if let Ok(mut file) = File::create(Self::FILENAME) {
            let default = Settings::default().to_table();
            let mut settings = self.to_table();

            // Remove the settings that are the default
            for (key, value) in default {
                if settings[&key] == value {
                    settings.remove(&key);
                }
            }

            // Save the rest
            if let Ok(buffer) = toml::to_vec(&Value::from(settings)) {
                if file.write_all(&buffer).is_err() {
                    error!("Warnings: Failed to write to '{}'", Self::FILENAME);
                }
            }
        } else {
            error!("Warning: Failed to open '{}'", Self::FILENAME);
        }
    }

    // Make sure the volume isn't too high
    pub fn clamp(&mut self) {
        self.volume = clamp(self.volume, 0, Self::DEFAULT_VOLUME);
    }

    // Reset the settings
    pub fn reset(&mut self) {
        self.volume = Self::DEFAULT_VOLUME;
    }

    // Convert the settings into a B-Tree map of key-value pairs
    fn to_table(&self) -> Table {
        // This should never fail
        Value::try_from(&self).ok().and_then(to_table).unwrap()
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum GameType {
    Local,
    Host,
    Connect,
}

impl GameType {
    pub fn as_str(&self) -> &str {
        match *self {
            GameType::Local => "Local",
            GameType::Host => "Host",
            GameType::Connect => "Connect",
        }
    }

    pub fn rotate_right(&mut self) {
        *self = match *self {
            GameType::Local => GameType::Host,
            GameType::Host => GameType::Connect,
            GameType::Connect => GameType::Local,
        }
    }

    pub fn rotate_left(&mut self) {
        self.rotate_right();
        self.rotate_right();
    }
}

// A struct for holding the initialization settings for a skirmish
pub struct SkirmishSettings {
    pub width: usize,
    pub height: usize,
    pub player_a_units: usize,
    pub player_b_units: usize,
    pub player_a_unit_type: UnitType,
    pub player_b_unit_type: UnitType,
    pub light: u8,
    pub game_type: GameType,
    pub address: String,
    pub save_game: Option<PathBuf>,
}

// The default skirmish settings
impl Default for SkirmishSettings {
    fn default() -> SkirmishSettings {
        SkirmishSettings {
            width: 30,
            height: 30,
            player_a_units: 6,
            player_b_units: 4,
            player_a_unit_type: UnitType::Squaddie,
            player_b_unit_type: UnitType::Machine,
            light: 10,
            game_type: GameType::Local,
            address: DEFAULT_ADDR.into(),
            save_game: None,
        }
    }
}

impl SkirmishSettings {
    const MIN_MAP_SIZE: usize = 10;
    const MAX_MAP_SIZE: usize = 60;

    // Ensure that the settings are between their min and max values
    pub fn clamp(&mut self) {
        self.width = clamp(self.width, Self::MIN_MAP_SIZE, Self::MAX_MAP_SIZE);
        self.height = clamp(self.height, Self::MIN_MAP_SIZE, Self::MAX_MAP_SIZE);
        self.player_a_units = clamp(self.player_a_units, 1, self.width);
        self.player_b_units = clamp(self.player_b_units, 1, self.width);
        self.light = clamp(self.light, 0, 10);
    }

    // Switch the player unit type
    pub fn change_player_a_unit_type(&mut self) {
        self.player_a_unit_type = match self.player_a_unit_type {
            UnitType::Squaddie => UnitType::Machine,
            UnitType::Machine => UnitType::Squaddie,
        }
    }

    // Switch the ai unit type
    pub fn change_player_b_unit_type(&mut self) {
        self.player_b_unit_type = match self.player_b_unit_type {
            UnitType::Squaddie => UnitType::Machine,
            UnitType::Machine => UnitType::Squaddie,
        }
    }

    pub fn set_savegame(&mut self, savegame: &str, settings: &Settings) {
        self.save_game = Some(PathBuf::from(&settings.savegames).join(savegame));
    }
}

#[test]
fn load_save() {
    let mut settings = Settings::default();

    // Test clamping the settings

    settings.volume = 255;
    settings.clamp();

    assert_eq!(settings.volume, Settings::DEFAULT_VOLUME);

    settings.volume = 99;

    // Test saving and loading the settings

    settings.save();

    let settings_2 = Settings::load();

    assert_eq!(settings_2.volume, 99);

    settings.reset();
    settings.save();
}
