extern crate sdl2;
extern crate rand;
extern crate pathfinding;

use sdl2::render::{Texture, TextureCreator};
use sdl2::video::WindowContext;
use sdl2::image::LoadTexture;
use sdl2::event::Event;
use sdl2::ttf;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::PixelFormatEnum;

mod map;
mod menu;
mod units;
mod ui;
mod weapons;
mod context;

use context::Context;
use map::map::Map;
use menu::Callback;

use std::collections::HashMap;

const TITLE: &str = "Assault";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

enum Mode {
    Menu,
    Game
}

pub struct Resources<'a> {
    texture_creator: &'a TextureCreator<WindowContext>,
    images: HashMap<&'a str, Texture<'a>>,
    font_context: &'a ttf::Sdl2TtfContext,
    fonts: HashMap<&'a str, ttf::Font<'a, 'a>>,
}

impl<'a> Resources<'a> {
    fn new(texture_creator: &'a TextureCreator<WindowContext>, font_context: &'a ttf::Sdl2TtfContext) -> Resources<'a> {
        Resources {
            texture_creator,
            images: HashMap::new(),
            font_context,
            fonts: HashMap::new(),
        }
    }

    fn load_image(&mut self, name: &'a str, path: &str) {
        self.images.insert(name, self.texture_creator.load_texture(path).unwrap());
    }

    fn image(&self, name: &str) -> &Texture {
        match self.images.get(name) {
            Some(texture) => &texture,
            None => panic!("Missing image: '{}'", name)
        }
    }

    fn create_texture(&self, width: u32, height: u32) -> Texture {
        self.texture_creator.create_texture_target(PixelFormatEnum::ARGB8888, width, height).unwrap()
    }

    fn load_font(&mut self, name: &'a str, path: &str, size: u16) {
        self.fonts.insert(name, self.font_context.load_font(path, size).unwrap());
    }

    fn render(&self, font: &str, text: &str) -> Texture {
        let colour = sdl2::pixels::Color {r:255, g:255, b:255,a:100};

        let rendered = self.fonts[font].render(text).solid(colour).unwrap();

        self.texture_creator.create_texture_from_surface(rendered).unwrap()
    }
}

struct State<'a> {
    ctx: &'a mut Context,
    resources: Resources<'a>,
    mode: Mode,
    menu: menu::Menu,
    map: Map,
}

impl<'a> State<'a> {
    fn run(ctx: &'a mut Context, resources: Resources<'a>) {
        let mut state = State {
            mode: Mode::Menu,
            menu: menu::Menu::new(),
            map: Map::new(&resources),
            ctx, resources,
        };

        let mut pump = state.ctx.event_pump();

        'main: while state.ctx.running {
            for event in pump.poll_iter() {
                match event {
                    Event::Quit {..} => break 'main,
                    Event::KeyDown {keycode, ..} => state.handle_key_down(keycode.unwrap()),
                    Event::KeyUp {keycode, ..} => state.handle_key_up(keycode.unwrap()),
                    Event::MouseMotion {x, y, ..} => state.handle_mouse_motion(x, y),
                    Event::MouseButtonDown {mouse_btn, x, y, ..} => state.handle_mouse_button(mouse_btn, x, y),
                    _ => {}
                }
            }

            state.update();
            state.draw();
        }
    }

    fn update(&mut self) {
        match self.mode {
            Mode::Game => self.map.update(),
            _ => {}
        }
    }

    fn handle_key_down(&mut self, key: Keycode) {
        match self.mode {
            Mode::Menu => match self.menu.handle_key(self.ctx, key) {
                Some(callback) => match callback {
                    Callback::Play => {
                        self.mode = Mode::Game;
                        self.map.start(self.menu.rows, self.menu.cols);
                    }
                },
                _ => {}
            },
            Mode::Game => self.map.handle_key(self.ctx, key, true)
        }
    }

    fn handle_key_up(&mut self, key: Keycode) {
        match self.mode {
            Mode::Game => self.map.handle_key(self.ctx, key, false),
            _ => {}
        }
    }

    fn handle_mouse_motion(&mut self, x: i32, y: i32) {
        match self.mode {
            Mode::Game => self.map.move_cursor(self.ctx, x as f32, y as f32),
            _ => {}
        }
    }

    fn handle_mouse_button(&mut self, button: MouseButton, x: i32, y: i32) {
        match self.mode {
            Mode::Game => self.map.mouse_button(self.ctx, button, x as f32, y as f32),
            _ => {}
        }
    }

    fn draw(&mut self) {
        self.ctx.clear();

        match self.mode {
            Mode::Game => self.map.draw(self.ctx, &self.resources),
            Mode::Menu => self.menu.draw(self.ctx, &self.resources)
        }

        self.ctx.present();
    }
}

pub fn main() {
    let mut ctx = Context::new(TITLE, WINDOW_WIDTH, WINDOW_HEIGHT);
    
    let font_context = sdl2::ttf::init().unwrap();
    let texture_creator = ctx.texture_creator();
    
    let mut resources = Resources::new(&texture_creator, &font_context);

    resources.load_image("title",               "resources/title.png");
    resources.load_image("base_1",              "resources/base/1.png");
    resources.load_image("base_2",              "resources/base/2.png");
    resources.load_image("friendly",            "resources/unit/friendly.png");
    resources.load_image("enemy",               "resources/unit/enemy.png");
    resources.load_image("dead_friendly",       "resources/unit/dead_friendly.png");
    resources.load_image("dead_enemy",          "resources/unit/dead_enemy.png");
    resources.load_image("bullet",              "resources/bullet/bullet.png");
    resources.load_image("cursor",              "resources/cursor/default.png");
    resources.load_image("cursor_unit",         "resources/cursor/unit.png");
    resources.load_image("cursor_unwalkable",   "resources/cursor/unwalkable.png");
    resources.load_image("cursor_crosshair",    "resources/cursor/crosshair.png");
    resources.load_image("ruin_1",              "resources/ruin/1.png");
    resources.load_image("ruin_2",              "resources/ruin/2.png");
    resources.load_image("ruin_3",              "resources/ruin/3.png");
    resources.load_image("pit_top",             "resources/pit/top.png");
    resources.load_image("pit_right",           "resources/pit/right.png");
    resources.load_image("pit_left",            "resources/pit/left.png");
    resources.load_image("pit_bottom",          "resources/pit/bottom.png");
    resources.load_image("pit_tl",              "resources/pit/tl.png");
    resources.load_image("pit_tr",              "resources/pit/tr.png");
    resources.load_image("pit_bl",              "resources/pit/bl.png");
    resources.load_image("pit_br",              "resources/pit/br.png");
    resources.load_image("pit_center",          "resources/pit/center.png");
    resources.load_image("path",                "resources/path/default.png");
    resources.load_image("path_no_weapon",      "resources/path/no_weapon.png");
    resources.load_image("path_unreachable",    "resources/path/unreachable.png");
    resources.load_image("edge_left",           "resources/edge/left.png");
    resources.load_image("edge_right",          "resources/edge/right.png");
    resources.load_image("edge_left_corner",    "resources/edge/left_corner.png");
    resources.load_image("edge_right_corner",   "resources/edge/right_corner.png");
    resources.load_image("edge_corner",         "resources/edge/corner.png");
    resources.load_image("skull",               "resources/decoration/skull.png");
    resources.load_image("fog",                 "resources/decoration/fog.png");
    resources.load_image("end_turn_button",     "resources/button/end_turn.png");
    resources.load_image("fire_button",         "resources/button/fire.png");
    
    resources.load_font("main", "resources/font.ttf", 35);

    State::run(&mut ctx, resources);
}
