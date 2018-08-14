extern crate rand;
extern crate pathfinding;
extern crate ord_subset;
extern crate odds;
extern crate line_drawing;
#[macro_use]
extern crate derive_is_enum_variant;
extern crate toml;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate bincode;
extern crate image;
extern crate rodio;
#[macro_use]
extern crate glium;
extern crate either;
#[macro_use]
extern crate log;
extern crate env_logger;

use std::path::*;
use std::time::*;

// Lazy way to pretend glutin is a direct dependency
pub use glium::glutin;

pub use either::*;

use glutin::*;
use glutin::dpi::LogicalPosition;

#[macro_use]
mod ui;
mod weapons;
mod items;
#[macro_use]
mod resources;
#[macro_use]
mod utils;
mod settings;
mod menu;
mod colours;
mod context;
mod battle;
mod networking;

use context::Context;
use settings::*;
use menu::*;
use battle::*;

// Which mode the game is in
enum Mode {
    Menu,
    Skirmish
}

// A struct for holding the game state
struct App {
    ctx: Context,
    mode: Mode,
    menu: MainMenu,
    skirmish: Option<Battle>,
    mouse: (f32, f32)
}

impl App {
    // Create a new state, starting on the menu
    fn new(mut ctx: Context) -> App {
        App {
            mode: Mode::Menu,
            menu: MainMenu::new(&mut ctx.settings),
            skirmish: None,
            mouse: (0.0, 0.0),
            ctx
        }
    }

    // Update the game
    fn update(&mut self, dt: f32) {
        if let Mode::Skirmish = self.mode {
            if let Some(ref mut skirmish) = self.skirmish {
                skirmish.update(&mut self.ctx, dt);
            }
        }
    }

    // Handle key presses
    fn handle_key_press(&mut self, key: VirtualKeyCode) -> bool {
        match self.mode {
            // If the mode is the menu, respond to callbacks
            Mode::Menu => if let Some(callback) = self.menu.handle_key(key, &mut self.ctx.settings) {
                match callback {
                    // Generate a new skirmish
                    MenuCallback::NewSkirmish => {
                        self.mode = Mode::Skirmish;
                        self.skirmish = Battle::new_singleplayer(Left(self.menu.skirmish_settings));
                    },
                    // Load a saved skirmish
                    MenuCallback::LoadSkirmish(filename) => {
                        let path = Path::new(&self.ctx.settings.savegames).join(&filename);

                        self.skirmish = Battle::new_singleplayer(Right(&path.as_path()));
                        if self.skirmish.is_some() {
                            self.mode = Mode::Skirmish;
                        } else {
                            println!("File '{}' could not be read", path.display());
                        }
                    },
                    MenuCallback::HostServer(addr) => {
                        self.skirmish = Battle::new_multiplayer_host(&addr, Left(self.menu.skirmish_settings));

                        if self.skirmish.is_some() {
                            self.mode = Mode::Skirmish;
                        } else {
                            // todo: print an error here
                            println!("Something went wrong in trying to host the server.");
                        }
                    },
                    MenuCallback::ConnectServer(addr) => {
                        self.skirmish = Battle::new_multiplayer_connect(&addr);

                        if self.skirmish.is_some() {
                            self.mode = Mode::Skirmish;
                        } else {
                            // todo: print an error here
                            println!("Something went wrong in trying to connect to the server.");
                        }
                    }
                    MenuCallback::Resume => self.mode = Mode::Skirmish,
                    // Quit
                    MenuCallback::Quit => return false
                }  
            },
            // If the skirmish returns false for a key press, switch to the menu
            Mode::Skirmish => if let Some(ref mut skirmish) = self.skirmish {
                if !skirmish.handle_key(&self.ctx.settings, key, true) {
                    self.mode = Mode::Menu;
                    self.menu.refresh(true);
                }
            }
        }

        true
    }

    // Handle key releases
    fn handle_key_release(&mut self, key: VirtualKeyCode) {
        if let Mode::Skirmish = self.mode {
            if let Some(ref mut skirmish) = self.skirmish {
                skirmish.handle_key(&self.ctx.settings, key, false);
            }
        }
    }

    // Handle mouse movement
    fn handle_mouse_motion(&mut self, x: f32, y: f32) {
        // Convert the coordinates
        let (x, y) = (
            x - self.ctx.width / 2.0,
            self.ctx.height / 2.0 - y
        );

        self.mouse = (x, y);

        if let Mode::Skirmish = self.mode {
            if let Some(ref mut skirmish) = self.skirmish {
                skirmish.move_cursor(x, y);
            }
        }
    }

    // Handle mouse button presses
    fn handle_mouse_button(&mut self, button: MouseButton) {
        if let Mode::Skirmish = self.mode {
            if let Some(ref mut skirmish) = self.skirmish {
                skirmish.mouse_button(button, self.mouse, &self.ctx);
            }
        }
    }

    // Clear, draw and present the context
    fn render(&mut self) {
        self.ctx.clear();

        match self.mode {
            Mode::Skirmish => if let Some(ref mut skirmish) = self.skirmish {
                skirmish.draw(&mut self.ctx);
            },
            Mode::Menu => self.menu.render(&mut self.ctx),
        }

        self.ctx.flush();
    }

    // Resize the context
    fn resize(&mut self, width: u32, height: u32) {
        self.ctx.resize(width, height);
    }
}

// The main function
fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .init();

    // Generate the event loop and the context
    let mut events_loop = EventsLoop::new();
    let ctx = Context::new(&events_loop, Settings::load());

    let mut app = App::new(ctx);
    let mut running = true;
    let mut start = Instant::now();

    while running {
        // Poll the window events
        events_loop.poll_events(|event| if let Event::WindowEvent {event, ..} = event {
            match event {
                WindowEvent::CloseRequested => running = false,
                WindowEvent::KeyboardInput {input: KeyboardInput {state, virtual_keycode: Some(key), ..}, ..} => {
                    match state {
                        ElementState::Pressed => running = app.handle_key_press(key),
                        ElementState::Released => app.handle_key_release(key)
                    }
                },

                WindowEvent::CursorMoved {position: LogicalPosition {x, y}, ..} => app.handle_mouse_motion(x as f32, y as f32),
                WindowEvent::MouseInput {state: ElementState::Pressed, button, ..} => app.handle_mouse_button(button),
                WindowEvent::Resized(size) => app.resize(size.width as u32, size.height as u32),
                _ => {},
            };
        });

        // Update the game with the delta time in seconds (divided by 1 billion)

        let now = Instant::now();
        let ns = now.duration_since(start).subsec_nanos();
        start = now;

        app.update(ns as f32 / 1_000_000_000.0);

        // Render the game

        app.render();
    }
}
