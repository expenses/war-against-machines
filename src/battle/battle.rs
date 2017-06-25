//! The main battle struct the handles actions

use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;

use std::fmt;

use battle::drawer::Drawer;
use battle::paths::{pathfind, PathPoint};
use battle::animations::Animations;
use battle::commands::{CommandQueue, Command, FireCommand, WalkCommand};
use battle::units::{Unit, UnitSide};
use battle::map::Map;
use battle::ai;
use context::Context;
use Resources;
use ui::{UI, Button, TextDisplay, VerticalAlignment, HorizontalAlignment};
use menu::SkirmishSettings;

const CAMERA_SPEED: f32 = 0.2;
const CAMERA_ZOOM_SPEED: f32 = 0.02;

/// A cursor on the map with a possible position
pub struct Cursor {
    pub position: Option<(usize, usize)>
}

/// What controls the actions
#[derive(Eq, PartialEq)]
pub enum Controller {
    Player,
    AI
}

impl fmt::Display for Controller {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            &Controller::Player => "Player",
            &Controller::AI => "AI"
        })
    }
}

/// The Battle struct
pub struct Battle {
    pub map: Map,
    drawer: Drawer,
    pub cursor: Cursor,
    keys: [bool; 6],
    pub selected: Option<usize>,
    pub path: Option<Vec<PathPoint>>,
    turn: u16,
    ui: UI,
    pub animations: Animations,
    pub command_queue: CommandQueue,
    controller: Controller
}

impl Battle {
    /// Create a new `Battle`
    pub fn new(resources: &Resources) -> Battle {
        let scale = 2.0;

        // Create the UI and add the buttons and text display

        let ui = UI::new(
            vec![
                Button::new(
                    "end_turn_button".into(),
                    0.0,
                    0.0,
                    scale,
                    resources,
                    VerticalAlignment::Right,
                    HorizontalAlignment::Bottom
                )
            ],
            vec![
                TextDisplay::new(0.0, 0.0, VerticalAlignment::Middle, HorizontalAlignment::Top)
            ]
        );

        Battle {
            map: Map::new(),
            drawer: Drawer::new(),
            cursor: Cursor { position: None },
            keys: [false; 6],
            selected: None,
            path: None,
            turn: 1,
            ui: ui,
            animations: Animations::new(),
            command_queue: CommandQueue::new(),
            controller: Controller::Player
        }
    }

    /// Start up the map
    pub fn start(&mut self, settings: &SkirmishSettings) {
        // Add player units
        for x in 0 .. settings.player_units {
            self.map.units.push(Unit::new(settings.player_unit_type, UnitSide::Player, x, 0));
        }

        // Add ai units
        for y in settings.cols - settings.ai_units .. settings.cols {
            self.map.units.push(Unit::new(settings.ai_unit_type, UnitSide::AI, y, settings.rows - 1));
        }
        
        // Generate tiles
        self.map.tiles.generate(settings.cols, settings.rows, &self.map.units);
    }

    /// Handle keypresses
    pub fn handle_key(&mut self, ctx: &mut Context, key: Keycode, pressed: bool) {
        match key {
            Keycode::Up    | Keycode::W => self.keys[0] = pressed,
            Keycode::Down  | Keycode::S => self.keys[1] = pressed,
            Keycode::Left  | Keycode::A => self.keys[2] = pressed,
            Keycode::Right | Keycode::D => self.keys[3] = pressed,
            Keycode::O                  => self.keys[4] = pressed,
            Keycode::P                  => self.keys[5] = pressed,
            Keycode::Escape             => ctx.quit(),
            _ => {}
        };
    }

    /// Update the battle
    pub fn update(&mut self) {
        // Change camera variables if a key is being pressed
        if self.keys[0] { self.drawer.camera.y -= CAMERA_SPEED; }
        if self.keys[1] { self.drawer.camera.y += CAMERA_SPEED; }
        if self.keys[2] { self.drawer.camera.x -= CAMERA_SPEED; }
        if self.keys[3] { self.drawer.camera.x += CAMERA_SPEED; }
        if self.keys[4] { self.drawer.zoom(-CAMERA_ZOOM_SPEED) }
        if self.keys[5] { self.drawer.zoom(CAMERA_ZOOM_SPEED) }

        if self.controller == Controller::AI && self.command_queue.is_empty() && self.animations.is_empty() {
            if !ai::make_move(&self.map, &mut self.command_queue) {
                self.controller = Controller::Player;
                self.turn += 1;
            }
        }


        if self.animations.is_empty() {
            self.command_queue.update(&mut self.map, &mut self.animations);
        }

        // Update the animation queue
        self.animations.update(&mut self.map);
    }

    /// Draw both the map and the UI
    pub fn draw(&mut self, ctx: &mut Context, resources: &Resources) {
        self.drawer.draw_battle(ctx, resources, &self);
        self.draw_ui(ctx, resources);
    }

    /// Draw the UI
    pub fn draw_ui(&mut self, ctx: &mut Context, resources: &Resources) {
        // Get a string of info about the selected unit
        let selected = match self.selected.and_then(|id| self.map.units.get(id)) {
            Some(unit) => {
                if unit.side == UnitSide::Player {
                    format!(
                        "(Name: {}, Moves: {}, Health: {}, Weapon: {})",
                        unit.name, unit.moves, unit.health, unit.weapon.name
                    )
                } else {
                    format!("(Name: {})", unit.name)
                }
            },
            _ => "~".into()
        };

        // Set the text of the UI text display
        self.ui.set_text(0, format!("Turn: {} - {}, Selected: {}", self.turn, self.controller, selected));

        // Draw the UI
        self.ui.draw(ctx, resources);
    }

    /// Move the cursor on the screen
    pub fn move_cursor(&mut self, ctx: &mut Context, x: f32, y: f32) {
        // Get the position where the cursor should be
        let (x, y) = self.drawer.tile_under_cursor(ctx, x, y);

        // Set cursor position if it is on the map and visible
        self.cursor.position = if x < self.map.tiles.cols && y < self.map.tiles.rows && self.map.tiles.at(x, y).visible() {
            Some((x, y))
        } else {
            None
        }
    }

    /// Respond to mouse presses
    pub fn mouse_button(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        match button {
            MouseButton::Left => match self.ui.clicked(ctx, x, y) {
                // Respond to UI clicks
                Some(0) => self.end_turn(),
                // Or check if cursor position
                _ => {
                    if let Some((x, y)) = self.cursor.position {
                        self.path = None;
                        self.selected = self.map.units.at_i(x, y);
                    }
                }
            },
            // Check if the cursor has a position and a unit is selected
            MouseButton::Right => if let Some((x, y, player_unit_id)) = self.cursor.position.and_then(|(x, y)|
                                                                        self.selected.map(|player_unit_id| (x, y, player_unit_id))) {
                match self.map.units.at_i(x, y) {
                    Some(ai_unit_id) => {
                        if self.map.units.get(ai_unit_id).unwrap().side == UnitSide::AI {
                            self.path = None;
                            self.command_queue.push(Command::Fire(FireCommand::new(player_unit_id, ai_unit_id)));
                        }
                    }
                    _ => {
                        let unit = self.map.units.get(player_unit_id).unwrap();

                        // Do nothing if the unit isn't a player unit
                        if unit.side != UnitSide::Player {
                            return;
                        // Or the the target location is selected
                        } else if self.map.taken(x, y) {
                            self.path = None;
                            return;
                        }

                        // Pathfind to get the path points and the cost
                        let points = match pathfind(unit, x, y, &self.map) {
                            Some((points, _)) => points,
                            _ => {
                                self.path = None;
                                return;
                            }
                        };

                        // Is the path is the same as existing one?
                        let same_path = match self.path {
                            Some(ref path) => path[path.len() - 1].at(x, y),
                            _ => false
                        };

                        // If the paths are the same and the player unit can move to the destination, get rid of the path
                        self.path = if same_path {
                            self.command_queue.push(Command::Walk(WalkCommand::new(player_unit_id, &self.map, points)));
                            None
                        } else {
                            Some(points)
                        }
                    }
                }
            },
            _ => {}
        }
    }

    /// Work out if the cursor is on an ai unit
    pub fn cursor_on_ai_unit(&self) -> bool {
        self.cursor.position
            .and_then(|(x, y)| self.map.units.at(x, y))
            .map(|(_, unit)| unit.side == UnitSide::AI)
            .unwrap_or(false)
    }

    /// End the current turn
    fn end_turn(&mut self) {
        if self.controller == Controller::Player {
            for (_, unit) in self.map.units.iter_mut() {
                unit.moves = unit.max_moves;
            }

            self.controller = Controller::AI;
        }   
    }
}