extern crate ggez;
extern crate rand;
extern crate pathfinding;

mod map;
mod menu;
mod images;
mod units;

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
        Ok(Resources {
            images: vec![
                load_image(ctx, "/mud.png")?,
                load_image(ctx, "/edge_left_corner.png")?,
                load_image(ctx, "/edge_left.png")?,
                load_image(ctx, "/edge_corner.png")?,
                load_image(ctx, "/edge_right.png")?,
                load_image(ctx, "/edge_right_corner.png")?,
                load_image(ctx, "/skull.png")?,
                load_image(ctx, "/mud_pool.png")?,
                load_image(ctx, "/cursor.png")?,
                load_image(ctx, "/cursor_unit.png")?,
                load_image(ctx, "/cursor_unwalkable.png")?,
                load_image(ctx, "/title.png")?,
                load_image(ctx, "/friendly.png")?,
                load_image(ctx, "/enemy.png")?,
                load_image(ctx, "/path.png")?,
                load_image(ctx, "/pit_left.png")?,
                load_image(ctx, "/pit_top.png")?,
                load_image(ctx, "/pit_right.png")?,
                load_image(ctx, "/pit_bottom.png")?,
                load_image(ctx, "/pit_tl.png")?,
                load_image(ctx, "/pit_tr.png")?,
                load_image(ctx, "/pit_br.png")?,
                load_image(ctx, "/pit_bl.png")?,
                load_image(ctx, "/pit_center.png")?
            ],
            font: Font::default_font().unwrap()
        })
    }
}

fn load_image(ctx: &mut Context, image: &str) -> GameResult<Image> {
    let mut image = Image::new(ctx, image)?;
    image.set_filter(FilterMode::Nearest);

    Ok(image)
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

        Ok(MainState {
            resources: Resources::new(ctx)?,
            map: map::Map::new(),
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
                Some(value) => match value {
                    0 => {
                        self.mode = Mode::Game;
                        self.map.generate();
                    },
                    1 => self.running = false,
                    _ => {}
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
            Mode::Game => self.map.move_cursor(x, y),
            _ => {}
        }
    }

    fn mouse_button_down_event(&mut self, button: MouseButton, x: i32, y: i32) {
        match self.mode {
            Mode::Game => self.map.mouse_button(button, x, y),
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
