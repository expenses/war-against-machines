extern crate sdl2;
extern crate rand;
extern crate pathfinding;
extern crate ord_subset;
extern crate odds;

use sdl2::render::{Texture, TextureCreator};
use sdl2::video::WindowContext;
use sdl2::event::Event;
use sdl2::ttf;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::{PixelFormatEnum, Color};
use sdl2::rwops::RWops;
use sdl2::image::ImageRWops;

mod battle;
mod menu;
mod ui;
mod weapons;
mod context;
mod utils;
mod colours;
mod items;

use context::Context;
use battle::battle::Battle;
use menu::Callback;

use std::collections::HashMap;

const TITLE: &str = "War Against Machines";
const WINDOW_WIDTH: u32 = 960;
const WINDOW_HEIGHT: u32 = 600;

// Which mode the game is in
enum Mode {
    Menu,
    Skirmish
}

// Load a resource into a SDL2 RWops struct at compile time
macro_rules! rw_ops {
    ($file:expr) => (
        RWops::from_bytes(include_bytes!(concat!("../resources/", $file))).unwrap()
    )
}

// A struct to hold resources for the game such as images and fonts
pub struct Resources<'a> {
    texture_creator: &'a TextureCreator<WindowContext>,
    images: HashMap<&'static str, Texture<'a>>,
    font_context: &'a ttf::Sdl2TtfContext,
    fonts: HashMap<&'static str, ttf::Font<'a, 'a>>,
}

impl<'a> Resources<'a> {
    // Create a new resource struct with a texture creator, font context and directory string
    fn new(texture_creator: &'a TextureCreator<WindowContext>,
           font_context: &'a ttf::Sdl2TtfContext) -> Resources<'a> {        
        Resources {
            texture_creator,
            images: HashMap::new(),
            font_context,
            fonts: HashMap::new(),
        }
    }

    // Load an image into the images hashmap from a RWops of a png
    fn load_image(&mut self, name: &'static str, rw_ops: RWops) {
        self.images.insert(name, self.texture_creator.create_texture_from_surface(
            rw_ops.load_png().unwrap()
        ).unwrap());
    }

    // Get an image from the hashmap or panic
    fn image(&self, name: &str) -> &Texture {
        self.images.get(name).expect(&format!("Image '{}' could not be found.", name))
    }

    // Create a new texture using the texture creator
    fn create_texture(&self, width: u32, height: u32) -> Texture {
        self.texture_creator.create_texture_target(PixelFormatEnum::ARGB8888, width, height).unwrap()
    }

    // Load a font into the fonts hashmap from a RWops of a font
    fn load_font(&mut self, name: &'static str, rw_ops: RWops<'a>, size: u16) {
        self.fonts.insert(name, self.font_context.load_font_from_rwops(
            rw_ops, size
        ).unwrap());
    }

    // Render a string of text using a font
    fn render(&self, font: &str, text: &str, colour: Color) -> Texture {
        // Render the text into a surface in a solid colour
        let rendered = self.fonts[font].render(text).solid(colour).unwrap();

        // Create a texture from that surface
        self.texture_creator.create_texture_from_surface(rendered).unwrap()
    }
}

// A struct for holding the game state
struct State<'a> {
    ctx: Context,
    resources: Resources<'a>,
    mode: Mode,
    menu: menu::Menu,
    skirmish: Battle,
}

impl<'a> State<'a> {
    // Create a new state, starting on the menu
    fn run(ctx: Context, resources: Resources<'a>) {
        let mut state = State {
            mode: Mode::Menu,
            menu: menu::Menu::new(),
            skirmish: Battle::new(&resources),
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

    // Update the game
    fn update(&mut self) {
        match self.mode {
            Mode::Skirmish => self.skirmish.update(),
            _ => {}
        }
    }

    // Handle key presses
    fn handle_key_down(&mut self, key: Keycode) {
        match self.mode {
            // If the mode is the menu, respond to callbacks
            Mode::Menu => if let Some(callback) = self.menu.handle_key(&mut self.ctx, key) {
                match callback {
                    Callback::Play => {
                        self.mode = Mode::Skirmish;
                        self.skirmish.start(&self.menu.skirmish_settings);
                    }
                }  
            },
            Mode::Skirmish => self.skirmish.handle_key(&mut self.ctx, key, true)
        }
    }

    // Handle key releases
    fn handle_key_up(&mut self, key: Keycode) {
        match self.mode {
            Mode::Skirmish => self.skirmish.handle_key(&mut self.ctx, key, false),
            _ => {}
        }
    }

    // Handle mouse movement
    fn handle_mouse_motion(&mut self, x: i32, y: i32) {
        match self.mode {
            Mode::Skirmish => self.skirmish.move_cursor(&mut self.ctx, x as f32, y as f32),
            _ => {}
        }
    }

    // Handle mouse button presses
    fn handle_mouse_button(&mut self, button: MouseButton, x: i32, y: i32) {
        match self.mode {
            Mode::Skirmish => self.skirmish.mouse_button(&mut self.ctx, button, x as f32, y as f32),
            _ => {}
        }
    }

    // Clear, draw and present the canvas
    fn draw(&mut self) {
        self.ctx.clear();

        match self.mode {
            Mode::Skirmish => self.skirmish.draw(&mut self.ctx, &self.resources),
            Mode::Menu => self.menu.draw(&mut self.ctx, &self.resources)
        }

        self.ctx.present();
    }
}

// The main function
fn main() {
    // Create the context
    let ctx = Context::new(TITLE, WINDOW_WIDTH, WINDOW_HEIGHT);
    
    let texture_creator = ctx.texture_creator();
    let font_context = ttf::init().unwrap();

    // Create the resources
    let mut resources = Resources::new(&texture_creator, &font_context);

    // Load the images into the binary
    resources.load_image("title", rw_ops!("title.png"));

    resources.load_image("base_1", rw_ops!("base/1.png"));
    resources.load_image("base_2", rw_ops!("base/2.png"));
    resources.load_image("fog",    rw_ops!("base/fog.png"));
    
    resources.load_image("squaddie", rw_ops!("unit/squaddie.png"));
    resources.load_image("machine",  rw_ops!("unit/machine.png"));
    
    resources.load_image("regular_bullet", rw_ops!("bullet/regular.png"));
    resources.load_image("plasma_bullet",  rw_ops!("bullet/plasma.png"));
    
    resources.load_image("cursor",            rw_ops!("cursor/default.png"));
    resources.load_image("cursor_unit",       rw_ops!("cursor/unit.png"));
    resources.load_image("cursor_unwalkable", rw_ops!("cursor/unwalkable.png"));
    resources.load_image("cursor_crosshair",  rw_ops!("cursor/crosshair.png"));
    
    resources.load_image("ruin_1", rw_ops!("ruin/1.png"));
    resources.load_image("ruin_2", rw_ops!("ruin/2.png"));
    resources.load_image("ruin_3", rw_ops!("ruin/3.png"));
    
    resources.load_image("pit_top",    rw_ops!("pit/top.png"));
    resources.load_image("pit_right",  rw_ops!("pit/right.png"));
    resources.load_image("pit_left",   rw_ops!("pit/left.png"));
    resources.load_image("pit_bottom", rw_ops!("pit/bottom.png"));
    resources.load_image("pit_tl",     rw_ops!("pit/tl.png"));
    resources.load_image("pit_tr",     rw_ops!("pit/tr.png"));
    resources.load_image("pit_bl",     rw_ops!("pit/bl.png"));
    resources.load_image("pit_br",     rw_ops!("pit/br.png"));
    resources.load_image("pit_center", rw_ops!("pit/center.png"));
    
    resources.load_image("path",             rw_ops!("path/default.png"));
    resources.load_image("path_no_weapon",   rw_ops!("path/no_weapon.png"));
    resources.load_image("path_unreachable", rw_ops!("path/unreachable.png"));
    
    resources.load_image("edge_left",         rw_ops!("edge/left.png"));
    resources.load_image("edge_right",        rw_ops!("edge/right.png"));
    resources.load_image("edge_left_corner",  rw_ops!("edge/left_corner.png"));
    resources.load_image("edge_right_corner", rw_ops!("edge/right_corner.png"));
    resources.load_image("edge_corner",       rw_ops!("edge/corner.png"));
        
    resources.load_image("end_turn_button",         rw_ops!("button/end_turn.png"));
    resources.load_image("inventory_button",        rw_ops!("button/inventory.png"));
    resources.load_image("change_fire_mode_button", rw_ops!("button/change_fire_mode.png"));

    resources.load_image("scrap",           rw_ops!("items/scrap.png"));
    resources.load_image("weapon",          rw_ops!("items/weapon.png"));
    resources.load_image("squaddie_corpse", rw_ops!("items/squaddie_corpse.png"));
    resources.load_image("machine_corpse",  rw_ops!("items/machine_corpse.png"));
    resources.load_image("skeleton",        rw_ops!("items/skeleton.png"));
    
    // Load the font
    resources.load_font("main", rw_ops!("font.ttf"), 35);

    // Start the game
    State::run(ctx, resources);
}
