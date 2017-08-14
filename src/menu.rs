// The main menu of the game

use glutin::VirtualKeyCode;

use std::fs::read_dir;

use context::Context;
use resources::Image;
use settings::{Settings, SkirmishSettings};
use ui::{Menu, Vertical, Horizontal};

const MAP_SIZE_CHANGE: usize = 5;
const TITLE_TOP_OFFSET: f32 = 50.0;
const VOLUME_CHANGE: u8 = 5;

macro_rules! item {
    ($item: expr) => (
        ($item.to_string(), true)
    );
    ($item: expr, $boolean: expr) => (
        ($item.to_string(), $boolean)
    );
    ($item: expr, $thing: expr, $boolean: expr) => (
        (format!($item, $thing), $boolean)
    )
}

macro_rules! menu {
    ($($item: expr),*) => (
        Menu::new(0.0, 0.0, Vertical::Middle, Horizontal::Middle, true, true, vec![$($item,)*])
    )
}

// Callbacks that can be returned from key presses
pub enum MenuCallback {
    NewSkirmish,
    LoadSkirmish(String),
    Resume,
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
                    item!("Skirmish"),
                    item!("Settings"),
                    item!("Quit")
                ),
                menu!(
                    item!("Back"),
                    item!("Resume Skirmish", false),
                    item!("New Skirmish"),
                    item!("Load Skirmish"),
                    item!("--Settings--", false),
                    item!("Cols: {}", skirmish_settings.cols, true),
                    item!("Rows: {}", skirmish_settings.rows, true),
                    item!("Player units: {}", skirmish_settings.player_units, true),
                    item!("AI units: {}", skirmish_settings.ai_units, true),
                    item!("Player unit type: {}", skirmish_settings.player_unit_type, true),
                    item!("AI unit type: {}", skirmish_settings.ai_unit_type, true)
                ),
                menu!(
                    item!("Back"),
                    item!("Volume: {}", settings.volume, true),
                    item!("UI Scale: {}", settings.ui_scale, true),
                    item!("Reset")
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
        let skirmish_submenu = &mut self.submenus[SKIRMISH];
        
        self.skirmish_settings.clamp();
        skirmish_submenu.list[5]  = item!("Cols: {}", self.skirmish_settings.cols, true);
        skirmish_submenu.list[6]  = item!("Rows: {}", self.skirmish_settings.rows, true);
        skirmish_submenu.list[7]  = item!("Player units: {}", self.skirmish_settings.player_units, true);
        skirmish_submenu.list[8]  = item!("AI units: {}", self.skirmish_settings.ai_units, true);
        skirmish_submenu.list[9]  = item!("Player unit type: {}", self.skirmish_settings.player_unit_type, true);
        skirmish_submenu.list[10] = item!("AI unit type: {}", self.skirmish_settings.ai_unit_type, true);
    }

    // refresh the settings submenu
    fn refresh_settings(&mut self, settings: &mut Settings) {
        let settings_submenu = &mut self.submenus[SETTINGS];

        settings.clamp();
        settings_submenu.list[1] = item!("Volume: {}", settings.volume, true);
        settings_submenu.list[2] = item!("UI Scale: {}", settings.ui_scale, true);
    }

    // refresh the saves submenu
    fn refresh_skirmish_saves(&mut self) {
        let mut files: Vec<(String, bool)> = read_dir("savegames/skirmishes").unwrap()
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
                    1 => self.submenu = SETTINGS,
                    2 => return Some(MenuCallback::Quit),
                    _ => {}
                },
                SKIRMISH => match self.submenus[SKIRMISH].selection {
                    0 => self.submenu = MAIN,
                    1 if self.submenus[SKIRMISH].enabled(1) => return Some(MenuCallback::Resume),
                    2 => return Some(MenuCallback::NewSkirmish),
                    3 => {
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
                        5  => self.skirmish_settings.cols -= MAP_SIZE_CHANGE,
                        6  => self.skirmish_settings.rows -= MAP_SIZE_CHANGE,
                        7  => self.skirmish_settings.player_units -= 1,
                        8  => self.skirmish_settings.ai_units -= 1,
                        9  => self.skirmish_settings.change_player_unit_type(),
                        10 => self.skirmish_settings.change_ai_unit_type(),
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
            _ => {}
        }

        None
    }
}

#[test]
fn test_menu_lengths() {
    let main_menu = MainMenu::new(&Settings::default());

    assert_eq!(main_menu.submenus.len(), 4);
    assert_eq!(main_menu.submenus[0].len(), 3);
    assert_eq!(main_menu.submenus[1].len(), 11);
    assert_eq!(main_menu.submenus[2].len(), 4);
    assert_eq!(main_menu.submenus[3].len(), 0);
}