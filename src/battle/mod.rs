// A battle in the game

pub mod units;
pub mod map;
mod drawer;
mod tiles;
mod paths;
mod animations;
mod ai;
mod commands;

use std::fmt;

use piston::input::{Key, MouseButton};
use graphics::Context;
use opengl_graphics::GlGraphics;

use battle::drawer::Drawer;
use battle::paths::{pathfind, PathPoint};
use battle::animations::{Animations, UpdateAnimations};
use battle::commands::{CommandQueue, Command, UpdateCommands, FireCommand, WalkCommand};
use battle::units::{Unit, UnitSide};
use battle::map::Map;
use resources::{Resources, SetImage};
use ui::{UI, Button, TextDisplay, VerticalAlignment, HorizontalAlignment};
use settings::SkirmishSettings;
use WindowSize;
use traits::Dimensions;

const CAMERA_SPEED: f64 = 10.0;
const CAMERA_ZOOM_SPEED: f64 = 1.0;

// A cursor on the map with a possible position
pub struct Cursor {
    pub position: Option<(usize, usize)>
}

// Whose turn is it
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

pub enum BattleCallback {
    Won,
    Lost
}

// The main Battle struct the handles actions
pub struct Battle {
    pub map: Map,
    drawer: Drawer,
    pub cursor: Cursor,
    keys: [bool; 6],
    pub selected: Option<usize>,
    pub path: Option<Vec<PathPoint>>,
    ui: UI,
    pub animations: Animations,
    pub command_queue: CommandQueue,
    controller: Controller,
}

impl Battle {
    // Create a new Battle
    pub fn new() -> Battle {
        let scale = 2.0;
        let width_offset = SetImage::EndTurnButton.width() * -scale;

        // Create the UI and add the buttons and text display

        let ui = UI::new(
            vec![
                Button::new(
                    SetImage::EndTurnButton,
                    0.0,
                    0.0,
                    scale,
                    VerticalAlignment::Right,
                    HorizontalAlignment::Bottom
                ),
                Button::new(
                    SetImage::InventoryButton,
                    width_offset,
                    0.0,
                    scale,
                    VerticalAlignment::Right,
                    HorizontalAlignment::Bottom
                ),
                Button::new(
                    SetImage::ChangeFireModeButton,
                    width_offset * 2.0,
                    0.0,
                    scale,
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
            ui: ui,
            animations: Animations::new(),
            command_queue: CommandQueue::new(),
            controller: Controller::Player,
        }
    }

    // Start up the map
    pub fn start(&mut self, settings: &SkirmishSettings) {
        // Add player units
        for x in 0 .. settings.player_units {
            self.map.units.add(settings.player_unit_type, UnitSide::Player, x, 0);
        }

        // Add ai units
        for y in settings.cols - settings.ai_units .. settings.cols {
            self.map.units.add(settings.ai_unit_type, UnitSide::AI, y, settings.rows - 1);
        }
        
        // Generate tiles
        self.map.tiles.generate(settings.cols, settings.rows, &self.map.units);
    }

    // Handle keypresses
    pub fn handle_key(&mut self, key: Key, pressed: bool) -> bool {
        match key {
            Key::Up    | Key::W => self.keys[0] = pressed,
            Key::Down  | Key::S => self.keys[1] = pressed,
            Key::Left  | Key::A => self.keys[2] = pressed,
            Key::Right | Key::D => self.keys[3] = pressed,
            Key::O              => self.keys[4] = pressed,
            Key::P              => self.keys[5] = pressed,
            Key::Escape         => {
                self.map.save_skrimish("autosave.sav");
                return false;
            }
            _ => {}
        };

        true
    }

    // Update the battle
    pub fn update(&mut self, resources: &Resources, dt: f64) -> Option<BattleCallback> {
        // Change camera variables if a key is being pressed
        if self.keys[0] { self.drawer.camera.y -= CAMERA_SPEED * dt; }
        if self.keys[1] { self.drawer.camera.y += CAMERA_SPEED * dt; }
        if self.keys[2] { self.drawer.camera.x -= CAMERA_SPEED * dt; }
        if self.keys[3] { self.drawer.camera.x += CAMERA_SPEED * dt; }
        if self.keys[4] { self.drawer.zoom(-CAMERA_ZOOM_SPEED * dt) }
        if self.keys[5] { self.drawer.zoom(CAMERA_ZOOM_SPEED  * dt) }

        if let Controller::AI = self.controller {
            if self.command_queue.is_empty() &&
               self.animations.is_empty() &&
               !ai::make_move(&self.map, &mut self.command_queue) {
                self.controller = Controller::Player;
                self.map.turn += 1;

                if !self.map.units.any_alive(UnitSide::Player) {
                    return Some(BattleCallback::Lost);
                } else if !self.map.units.any_alive(UnitSide::AI) {
                    return Some(BattleCallback::Won);
                }
            }
        }
        
        // Update the command queue if there are no animations in progress
        if self.animations.is_empty() {
            self.command_queue.update(&mut self.map, &mut self.animations);
        }
        // Update the animations
        self.animations.update(&mut self.map, resources, dt);

        None
    }

    // Draw both the map and the UI
    pub fn draw(&mut self, ctx: &Context, gl: &mut GlGraphics, resources: &mut Resources) {
        self.drawer.draw_battle(ctx, gl, resources, self);
        self.draw_ui(ctx, gl, resources);
    }

    // Draw the UI
    fn draw_ui(&mut self, ctx: &Context, gl: &mut GlGraphics, resources: &mut Resources) {
        // Get a string of info about the selected unit
        let selected = match self.selected_unit() {
            Some(unit) => format!(
                "Name: {}, Moves: {}, Health: {}\nWeapon: {}",
                unit.name, unit.moves, unit.health, unit.weapon
            ),
            _ => String::new()
        };

        // Set the text of the UI text display
        self.ui.set_text(0, format!("Turn {} - {}\n{}", self.map.turn, self.controller, selected));

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
        self.ui.draw(ctx, gl, resources);
    }

    // Move the cursor on the screen
    pub fn move_cursor(&mut self, x: f64, y: f64, window_size: &WindowSize) {
        // Get the position where the cursor should be
        let (x, y) = self.drawer.tile_under_cursor(x, y, window_size);

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
    pub fn mouse_button(&mut self, button: MouseButton, window_size: &WindowSize, mouse: (f64, f64)) {
        match button {
            MouseButton::Left => match self.ui.clicked(window_size, mouse) {
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
                        Some(unit) => if unit.side == UnitSide::Player {
                            Some(unit.id)
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
                    if let Controller::AI = self.controller {
                        return;
                    }

                    match self.map.units.at(x, y) {
                        // If an AI unit is under the cursor, push a fire command
                        Some(ai_unit) => {
                            if ai_unit.side == UnitSide::AI {
                                self.path = None;
                                self.command_queue.push(Command::Fire(FireCommand::new(player_unit_id, ai_unit.id)));
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
            .map(|unit| unit.side == UnitSide::AI)
            .unwrap_or(false)
    }

    // End the current turn
    fn end_turn(&mut self) {
        if let Controller::Player = self.controller {
            for unit in self.map.units.iter_mut() {
                unit.moves = unit.max_moves;
            }

            self.controller = Controller::AI;
        }   
    }
}