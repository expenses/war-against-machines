// A battle in the game

pub mod units;
pub mod map;
mod drawer;
mod tiles;
mod paths;
mod animations;
mod ai;
mod commands;
mod walls;

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

// An optional callback that is returned by the key handler. It species if the game ended or was quit
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
    inventory: UI,
    controller: Controller,
}

impl Battle {
    // Create a new Battle
    pub fn new(ctx: &Context, settings: &SkirmishSettings, map: Option<Map>) -> Battle {
        let width_offset = - Image::EndTurnButton.width() * ctx.ui_scale;

        // Create the base UI

        let mut ui = UI::new(true);

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
            TextDisplay::new(10.0, -10.0, Vertical::Left, Horizontal::Bottom, true)
        ]);

        ui.add_text_inputs(vec![
            TextInput::new(0.0, 0.0, Vertical::Middle, Horizontal::Middle, false, ctx, "Save game to:")
        ]);

        ui.add_menus(vec![
            Menu::new(0.0, 0.0, Vertical::Middle, Horizontal::Middle, false, true, Vec::new())
        ]);

        // Create the inventory UI

        let mut inventory = UI::new(false);
        
        inventory.add_text_displays(vec![
            TextDisplay::new(-150.0, 100.0, Vertical::Middle, Horizontal::Top, true),
            TextDisplay::new(150.0, 100.0, Vertical::Middle, Horizontal::Top, true)
        ]);

        inventory.add_menus(vec![
            Menu::new(-150.0, 165.0, Vertical::Middle, Horizontal::Top, true, true, Vec::new()),
            Menu::new(150.0, 125.0, Vertical::Middle, Horizontal::Top, true, false, Vec::new())
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
            inventory: inventory,
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

        // If the escape key was pressed, create an autosave and quit the game
        if key == VirtualKeyCode::Escape && pressed {
            self.map.save(None);
            return Some(BattleCallback::Quit);
        }

        // Respond to key presses when the text input is open
        if self.ui.text_input(0).active && pressed {
            if key == VirtualKeyCode::Return {
                if let Some(save) = self.map.save(Some(self.ui.text_input(0).text())) {
                    self.ui.text_display(1).append(&format!("Saved to '{}'", save.display()));
                } else {
                    self.ui.text_display(1).append("Failed to save game");
                }

                self.ui.text_input(0).toggle();
            } else {
                self.ui.text_input(0).handle_key(key);
            }

            return None;
        }

        // Respond to key presses when the inventory is open
        if self.inventory.active && pressed {
            // Get the active/inactive menu
            let (active, inactive) = if self.inventory.menu(0).selected {(0, 1)} else {(1, 0)};
            
            match key {
                // Toggle the inventory
                VirtualKeyCode::I => self.inventory.toggle(),
                // Rotate the selection up
                VirtualKeyCode::Up   | VirtualKeyCode::W => self.inventory.menu(active).rotate_up(),
                // Rotate the selection down
                VirtualKeyCode::Down | VirtualKeyCode::S => self.inventory.menu(active).rotate_down(),
                // Switch which menu is selected
                VirtualKeyCode::Left | VirtualKeyCode::Right |
                VirtualKeyCode::A    | VirtualKeyCode::D => {
                    self.inventory.menu(active).selected = false;
                    self.inventory.menu(inactive).selected = true;
                },
                // Pick up / drop an item
                VirtualKeyCode::Return => if let Some(selected) = self.selected {
                    let index = self.inventory.menu(active).selection;

                    if active == 0 {
                        self.map.drop_item(selected, index)
                    } else {
                        self.map.pick_up_item(selected, index)
                    };

                    let new_len = self.inventory.menu(active).len() - 1;

                    if index >= new_len {
                        self.inventory.menu(active).selection = match new_len {
                            0 => 0,
                            _ => new_len - 1
                        }
                    }
                },
                // Use an item
                VirtualKeyCode::E => if let Some(selected) = self.selected {
                    if active == 0 {
                        let index = self.inventory.menu(active).selection;

                        if self.map.use_item(selected, index) {
                            let new_len = self.inventory.menu(active).len() - 1;

                            if index >= new_len {
                                self.inventory.menu(active).selection = match new_len {
                                    0 => 0,
                                    _ => new_len - 1
                                }
                            }
                        }
                    }
                },
                _ => {}
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
            VirtualKeyCode::I => if pressed && self.selected.is_some() {
                self.inventory.toggle();
            }
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
            self.ui.text_display(1).append(&format!("Turn {} started", self.map.turn));

            // Get the number of alive units on both sides
            let player_count = self.map.units.count(UnitSide::Player);
            let ai_count = self.map.units.count(UnitSide::AI);

            // If one side has lost all their units, Set the score screen menu
            if player_count == 0 || ai_count == 0 {
                self.ui.menu(0).active = true;
                self.ui.menu(0).list = vec![
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
            self.command_queue.update(&mut self.map, &mut self.animations, &mut self.ui.text_display(1));
        }
        // Update the animations
        self.animations.update(&mut self.map, ctx, dt);
    }

    // Draw both the map and the UI
    pub fn draw(&mut self, ctx: &mut Context) {
        // If the score screen menu is active, draw it
        if self.ui.menu(0).active {
            self.ui.menu(0).render(ctx);
        // Otherwise draw the battle and UI
        } else {
            self.drawer.draw_battle(ctx, self);
            self.draw_ui(ctx);
        }
    }

    // Set and draw the UI
    fn draw_ui(&mut self, ctx: &mut Context) {
        // Get a string of info about the selected unit
        let selected = match self.selected() {
            Some(unit) => format!(
                "Name: {}, Moves: {}, Health: {}\nWeapon: {}",
                unit.name, unit.moves, unit.health, unit.weapon
            ),
            _ => String::new()
        };

        // Set the text of the UI text display
        self.ui.text_display(0).text = format!("Turn {} - {}\n{}", self.map.turn, self.controller, selected);

        // Set the inventory
        if self.inventory.active {
            // Get the name of the selected unit, it's items and the items on the ground
            let info = self.selected().map(|unit| {
                let mut weight = unit.weapon.tag.weight();

                // Collect the unit's items into a vec
                let items: Vec<String> = unit.inventory.iter()
                    .inspect(|item| weight += item.weight())
                    .map(|item| item.to_string())
                    .collect();
                // Collect the items on the ground into a vec
                let ground: Vec<String> = self.map.tiles.at(unit.x, unit.y).items.iter()
                    .map(|item| item.to_string())
                    .collect();
                
                (
                    format!(
                        "{}\n{} ({}/{})\nCarry Capacity: {}/{} kg",
                        unit.name,
                        unit.weapon.tag, unit.weapon.ammo, unit.weapon.tag.capacity(),
                        weight, unit.tag.capacity()
                    ),
                    items,
                    ground
                )
            });

            // Set the inventory UI
            if let Some((unit_string, items, ground)) = info {
                self.inventory.text_display(0).text = unit_string;
                self.inventory.text_display(1).text = "Ground".into();
                self.inventory.menu(0).list = vec_or_default!(items, vec!["No items".into()]);
                self.inventory.menu(1).list = vec_or_default!(ground, vec!["No items".into()]);
            }
        }
        
        // Draw the UI
        self.ui.draw(ctx);
        self.inventory.draw(ctx);
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
                Some(1) => if self.selected.is_some() {
                    self.inventory.toggle();
                },
                // Change the selected units fire mode
                Some(2) => if let Some(unit) = self.selected_mut() {
                    unit.weapon.change_mode();
                },
                // Toggle the save game input
                Some(3) => self.ui.text_input(0).toggle(),
                // Or select/deselect a unit
                _ => if let Some((x, y)) = self.cursor.position {
                    // Clear the path
                    self.path = None;
                    // Set the selection
                    self.selected = match self.map.units.at(x, y) {
                        Some(unit) => if unit.side == UnitSide::Player {
                            Some(unit.id)
                        } else {
                            None
                        },
                        _ => None
                    };

                    // Make sure the inventory is only active if a unit is selected
                    self.inventory.active = self.inventory.active && self.selected.is_some();
                }
            },
            // Check if the cursor has a position and a unit is selected
            MouseButton::Right => if let Some((x, y)) = self.cursor.position {
                if let Some(selected_id) = self.selected {
                    // Don't do anything if it's the AI's turn or if there is a command in progress
                    if self.controller == Controller::AI || !self.command_queue.is_empty() {
                        return;
                    }

                    match self.map.units.at(x, y) {
                        // If an AI unit is under the cursor, push a fire command
                        Some(ai_unit) => {
                            if ai_unit.side == UnitSide::AI {
                                self.path = None;
                                
                                if let Some(true) = self.selected().map(|unit| unit.weapon.can_fire()) {
                                    self.command_queue.push(Command::Fire(FireCommand::new(selected_id, ai_unit.id)));
                                }
                            }
                        }
                        _ => if let Some(unit) = self.map.units.get(selected_id) {
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
                                self.command_queue.push(Command::Walk(WalkCommand::new(selected_id, &self.map, points)));
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
    fn selected(&self) -> Option<&Unit> {
        self.selected.and_then(move |selected| self.map.units.get(selected))
    }

    fn selected_mut(&mut self) -> Option<&mut Unit> {
        self.selected.and_then(move |selected| self.map.units.get_mut(selected))
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
                unit.moves = unit.tag.moves();
            }

            self.controller = Controller::AI;
        }   
    }
}