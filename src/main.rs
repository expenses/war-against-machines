extern crate rand;
extern crate pathfinding;
extern crate ord_subset;
extern crate odds;
extern crate toml;
#[macro_use]
extern crate serde_derive;
extern crate bincode;
#[macro_use]
extern crate gfx;
extern crate gfx_device_gl;
extern crate glutin;
extern crate gfx_window_glutin;
extern crate image;
extern crate rodio;

use std::time::Instant;

use glutin::{Event, WindowEvent, ElementState, VirtualKeyCode, MouseButton};

mod battle;
mod weapons;
mod items;
mod ui;
#[macro_use]
mod resources;
#[macro_use]
mod utils;
mod settings;
mod menu;
mod colours;
mod context;

use context::Context;
use settings::Settings;
use menu::{Menu, MenuCallback};
use battle::Battle;
use battle::map::Map;

const TITLE: &str = "War Against Machines";
const WINDOW_WIDTH: u32 = 960;
const WINDOW_HEIGHT: u32 = 540;

// Which mode the game is in
enum Mode {
    Menu,
    Skirmish
}

// A struct for holding the game state
struct App {
    ctx: Context,
    mode: Mode,
    menu: menu::Menu,
    skirmish: Battle,
    mouse: (f32, f32)
}

impl App {
    // Create a new state, starting on the menu
    fn new(ctx: Context, settings: Settings) -> App {
        App {
            mode: Mode::Menu,
            menu: Menu::new(settings),
            skirmish: Battle::new(&ctx),
            mouse: (0.0, 0.0),
            ctx
        }
    }

    // Update the game
    fn update(&mut self, dt: f32) {
        if let Mode::Skirmish = self.mode {
            if self.skirmish.update(&self.ctx, dt).is_some() {
                self.mode = Mode::Menu;
                self.skirmish = Battle::new(&self.ctx);
            }
        }
    }

    // Handle key presses
    fn handle_key_press(&mut self, key: VirtualKeyCode) -> bool {
        match self.mode {
            // If the mode is the menu, respond to callbacks
            Mode::Menu => if let Some(callback) = self.menu.handle_key(key, &mut self.ctx) {
                match callback {
                    MenuCallback::NewSkirmish => {
                        self.mode = Mode::Skirmish;
                        self.skirmish.start(&self.menu.skirmish_settings);
                    },
                    MenuCallback::LoadSkirmish(filename) => {
                        if let Some(map) = Map::load(&filename) {
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
    fn handle_key_release(&mut self, key: VirtualKeyCode) {
        if let Mode::Skirmish = self.mode {
            self.skirmish.handle_key(key, false);
        }
    }

    // Handle mouse movement
    fn handle_mouse_motion(&mut self, x: i32, y: i32) {
        let (x, y) = (x as f32 - self.ctx.width / 2.0, self.ctx.height / 2.0 - y as f32);

        self.mouse = (x, y);

        if let Mode::Skirmish = self.mode {
            self.skirmish.move_cursor(x, y);
        }
    }

    // Handle mouse button presses
    fn handle_mouse_button(&mut self, button: MouseButton) {
        if let Mode::Skirmish = self.mode {
            self.skirmish.mouse_button(button, self.mouse, &self.ctx);
        }
    }

    // Clear, draw and present the canvas
    fn render(&mut self) {
        self.ctx.clear();

        match self.mode {
            Mode::Skirmish => self.skirmish.draw(&mut self.ctx),
            Mode::Menu => self.menu.render(&mut self.ctx),
        }

        self.ctx.flush();
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.ctx.resize(width as f32, height as f32);
    }
}

// The main function
fn main() {
    // Load (or use the default) settings
    let settings = Settings::load();

    let events_loop = glutin::EventsLoop::new();
    let mut ctx = Context::new(
        &events_loop,
        TITLE.into(), WINDOW_WIDTH, WINDOW_HEIGHT,
        bytes!("tileset.png"),
        [
            bytes!("audio/walk.ogg"),
            bytes!("audio/regular_shot.ogg"),
            bytes!("audio/plasma_shot.ogg")
        ]
    );
    ctx.set(&settings);

    let mut app = App::new(ctx, settings);
    let mut running = true;
    let mut start = Instant::now();

    while running {
        events_loop.poll_events(|Event::WindowEvent{event, ..}| {
            match event {
                WindowEvent::Closed => running = false,
                WindowEvent::KeyboardInput(ElementState::Pressed, _, Some(key), _) => running = app.handle_key_press(key),
                WindowEvent::KeyboardInput(ElementState::Released, _, Some(key), _) => app.handle_key_release(key),
                WindowEvent::MouseMoved(x, y) => app.handle_mouse_motion(x, y),
                WindowEvent::MouseInput(ElementState::Pressed, button) => app.handle_mouse_button(button),
                WindowEvent::Resized(width, height) => app.resize(width, height),
                _ => {},
            };
        });

        let now = Instant::now();
        let ns = now.duration_since(start).subsec_nanos();
        start = now;

        // Update the game with the delta time in seconds (divided by 1 billion)
        app.update(ns as f32 / 1_000_000_000.0);

        app.render();
    }
}
