extern crate ggez;
extern crate rand;
extern crate pathfinding;

mod map;
mod menu;
mod images;
mod units;
mod tiles;
mod ui;
mod weapons;
mod paths;

use menu::Callback;

use std::time::Duration;

use ggez::conf;
use ggez::event;
use ggez::event::{Mod, Keycode, MouseState, MouseButton};
use ggez::{GameResult, Context};
use ggez::graphics;
use ggez::graphics::{Image, FilterMode, Color, Font};

enum Mode {
    Menu,
    Game
}

const TITLE: &str = "Assault";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

pub struct Resources {
    images: Vec<Image>,
    font: Font
}

impl Resources {
    fn new(ctx: &mut Context) -> GameResult<Resources> {
        graphics::set_default_filter(ctx, FilterMode::Nearest);

        Ok(Resources {
            images: vec![
                Image::new(ctx, "/base_1.png")?,
                Image::new(ctx, "/base_2.png")?,
                Image::new(ctx, "/edge_left_corner.png")?,
                Image::new(ctx, "/edge_left.png")?,
                Image::new(ctx, "/edge_corner.png")?,
                Image::new(ctx, "/edge_right.png")?,
                Image::new(ctx, "/edge_right_corner.png")?,
                Image::new(ctx, "/skull.png")?,
                Image::new(ctx, "/cursor.png")?,
                Image::new(ctx, "/cursor_unit.png")?,
                Image::new(ctx, "/cursor_unwalkable.png")?,
                Image::new(ctx, "/cursor_crosshair.png")?,
                Image::new(ctx, "/title.png")?,
                Image::new(ctx, "/friendly.png")?,
                Image::new(ctx, "/enemy.png")?,
                Image::new(ctx, "/dead_friendly.png")?,
                Image::new(ctx, "/dead_enemy.png")?,
                Image::new(ctx, "/path.png")?,
                Image::new(ctx, "/pit_left.png")?,
                Image::new(ctx, "/pit_top.png")?,
                Image::new(ctx, "/pit_right.png")?,
                Image::new(ctx, "/pit_bottom.png")?,
                Image::new(ctx, "/pit_tl.png")?,
                Image::new(ctx, "/pit_tr.png")?,
                Image::new(ctx, "/pit_br.png")?,
                Image::new(ctx, "/pit_bl.png")?,
                Image::new(ctx, "/pit_center.png")?,
                Image::new(ctx, "/end_turn_button.png")?,
                Image::new(ctx, "/fire_button.png")?,
                Image::new(ctx, "/ruin_1.png")?,
                Image::new(ctx, "/ruin_2.png")?
                // Image::new(ctx, "/fog.png")?
            ],
            font: Font::new(ctx, "/font.ttf", 12)?
        })
    }
}

struct MainState {
    resources: Resources,
    mode: Mode,
    map: map::Map,
    menu: menu::Menu,
    // The state of the game, used to trigger ctx.quit() when ctx might not be avaliable (event listeners)
    running: bool
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        graphics::set_background_color(ctx, Color::new(0.0, 0.0, 0.0, 0.0));

        let resources = Resources::new(ctx)?;

        Ok(MainState {
            map: map::Map::new(&resources),
            resources,
            menu: menu::Menu::new(), 
            mode: Mode::Menu,
            running: true
        })
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context, _dt: Duration) -> GameResult<()> {
        if !self.running { ctx.quit()?; }

        match self.mode {
            Mode::Game => self.map.update(),
            _ => {}
        };
        
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        match self.mode {
            Mode::Game => self.map.draw(ctx, &self.resources),
            Mode::Menu => self.menu.draw(ctx, &self.resources)
        };

        graphics::present(ctx);

        Ok(())
    }

    fn key_down_event(&mut self, key: Keycode, _mod: Mod, _repeat: bool) {
        match self.mode {
            Mode::Game => self.map.handle_key(key, true),
            Mode::Menu => match self.menu.handle_key(key) {
                Some(callback) => match callback {
                    Callback::Play => {
                        self.mode = Mode::Game;
                        self.map.start(self.menu.rows, self.menu.cols);
                    },
                    Callback::Quit => self.running = false,
                },
                _ => {}
            }
        };
    }

    fn key_up_event(&mut self, key: Keycode, _mod: Mod, _repeat: bool) {
        match self.mode {
            Mode::Game => self.map.handle_key(key, false),
            _ => {}
        }
    }

    fn mouse_motion_event(&mut self, _state: MouseState, x: i32, y: i32, _xrel: i32, _yrel: i32) {
        match self.mode {
            Mode::Game => self.map.move_cursor(x as f32, y as f32),
            _ => {}
        }
    }

    fn mouse_button_down_event(&mut self, button: MouseButton, x: i32, y: i32) {
        match self.mode {
            Mode::Game => self.map.mouse_button(button, x as f32, y as f32),
            _ => {}
        }
    }
}

pub fn main() {
    let c = conf::Conf {
        window_title: String::from(TITLE),
        vsync: true,
        window_width: WINDOW_WIDTH,
        window_height: WINDOW_HEIGHT,
        ..Default::default()
    };

    let mut ctx = Context::load_from_conf(TITLE, "ggez", c).unwrap();
    let mut state = MainState::new(&mut ctx).unwrap();
    event::run(&mut ctx, &mut state).unwrap();
}
