extern crate ggez;
extern crate rand;
extern crate pathfinding;

use ggez::conf;
use ggez::event;
use ggez::event::{Mod, Keycode, MouseState, MouseButton};
use ggez::{GameResult, Context};
use ggez::graphics;
use ggez::graphics::{Image, FilterMode, Color, Font};

use std::time::Duration;

mod map;
mod menu;
mod images;
mod units;
mod ui;
mod weapons;

use map::map::Map;
use menu::Callback;

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
                Image::new(ctx, "/base/1.png")?,
                Image::new(ctx, "/base/2.png")?,
                Image::new(ctx, "/edge/left_corner.png")?,
                Image::new(ctx, "/edge/left.png")?,
                Image::new(ctx, "/edge/corner.png")?,
                Image::new(ctx, "/edge/right.png")?,
                Image::new(ctx, "/edge/right_corner.png")?,
                Image::new(ctx, "/decoration/skull.png")?,
                Image::new(ctx, "/cursor/default.png")?,
                Image::new(ctx, "/cursor/unit.png")?,
                Image::new(ctx, "/cursor/unwalkable.png")?,
                Image::new(ctx, "/cursor/crosshair.png")?,
                Image::new(ctx, "/title.png")?,
                Image::new(ctx, "/unit/friendly.png")?,
                Image::new(ctx, "/unit/enemy.png")?,
                Image::new(ctx, "/unit/dead_friendly.png")?,
                Image::new(ctx, "/unit/dead_enemy.png")?,
                Image::new(ctx, "/path/default.png")?,
                Image::new(ctx, "/path/no_weapon.png")?,
                Image::new(ctx, "/path/unreachable.png")?,
                Image::new(ctx, "/pit/left.png")?,
                Image::new(ctx, "/pit/top.png")?,
                Image::new(ctx, "/pit/right.png")?,
                Image::new(ctx, "/pit/bottom.png")?,
                Image::new(ctx, "/pit/tl.png")?,
                Image::new(ctx, "/pit/tr.png")?,
                Image::new(ctx, "/pit/br.png")?,
                Image::new(ctx, "/pit/bl.png")?,
                Image::new(ctx, "/pit/center.png")?,
                Image::new(ctx, "/button/end_turn.png")?,
                Image::new(ctx, "/button/fire.png")?,
                Image::new(ctx, "/ruin/1.png")?,
                Image::new(ctx, "/ruin/2.png")?,
                Image::new(ctx, "/ruin/3.png")?,
                Image::new(ctx, "/bullet/bullet.png")?
                // Image::new(ctx, "/fog.png")?
            ],
            font: Font::new(ctx, "/font.ttf", 12)?
        })
    }
}

struct MainState {
    resources: Resources,
    mode: Mode,
    map: Map,
    menu: menu::Menu
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        graphics::set_background_color(ctx, Color::new(0.0, 0.0, 0.0, 0.0));

        let resources = Resources::new(ctx)?;

        Ok(MainState {
            map: Map::new(ctx, &resources),
            resources,
            menu: menu::Menu::new(), 
            mode: Mode::Menu,
        })
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context, _dt: Duration) -> GameResult<()> {
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

    fn key_down_event(&mut self, ctx: &mut Context, key: Keycode, _mod: Mod, _repeat: bool) {
        match self.mode {
            Mode::Game => self.map.handle_key(ctx, key, true),
            Mode::Menu => match self.menu.handle_key(key) {
                Some(callback) => match callback {
                    Callback::Play => {
                        self.mode = Mode::Game;
                        self.map.start(self.menu.rows, self.menu.cols);
                    },
                    Callback::Quit => ctx.quit().unwrap()
                },
                _ => {}
            }
        };
    }

    fn key_up_event(&mut self, ctx: &mut Context, key: Keycode, _mod: Mod, _repeat: bool) {
        match self.mode {
            Mode::Game => self.map.handle_key(ctx, key, false),
            _ => {}
        }
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, _state: MouseState, x: i32, y: i32, _xrel: i32, _yrel: i32) {
        match self.mode {
            Mode::Game => self.map.move_cursor(ctx, x as f32, y as f32),
            _ => {}
        }
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: i32, y: i32) {
        match self.mode {
            Mode::Game => self.map.mouse_button(button, x as f32, y as f32),
            _ => {}
        }
    }

    fn quit_event(&mut self) -> bool { false }
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
