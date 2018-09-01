// The main menu of the game

use context::Context;
use resources::Image;
use settings::*;

use pedot::*;
use ui::*;

use glutin::*;

use std::fs::*;

const MAP_SIZE_CHANGE: usize = 5;
const TITLE_TOP_OFFSET: f32 = 50.0;
const VOLUME_CHANGE: u8 = 5;
const LIGHT_LEVEL_CHANGE: u8 = 2;

#[derive(PartialEq)]
enum Submenu {
    Main,
    Skirmish,
    SkirmishSettings,
    SkirmishSaves,
    Settings
}

impl Submenu {
    fn index(&self) -> usize {
        match *self {
            Submenu::Main => 0,
            Submenu::Skirmish => 1,
            Submenu::SkirmishSettings => 2,
            Submenu::Settings => 3,
            Submenu::SkirmishSaves => 4
        }
    }
}

// Callbacks that can be returned from key presses
pub enum MenuCallback<'a> {
    NewSkirmish(&'a SkirmishSettings),
    Resume,
    Quit
}

// The main menu struct
pub struct MainMenu {
    settings: SkirmishSettings,
    submenu: Submenu,
    submenus: [List<ListItem>; 5]
}

// todo: making a server should also be under the skirmish submenu

impl MainMenu {
    // Create a new Menu
    pub fn new(ctx: &mut Context) -> MainMenu {
        let mut menu = Self {
            submenu: Submenu::Main,
            submenus: [
                list!(
                    ListItem::new(0.0, 50.0, "Skirmish"),
                    ListItem::new(0.0, 30.0, "Settings"),
                    ListItem::new(0.0, 10.0, "Quit")
                ),
                list!(
                    ListItem::new(0.0, 50.0, "Back"),
                    ListItem::new(0.0, 30.0, "Resume Skirmish"),
                    ListItem::new(0.0, 10.0, "New Skirmish"),
                    ListItem::new(0.0, -10.0, "Game Type:").unselectable(),
                    ListItem::new(0.0, -30.0, "<Game Type>"),
                    ListItem::new(0.0, -50.0, "Load Save"),
                    ListItem::new(0.0, -70.0, "Settings"),
                    ListItem::new(0.0, -90.0, "Address:").unselectable(),
                    ListItem::new(0.0, -110.0, "<Address>")
                ),
                list!(
                    ListItem::new(0.0, 50.0, "Back"),
                    ListItem::new(0.0, 30.0, "<Width>"),
                    ListItem::new(0.0, 10.0, "<Height>"),
                    ListItem::new(0.0, -10.0, "<Player A Units>"),
                    ListItem::new(0.0, -30.0, "<Player B Units>"),
                    ListItem::new(0.0, -50.0, "<Player A Unit Type>"),
                    ListItem::new(0.0, -70.0, "<Player A Unit Type>"),
                    ListItem::new(0.0, -90.0, "<Light Level>")
                ),
                list!(
                    ListItem::new(0.0, 50.0, "Back"),
                    ListItem::new(0.0, 30.0, "<Volume>"),
                    ListItem::new(0.0, 10.0, "<UI Scale>"),
                    ListItem::new(0.0, -10.0, "Reset")
                ),
                list!()
            ],
            settings: SkirmishSettings::default()
        };

        menu.refresh_skirmish(false);
        menu.refresh_skirmish_settings();
        menu.refresh_settings(ctx);
        menu.refresh_skirmish_saves(ctx);

        menu
    }

    fn rotate_up(&mut self) {
        self.submenus[self.submenu.index()].rotate_up();
        if !self.submenus[self.submenu.index()].get().selectable() {
            self.rotate_up();
        }
    }

    fn rotate_down(&mut self) {
        self.submenus[self.submenu.index()].rotate_down();
        if !self.submenus[self.submenu.index()].get().selectable() {
            self.rotate_down();
        }
    }

    fn refresh_skirmish_settings(&mut self) {
        let skirmish_settings = Submenu::SkirmishSettings.index();

        self.submenus[skirmish_settings][1].set_text(&format!("Width: {}", self.settings.width));
        self.submenus[skirmish_settings][2].set_text(&format!("Height: {}", self.settings.height));
        self.submenus[skirmish_settings][3].set_text(&format!("Player A Units: {}", self.settings.player_a_units));
        self.submenus[skirmish_settings][4].set_text(&format!("Player B Units: {}", self.settings.player_b_units));
        self.submenus[skirmish_settings][5].set_text(&format!("Player A Unit Type: {}", self.settings.player_a_unit_type));
        self.submenus[skirmish_settings][6].set_text(&format!("Player B Unit Type: {}", self.settings.player_b_unit_type));
        self.submenus[skirmish_settings][7].set_text(&format!("Light Level: {}", f32::from(self.settings.light) / 10.0));
    }

    fn refresh_skirmish(&mut self, game_in_progress: bool) {
        let skirmish = Submenu::Skirmish.index();
        
        self.settings.clamp();

        self.submenus[skirmish][1].set_selectable(game_in_progress);
        self.submenus[skirmish][4].set_text(self.settings.game_type.as_str());
        self.submenus[skirmish][5].set_selectable(self.settings.game_type != GameType::Connect);
        self.submenus[skirmish][6].set_selectable(self.settings.game_type != GameType::Connect);
        self.submenus[skirmish][8].set_text(&self.settings.address).set_selectable(self.settings.game_type != GameType::Local);
    }

    fn refresh_settings(&mut self, ctx: &mut Context) {
        let settings = Submenu::Settings.index();

        ctx.settings.clamp();
        self.submenus[settings][1].set_text(&format!("Volume: {}", ctx.settings.volume));
        self.submenus[settings][2].set_text(&format!("UI Scale: {}", ctx.settings.ui_scale));
    }

    fn refresh_skirmish_saves(&mut self, ctx: &Context) {
        let submenu = &mut self.submenus[Submenu::SkirmishSaves.index()];

        submenu.clear_entries();

        submenu.push_entry(ListItem::new(0.0, 50.0, "Back"));
        submenu.push_entry(ListItem::new(0.0, 30.0, "Refresh"));
        submenu.push_entry(ListItem::new(0.0, 10.0, "None"));

        let mut y = 10.0;

        read_dir(&ctx.settings.savegames).unwrap()
            .filter_map(|entry| entry.ok().and_then(|entry| entry.file_name().into_string().ok()))
            .filter(|entry| !entry.starts_with('.'))
            .map(|entry| {
                y -= 20.0;
                ListItem::new(0.0, y, &entry)
            })
            .for_each(|entry| submenu.push_entry(entry));
    }

    pub fn update(&mut self, ctx: &mut Context, game_in_progress: bool) -> Option<MenuCallback> {
        let enter_pressed = ctx.gui.key_pressed(VirtualKeyCode::Return);

        if ctx.gui.key_pressed(VirtualKeyCode::Up) {
            self.rotate_up();
        }

        if ctx.gui.key_pressed(VirtualKeyCode::Down) {
            self.rotate_down();
        }

        let movement_left = ctx.gui.key_pressed(VirtualKeyCode::Left);
        let movement_right = ctx.gui.key_pressed(VirtualKeyCode::Right);

        let index = self.submenus[self.submenu.index()].index();

        match self.submenu {
            Submenu::Main => {
                match index {
                    0 if enter_pressed => self.submenu = Submenu::Skirmish,
                    1 if enter_pressed => self.submenu = Submenu::Settings,
                    2 if enter_pressed => return Some(MenuCallback::Quit),
                    _ => {}
                }
            },
            Submenu::Skirmish => return self.update_skirmish(ctx, game_in_progress),
            Submenu::SkirmishSettings => self.update_skirmish_settings(ctx),
            Submenu::Settings => {
                match index {
                    0 if enter_pressed => self.submenu = Submenu::Main,
                    1 if movement_left => ctx.settings.volume -= VOLUME_CHANGE,
                    1 if movement_right => ctx.settings.volume += VOLUME_CHANGE,
                    2 if movement_left => ctx.settings.ui_scale -= 1,
                    2 if movement_right => ctx.settings.ui_scale += 1,
                    3 if enter_pressed => ctx.settings.reset(),
                    _ => {}
                }

                if enter_pressed || movement_left || movement_right {
                    self.refresh_settings(ctx);
                }
            },
            Submenu::SkirmishSaves => {
                match index {
                    0 if enter_pressed => self.submenu = Submenu::Skirmish,
                    1 if enter_pressed => self.refresh_skirmish_saves(ctx),
                    2 if enter_pressed => self.settings.save_game = None,
                    _ if enter_pressed => {
                        let save_game = self.submenus[self.submenu.index()].get().text();
                        self.settings.set_savegame(save_game, &ctx.settings);
                    },
                    _ => {}
                }
            }
        }

        None
    }

    pub fn update_skirmish(&mut self, ctx: &Context, game_in_progress: bool) -> Option<MenuCallback> {
        let enter_pressed = ctx.gui.key_pressed(VirtualKeyCode::Return);
        let movement_left = ctx.gui.key_pressed(VirtualKeyCode::Left);
        let movement_right = ctx.gui.key_pressed(VirtualKeyCode::Right);
        let back_pressed = ctx.gui.key_pressed(VirtualKeyCode::Back);
        
        let index = &self.submenus[self.submenu.index()].index();

        let mut key_input = false;

        match index {
            0 if enter_pressed  => self.submenu = Submenu::Main,
            1 if enter_pressed  => return Some(MenuCallback::Resume),
            2 if enter_pressed  => return Some(MenuCallback::NewSkirmish(&self.settings)),
            4 if movement_left  => self.settings.game_type.rotate_left(),
            4 if movement_right => self.settings.game_type.rotate_right(),
            5 if enter_pressed  => self.submenu = Submenu::SkirmishSaves,
            6 if enter_pressed  => self.submenu = Submenu::SkirmishSettings,
            8 if back_pressed   => {
                self.settings.address.pop();
            },
            8 => {
                key_input = ctx.gui.key_input(&mut self.settings.address, |c| c.is_ascii_digit() || c == '.' || c == ':');
            },
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

            7 if movement_left => self.settings.light = self.settings.light.saturating_sub(LIGHT_LEVEL_CHANGE),
            7 if movement_right => self.settings.light += LIGHT_LEVEL_CHANGE,
            _ => {}
        }

        if movement_left || movement_right {
            self.refresh_skirmish_settings();
        }
    }

    pub fn render(&self, ctx: &mut Context) {
        // Draw the title
        let dest = [0.0, ctx.height / 2.0 - TITLE_TOP_OFFSET];
        ctx.render(Image::Title, dest, 1.0);

        render_list(&self.submenus[self.submenu.index()], ctx);
    }
}