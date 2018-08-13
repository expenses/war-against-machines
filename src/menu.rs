// The main menu of the game

use *;

use std::fs::read_dir;

use context::Context;
use resources::Image;
use settings::{Settings, SkirmishSettings};
use ui::{Menu, MenuItem, Vertical, Horizontal};

// Look into alternative ui systems, potentially a lib, because this is pretty messy

const MAP_SIZE_CHANGE: usize = 5;
const TITLE_TOP_OFFSET: f32 = 50.0;
const VOLUME_CHANGE: u8 = 5;
const LIGHT_LEVEL_CHANGE: f32 = 0.2;

const MAIN: usize = 0;
const SKIRMISH: usize = 1;
const MULTIPLAYER: usize = 2;
const SETTINGS: usize = 3;
const SKIRMISH_SAVES: usize = 4;

// Callbacks that can be returned from key presses
pub enum MenuCallback {
    NewSkirmish,
    LoadSkirmish(String),
    HostServer(String),
    ConnectServer(String),
    Resume,
    Quit
}

// The main menu struct
pub struct MainMenu {
    pub skirmish_settings: SkirmishSettings,
    submenu: usize,
    submenus: [Menu; 5]
}

impl MainMenu {
    // Create a new Menu
    pub fn new(settings: &mut Settings) -> MainMenu {
        let skirmish_settings = SkirmishSettings::default();

        let mut menu = MainMenu {
            submenu: MAIN,
            submenus: [
                menu!(
                    item!("Skirmish"),
                    item!("Multiplayer"),
                    item!("Settings"),
                    item!("Quit")
                ),
                menu!(
                    item!("Back"),
                    item!("Resume Skirmish", false),
                    item!("New Skirmish"),
                    item!("Load Skirmish"),
                    item!("--Settings--", false),
                    item!(),
                    item!(),
                    item!(),
                    item!(),
                    item!(),
                    item!(),
                    item!()
                ),
                menu!(
                    item!("Back"),
                    item!("Host"),
                    item!("Connect"),
                    item!("--Address--", false),
                    item!("0.0.0.0:6666")
                ),
                menu!(
                    item!("Back"),
                    item!(),
                    item!(),
                    item!("Reset")
                ),
                menu!()
            ],
            skirmish_settings
        };

        menu.refresh_skirmish();
        menu.refresh_settings(settings);

        menu
    }

    // Draw the menu
    pub fn render(&self, ctx: &mut Context) {
        // Draw the title
        let dest = [0.0, ctx.height / 2.0 - TITLE_TOP_OFFSET];
        ctx.render(Image::Title, dest, 1.0);

        // Draw the selected submenu
        self.submenus[self.submenu].render(ctx);
    }

    // Refresh the skirmish submenu
    fn refresh_skirmish(&mut self) {
        let skirmish_submenu = &mut self.submenus[SKIRMISH];
        
        self.skirmish_settings.clamp();
        skirmish_submenu.list[5]  = item!("Cols: {}", self.skirmish_settings.cols, true);
        skirmish_submenu.list[6]  = item!("Rows: {}", self.skirmish_settings.rows, true);
        skirmish_submenu.list[7]  = item!("Player units: {}", self.skirmish_settings.player_units, true);
        skirmish_submenu.list[8]  = item!("AI units: {}", self.skirmish_settings.ai_units, true);
        skirmish_submenu.list[9]  = item!("Player unit type: {}", self.skirmish_settings.player_unit_type, true);
        skirmish_submenu.list[10] = item!("AI unit type: {}", self.skirmish_settings.ai_unit_type, true);
        skirmish_submenu.list[11] = item!("Light level: {:.1}", self.skirmish_settings.light, true)
    }

    // refresh the settings submenu
    fn refresh_settings(&mut self, settings: &mut Settings) {
        let settings_submenu = &mut self.submenus[SETTINGS];

        settings.clamp();
        settings_submenu.list[1] = item!("Volume: {}", settings.volume, true);
        settings_submenu.list[2] = item!("UI Scale: {}", settings.ui_scale, true);
    }

    // refresh the saves submenu
    fn refresh_skirmish_saves(&mut self, settings: &Settings) {
        let mut files: Vec<MenuItem> = read_dir(&settings.savegames).unwrap()
            .filter_map(|entry| entry.ok().and_then(|entry| entry.file_name().into_string().ok()))
            .filter(|entry| !entry.starts_with('.'))
            .map(|entry| item!(entry))
            .collect();

        self.submenus[SKIRMISH_SAVES].list = vec![item!("Back"), item!("Refresh")];
        self.submenus[SKIRMISH_SAVES].list.append(&mut files);
    }

    pub fn refresh(&mut self, skirmish_open: bool) {
        self.submenus[SKIRMISH].set_enabled(1, skirmish_open);
        self.submenus[SKIRMISH].selection = 1;
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
                    1 => self.submenu = MULTIPLAYER,
                    2 => self.submenu = SETTINGS,
                    3 => return Some(MenuCallback::Quit),
                    _ => {}
                },
                SKIRMISH => match self.submenus[SKIRMISH].selection {
                    0 => self.submenu = MAIN,
                    1 if self.submenus[SKIRMISH].enabled(1) => return Some(MenuCallback::Resume),
                    2 => return Some(MenuCallback::NewSkirmish),
                    3 => {
                        self.submenu = SKIRMISH_SAVES;
                        self.refresh_skirmish_saves(settings);
                    }
                    _ => {}
                },
                MULTIPLAYER => match self.submenus[MULTIPLAYER].selection {
                    0 => self.submenu = MAIN,
                    1 => return Some(MenuCallback::HostServer(self.submenus[MULTIPLAYER].list[4].content())),
                    2 => return Some(MenuCallback::ConnectServer(self.submenus[MULTIPLAYER].list[4].content())),
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
                    1 => self.refresh_skirmish_saves(settings),
                    _ => return Some(MenuCallback::LoadSkirmish(self.submenus[SKIRMISH_SAVES].selected()))
                },
                _ => {}
            },
            // Lower the skimish settings
            VirtualKeyCode::Left | VirtualKeyCode::A => match self.submenu {
                SKIRMISH => {
                    match self.submenus[SKIRMISH].selection {
                        5  => self.skirmish_settings.cols -= MAP_SIZE_CHANGE,
                        6  => self.skirmish_settings.rows -= MAP_SIZE_CHANGE,
                        7  => self.skirmish_settings.player_units -= 1,
                        8  => self.skirmish_settings.ai_units -= 1,
                        9  => self.skirmish_settings.change_player_unit_type(),
                        10 => self.skirmish_settings.change_ai_unit_type(),
                        11 => self.skirmish_settings.light -= LIGHT_LEVEL_CHANGE,
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
                        5  => self.skirmish_settings.cols += MAP_SIZE_CHANGE,
                        6  => self.skirmish_settings.rows += MAP_SIZE_CHANGE,
                        7  => self.skirmish_settings.player_units += 1,
                        8  => self.skirmish_settings.ai_units += 1,
                        9  => self.skirmish_settings.change_player_unit_type(),
                        10 => self.skirmish_settings.change_ai_unit_type(),
                        11 => self.skirmish_settings.light += LIGHT_LEVEL_CHANGE,
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
            key => if self.submenu == MULTIPLAYER && self.submenus[MULTIPLAYER].selection == 4 {
                self.submenus[MULTIPLAYER].list[4].handle_key(key);
            }
        }

        None
    }
}