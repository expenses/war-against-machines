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

pub mod battle;
pub mod menu;
pub mod ui;
pub mod weapons;
pub mod context;
pub mod utils;
pub mod colours;
pub mod items;

use context::Context;
use battle::battle::Battle;
use menu::Callback;

use std::collections::HashMap;

const TITLE: &str = "War Against Machines";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

/// Which mode the game is in
pub enum Mode {
    Menu,
    Skirmish
}

/// A struct to hold resources for the game such as images and fonts
pub struct Resources<'a> {
    texture_creator: &'a TextureCreator<WindowContext>,
    images: HashMap<String, Texture<'a>>,
    font_context: &'a ttf::Sdl2TtfContext,
    fonts: HashMap<String, ttf::Font<'a, 'a>>,
}

impl<'a> Resources<'a> {
    /// Create a new resource struct with a texture creator, font context and directory string
    pub fn new(texture_creator: &'a TextureCreator<WindowContext>,
           font_context: &'a ttf::Sdl2TtfContext) -> Resources<'a> {        
        Resources {
            texture_creator,
            images: HashMap::new(),
            font_context,
            fonts: HashMap::new(),
        }
    }

    /// Load an image into the images hashmap from the bytes of a png
    pub fn load_image(&mut self, name: &str, bytes: &[u8]) {
        let rw_ops = RWops::from_bytes(bytes).unwrap();

        self.images.insert(name.into(),
            self.texture_creator.create_texture_from_surface(
                rw_ops.load_png().unwrap()
            ).unwrap()
        );
    }

    /// Get an image from the hashmap or panic
    pub fn image(&self, name: &str) -> &Texture {
        self.images.get(name).expect(&format!("Image '{}' could not be found.", name))
    }

    /// Create a new texture using the texture creator
    pub fn create_texture(&self, width: u32, height: u32) -> Texture {
        self.texture_creator.create_texture_target(PixelFormatEnum::ARGB8888, width, height).unwrap()
    }

    /// Load a font into the fonts hashmap from the bytes of a font
    pub fn load_font(&mut self, name: &str, bytes: &'a [u8], size: u16) {
        let rw_ops = RWops::from_bytes(bytes).unwrap();

        self.fonts.insert(name.into(),
            self.font_context.load_font_from_rwops(
                rw_ops, size
            ).unwrap()
        );
    }

    /// Render a string of text using a font
    pub fn render(&self, font: &str, text: &str, colour: Color) -> Texture {
        // Render the text into a surface in a solid colour
        let rendered = self.fonts[font].render(text).solid(colour).unwrap();

        // Create a texture from that surface
        self.texture_creator.create_texture_from_surface(rendered).unwrap()
    }
}

/// A struct for holding the game state
struct State<'a> {
    ctx: Context,
    resources: Resources<'a>,
    mode: Mode,
    menu: menu::Menu,
    skirmish: Battle,
}

impl<'a> State<'a> {
    /// Create a new state, starting on the menu
    pub fn run(ctx: Context, resources: Resources<'a>) {
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

    /// Update the game
    pub fn update(&mut self) {
        match self.mode {
            Mode::Skirmish => self.skirmish.update(),
            _ => {}
        }
    }

    /// Handle key presses
    pub fn handle_key_down(&mut self, key: Keycode) {
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

    /// Handle key releases
    pub fn handle_key_up(&mut self, key: Keycode) {
        match self.mode {
            Mode::Skirmish => self.skirmish.handle_key(&mut self.ctx, key, false),
            _ => {}
        }
    }

    /// Handle mouse movement
    pub fn handle_mouse_motion(&mut self, x: i32, y: i32) {
        match self.mode {
            Mode::Skirmish => self.skirmish.move_cursor(&mut self.ctx, x as f32, y as f32),
            _ => {}
        }
    }

    /// Handle mouse button presses
    pub fn handle_mouse_button(&mut self, button: MouseButton, x: i32, y: i32) {
        match self.mode {
            Mode::Skirmish => self.skirmish.mouse_button(&mut self.ctx, button, x as f32, y as f32),
            _ => {}
        }
    }

    /// Clear, draw and present the canvas
    pub fn draw(&mut self) {
        self.ctx.clear();

        match self.mode {
            Mode::Skirmish => self.skirmish.draw(&mut self.ctx, &self.resources),
            Mode::Menu => self.menu.draw(&mut self.ctx, &self.resources)
        }

        self.ctx.present();
    }
}

/// The main function
pub fn main() {
    // Create the context
    let ctx = Context::new(TITLE, WINDOW_WIDTH, WINDOW_HEIGHT);
    
    let texture_creator = ctx.texture_creator();
    let font_context = ttf::init().unwrap();

    // Create the resources
    let mut resources = Resources::new(&texture_creator, &font_context);

    // Load the images into the binary
    resources.load_image("title",   include_bytes!("../resources/title.png"));

    resources.load_image("base_1", include_bytes!("../resources/base/1.png"));
    resources.load_image("base_2", include_bytes!("../resources/base/2.png"));
    resources.load_image("fog",    include_bytes!("../resources/base/fog.png"));
    
    resources.load_image("squaddie", include_bytes!("../resources/unit/squaddie.png"));
    resources.load_image("machine",  include_bytes!("../resources/unit/machine.png"));
    
    resources.load_image("rifle_round",       include_bytes!("../resources/bullet/rifle_round.png"));
    resources.load_image("machine_gun_round", include_bytes!("../resources/bullet/machine_gun_round.png"));
    resources.load_image("plasma_round",      include_bytes!("../resources/bullet/plasma_round.png"));
    
    resources.load_image("cursor",            include_bytes!("../resources/cursor/default.png"));
    resources.load_image("cursor_unit",       include_bytes!("../resources/cursor/unit.png"));
    resources.load_image("cursor_unwalkable", include_bytes!("../resources/cursor/unwalkable.png"));
    resources.load_image("cursor_crosshair",  include_bytes!("../resources/cursor/crosshair.png"));
    
    resources.load_image("ruin_1", include_bytes!("../resources/ruin/1.png"));
    resources.load_image("ruin_2", include_bytes!("../resources/ruin/2.png"));
    resources.load_image("ruin_3", include_bytes!("../resources/ruin/3.png"));
    
    resources.load_image("pit_top",    include_bytes!("../resources/pit/top.png"));
    resources.load_image("pit_right",  include_bytes!("../resources/pit/right.png"));
    resources.load_image("pit_left",   include_bytes!("../resources/pit/left.png"));
    resources.load_image("pit_bottom", include_bytes!("../resources/pit/bottom.png"));
    resources.load_image("pit_tl",     include_bytes!("../resources/pit/tl.png"));
    resources.load_image("pit_tr",     include_bytes!("../resources/pit/tr.png"));
    resources.load_image("pit_bl",     include_bytes!("../resources/pit/bl.png"));
    resources.load_image("pit_br",     include_bytes!("../resources/pit/br.png"));
    resources.load_image("pit_center", include_bytes!("../resources/pit/center.png"));
    
    resources.load_image("path",             include_bytes!("../resources/path/default.png"));
    resources.load_image("path_no_weapon",   include_bytes!("../resources/path/no_weapon.png"));
    resources.load_image("path_unreachable", include_bytes!("../resources/path/unreachable.png"));
    
    resources.load_image("edge_left",         include_bytes!("../resources/edge/left.png"));
    resources.load_image("edge_right",        include_bytes!("../resources/edge/right.png"));
    resources.load_image("edge_left_corner",  include_bytes!("../resources/edge/left_corner.png"));
    resources.load_image("edge_right_corner", include_bytes!("../resources/edge/right_corner.png"));
    resources.load_image("edge_corner",       include_bytes!("../resources/edge/corner.png"));
        
    resources.load_image("end_turn_button",  include_bytes!("../resources/button/end_turn.png"));
    resources.load_image("inventory_button", include_bytes!("../resources/button/inventory.png"));

    resources.load_image("scrap",           include_bytes!("../resources/items/scrap.png"));
    resources.load_image("weapon",          include_bytes!("../resources/items/weapon.png"));
    resources.load_image("squaddie_corpse", include_bytes!("../resources/items/squaddie_corpse.png"));
    resources.load_image("machine_corpse",  include_bytes!("../resources/items/machine_corpse.png"));
    resources.load_image("skeleton",        include_bytes!("../resources/items/skeleton.png"));
    
    // Load the font
    resources.load_font("main", include_bytes!("../resources/font.ttf"), 35);

    // Start the game
    State::run(ctx, resources);
}
