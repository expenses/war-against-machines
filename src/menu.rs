// The main menu of the game

use glutin::VirtualKeyCode;

use std::fs::read_dir;

use colours::WHITE;
use context::Context;
use resources::Image;
use settings::{Settings, SkirmishSettings};

const MAP_SIZE_CHANGE: usize = 5;
const TITLE_TOP_OFFSET: f32 = 50.0;
const TOP_ITEM_OFFSET: f32 = 150.0;
const VOLUME_CHANGE: u8 = 5;

// Callbacks that can be returned from key presses
pub enum MenuCallback {
    NewSkirmish,
    LoadSkirmish(String),
    Quit
}

// A submenu inside the main menu
struct Submenu {
    selection: usize,
    list: Vec<String>,
}

impl Submenu {
    // Create a new submenu
    fn new(list: Vec<String>) -> Submenu {
        Submenu {
            selection: 0,
            list,
        }
    }

    // Draw the items in the submenu
    fn render(&self, ctx: &mut Context) {
        for (i, item) in self.list.iter().enumerate() {
            let mut string = item.clone();

            // If the index is the same as the selection index, push a '>' to indicate that the option is selected
            if i == self.selection { string.insert_str(0, "> "); }

            // Render the string
            let y = ctx.height / 2.0 - TOP_ITEM_OFFSET - i as f32 * 20.0;

            ctx.render_text(&string, 0.0, y, WHITE);
        }
    }

    fn selected(&self) -> String {
        self.list[self.selection].clone()
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
const SKIRMISH_SAVES: usize = 3;

// The main menu struct
pub struct Menu {
    pub skirmish_settings: SkirmishSettings,
    pub settings: Settings,
    submenu: usize,
    submenus: [Submenu; 4]
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
                    "Back".into(),
                    "New Skirmish".into(),
                    "Load Skirmish".into(),
                    "--Settings--".into(),
                    format!("Cols: {}", skirmish_settings.cols),
                    format!("Rows: {}", skirmish_settings.rows),
                    format!("Player units: {}", skirmish_settings.player_units),
                    format!("AI units: {}", skirmish_settings.ai_units),
                    format!("Player unit type: {}", skirmish_settings.player_unit_type),
                    format!("AI unit type: {}", skirmish_settings.ai_unit_type),
                ]),
                Submenu::new(vec![
                    "Back".into(),
                    format!("Volume: {:.2}", settings.volume),
                    "Reset".into(),
                    "Save".into()
                ]),
                Submenu::new(Vec::new())
            ],
            skirmish_settings, settings
        }
    }

    // Draw the menu
    pub fn render(&self, ctx: &mut Context) {
        // Draw the title
        let y = ctx.height / 2.0 - TITLE_TOP_OFFSET;
        ctx.render(&Image::Title, 0.0, y, 1.0);

        // Draw the selected submenu
        self.submenus[self.submenu].render(ctx);
    }

    // Refresh the skirmish settings
    fn refresh_skirmish(&mut self) {
        let skirmish = &mut self.submenus[SKIRMISH];
        
        self.skirmish_settings.clamp();
        skirmish.set_item(4, format!("Cols: {}", self.skirmish_settings.cols));
        skirmish.set_item(5, format!("Rows: {}", self.skirmish_settings.rows));
        skirmish.set_item(6, format!("Player units: {}", self.skirmish_settings.player_units));
        skirmish.set_item(7, format!("AI units: {}", self.skirmish_settings.ai_units));
        skirmish.set_item(8, format!("Player unit type: {}", self.skirmish_settings.player_unit_type));
        skirmish.set_item(9, format!("AI unit type: {}", self.skirmish_settings.ai_unit_type));
    }

    fn refresh_settings(&mut self) {
        let settings = &mut self.submenus[SETTINGS];

        self.settings.clamp();
        settings.set_item(1, format!("Volume: {:.2}", self.settings.volume));
    }

    fn refresh_skirmish_saves(&mut self) {
        let mut files: Vec<String> = read_dir("savegames/skirmishes").unwrap()
            .filter_map(|entry| entry.ok().and_then(|entry| entry.file_name().into_string().ok()))
            .filter(|entry| !entry.starts_with('.'))
            .collect();

        self.submenus[SKIRMISH_SAVES].list = vec![
            "Back".into(),
            "Refresh".into()
        ];

        self.submenus[SKIRMISH_SAVES].list.append(&mut files);
    }

    // Handle key presses, returning an optional callback
    pub fn handle_key(&mut self, key: VirtualKeyCode, ctx: &mut Context) -> Option<MenuCallback> {
        match key {
            // Rotate the selections up
            VirtualKeyCode::Up | VirtualKeyCode::W => self.submenus[self.submenu].rotate_up(),
            // Rotate the selections down
            VirtualKeyCode::Down | VirtualKeyCode::S => self.submenus[self.submenu].rotate_down(),
            // Perform actions on the selection 
            VirtualKeyCode::Return => match self.submenu {
                MAIN => match self.submenus[MAIN].selection {
                    0 => self.submenu = SKIRMISH,
                    1 => self.submenu = SETTINGS,
                    2 => return Some(MenuCallback::Quit),
                    _ => {}
                },
                SKIRMISH => match self.submenus[SKIRMISH].selection {
                    0 => self.submenu = MAIN,
                    1 => return Some(MenuCallback::NewSkirmish),
                    2 => {
                        self.submenu = SKIRMISH_SAVES;
                        self.refresh_skirmish_saves();
                    }
                    _ => {}
                },
                SETTINGS => match self.submenus[SETTINGS].selection {
                    0 => self.submenu = MAIN,
                    2 => {
                        self.settings = Settings::default();
                        ctx.set(&self.settings);
                        self.refresh_settings();
                    },
                    3 => {
                        ctx.set(&self.settings);
                        self.settings.save();
                    },
                    _ => {}
                },
                SKIRMISH_SAVES => match self.submenus[SKIRMISH_SAVES].selection {
                    0 => self.submenu = SKIRMISH,
                    1 => self.refresh_skirmish_saves(),
                    _ => return Some(MenuCallback::LoadSkirmish(self.submenus[SKIRMISH_SAVES].selected()))
                },
                _ => {}
            },
            // Lower the skimish settings
            VirtualKeyCode::Left | VirtualKeyCode::A => match self.submenu {
                SKIRMISH => {
                    match self.submenus[SKIRMISH].selection {
                        4 => self.skirmish_settings.cols -= MAP_SIZE_CHANGE,
                        5 => self.skirmish_settings.rows -= MAP_SIZE_CHANGE,
                        6 => self.skirmish_settings.player_units -= 1,
                        7 => self.skirmish_settings.ai_units -= 1,
                        8 => self.skirmish_settings.change_player_unit_type(),
                        9 => self.skirmish_settings.change_ai_unit_type(),
                        _ => {}
                    }
                    self.refresh_skirmish();
                }
                SETTINGS => {
                    match self.submenus[SETTINGS].selection {
                        1 => if self.settings.volume > 0 {
                            self.settings.volume -= VOLUME_CHANGE;
                        },
                        _ => {}
                    }
                    self.refresh_settings();
                }
                _ => {}
            },
            // Raise the skimish settings
            VirtualKeyCode::Right | VirtualKeyCode::D => match self.submenu {
                SKIRMISH => {
                    match self.submenus[SKIRMISH].selection {
                        4 => self.skirmish_settings.cols += MAP_SIZE_CHANGE,
                        5 => self.skirmish_settings.rows += MAP_SIZE_CHANGE,
                        6 => self.skirmish_settings.player_units += 1,
                        7 => self.skirmish_settings.ai_units += 1,
                        8 => self.skirmish_settings.change_player_unit_type(),
                        9 => self.skirmish_settings.change_ai_unit_type(),
                        _ => {}
                    }
                    self.refresh_skirmish();
                },
                SETTINGS => {
                    match self.submenus[SETTINGS].selection {
                        1 => self.settings.volume += VOLUME_CHANGE,
                        _ => {}
                    }
                    self.refresh_settings();
                }
                _ => {}
            },
            VirtualKeyCode::Escape => return Some(MenuCallback::Quit),
            _ => {}
        }

        None
    }
}