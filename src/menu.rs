//! The main menu of the game

use sdl2::keyboard::Keycode;
use colours::WHITE;

use Resources;
use context::Context;
use utils::bound;
use battle::units::UnitType;

const MIN_SIZE: usize = 10;
const MAX_SIZE: usize = 50;
const DEFAULT_SIZE: usize = 20;
const SIZE_CHANGE: usize = 5;
const TITLE_TOP_OFFSET: f32 = 50.0;

const DEFAULT_UNITS: usize = 3;
const DEFAULT_UNIT_TYPE: UnitType = UnitType::Squaddie;
const DEFAULT_ENEMY_TYPE: UnitType = UnitType::Machine;

/// Callbacks that can be returned from key presses
pub enum Callback {
    Play,
}

/// A submenu inside the main menu
pub struct Submenu {
    selection: usize,
    list: Vec<String>
}

impl Submenu {
    /// Create a new submenu
    pub fn new(list: Vec<String>) -> Submenu {
        Submenu {
            selection: 0,
            list
        }
    }

    /// Draw the items in the submenu
    pub fn draw(&self, ctx: &mut Context, resources: &Resources) {
        for (i, item) in self.list.iter().enumerate() {
            let mut string = item.clone();

            // If the index is the same as the selection index, push a '>' to indicate that the option is selected
            if i == self.selection { string.insert_str(0, "> "); }

            // Render the string
            let rendered = resources.render("main", &string, WHITE);

            // Get the center of the rendered string
            let center = (ctx.width() - rendered.query().width) as f32 / 2.0;

            // Draw the string
            ctx.draw(&rendered, center, 150.0 + i as f32 * 20.0, 1.0);
        }
    }

    /// Change an item in the list
    pub fn set_item(&mut self, i: usize, string: String) {
        self.list[i] = string;
    }

    /// Rotate the selection up
    pub fn rotate_up(&mut self) {
        self.selection = match self.selection {
            0 => self.list.len() - 1,
            _ => self.selection - 1
        }
    }

    /// Rotate the selection down
    pub fn rotate_down(&mut self) {
        self.selection = (self.selection + 1) % self.list.len();
    }
}

/// A struct for holding the settings of a skirmish
pub struct SkirmishSettings {
    pub cols: usize,
    pub rows: usize,
    pub units: usize,
    pub enemies: usize,
    pub player_unit_type: UnitType,
    pub ai_unit_type: UnitType
}

impl SkirmishSettings {
    /// Create a new `SkirmishSettings` using the defaults
    pub fn new() -> SkirmishSettings {
        SkirmishSettings {
            cols: DEFAULT_SIZE,
            rows: DEFAULT_SIZE,
            units: DEFAULT_UNITS,
            enemies: DEFAULT_UNITS,
            player_unit_type: DEFAULT_UNIT_TYPE,
            ai_unit_type: DEFAULT_ENEMY_TYPE
        }
    }

    /// Ensure that the settings are between their upper and lower bounds
    pub fn bound(&mut self) {
        self.cols = bound(self.cols, MIN_SIZE, MAX_SIZE);
        self.rows = bound(self.rows, MIN_SIZE, MAX_SIZE);
        self.units = bound(self.units, 1, self.cols);
        self.enemies = bound(self.enemies, 1, self.cols);
    }

    /// Switch the player unit type
    pub fn change_player_unit_type(&mut self) {
        self.player_unit_type = match self.player_unit_type {
            UnitType::Squaddie => UnitType::Machine,
            UnitType::Machine => UnitType::Squaddie
        }
    }

    /// Switch the ai unit type
    pub fn change_ai_unit_type(&mut self) {
        self.ai_unit_type = match self.ai_unit_type {
            UnitType::Squaddie => UnitType::Machine,
            UnitType::Machine => UnitType::Squaddie
        }
    }
}

/// Which submenu is selected
pub enum Selected {
    Main,
    Skirmish
}

/// The main menu struct
pub struct Menu {
    pub skirmish_settings: SkirmishSettings,
    main: Submenu,
    skirmish: Submenu,
    submenu: Selected
}

impl Menu {
    /// Create a new `Menu`
    pub fn new() -> Menu {
        Menu {
            skirmish_settings: SkirmishSettings::new(),
            main: Submenu::new(vec!["Skirmish".into(), "Quit".into()]),
            skirmish: Submenu::new(vec![
                    "Play".into(),
                    format!("Cols: {}", DEFAULT_SIZE),
                    format!("Rows: {}", DEFAULT_SIZE),
                    format!("Units: {}", DEFAULT_UNITS),
                    format!("Enemies: {}", DEFAULT_UNITS),
                    format!("Unit type: {}", DEFAULT_UNIT_TYPE),
                    format!("Enemy type: {}", DEFAULT_ENEMY_TYPE),
                    "Back".into()
            ]),
            submenu: Selected::Main
        }
    }

    /// Draw the menu
    pub fn draw(&self, ctx: &mut Context, resources: &Resources) {
        // Draw the title
        let title = resources.image(&"title".into());
        let center = (ctx.width() - title.query().width) as f32 / 2.0;
        ctx.draw(title, center, TITLE_TOP_OFFSET, 1.0);

        // Draw the selected submenu
        match self.submenu {
            Selected::Main => self.main.draw(ctx, resources),
            Selected::Skirmish => self.skirmish.draw(ctx, resources)
        }
    }

    /// Refresh the skirmish submenu
    fn refresh_skirmish(&mut self) {
        self.skirmish_settings.bound();
        self.skirmish.set_item(1, format!("Cols: {}", self.skirmish_settings.cols));
        self.skirmish.set_item(2, format!("Rows: {}", self.skirmish_settings.rows));
        self.skirmish.set_item(3, format!("Units: {}", self.skirmish_settings.units));
        self.skirmish.set_item(4, format!("Enemies: {}", self.skirmish_settings.enemies));
        self.skirmish.set_item(5, format!("Unit type: {}", self.skirmish_settings.player_unit_type));
        self.skirmish.set_item(6, format!("Enemy type: {}", self.skirmish_settings.ai_unit_type));
    }

    /// Handle key presses, returning an optional callback
    pub fn handle_key(&mut self, ctx: &mut Context, key: Keycode) -> Option<Callback> {
        match key {
            // Rotate the selections up
            Keycode::Up => match self.submenu {
                Selected::Main => self.main.rotate_up(),
                Selected::Skirmish => self.skirmish.rotate_up()
            },
            // Rotate the selections down
            Keycode::Down => match self.submenu {
                Selected::Main => self.main.rotate_down(),
                Selected::Skirmish => self.skirmish.rotate_down()
            },
            // Perform actions on the selection 
            Keycode::Return => match self.submenu {
                Selected::Main => match self.main.selection {
                    0 => self.submenu = Selected::Skirmish,
                    1 => ctx.quit(),
                    _ => {}
                },
                Selected::Skirmish => match self.skirmish.selection {
                    0 => return Some(Callback::Play),
                    7 => self.submenu = Selected::Main,
                    _ => {}
                }
            },
            // Lower the cols/rows values
            Keycode::Left => match self.submenu {
                Selected::Skirmish => match self.skirmish.selection {
                    1 => { self.skirmish_settings.cols -= SIZE_CHANGE; self.refresh_skirmish(); },
                    2 => { self.skirmish_settings.rows -= SIZE_CHANGE; self.refresh_skirmish(); },
                    3 => { self.skirmish_settings.units -= 1; self.refresh_skirmish(); },
                    4 => { self.skirmish_settings.enemies -= 1; self.refresh_skirmish(); },
                    5 => { self.skirmish_settings.change_player_unit_type(); self.refresh_skirmish(); },
                    6 => { self.skirmish_settings.change_ai_unit_type(); self.refresh_skirmish(); },
                    _ => {}
                },
                _ => {}
            },
            // Raise the cols/rows values
            Keycode::Right => match self.submenu {
                Selected::Skirmish => match self.skirmish.selection {
                    1 => { self.skirmish_settings.cols += SIZE_CHANGE; self.refresh_skirmish(); },
                    2 => { self.skirmish_settings.rows += SIZE_CHANGE; self.refresh_skirmish(); },
                    3 => { self.skirmish_settings.units += 1; self.refresh_skirmish(); },
                    4 => { self.skirmish_settings.enemies += 1; self.refresh_skirmish(); },
                    5 => { self.skirmish_settings.change_player_unit_type(); self.refresh_skirmish(); },
                    6 => { self.skirmish_settings.change_ai_unit_type(); self.refresh_skirmish(); },
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        }

        None
    }
}