// A battle in the game

pub mod units;
pub mod map;
mod drawer;
mod tiles;
mod paths;
mod animations;
mod ai;
mod commands;

use glutin::{VirtualKeyCode, MouseButton};

use std::fmt;

use self::drawer::Drawer;
use self::paths::{pathfind, PathPoint};
use self::animations::{Animations, UpdateAnimations};
use self::commands::{CommandQueue, Command, UpdateCommands, FireCommand, WalkCommand};
use self::units::{Unit, UnitSide};
use self::map::Map;
use resources::{ImageSource, Image};
use context::Context;
use ui::{UI, Button, TextDisplay, TextInput, Vertical, Horizontal, Menu};
use settings::SkirmishSettings;

const CAMERA_SPEED: f32 = 10.0;
const CAMERA_ZOOM_SPEED: f32 = 1.0;

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

pub enum BattleCallback {
    Ended,
    Quit
}

// The main Battle struct the handles actions
pub struct Battle {
    pub map: Map,
    pub cursor: Cursor,
    pub selected: Option<u8>,
    pub path: Option<Vec<PathPoint>>,
    pub animations: Animations,
    pub command_queue: CommandQueue,
    drawer: Drawer,
    keys: [bool; 6],
    ui: UI,
    controller: Controller,
}

impl Battle {
    // Create a new Battle
    pub fn new(ctx: &Context, settings: &SkirmishSettings, map: Option<Map>) -> Battle {
        let width_offset = - Image::EndTurnButton.width() * ctx.ui_scale;

        // Create the UI and add the buttons and text display

        let mut ui = UI::new();

        ui.add_buttons(vec![
            Button::new(
                Image::EndTurnButton,
                0.0, 0.0, ctx.ui_scale,
                Vertical::Right, Horizontal::Bottom
            ),
            Button::new(
                Image::InventoryButton,
                width_offset, 0.0, ctx.ui_scale,
                Vertical::Right, Horizontal::Bottom
            ),
            Button::new(
                Image::ChangeFireModeButton,
                width_offset * 2.0, 0.0, ctx.ui_scale,
                Vertical::Right, Horizontal::Bottom
            ),
            Button::new(
                Image::SaveGameButton,
                width_offset * 3.0, 0.0, ctx.ui_scale,
                Vertical::Right, Horizontal::Bottom
            )
        ]);

        ui.add_text_displays(vec![
            TextDisplay::new(0.0, 10.0, Vertical::Middle, Horizontal::Top, true),
            TextDisplay::new(0.0, 0.0, Vertical::Middle, Horizontal::Middle, false),
            TextDisplay::new(10.0, -10.0, Vertical::Left, Horizontal::Bottom, true)
        ]);

        ui.add_text_inputs(vec![
            TextInput::new(0.0, 0.0, Vertical::Middle, Horizontal::Middle, false, ctx, "Save game to:")
        ]);

        ui.add_menus(vec![
            Menu::new(0.0, 0.0, Vertical::Middle, Horizontal::Middle, false, Vec::new())
        ]);

        // Attempt to unwrap the loaded map or generate a new one based off the skirmish settings
        let map = map.unwrap_or_else(|| {
            let mut map = Map::new();

            // Add player units
            for x in 0 .. settings.player_units {
                map.units.add(settings.player_unit_type, UnitSide::Player, x, 0);
            }

            // Add ai units
            for y in settings.cols - settings.ai_units .. settings.cols {
                map.units.add(settings.ai_unit_type, UnitSide::AI, y, settings.rows - 1);
            }
            
            // Generate tiles
            map.tiles.generate(settings.cols, settings.rows, &map.units);

            map
        });

        // Create the battle
        Battle {
            map: map,
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

    // Handle keypresses
    pub fn handle_key(&mut self, key: VirtualKeyCode, pressed: bool) -> Option<BattleCallback> {
        // Respond to key presses on the score screen            
        if self.ui.menu(0).active {
            if pressed {
                match key {
                    VirtualKeyCode::Return => match self.ui.menu(0).selection {
                        3 => return Some(BattleCallback::Ended),
                        4 => return Some(BattleCallback::Quit),
                        _ => {}
                    },
                    VirtualKeyCode::Up => self.ui.menu(0).rotate_up(),
                    VirtualKeyCode::Down => self.ui.menu(0).rotate_down(),
                    VirtualKeyCode::Escape => return Some(BattleCallback::Quit),
                    _ => {}
                };
            }

            return None;
        }

        // Creating an autosave and quit the gme
        if key == VirtualKeyCode::Escape && pressed {
            self.map.save(None);
            return Some(BattleCallback::Quit);
        }

        // Respond to key presses when the text input is open
        if self.ui.text_input(0).active && pressed {
            if key == VirtualKeyCode::Return {
                if let Some(save) = self.map.save(Some(self.ui.text_input(0).text())) {
                    self.ui.text_display(2).append(&format!("Saved to '{}'", save.display()));
                } else {
                    self.ui.text_display(2).append("Failed to save game");
                }

                self.ui.text_input(0).toggle();
            } else {
                self.ui.text_input(0).handle_key(key);
            }

            return None;
        }
    
        // Respond to keys normally!
        match key {
            VirtualKeyCode::Up    | VirtualKeyCode::W => self.keys[0] = pressed,
            VirtualKeyCode::Down  | VirtualKeyCode::S => self.keys[1] = pressed,
            VirtualKeyCode::Left  | VirtualKeyCode::A => self.keys[2] = pressed,
            VirtualKeyCode::Right | VirtualKeyCode::D => self.keys[3] = pressed,
            VirtualKeyCode::O => self.keys[4] = pressed,
            VirtualKeyCode::P => self.keys[5] = pressed,
            _ => {}
        }

        None
    }

    // Update the battle
    pub fn update(&mut self, ctx: &Context, dt: f32) {
        // Change camera variables if a key is being pressed
        if self.keys[0] { self.drawer.y += CAMERA_SPEED * dt; }
        if self.keys[1] { self.drawer.y -= CAMERA_SPEED * dt; }
        if self.keys[2] { self.drawer.x -= CAMERA_SPEED * dt; }
        if self.keys[3] { self.drawer.x += CAMERA_SPEED * dt; }
        if self.keys[4] { self.drawer.zoom(-CAMERA_ZOOM_SPEED * dt); }
        if self.keys[5] { self.drawer.zoom(CAMERA_ZOOM_SPEED  * dt); }

        // If the controller is the AI and the command queue and animations are empty, make a ai move
        // If that returns false, switch control back to the player
        if self.controller == Controller::AI &&
           self.command_queue.is_empty() &&
           self.animations.is_empty() &&
           !ai::make_move(&self.map, &mut self.command_queue) {
            
            self.controller = Controller::Player;
            // Update the turn
            self.map.turn += 1;
            // Log to the text display
            self.ui.text_display(2).append(&format!("Turn {} started", self.map.turn));

            // Get the number of alive units on both sides
            let player_count = self.map.units.count(UnitSide::Player);
            let ai_count = self.map.units.count(UnitSide::AI);

            // If one side has lost all their units, Set the score screen menu
            if player_count == 0 || ai_count == 0 {
                let score_screen = self.ui.menu(0);
                score_screen.active = true;
                score_screen.list = vec![
                    "Skirmish Ended".into(),
                    format!("Units lost: {}", self.map.units.max_player_units - player_count),
                    format!("Units killed: {}", self.map.units.max_ai_units - ai_count),
                    "Close".into(),
                    "Quit".into()
                ];
            }
        }
        
        // Update the command queue if there are no animations in progress
        if self.animations.is_empty() {
            self.command_queue.update(&mut self.map, &mut self.animations, &mut self.ui.text_display(2));
        }
        // Update the animations
        self.animations.update(&mut self.map, ctx, dt);
    }

    // Draw both the map and the UI
    pub fn draw(&mut self, ctx: &mut Context) {
        if self.ui.menu(0).active {
            self.ui.menu(0).render(ctx);
        } else {
            self.drawer.draw_battle(ctx, self);
            self.draw_ui(ctx);
        }
    }

    // Draw the UI
    fn draw_ui(&mut self, ctx: &mut Context) {
        // Get a string of info about the selected unit
        let selected = match self.selected_unit() {
            Some(unit) => format!(
                "Name: {}, Moves: {}, Health: {}\nWeapon: {}",
                unit.name, unit.moves, unit.health, unit.weapon
            ),
            _ => String::new()
        };

        // Set the text of the UI text display
        self.ui.text_display(0).text = format!("Turn {} - {}\n{}", self.map.turn, self.controller, selected);

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
        
        self.ui.text_display(1).text = inventory_string;

        // Draw the UI
        self.ui.draw(ctx);
    }

    // Move the cursor on the screen
    pub fn move_cursor(&mut self, x: f32, y: f32) {
        // Get the position where the cursor should be
        let (x, y) = self.drawer.tile_under_cursor(x, y);

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
    pub fn mouse_button(&mut self, button: MouseButton, mouse: (f32, f32), ctx: &Context) {
        match button {
            MouseButton::Left => match self.ui.clicked(ctx, mouse) {
                // End the turn
                Some(0) => self.end_turn(),
                // Toggle the inventory
                Some(1) => self.ui.text_display(1).toggle(),
                // Change the selected units fire mode
                Some(2) => if let Some(unit) = self.selected.and_then(|selected| self.map.units.get_mut(selected)) {
                    unit.weapon.change_mode();
                },
                Some(3) => self.ui.text_input(0).toggle(),
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
                    // Don't do anything if it's the AI's turn or if there is a command in progress
                    if self.controller == Controller::AI || !self.command_queue.is_empty() {
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
        if self.controller == Controller::Player {
            for unit in self.map.units.iter_mut() {
                unit.moves = unit.max_moves;
            }

            self.controller = Controller::AI;
        }   
    }
}