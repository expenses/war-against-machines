extern crate ggez;
extern crate rand;

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

struct MainState {
    images: Vec<Image>,
    font: Font,
    mode: Mode,
    map: map::Map,
    menu: menu::Menu,
    // The state of the game, used to trigger ctx.quit() when ctx might not be avaliable (event listeners)
    running: bool
}

fn load_image(ctx: &mut Context, image: &str) -> GameResult<Image> {
    let mut image = Image::new(ctx, image)?;
    image.set_filter(FilterMode::Nearest);

    Ok(image)
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let mut images: Vec<Image> = Vec::new();

        images.push(load_image(ctx, "/mud.png")?);
        images.push(load_image(ctx, "/edge_left_corner.png")?);
        images.push(load_image(ctx, "/edge_left.png")?);
        images.push(load_image(ctx, "/edge_corner.png")?);
        images.push(load_image(ctx, "/edge_right.png")?);
        images.push(load_image(ctx, "/edge_right_corner.png")?);
        images.push(load_image(ctx, "/skull.png")?);
        images.push(load_image(ctx, "/mud_pool.png")?);
        images.push(load_image(ctx, "/cursor.png")?);
        images.push(load_image(ctx, "/cursor_selected.png")?);
        images.push(load_image(ctx, "/title.png")?);
        images.push(load_image(ctx, "/friendly.png")?);
        images.push(load_image(ctx, "/enemy.png")?);

        graphics::set_background_color(ctx, Color::new(0.0, 0.0, 0.0, 0.0));

        let state = MainState {
            images: images,
            font: Font::default_font().unwrap(),
            map: map::Map::new(),
            menu: menu::Menu::new(), 
            mode: Mode::Menu,
            running: true
        };

        Ok(state)
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
            Mode::Game => self.map.draw(ctx, &self.images, &self.font),
            Mode::Menu => self.menu.draw(ctx, &self.images, &self.font)
        };

        graphics::present(ctx);

        Ok(())
    }

    fn key_down_event(&mut self, key: Keycode, _mod: Mod, _repeat: bool) {
        match self.mode {
            Mode::Game => self.map.handle_key(key, true),
            Mode::Menu => match self.menu.handle_key(key) {
                Some(value) => match value {
                    0 => { self.mode = Mode::Game; self.map.generate(); },
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
            Mode::Game => self.map.move_cursor(x as f32, y as f32),
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