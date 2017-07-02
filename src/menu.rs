// The main menu of the game

use sdl2::keyboard::Keycode;
use colours::WHITE;

use Resources;
use context::Context;
use settings::{Settings, SkirmishSettings};

const MAP_SIZE_CHANGE: usize = 5;
const TITLE_TOP_OFFSET: f32 = 50.0;
const WINDOW_SIZE_CHANGE: u32 = 10;
const VOLUME_CHANGE: i32 = 4;

// Callbacks that can be returned from key presses
pub enum Callback {
    NewSkirmish,
    LoadSkirmish
}

// A submenu inside the main menu
struct Submenu {
    selection: usize,
    list: Vec<String>
}

impl Submenu {
    // Create a new submenu
    fn new(list: Vec<String>) -> Submenu {
        Submenu {
            selection: 0,
            list
        }
    }

    // Draw the items in the submenu
    fn draw(&self, ctx: &mut Context, resources: &Resources) {
        for (i, item) in self.list.iter().enumerate() {
            let mut string = item.clone();

            // If the index is the same as the selection index, push a '>' to indicate that the option is selected
            if i == self.selection { string.insert_str(0, "> "); }

            // Render the string
            let rendered = resources.render("main", &string, WHITE);

            // Get the center of the rendered string
            let center = (ctx.get_width() - rendered.query().width) as f32 / 2.0;

            // Draw the string
            ctx.draw(&rendered, center, 150.0 + i as f32 * 20.0, 1.0);
        }
    }

    // Change an item in the list
    fn set_item(&mut self, i: usize, string: String) {
        self.list[i] = string;
    }

    // Rotate the selection up
    fn rotate_up(&mut self) {
        self.selection = match self.selection {
            0 => self.list.len() - 1,
            _ => self.selection - 1
        }
    }

    // Rotate the selection down
    fn rotate_down(&mut self) {
        self.selection = (self.selection + 1) % self.list.len();
    }
}

const MAIN: usize = 0;
const SKIRMISH: usize = 1;
const SETTINGS: usize = 2;

// The main menu struct
pub struct Menu {
    pub skirmish_settings: SkirmishSettings,
    pub settings: Settings,
    submenu: usize,
    submenus: [Submenu; 3]
}

impl Menu {
    // Create a new Menu
    pub fn new(settings: Settings) -> Menu {
        let skirmish_settings = SkirmishSettings::default();

        Menu {
            submenu: MAIN,
            submenus: [
                Submenu::new(vec![
                    "Skirmish".into(),
                    "Settings".into(),
                    "Quit".into(),
                ]),
                Submenu::new(vec![
                    "New".into(),
                    "Load Skirmish".into(),
                    format!("Cols: {}", skirmish_settings.cols),
                    format!("Rows: {}", skirmish_settings.rows),
                    format!("Player units: {}", skirmish_settings.player_units),
                    format!("AI units: {}", skirmish_settings.ai_units),
                    format!("Player unit type: {}", skirmish_settings.player_unit_type),
                    format!("AI unit type: {}", skirmish_settings.ai_unit_type),
                    "Back".into()
                ]),
                Submenu::new(vec![
                    "Back".into(),
                    format!("Volume: {}", settings.volume),
                    format!("Width: {}", settings.width),
                    format!("Height: {}", settings.height),
                    "From screen size".into(),
                    format!("Fullscreen: {}", settings.fullscreen),
                    "Reset".into(),
                    "Save".into()
                ])
            ],
            skirmish_settings, settings
        }
    }

    // Draw the menu
    pub fn draw(&self, ctx: &mut Context, resources: &Resources) {
        // Draw the title
        let title = resources.image("title");
        let center = (ctx.get_width() - title.query().width) as f32 / 2.0;
        ctx.draw(title, center, TITLE_TOP_OFFSET, 1.0);

        // Draw the selected submenu
        self.submenus[self.submenu].draw(ctx, resources);
    }

    // Refresh the skirmish settings
    fn refresh_skirmish(&mut self) {
        let skirmish = &mut self.submenus[SKIRMISH];
        
        self.skirmish_settings.clamp();
        skirmish.set_item(2, format!("Cols: {}", self.skirmish_settings.cols));
        skirmish.set_item(3, format!("Rows: {}", self.skirmish_settings.rows));
        skirmish.set_item(4, format!("Player units: {}", self.skirmish_settings.player_units));
        skirmish.set_item(5, format!("AI units: {}", self.skirmish_settings.ai_units));
        skirmish.set_item(6, format!("Player unit type: {}", self.skirmish_settings.player_unit_type));
        skirmish.set_item(7, format!("AI unit type: {}", self.skirmish_settings.ai_unit_type));
    }

    fn refresh_settings(&mut self) {
        let settings = &mut self.submenus[SETTINGS];

        self.settings.clamp();
        settings.set_item(1, format!("Volume: {}", self.settings.volume));
        settings.set_item(2, format!("Width: {}", self.settings.width));
        settings.set_item(3, format!("Height: {}", self.settings.height));
        settings.set_item(5, format!("Fullscreen: {}", self.settings.fullscreen));
    }

    // Handle key presses, returning an optional callback
    pub fn handle_key(&mut self, ctx: &mut Context, key: Keycode) -> Option<Callback> {
        match key {
            // Rotate the selections up
            Keycode::Up | Keycode::W => self.submenus[self.submenu].rotate_up(),
            // Rotate the selections down
            Keycode::Down | Keycode::S => self.submenus[self.submenu].rotate_down(),
            // Perform actions on the selection 
            Keycode::Return => match self.submenu {
                MAIN => match self.submenus[MAIN].selection {
                    0 => self.submenu = SKIRMISH,
                    1 => self.submenu = SETTINGS,
                    2 => ctx.quit(),
                    _ => {}
                },
                SKIRMISH => match self.submenus[SKIRMISH].selection {
                    0 => return Some(Callback::NewSkirmish),
                    1 => return Some(Callback::LoadSkirmish),
                    7 => self.submenu = MAIN,
                    _ => {}
                },
                SETTINGS => match self.submenus[SETTINGS].selection {
                    0 => self.submenu = MAIN,
                    4 => {
                        self.settings.width = ctx.get_width();
                        self.settings.height = ctx.get_height();
                        self.refresh_settings();
                    },
                    6 => {
                        self.settings = Settings::default();
                        ctx.set(&self.settings);
                        self.refresh_settings();
                    },
                    7 => {
                        ctx.set(&self.settings);
                        self.settings.save();
                    },
                    _ => {}
                },
                _ => {}
            },
            // Lower the skimish settings
            Keycode::Left | Keycode::A => match self.submenu {
                SKIRMISH => {
                    match self.submenus[SKIRMISH].selection {
                        2 => self.skirmish_settings.cols -= MAP_SIZE_CHANGE,
                        3 => self.skirmish_settings.rows -= MAP_SIZE_CHANGE,
                        4 => self.skirmish_settings.player_units -= 1,
                        5 => self.skirmish_settings.ai_units -= 1,
                        6 => self.skirmish_settings.change_player_unit_type(),
                        7 => self.skirmish_settings.change_ai_unit_type(),
                        _ => {}
                    }
                    self.refresh_skirmish();
                }
                SETTINGS => {
                    match self.submenus[SETTINGS].selection {
                        1 => self.settings.volume -= VOLUME_CHANGE,
                        2 => self.settings.width -= WINDOW_SIZE_CHANGE,
                        3 => self.settings.height -= WINDOW_SIZE_CHANGE,
                        5 => self.settings.toggle_fullscreen(),
                        _ => {}
                    }
                    self.refresh_settings();
                }
                _ => {}
            },
            // Raise the skimish settings
            Keycode::Right | Keycode::D => match self.submenu {
                SKIRMISH => {
                    match self.submenus[SKIRMISH].selection {
                        2 => self.skirmish_settings.cols += MAP_SIZE_CHANGE,
                        3 => self.skirmish_settings.rows += MAP_SIZE_CHANGE,
                        4 => self.skirmish_settings.player_units += 1,
                        5 => self.skirmish_settings.ai_units += 1,
                        6 => self.skirmish_settings.change_player_unit_type(),
                        7 => self.skirmish_settings.change_ai_unit_type(),
                        _ => {}
                    }
                    self.refresh_skirmish();
                },
                SETTINGS => {
                    match self.submenus[SETTINGS].selection {
                        1 => self.settings.volume += VOLUME_CHANGE,
                        2 => self.settings.width += WINDOW_SIZE_CHANGE,
                        3 => self.settings.height += WINDOW_SIZE_CHANGE,
                        5 => self.settings.toggle_fullscreen(),
                        _ => {}
                    }
                    self.refresh_settings();
                }
                _ => {}
            },
            Keycode::Escape => ctx.quit(),
            _ => {}
        }

        None
    }
}