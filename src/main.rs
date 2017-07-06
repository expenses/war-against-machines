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
extern crate rodio;

use piston::window::WindowSettings;
use piston::event_loop::{Events, EventSettings, EventLoop};
use piston::input::{Input, Key, Button, MouseButton, Motion};
use piston::window::Window;
use graphics::{Context, clear};
use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, OpenGL};

mod battle;
mod menu;
mod ui;
mod weapons;
#[macro_use]
mod utils;
#[macro_use]
mod resources;
mod colours;
mod items;
mod settings;

use colours::BLACK;
use battle::Battle;
use menu::{Menu, MenuCallback};
use resources::Resources;
use settings::Settings;
use battle::map::Map;

const TITLE: &str = "War Against Machines";

// Which mode the game is in
enum Mode {
    Menu,
    Skirmish
}

pub struct WindowSize {
    width: f64,
    height: f64
}

impl WindowSize {
    fn update(&mut self, window: &GlutinWindow) {
        let size = window.draw_size();
        self.width = size.width as f64;
        self.height = size.height as f64;
    }
}

// A struct for holding the game state
struct App {
    resources: Resources,
    mode: Mode,
    menu: menu::Menu,
    skirmish: Battle,
    window_size: WindowSize
}

impl App {
    // Create a new state, starting on the menu
    fn new(resources: Resources, settings: Settings) -> App {
        App {
            mode: Mode::Menu,
            menu: Menu::new(settings),
            skirmish: Battle::new(),
            window_size: WindowSize {width: 0.0, height: 0.0},
            resources,
        }
    }

    // Update the game
    fn update(&mut self, dt: f64, window: &GlutinWindow) -> bool {
        self.window_size.update(window);

        if let Mode::Skirmish = self.mode {
            if self.skirmish.update(&self.resources, dt).is_some() {
                self.mode = Mode::Menu;
                self.skirmish = Battle::new();
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
                        self.skirmish.start(&self.menu.skirmish_settings);
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
    fn handle_mouse_motion(&mut self, x: f64, y: f64) -> bool {
        if let Mode::Skirmish = self.mode {
            self.skirmish.move_cursor(x, y, &self.window_size);
        }

        true
    }

    // Handle mouse button presses
    fn handle_mouse_button(&mut self, button: MouseButton) -> bool {
        if let Mode::Skirmish = self.mode {
            self.skirmish.mouse_button(button, &self.window_size);
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
    let mut events = Events::new(EventSettings::new().ups(60));

    let resources = Resources::new(
        bytes!("tileset.png"),
        bytes!("font.ttf"), 22,
        [bytes!("audio/plasma.ogg"), bytes!("audio/walk.ogg")]
    );

    let mut app = App::new(resources, settings);

    while let Some(event) = events.next(&mut window) {
        let running = match event {
            Input::Press(Button::Keyboard(key)) => app.handle_key_press(key),
            Input::Press(Button::Mouse(button)) => app.handle_mouse_button(button),
            Input::Release(Button::Keyboard(key)) => app.handle_key_release(key),
            Input::Move(Motion::MouseCursor(x, y)) => app.handle_mouse_motion(x, y),
            Input::Render(args) => {
                gl.draw(args.viewport(), |ctx, gl| app.render(&ctx, gl));
                true
            },
            Input::Update(args) => app.update(args.dt, &window),
            _ => true
        };

        if !running {
            break;
        }
    }
}
