extern crate sdl2;
extern crate rand;
extern crate pathfinding;
extern crate ord_subset;
extern crate odds;
extern crate toml;
#[macro_use]
extern crate serde_derive;
extern crate bincode;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::rwops::RWops;

mod battle;
mod menu;
mod ui;
mod weapons;
mod context;
#[macro_use]
mod utils;
#[macro_use]
mod resources;
mod colours;
mod items;
mod settings;

use context::Context;
use battle::Battle;
use menu::Callback;
use resources::Resources;
use settings::Settings;
use battle::map::Map;

pub const TITLE: &str = "War Against Machines";

// Which mode the game is in
enum Mode {
    Menu,
    Skirmish
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
    fn run(ctx: Context, resources: Resources<'a>, settings: Settings) {
        let mut state = State {
            mode: Mode::Menu,
            menu: menu::Menu::new(settings),
            skirmish: Battle::new(&resources),
            ctx, resources
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
        if let Mode::Skirmish = self.mode {
            self.skirmish.update(&self.resources);
        }
    }

    // Handle key presses
    fn handle_key_down(&mut self, key: Keycode) {
        match self.mode {
            // If the mode is the menu, respond to callbacks
            Mode::Menu => if let Some(callback) = self.menu.handle_key(&mut self.ctx, key) {
                match callback {
                    Callback::NewSkirmish => {
                        self.mode = Mode::Skirmish;
                        self.skirmish.start(&self.menu.skirmish_settings);
                    },
                    Callback::LoadSkirmish => {
                        if let Some(map) = Map::load() {
                            self.skirmish.map = map;
                            self.mode = Mode::Skirmish;
                        }
                    }
                }  
            },
            Mode::Skirmish => self.skirmish.handle_key(&mut self.ctx, key, true)
        }
    }

    // Handle key releases
    fn handle_key_up(&mut self, key: Keycode) {
        if let Mode::Skirmish = self.mode {
            self.skirmish.handle_key(&mut self.ctx, key, false);
        }
    }

    // Handle mouse movement
    fn handle_mouse_motion(&mut self, x: i32, y: i32) {
        if let Mode::Skirmish = self.mode {
            self.skirmish.move_cursor(&mut self.ctx, x as f32, y as f32);
        }
    }

    // Handle mouse button presses
    fn handle_mouse_button(&mut self, button: MouseButton, x: i32, y: i32) {
        if let Mode::Skirmish = self.mode {
            self.skirmish.mouse_button(&mut self.ctx, button, x as f32, y as f32);
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
    // Load (or use the default) settings
    let settings = Settings::load();

    // Create the context
    let mut ctx = Context::new(settings.width, settings.height);

    // Set the settings
    ctx.set(&settings);

    let texture_creator = ctx.texture_creator();
    let font_context = ctx.font_context();

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
    
    resources.load_image("left_edge",  rw_ops!("edge/left.png"));
    resources.load_image("right_edge", rw_ops!("edge/right.png"));
        
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

    // Load audio resources
    resources.load_audio("plasma",  rw_ops!("audio/plasma.ogg"));
    resources.load_audio("walk",    rw_ops!("audio/walk.ogg"));

    // Start the game
    State::run(ctx, resources, settings);
}
