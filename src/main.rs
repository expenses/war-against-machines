extern crate line_drawing;
extern crate odds;
extern crate ord_subset;
extern crate pathfinding;
extern crate rand;
#[macro_use]
extern crate derive_is_enum_variant;
extern crate serde;
extern crate toml;
#[macro_use]
extern crate serde_derive;
extern crate bincode;
extern crate glium;
extern crate image;
extern crate rodio;
#[macro_use]
extern crate log;
extern crate env_logger;
#[macro_use]
extern crate error_chain;
extern crate pedot;
extern crate runic;
extern crate skynet;

use std::time::*;

use glium::glutin;
use glutin::dpi::LogicalPosition;
use glutin::*;

#[macro_use]
mod ui;
mod items;
mod weapons;
#[macro_use]
mod resources;
mod battle;
mod colours;
mod context;
mod error;
mod menu;
mod networking;
mod settings;
mod utils;

use battle::*;
use context::Context;
use error::*;
use menu::*;
use settings::*;

// Which mode the game is in
enum Mode {
    Menu,
    Skirmish,
}

// A struct for holding the game state
struct App {
    ctx: Context,
    mode: Mode,
    menu: MainMenu,
    skirmish: Option<Battle>,
    mouse: (f32, f32),
}

impl App {
    // Create a new state, starting on the menu
    fn new(events_loop: &EventsLoop, settings: Settings) -> Self {
        let mut ctx = Context::new(events_loop, settings);

        Self {
            mode: Mode::Menu,
            menu: MainMenu::new(&mut ctx),
            skirmish: None,
            mouse: (0.0, 0.0),
            ctx,
        }
    }

    // Update the game
    fn update(&mut self, dt: f32) -> bool {
        match self.mode {
            Mode::Skirmish => {
                if let Some(ref mut skirmish) = self.skirmish {
                    skirmish.update(&mut self.ctx, dt);
                }
            }
            Mode::Menu => {
                if let Some(callback) = self.menu.update(&mut self.ctx, self.skirmish.is_some()) {
                    match callback {
                        // Generate a new skirmish
                        MenuCallback::NewSkirmish(settings) => {
                            match Battle::new_from_settings(settings, self.ctx.settings.clone()) {
                                Ok(skirmish) => {
                                    self.mode = Mode::Skirmish;
                                    self.skirmish = Some(skirmish);
                                }
                                Err(error) => display_error(&error),
                            }
                        }
                        MenuCallback::Resume => self.mode = Mode::Skirmish,
                        // Quit
                        MenuCallback::Quit => return false,
                    }
                }
            }
        }

        true
    }

    // Handle key presses
    fn handle_key_press(&mut self, key: VirtualKeyCode) -> bool {
        if let Mode::Skirmish = self.mode {
            let key_response = self
                .skirmish
                .as_mut()
                .map(|skirmish| skirmish.handle_key(key, true));

            if let Some(response) = key_response {
                match response {
                    KeyResponse::GameOver => {
                        self.mode = Mode::Menu;
                        self.menu.reset_submenu();
                        self.skirmish = None;
                    }
                    KeyResponse::OpenMenu => {
                        self.mode = Mode::Menu;
                    }
                    _ => {}
                }
            }
        }

        true
    }

    // Handle key releases
    fn handle_key_release(&mut self, key: VirtualKeyCode) {
        if let Mode::Skirmish = self.mode {
            if let Some(ref mut skirmish) = self.skirmish {
                skirmish.handle_key(key, false);
            }
        }
    }

    // Handle mouse movement
    fn handle_mouse_motion(&mut self, x: f32, y: f32) {
        self.mouse = (x, y);

        if let Mode::Skirmish = self.mode {
            if let Some(ref mut skirmish) = self.skirmish {
                skirmish.move_cursor(x, y, &self.ctx);
            }
        }
    }

    // Handle mouse button presses
    fn handle_mouse_button(&mut self, button: MouseButton) {
        if let Mode::Skirmish = self.mode {
            if let Some(ref mut skirmish) = self.skirmish {
                skirmish.mouse_button(button, &self.ctx);
            }
        }
    }

    // Clear, draw and present the context
    fn render(&mut self) {
        self.ctx.clear();

        match self.mode {
            Mode::Skirmish => {
                if let Some(ref mut skirmish) = self.skirmish {
                    skirmish.draw(&mut self.ctx);
                }
            }
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
    let env = env_logger::Env::new().filter_or("RUST_LOG", "info");
    env_logger::init_from_env(env);

    // Create the app
    let settings = Settings::load();
    let mut events_loop = EventsLoop::new();
    let mut app = App::new(&events_loop, settings);

    let mut start = Instant::now();
    let mut running = true;
    while running {
        app.ctx.clear_gui();

        // Poll the window events
        events_loop.poll_events(|event| {
            if let Event::WindowEvent { event, .. } = event {
                app.ctx.update_gui(&event);

                match event {
                    WindowEvent::CloseRequested => running = false,
                    // Respond to key presses / releases
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state,
                                virtual_keycode: Some(key),
                                ..
                            },
                        ..
                    } => match state {
                        ElementState::Pressed => running = app.handle_key_press(key),
                        ElementState::Released => app.handle_key_release(key),
                    },

                    // Respond to cursor movements
                    WindowEvent::CursorMoved {
                        position: LogicalPosition { x, y },
                        ..
                    } => app.handle_mouse_motion(x as f32, y as f32),
                    // Respond to mouse clicks
                    WindowEvent::MouseInput {
                        state: ElementState::Pressed,
                        button,
                        ..
                    } => app.handle_mouse_button(button),
                    // Respond to resize events
                    WindowEvent::Resized(size) => app.resize(size.width as u32, size.height as u32),
                    _ => {}
                };
            }
        });

        // Get delta time
        let now = Instant::now();
        let ns = now.duration_since(start).subsec_nanos();
        // Convert nanoseconds to milliseconds
        let ms = ns as f32 / 1_000_000_000.0;
        start = now;

        running &= app.update(ms);
        app.render();
    }
}
