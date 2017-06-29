// A battle in the game

pub mod units;
mod drawer;
mod map;
mod tiles;
mod paths;
mod animations;
mod ai;
mod commands;

use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;

use std::fmt;

use battle::drawer::Drawer;
use battle::paths::{pathfind, PathPoint};
use battle::animations::Animations;
use battle::commands::{CommandQueue, Command, FireCommand, WalkCommand};
use battle::units::{Unit, UnitSide};
use battle::map::Map;
use context::Context;
use Resources;
use ui::{UI, Button, TextDisplay, VerticalAlignment, HorizontalAlignment};
use menu::SkirmishSettings;

const CAMERA_SPEED: f32 = 0.2;
const CAMERA_ZOOM_SPEED: f32 = 0.02;

// A cursor on the map with a possible position
pub struct Cursor {
    pub position: Option<(usize, usize)>
}

// Whose turn is it
#[derive(Eq, PartialEq)]
enum Controller {
    Player,
    AI
}

impl fmt::Display for Controller {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            Controller::Player => "Player",
            Controller::AI => "AI"
        })
    }
}

// The main Battle struct the handles actions
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
    // Create a new Battle
    pub fn new(resources: &Resources) -> Battle {
        let scale = 2.0;
        let width_offset = resources.image("end_turn_button").query().width as f32 * -scale;

        // Create the UI and add the buttons and text display

        let ui = UI::new(
            vec![
                Button::new(
                    "end_turn_button",
                    0.0,
                    0.0,
                    scale,
                    resources,
                    VerticalAlignment::Right,
                    HorizontalAlignment::Bottom
                ),
                Button::new(
                    "inventory_button",
                    width_offset,
                    0.0,
                    scale,
                    resources,
                    VerticalAlignment::Right,
                    HorizontalAlignment::Bottom
                ),
                Button::new(
                    "change_fire_mode_button",
                    width_offset * 2.0,
                    0.0,
                    scale,
                    resources,
                    VerticalAlignment::Right,
                    HorizontalAlignment::Bottom
                )
            ],
            vec![
                TextDisplay::new(0.0, 0.0, VerticalAlignment::Middle, HorizontalAlignment::Top, true, "-"),
                TextDisplay::new(0.0, -100.0, VerticalAlignment::Middle, HorizontalAlignment::Middle, false, "-")
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

    // Start up the map
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

    // Handle keypresses
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

    // Update the battle
    pub fn update(&mut self) {
        // Change camera variables if a key is being pressed
        if self.keys[0] { self.drawer.camera.y -= CAMERA_SPEED; }
        if self.keys[1] { self.drawer.camera.y += CAMERA_SPEED; }
        if self.keys[2] { self.drawer.camera.x -= CAMERA_SPEED; }
        if self.keys[3] { self.drawer.camera.x += CAMERA_SPEED; }
        if self.keys[4] { self.drawer.zoom(-CAMERA_ZOOM_SPEED) }
        if self.keys[5] { self.drawer.zoom(CAMERA_ZOOM_SPEED) }

        if self.controller == Controller::AI &&
           self.command_queue.is_empty() &&
           self.animations.is_empty() &&
           !ai::make_move(&self.map, &mut self.command_queue) {
            self.controller = Controller::Player;
            self.turn += 1;
        }

        if self.animations.is_empty() {
            self.command_queue.update(&mut self.map, &mut self.animations);
        }

        // Update the animation queue
        self.animations.update(&mut self.map);
    }

    // Draw both the map and the UI
    pub fn draw(&mut self, ctx: &mut Context, resources: &Resources) {
        self.drawer.draw_battle(ctx, resources, self);
        self.draw_ui(ctx, resources);
    }

    // Draw the UI
    fn draw_ui(&mut self, ctx: &mut Context, resources: &Resources) {
        // Get a string of info about the selected unit
        let selected = match self.selected_unit() {
            Some(unit) => format!(
                "Name: {}, Moves: {}, Health: {}\nWeapon: {}",
                unit.name, unit.moves, unit.health, unit.weapon
            ),
            _ => String::new()
        };

        // Set the text of the UI text display
        self.ui.set_text(0, format!("Turn {} - {}\n{}", self.turn, self.controller, selected));

        // Create the inventory string
        let inventory_string = match self.selected_unit() {
            Some(unit) => {
                let mut string = String::new();

                // Add unit items
                if !unit.inventory.is_empty() {
                    string.push_str(&format!("Inventory for {}:\n", unit.name));

                    for item in &unit.inventory {
                        string.push_str(&format!("{}\n", item));
                    }
                } else {
                    string.push_str(&format!("{} has an empty inventory\n", unit.name));
                }
                
                // Add tile items
                let tile = self.map.tiles.at(unit.x, unit.y);

                if !tile.items.is_empty() {
                    string.push_str("Items on the ground:");

                    for item in &tile.items {
                        string.push_str(&format!("\n{}", item));
                    }
                } else {
                    string.push_str("There are no items on the ground");
                }

                string
            }
            _ => "No unit selected".into()
        };
        
        self.ui.set_text(1, inventory_string);

        // Draw the UI
        self.ui.draw(ctx, resources);
    }

    // Move the cursor on the screen
    pub fn move_cursor(&mut self, ctx: &mut Context, x: f32, y: f32) {
        // Get the position where the cursor should be
        let (x, y) = self.drawer.tile_under_cursor(ctx, x, y);

        // Set cursor position if it is on the map and visible
        self.cursor.position = if x < self.map.tiles.cols &&
                                  y < self.map.tiles.rows &&
                                  self.map.tiles.at(x, y).visible() {
            Some((x, y))
        } else {
            None
        }
    }

    // Respond to mouse presses
    pub fn mouse_button(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        match button {
            MouseButton::Left => match self.ui.clicked(ctx, x, y) {
                // End the turn
                Some(0) => self.end_turn(),
                // Toggle the inventory
                Some(1) => self.ui.toggle_text_display(1),
                // Change the selected units fire mode
                Some(2) => if let Some(unit) = self.selected.and_then(|selected| self.map.units.get_mut(selected)) {
                    unit.weapon.change_mode();
                },
                // Or select/deselect a unit
                _ => if let Some((x, y)) = self.cursor.position {
                    self.path = None;
                    self.selected = match self.map.units.at(x, y) {
                        Some((id, unit)) => if unit.side == UnitSide::Player {
                            Some(id)
                        } else {
                            None
                        },
                        _ => None
                    }
                }
            },
            // Check if the cursor has a position and a unit is selected
            MouseButton::Right => if let Some((x, y)) = self.cursor.position {
                if let Some(player_unit_id) = self.selected {
                    // Don't do anything if it's the AI's turn
                    if self.controller == Controller::AI {
                        return;
                    }

                    match self.map.units.at(x, y) {
                        // If an AI unit is under the cursor, push a fire command
                        Some((ai_unit_id, ai_unit)) => {
                            if ai_unit.side == UnitSide::AI {
                                self.path = None;
                                self.command_queue.push(Command::Fire(FireCommand::new(player_unit_id, ai_unit_id)));
                            }
                        }
                        _ => {
                            let unit = self.map.units.get(player_unit_id).unwrap();

                            // return if the target location is taken
                            if self.map.taken(x, y) {
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
                }                                  
            },
            _ => {}
        }
    }

    // Get a reference to the unit that is selected
    fn selected_unit(&self) -> Option<&Unit> {
        self.selected.and_then(|selected| self.map.units.get(selected))
    }

    // Work out if the cursor is on an ai unit
    pub fn cursor_on_ai_unit(&self) -> bool {
        self.cursor.position
            .and_then(|(x, y)| self.map.units.at(x, y))
            .map(|(_, unit)| unit.side == UnitSide::AI)
            .unwrap_or(false)
    }

    // End the current turn
    fn end_turn(&mut self) {
        if self.controller == Controller::Player {
            for (_, unit) in self.map.units.iter_mut() {
                unit.moves = unit.max_moves;
            }

            self.controller = Controller::AI;
        }   
    }
}