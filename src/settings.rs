use std::fs::File;
use std::io::{Read, Write};

use battle::units::UnitType;
use resources::{FONT_HEIGHT, CHARACTER_GAP, ImageSource};
use utils::clamp;

use toml;
use toml::Value;

use std::collections::BTreeMap;
use std::collections::btree_map::Entry;


type Table = BTreeMap<String, Value>;

// Extract the table out of a toml value
fn to_table(value: Value) -> Option<Table> {
    if let Value::Table(table) = value {
        Some(table)
    } else {
        None
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub volume: u8,
    pub ui_scale: u8,
    pub window_width: u32,
    pub window_height: u32,
    pub fullscreen: bool,
    pub savegames: String
}

// The default settings
impl Default for Settings {
    fn default() -> Settings {
        Settings {
            volume: Self::DEFAULT_VOLUME,
            ui_scale: Self::DEFAULT_UI_SCALE,
            window_width: 960,
            window_height: 540,
            fullscreen: false,
            savegames: "savegames".into()
        }
    }
}

impl Settings {
    const DEFAULT_VOLUME: u8 = 100;
    const DEFAULT_UI_SCALE: u8 = 2;
    const MAX_UI_SCALE: u8 = 4;
    const FILENAME: &'static str = "settings.toml";

    // Load the settings or use the defaults
    pub fn load() -> Settings {
        let mut buffer = String::new();
        let mut settings = Settings::default();

        // If the settings were able to be loaded and parsed into a toml table
        if let Some(loaded) = File::open(Self::FILENAME).ok()
            .and_then(|mut file| file.read_to_string(&mut buffer).ok())
            .and_then(|_| buffer.parse::<Value>().ok())
            .and_then(to_table) {

            let mut settings_table = settings.to_table();

            // Change the default settings to the new keys
            for (key, value) in loaded {
                match settings_table.entry(key) {
                    Entry::Occupied(mut entry) => {
                        entry.insert(value);
                    },
                    Entry::Vacant(entry) => eprintln!("Warning: '{}' key '{}' does not exist.", Self::FILENAME, entry.key())
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
    pub fn save(&self) {
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
                    eprintln!("Warnings: Failed to write to '{}'", Self::FILENAME);
                }
            }
        } else {
            eprintln!("Warning: Failed to open '{}'", Self::FILENAME);
        }
    }

    pub fn ui_scale(&self) -> f32 {
        f32::from(self.ui_scale)
    }

    // Make sure the volume isn't too high
    pub fn clamp(&mut self) {
        self.volume = clamp(self.volume, 0, Self::DEFAULT_VOLUME);
        self.ui_scale = clamp(self.ui_scale, 1, Self::MAX_UI_SCALE);
    }

    // Get the height that a string would be rendered at
    pub fn font_height(&self) -> f32 {
        FONT_HEIGHT * self.ui_scale()
    }

    // Get the width that a string would be rendered at
    pub fn font_width(&self, string: &str) -> f32 {
        string.chars().fold(0.0, |total, character| total + (character.width() + CHARACTER_GAP) * self.ui_scale())
    }

    // Reset the settings
    pub fn reset(&mut self) {
        self.volume = Self::DEFAULT_VOLUME;
        self.ui_scale = Self::DEFAULT_UI_SCALE;
    }

    // Convert the settings into a B-Tree map of key-value pairs
    fn to_table(&self) -> Table {
        // This should never fail
        Value::try_from(&self).ok().and_then(to_table).unwrap()
    }
}

// A struct for holding the initialization settings for a skirmish
pub struct SkirmishSettings {
    pub cols: usize,
    pub rows: usize,
    pub player_units: usize,
    pub ai_units: usize,
    pub player_unit_type: UnitType,
    pub ai_unit_type: UnitType,
    pub light: f32
}

// The default skirmish settings
impl Default for SkirmishSettings {
    fn default() -> SkirmishSettings {
        SkirmishSettings {
            cols: 30,
            rows: 30,
            player_units: 6,
            ai_units: 4,
            player_unit_type: UnitType::Squaddie,
            ai_unit_type: UnitType::Machine,
            light: 1.0
        }
    }
}

impl SkirmishSettings {
    const MIN_MAP_SIZE: usize = 10;
    const MAX_MAP_SIZE: usize = 60;

    // Ensure that the settings are between their min and max values
    pub fn clamp(&mut self) {
        self.cols = clamp(self.cols, Self::MIN_MAP_SIZE, Self::MAX_MAP_SIZE);
        self.rows = clamp(self.rows, Self::MIN_MAP_SIZE, Self::MAX_MAP_SIZE);
        self.player_units = clamp(self.player_units, 1, self.cols);
        self.ai_units = clamp(self.ai_units, 1, self.cols);
        self.light = clamp(self.light, 0.0, 1.0);
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

#[test]
fn load_save() {
    let mut settings = Settings::default();

    // Test clamping the settings

    settings.volume = 255;
    settings.ui_scale = 100;

    settings.clamp();

    assert_eq!(settings.volume, Settings::DEFAULT_VOLUME);
    assert_eq!(settings.ui_scale, Settings::MAX_UI_SCALE);

    settings.volume = 99;

    // Test saving and loading the settings

    settings.save();

    let settings_2 = Settings::load();

    assert_eq!(settings_2.volume, 99);

    settings.reset();
    settings.save();
}