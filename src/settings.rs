use std::fs::File;
use std::io::{Read, Write};

use battle::units::UnitType;
use resources::{FONT_HEIGHT, CHARACTER_GAP, ImageSource};

use toml;

const FILENAME: &str = "settings.toml";
const MIN_MAP_SIZE: usize = 10;
const MAX_MAP_SIZE: usize = 60;
const DEFAULT_VOLUME: u8 = 100;
const DEFAULT_UI_SCALE: u8 = 2;
const MAX_UI_SCALE: u8 = 4;

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub volume: u8,
    pub ui_scale: u8
}

// The default settings
impl Default for Settings {
    fn default() -> Settings {
        Settings {
            volume: DEFAULT_VOLUME,
            ui_scale: DEFAULT_UI_SCALE
        }
    }
}

impl Settings {
    // Load the settings or use the defaults
    pub fn load() -> Settings {
        let mut string = String::new();

        let mut settings: Settings = File::open(FILENAME).ok()
            .and_then(|mut file| file.read_to_string(&mut string).ok())
            .and_then(|_| toml::from_str(&string).ok())
            .unwrap_or_default();

        settings.clamp();

        settings
    }

    // Save the settings
    pub fn save(&self) {
        let mut file = File::create(FILENAME).unwrap();
        let buffer = toml::to_vec(self).unwrap();
        file.write_all(&buffer).unwrap();
    }

    pub fn ui_scale(&self) -> f32 {
        self.ui_scale as f32
    }

    // Make sure the volume isn't too high
    pub fn clamp(&mut self) {
        self.volume = clamp!(self.volume, 0, DEFAULT_VOLUME);
        self.ui_scale = clamp!(self.ui_scale, 1, MAX_UI_SCALE);
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
        self.volume = DEFAULT_VOLUME;
        self.ui_scale = DEFAULT_UI_SCALE;
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

// The default skirmish settings
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

#[test]
fn load_save() {
    let mut settings = Settings::default();

    // Test clamping the settings

    settings.volume = 255;
    settings.ui_scale = 100;

    settings.clamp();

    assert_eq!(settings.volume, DEFAULT_VOLUME);
    assert_eq!(settings.ui_scale, MAX_UI_SCALE);

    settings.volume = 99;

    // Test saving and loading the settings

    settings.save();

    let settings_2 = Settings::load();

    assert_eq!(settings_2.volume, 99);

    settings.reset();
    settings.save();
}