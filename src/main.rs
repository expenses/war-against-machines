extern crate rand;
extern crate pathfinding;
extern crate ord_subset;
extern crate odds;
extern crate toml;
#[macro_use]
extern crate serde_derive;
extern crate bincode;
extern crate piston;
extern crate graphics;
extern crate opengl_graphics;
extern crate glutin_window;
extern crate image;

use piston::window::WindowSettings;
use piston::event_loop::{Events, EventLoop, EventSettings};
use piston::input::{Input, Key, Button, UpdateArgs, MouseButton};
use graphics::{Context, clear};
use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, OpenGL};

mod battle;
mod menu;
mod ui;
mod weapons;
#[macro_use]
mod utils;
mod resources;
mod constants;
mod items;
mod settings;

use constants::BLACK;
use battle::Battle;
use menu::{Menu, MenuCallback};
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
struct App {
    resources: Resources,
    mode: Mode,
    menu: menu::Menu,
    skirmish: Battle,
}

impl App {
    // Create a new state, starting on the menu
    fn new(resources: Resources, settings: Settings) -> App {
        App {
            mode: Mode::Menu,
            menu: Menu::new(settings),
            skirmish: Battle::new(&resources),
            resources
        }
    }

    // Update the game
    fn update(&mut self, _dt: f64) -> bool {
        if let Mode::Skirmish = self.mode {
            if self.skirmish.update(&self.resources).is_some() {
                self.mode = Mode::Menu;
                self.skirmish = Battle::new(&self.resources);
            }
        }

        true
    }

    // Handle key presses
    fn handle_key_press(&mut self, key: Key) -> bool {
        match self.mode {
            // If the mode is the menu, respond to callbacks
            Mode::Menu => if let Some(callback) = self.menu.handle_key(key) {
                match callback {
                    MenuCallback::NewSkirmish => {
                        self.mode = Mode::Skirmish;
                        //self.skirmish.start(&self.menu.skirmish_settings);
                    },
                    MenuCallback::LoadSkirmish(filename) => {
                        if let Some(map) = Map::load_skirmish(&filename) {
                            self.skirmish.map = map;
                            self.mode = Mode::Skirmish;
                        }
                    },
                    MenuCallback::Quit => return false
                }  
            },
            Mode::Skirmish => return self.skirmish.handle_key(key, true)
        }

        true
    }

    // Handle key releases
    fn handle_key_release(&mut self, key: Key) -> bool {
        if let Mode::Skirmish = self.mode {
            return self.skirmish.handle_key(key, false);
        }

        true
    }

    // Handle mouse movement
    fn _handle_mouse_motion(&mut self, _x: i32, _y: i32) {
        if let Mode::Skirmish = self.mode {
            //self.skirmish.move_cursor(x as f32, y as f32);
        }
    }

    // Handle mouse button presses
    fn _handle_mouse_button(&mut self, _button: MouseButton, _x: i32, _y: i32) -> bool {
        if let Mode::Skirmish = self.mode {
            //self.skirmish.mouse_button(button, x as f32, y as f32);
        }

        true
    }

    // Clear, draw and present the canvas
    fn render(&mut self, ctx: &Context, gl: &mut GlGraphics) {
        clear(BLACK, gl);

        match self.mode {
            Mode::Skirmish => self.skirmish.draw(ctx, gl, &mut self.resources),
            Mode::Menu => self.menu.render(ctx, gl, &mut self.resources)
        }
    }
}

// The main function
fn main() {
    // Set opengl version
    let opengl = OpenGL::V3_2;

    // Load (or use the default) settings
    let settings = Settings::load();

    let mut window: GlutinWindow = WindowSettings::new(TITLE, (settings.width, settings.height))
        .vsync(true)
        .opengl(opengl)
        .build()
        .unwrap();

    let mut gl = GlGraphics::new(opengl);
    let mut events = Events::new(EventSettings::new());

    let mut resources = Resources::new("resources/tileset.png", "resources/font.ttf");

    resources.load_image("title", "resources/title.png");
    resources.load_image("end_turn_button", "resources/button/end_turn.png");
    resources.load_image("inventory_button", "resources/button/inventory.png");
    resources.load_image("change_fire_mode_button", "resources/button/change_fire_mode.png");

    let mut app = App::new(resources, settings);

    while let Some(event) = events.next(&mut window) {
        let running = match event {
            Input::Press(Button::Keyboard(key)) => app.handle_key_press(key),
            Input::Release(Button::Keyboard(key)) => app.handle_key_release(key),
            Input::Render(args) => {
                gl.draw(args.viewport(), |ctx, gl| app.render(&ctx, gl));
                true
            },
            Input::Update(args) => app.update(args.dt),
            _ => true
        };

        if !running {
            break;
        }
    }
}
