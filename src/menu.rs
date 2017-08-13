// The main menu of the game

use glutin::VirtualKeyCode;

use std::fs::read_dir;

use context::Context;
use resources::Image;
use settings::{Settings, SkirmishSettings};
use ui::{Menu, Vertical, Horizontal};

const MAP_SIZE_CHANGE: usize = 5;
const TITLE_TOP_OFFSET: f32 = 50.0;
const TOP_ITEM_OFFSET: f32 = 150.0;
const VOLUME_CHANGE: u8 = 5;

macro_rules! menu {
    ($($item: expr),*) => (
        Menu::new(0.0, TOP_ITEM_OFFSET, Vertical::Middle, Horizontal::Top, true, true, vec![$($item,)*])
    )
}

// Callbacks that can be returned from key presses
pub enum MenuCallback {
    NewSkirmish,
    LoadSkirmish(String),
    Quit
}

const MAIN: usize = 0;
const SKIRMISH: usize = 1;
const SETTINGS: usize = 2;
const SKIRMISH_SAVES: usize = 3;

// The main menu struct
pub struct MainMenu {
    pub skirmish_settings: SkirmishSettings,
    submenu: usize,
    submenus: [Menu; 4]
}

impl MainMenu {
    // Create a new Menu
    pub fn new(settings: &Settings) -> MainMenu {
        let skirmish_settings = SkirmishSettings::default();

        MainMenu {
            submenu: MAIN,
            submenus: [
                menu!(
                    "Skirmish".into(),
                    "Settings".into(),
                    "Quit".into()
                ),
                menu!(
                    "Back".into(),
                    "New Skirmish".into(),
                    "Load Skirmish".into(),
                    "--Settings--".into(),
                    format!("Cols: {}", skirmish_settings.cols),
                    format!("Rows: {}", skirmish_settings.rows),
                    format!("Player units: {}", skirmish_settings.player_units),
                    format!("AI units: {}", skirmish_settings.ai_units),
                    format!("Player unit type: {}", skirmish_settings.player_unit_type),
                    format!("AI unit type: {}", skirmish_settings.ai_unit_type)
                ),
                menu!(
                    "Back".into(),
                    format!("Volume: {}", settings.volume),
                    format!("UI Scale: {}", settings.ui_scale),
                    "Reset".into()
                ),
                menu!()
            ],
            skirmish_settings
        }
    }

    // Draw the menu
    pub fn render(&self, ctx: &mut Context) {
        // Draw the title
        let dest = [0.0, ctx.height / 2.0 - TITLE_TOP_OFFSET];
        ctx.render(&Image::Title, dest, 1.0);

        // Draw the selected submenu
        self.submenus[self.submenu].render(ctx);
    }

    // Refresh the skirmish submenu
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

    // refresh the settings submenu
    fn refresh_settings(&mut self, settings: &mut Settings) {
        let settings_submenu = &mut self.submenus[SETTINGS];

        settings.clamp();
        settings_submenu.set_item(1, format!("Volume: {}", settings.volume));
        settings_submenu.set_item(2, format!("UI Scale: {}", settings.ui_scale));
    }

    // refresh the saves submenu
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
    pub fn handle_key(&mut self, key: VirtualKeyCode, settings: &mut Settings) -> Option<MenuCallback> {
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
                    0 => {
                        self.submenu = MAIN;
                        settings.save();
                    },
                    3 => {
                        settings.reset();
                        self.refresh_settings(settings);
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
                        1 => if settings.volume > 0 {
                            settings.volume -= VOLUME_CHANGE;
                        },
                        2 => settings.ui_scale -= 1,
                        _ => {}
                    }
                    self.refresh_settings(settings);
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
                        1 => settings.volume += VOLUME_CHANGE,
                        2 => settings.ui_scale += 1,
                        _ => {}
                    }
                    self.refresh_settings(settings);
                }
                _ => {}
            },
            VirtualKeyCode::Escape => return Some(MenuCallback::Quit),
            _ => {}
        }

        None
    }
}