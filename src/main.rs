extern crate sdl2;
extern crate rand;
extern crate pathfinding;
extern crate ord_subset;

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
mod ui;
mod weapons;
mod context;
mod utils;

use context::Context;
use map::map::Map;
use menu::Callback;

use std::collections::HashMap;
use std::path::Path;

const TITLE: &str = "Assault";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

enum Mode {
    Menu,
    Game
}

// A struct to hold resources for the game such as images and fonts
pub struct Resources<'a> {
    texture_creator: &'a TextureCreator<WindowContext>,
    directory: &'a Path,
    images: HashMap<String, Texture<'a>>,
    font_context: &'a ttf::Sdl2TtfContext,
    fonts: HashMap<String, ttf::Font<'a, 'a>>,
}

impl<'a> Resources<'a> {
    // Create a new resource struct with a texture creator, font context and directory string
    fn new(texture_creator: &'a TextureCreator<WindowContext>,
           font_context: &'a ttf::Sdl2TtfContext, directory: &'a str) -> Resources<'a> {        
        Resources {
            texture_creator,
            directory: Path::new(directory),
            images: HashMap::new(),
            font_context,
            fonts: HashMap::new(),
        }
    }

    // Load an image into the images hashmap
    fn load_image(&mut self, name: &str, path: &str) {
        let path = self.directory.join(path);

        self.images.insert(name.into(), self.texture_creator.load_texture(path).unwrap());
    }

    // Get an image from the hashmap or panic
    fn image(&self, name: &String) -> &Texture {
        match self.images.get(name) {
            Some(texture) => &texture,
            None => panic!("Loaded image '{}' could not be found.", name)
        }
    }

    // Create a new texture using the texture creator
    fn create_texture(&self, width: u32, height: u32) -> Texture {
        self.texture_creator.create_texture_target(PixelFormatEnum::ARGB8888, width, height).unwrap()
    }

    // Load a font into the fonts hashmap
    fn load_font(&mut self, name: &str, path: &str, size: u16) {
        let path = self.directory.join(path);

        self.fonts.insert(name.into(), self.font_context.load_font(path, size).unwrap());
    }

    // Render a string of text using a font
    fn render(&self, font: &str, text: &String) -> Texture {
        let colour = sdl2::pixels::Color {r: 255, g: 255, b: 255, a: 255};

        let rendered = self.fonts[font].render(text).solid(colour).unwrap();

        self.texture_creator.create_texture_from_surface(rendered).unwrap()
    }
}

// A struct for holding the game state
struct State<'a> {
    ctx: Context,
    resources: Resources<'a>,
    mode: Mode,
    menu: menu::Menu,
    map: Map,
}

impl<'a> State<'a> {
    // Create a new state, starting on the menu
    fn run(ctx: Context, resources: Resources<'a>) {
        let mut state = State {
            mode: Mode::Menu,
            menu: menu::Menu::new(),
            map: Map::new(&resources),
            ctx, resources,
        };

        // Get the event pump
        let mut pump = state.ctx.event_pump();

        // Loop through events while the game is running
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

    // Update the parts of the game
    fn update(&mut self) {
        match self.mode {
            Mode::Game => self.map.update(),
            _ => {}
        }
    }

    // Handle key presses
    fn handle_key_down(&mut self, key: Keycode) {
        match self.mode {
            // If the mode is the menu, respond to callbacks
            Mode::Menu => match self.menu.handle_key(&mut self.ctx, key) {
                Some(callback) => match callback {
                    Callback::Play => {
                        self.mode = Mode::Game;
                        self.map.start(self.menu.rows, self.menu.cols);
                    }
                },
                _ => {}
            },
            Mode::Game => self.map.handle_key(&mut self.ctx, key, true)
        }
    }

    // Handle key releases
    fn handle_key_up(&mut self, key: Keycode) {
        match self.mode {
            Mode::Game => self.map.handle_key(&mut self.ctx, key, false),
            _ => {}
        }
    }

    // Handle mouse motion
    fn handle_mouse_motion(&mut self, x: i32, y: i32) {
        match self.mode {
            Mode::Game => self.map.move_cursor(&mut self.ctx, x as f32, y as f32),
            _ => {}
        }
    }

    // Handle mouse button presses
    fn handle_mouse_button(&mut self, button: MouseButton, x: i32, y: i32) {
        match self.mode {
            Mode::Game => self.map.mouse_button(&mut self.ctx, button, x as f32, y as f32),
            _ => {}
        }
    }

    // Draw on the canvas
    fn draw(&mut self) {
        self.ctx.clear();

        match self.mode {
            Mode::Game => self.map.draw(&mut self.ctx, &self.resources),
            Mode::Menu => self.menu.draw(&mut self.ctx, &self.resources)
        }

        self.ctx.present();
    }
}

pub fn main() {
    // Create the context
    let ctx = Context::new(TITLE, WINDOW_WIDTH, WINDOW_HEIGHT);
    
    let texture_creator = ctx.texture_creator();
    let font_context = ttf::init().unwrap();

    // Create the resources
    let mut resources = Resources::new(&texture_creator, &font_context, "resources");

    // Load the images
    resources.load_image("title",   "title.png");

    resources.load_image("base_1",  "base/1.png");
    resources.load_image("base_2",  "base/2.png");
    
    resources.load_image("friendly_squaddie",       "unit/friendly_squaddie.png");
    resources.load_image("enemy_squaddie",          "unit/enemy_squaddie.png");
    resources.load_image("dead_friendly_squaddie",  "unit/dead_friendly_squaddie.png");
    resources.load_image("dead_enemy_squaddie",     "unit/dead_enemy_squaddie.png");
    resources.load_image("friendly_robot",          "unit/friendly_robot.png");
    resources.load_image("enemy_robot",             "unit/enemy_robot.png");
    resources.load_image("dead_friendly_robot",     "unit/dead_friendly_robot.png");
    resources.load_image("dead_enemy_robot",        "unit/dead_enemy_robot.png");
    
    
    resources.load_image("bullet",  "bullet/bullet.png");
    
    resources.load_image("cursor",              "cursor/default.png");
    resources.load_image("cursor_unit",         "cursor/unit.png");
    resources.load_image("cursor_unwalkable",   "cursor/unwalkable.png");
    resources.load_image("cursor_crosshair",    "cursor/crosshair.png");
    
    resources.load_image("ruin_1",  "ruin/1.png");
    resources.load_image("ruin_2",  "ruin/2.png");
    resources.load_image("ruin_3",  "ruin/3.png");
    
    resources.load_image("pit_top",     "pit/top.png");
    resources.load_image("pit_right",   "pit/right.png");
    resources.load_image("pit_left",    "pit/left.png");
    resources.load_image("pit_bottom",  "pit/bottom.png");
    resources.load_image("pit_tl",      "pit/tl.png");
    resources.load_image("pit_tr",      "pit/tr.png");
    resources.load_image("pit_bl",      "pit/bl.png");
    resources.load_image("pit_br",      "pit/br.png");
    resources.load_image("pit_center",  "pit/center.png");
    
    resources.load_image("path",                "path/default.png");
    resources.load_image("path_no_weapon",      "path/no_weapon.png");
    resources.load_image("path_unreachable",    "path/unreachable.png");
    
    resources.load_image("edge_left",           "edge/left.png");
    resources.load_image("edge_right",          "edge/right.png");
    resources.load_image("edge_left_corner",    "edge/left_corner.png");
    resources.load_image("edge_right_corner",   "edge/right_corner.png");
    resources.load_image("edge_corner",         "edge/corner.png");
    
    resources.load_image("skull",   "decoration/skull.png");
    resources.load_image("fog",     "decoration/fog.png");
    
    resources.load_image("end_turn_button", "button/end_turn.png");
    resources.load_image("fire_button",     "button/fire.png");
    
    // Load the font
    resources.load_font("main", "font.ttf", 35);

    // Start the game
    State::run(ctx, resources);
}
