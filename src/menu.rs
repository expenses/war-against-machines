// The main menu of the game

use context::Context;
use resources::Image;
use settings::*;
use ui::*;

use glutin::*;

use std::fs::*;

const MAP_SIZE_CHANGE: usize = 5;
const TITLE_TOP_OFFSET: f32 = 50.0;
const VOLUME_CHANGE: u8 = 5;
const LIGHT_LEVEL_CHANGE: u8 = 2;

macro_rules! list {
    ($x:expr, $y:expr, $($widget: expr),*) => (
        List::new($x, $y, vec![$($widget,)*])
    )
}

#[derive(PartialEq)]
enum Submenu {
    Main,
    Skirmish,
    SkirmishSettings,
    SkirmishSaves,
    Settings,
}

impl Submenu {
    fn index(&self) -> usize {
        match *self {
            Submenu::Main => 0,
            Submenu::Skirmish => 1,
            Submenu::SkirmishSettings => 2,
            Submenu::Settings => 3,
            Submenu::SkirmishSaves => 4,
        }
    }
}

// Callbacks that can be returned from key presses
pub enum MenuCallback<'a> {
    NewSkirmish(&'a SkirmishSettings),
    Resume,
    Quit,
}

// The main menu struct
pub struct MainMenu {
    settings: SkirmishSettings,
    submenu: Submenu,
    submenus: [List; 5],
}

impl MainMenu {
    // Create a new Menu
    pub fn new(ctx: &mut Context) -> MainMenu {
        let mut menu = Self {
            submenu: Submenu::Main,
            submenus: [
                list!(
                    0.0,
                    50.0,
                    ListItem::new("Skirmish"),
                    ListItem::new("Settings"),
                    ListItem::new("Quit")
                ),
                list!(
                    0.0,
                    50.0,
                    ListItem::new("Back"),
                    ListItem::new("Resume Skirmish"),
                    ListItem::new("New Skirmish"),
                    ListItem::new("<Game Type>"),
                    ListItem::new("Load Save"),
                    ListItem::new("Settings"),
                    ListItem::new("<Address>")
                ),
                list!(
                    0.0,
                    50.0,
                    ListItem::new("Back"),
                    ListItem::new("<Width>"),
                    ListItem::new("<Height>"),
                    ListItem::new("<Player A Units>"),
                    ListItem::new("<Player B Units>"),
                    ListItem::new("<Player A Unit Type>"),
                    ListItem::new("<Player A Unit Type>"),
                    ListItem::new("<Light Level>")
                ),
                list!(
                    0.0,
                    50.0,
                    ListItem::new("Back"),
                    ListItem::new("<Volume>"),
                    ListItem::new("Reset")
                ),
                List::new(0.0, 50.0, Vec::new()),
            ],
            settings: SkirmishSettings::default(),
        };

        menu.refresh_skirmish(false);
        menu.refresh_skirmish_settings();
        menu.refresh_settings(ctx);
        menu.refresh_skirmish_saves(ctx);

        menu
    }

    pub fn reset_submenu(&mut self) {
        self.submenu = Submenu::Main;
    }

    fn refresh_skirmish_settings(&mut self) {
        self.settings.clamp();

        let skirmish_settings = &mut self.submenus[Submenu::SkirmishSettings.index()];
        skirmish_settings[1].set_text(&format!("Width: {}", self.settings.width));
        skirmish_settings[2].set_text(&format!("Height: {}", self.settings.height));
        skirmish_settings[3].set_text(&format!("Player A Units: {}", self.settings.player_a_units));
        skirmish_settings[4].set_text(&format!("Player B Units: {}", self.settings.player_b_units));
        skirmish_settings[5].set_text(&format!(
            "Player A Unit Type: {}",
            self.settings.player_a_unit_type
        ));
        skirmish_settings[6].set_text(&format!(
            "Player B Unit Type: {}",
            self.settings.player_b_unit_type
        ));
        skirmish_settings[7].set_text(&format!(
            "Light Level: {}",
            f32::from(self.settings.light) / 10.0
        ));
    }

    fn refresh_skirmish(&mut self, game_in_progress: bool) {
        let skirmish = &mut self.submenus[Submenu::Skirmish.index()];
        skirmish[1].set_selectable(game_in_progress);
        skirmish[3].set_text(&format!("Game Type: {}", self.settings.game_type.as_str()));
        skirmish[4].set_selectable(self.settings.game_type != GameType::Connect);
        skirmish[5].set_selectable(self.settings.game_type != GameType::Connect);
        skirmish[6]
            .set_text(&format!("Address: {}", self.settings.address))
            .set_selectable(self.settings.game_type != GameType::Local);
    }

    fn refresh_settings(&mut self, ctx: &mut Context) {
        ctx.settings.clamp();

        let settings = &mut self.submenus[Submenu::Settings.index()];
        settings[1].set_text(&format!("Volume: {}", ctx.settings.volume));
    }

    fn refresh_skirmish_saves(&mut self, ctx: &Context) {
        let submenu = &mut self.submenus[Submenu::SkirmishSaves.index()];

        submenu.clear_entries();

        submenu.push_entry(ListItem::new("Back"));
        submenu.push_entry(ListItem::new("Refresh"));
        submenu.push_entry(ListItem::new("None"));

        read_dir(&ctx.settings.savegames)
            .into_iter()
            .flat_map(|dir| dir)
            .filter_map(|entry| {
                entry
                    .ok()
                    .and_then(|entry| entry.file_name().into_string().ok())
            })
            .filter(|entry| !entry.starts_with('.'))
            .map(|entry| ListItem::new(&entry))
            .for_each(|entry| submenu.push_entry(entry));
    }

    pub fn update(&mut self, ctx: &mut Context, game_in_progress: bool) -> Option<MenuCallback> {
        let enter_pressed = ctx.gui.key_pressed(VirtualKeyCode::Return);

        if ctx.gui.key_pressed(VirtualKeyCode::Up) {
            self.submenus[self.submenu.index()].rotate_up();
        }

        if ctx.gui.key_pressed(VirtualKeyCode::Down) {
            self.submenus[self.submenu.index()].rotate_down();
        }

        let movement_left = ctx.gui.key_pressed(VirtualKeyCode::Left);
        let movement_right = ctx.gui.key_pressed(VirtualKeyCode::Right);

        let index = self.submenus[self.submenu.index()].index();

        match self.submenu {
            Submenu::Main => match index {
                0 if enter_pressed => self.submenu = Submenu::Skirmish,
                1 if enter_pressed => self.submenu = Submenu::Settings,
                2 if enter_pressed => return Some(MenuCallback::Quit),
                _ => {}
            },
            Submenu::Skirmish => return self.update_skirmish(ctx, game_in_progress),
            Submenu::SkirmishSettings => self.update_skirmish_settings(ctx),
            Submenu::Settings => {
                match index {
                    0 if enter_pressed => self.submenu = Submenu::Main,
                    1 if movement_left => ctx.settings.volume -= VOLUME_CHANGE,
                    1 if movement_right => ctx.settings.volume += VOLUME_CHANGE,
                    2 if enter_pressed => ctx.settings.reset(),
                    _ => {}
                }

                if enter_pressed || movement_left || movement_right {
                    self.refresh_settings(ctx);
                }
            }
            Submenu::SkirmishSaves => match index {
                0 if enter_pressed => self.submenu = Submenu::Skirmish,
                1 if enter_pressed => self.refresh_skirmish_saves(ctx),
                2 if enter_pressed => self.settings.save_game = None,
                _ if enter_pressed => {
                    let save_game = self.submenus[self.submenu.index()].get().text();
                    self.settings.set_savegame(save_game, &ctx.settings);
                }
                _ => {}
            },
        }

        None
    }

    pub fn update_skirmish(
        &mut self,
        ctx: &Context,
        game_in_progress: bool,
    ) -> Option<MenuCallback> {
        let enter_pressed = ctx.gui.key_pressed(VirtualKeyCode::Return);
        let movement_left = ctx.gui.key_pressed(VirtualKeyCode::Left);
        let movement_right = ctx.gui.key_pressed(VirtualKeyCode::Right);
        let back_pressed = ctx.gui.key_pressed(VirtualKeyCode::Back);

        let index = &self.submenus[self.submenu.index()].index();

        let mut key_input = false;

        match index {
            0 if enter_pressed => self.submenu = Submenu::Main,
            1 if enter_pressed => return Some(MenuCallback::Resume),
            2 if enter_pressed => return Some(MenuCallback::NewSkirmish(&self.settings)),
            3 if movement_left => self.settings.game_type.rotate_left(),
            3 if movement_right => self.settings.game_type.rotate_right(),
            4 if enter_pressed => self.submenu = Submenu::SkirmishSaves,
            5 if enter_pressed => self.submenu = Submenu::SkirmishSettings,
            6 if back_pressed => {
                self.settings.address.pop();
            }
            6 => {
                key_input = ctx.gui.key_input(&mut self.settings.address, |c| {
                    c.is_ascii_digit() || c == '.' || c == ':'
                });
            }
            _ => {}
        }

        // This could be cleaner
        if movement_left || movement_right || back_pressed || key_input {
            self.refresh_skirmish(game_in_progress);
        }

        None
    }

    pub fn update_skirmish_settings(&mut self, ctx: &Context) {
        let enter_pressed = ctx.gui.key_pressed(VirtualKeyCode::Return);
        let movement_left = ctx.gui.key_pressed(VirtualKeyCode::Left);
        let movement_right = ctx.gui.key_pressed(VirtualKeyCode::Right);
        let index = &self.submenus[self.submenu.index()].index();

        match index {
            0 if enter_pressed => self.submenu = Submenu::Skirmish,

            1 if movement_left => self.settings.width -= MAP_SIZE_CHANGE,
            1 if movement_right => self.settings.width += MAP_SIZE_CHANGE,
            2 if movement_left => self.settings.height -= MAP_SIZE_CHANGE,
            2 if movement_right => self.settings.height += MAP_SIZE_CHANGE,

            3 if movement_left => self.settings.player_a_units -= 1,
            3 if movement_right => self.settings.player_a_units += 1,
            4 if movement_left => self.settings.player_b_units -= 1,
            4 if movement_right => self.settings.player_b_units += 1,

            5 if movement_left || movement_right => self.settings.change_player_a_unit_type(),
            6 if movement_left || movement_right => self.settings.change_player_b_unit_type(),

            7 if movement_left => {
                self.settings.light = self.settings.light.saturating_sub(LIGHT_LEVEL_CHANGE)
            }
            7 if movement_right => self.settings.light += LIGHT_LEVEL_CHANGE,
            _ => {}
        }

        if movement_left || movement_right {
            self.refresh_skirmish_settings();
        }
    }

    pub fn render(&self, ctx: &mut Context) {
        // Draw the title
        let dest = [ctx.width / 2.0, TITLE_TOP_OFFSET];
        ctx.render(Image::Title, dest, 1.0);

        self.submenus[self.submenu.index()].render(ctx);
    }
}
