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
mod iter_2d;

use glutin::{VirtualKeyCode, MouseButton};

use std::fmt;

use self::drawer::{draw_battle, tile_under_cursor, Camera};
use self::paths::{pathfind, PathPoint};
use self::animations::{Animations, UpdateAnimations};
use self::commands::{CommandQueue, FireCommand, WalkCommand, ThrowItemCommand, TurnCommand};
use self::units::{Unit, UnitSide, UnitFacing};
use self::map::Map;
use resources::{ImageSource, Image};
use context::Context;
use ui::{UI, Button, TextDisplay, TextInput, Vertical, Horizontal, Menu, MenuItem};
use utils::distance_under;
use settings::{Settings, SkirmishSettings};

// Whose turn is it
#[derive(Debug, PartialEq)]
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

#[derive(Default)]
struct Keys {
    up: bool,
    left: bool,
    right: bool,
    down: bool,
    zoom_out: bool,
    zoom_in: bool,
    force_fire: bool
}

// The main Battle struct the handles actions
pub struct Battle {
    pub map: Map,
    pub cursor: Option<(usize, usize)>,
    pub selected: u8,
    pub path: Option<Vec<PathPoint>>,
    pub animations: Animations,
    pub command_queue: CommandQueue,
    keys: Keys,
    ui: UI,
    inventory: UI,
    controller: Controller,
}

impl Battle {
    // Create a new Battle
    pub fn new(skirmish_settings: &SkirmishSettings, map: Option<Map>) -> Battle {
        let width_offset = -Image::EndTurnButton.width();

        // Create the base UI

        let mut ui = UI::new(true);

        ui.add_buttons(vec![
            Button::new(Image::EndTurnButton, 0.0, 0.0, Vertical::Right, Horizontal::Bottom),
            Button::new(Image::InventoryButton, width_offset, 0.0, Vertical::Right, Horizontal::Bottom),
            Button::new(Image::SaveGameButton, width_offset * 2.0, 0.0, Vertical::Right, Horizontal::Bottom)
        ]);

        ui.add_text_displays(vec![
            TextDisplay::new(0.0, 10.0, Vertical::Middle, Horizontal::Top, true),
            TextDisplay::new(10.0, -10.0, Vertical::Left, Horizontal::Bottom, true)
        ]);

        ui.add_text_inputs(vec![
            TextInput::new(0.0, 0.0, Vertical::Middle, Horizontal::Middle, false, "Save game to:")
        ]);

        ui.add_menus(vec![
            Menu::new(0.0, 0.0, Vertical::Middle, Horizontal::Middle, false, true, Vec::new())
        ]);

        // Create the inventory UI

        let mut inventory = UI::new(false);
        
        inventory.add_text_displays(vec![
            TextDisplay::new(-75.0, 50.0, Vertical::Middle, Horizontal::Top, true),
            TextDisplay::new(75.0, 50.0, Vertical::Middle, Horizontal::Top, true)
        ]);

        inventory.add_menus(vec![
            Menu::new(-75.0, 82.5, Vertical::Middle, Horizontal::Top, true, true, Vec::new()),
            Menu::new(75.0, 62.5, Vertical::Middle, Horizontal::Top, true, false, Vec::new())
        ]);


        // Attempt to unwrap the loaded map or generate a new one based off the skirmish settings
        let map = map.unwrap_or_else(|| {
            let mut map = Map::new(skirmish_settings.cols, skirmish_settings.rows, skirmish_settings.light);

            // Add player units
            for x in 0 .. skirmish_settings.player_units {
                map.units.add(skirmish_settings.player_unit_type, UnitSide::Player, x, 0, UnitFacing::Bottom);
            }

            // Add ai units
            for y in skirmish_settings.cols - skirmish_settings.ai_units .. skirmish_settings.cols {
                map.units.add(skirmish_settings.ai_unit_type, UnitSide::AI, y, skirmish_settings.rows - 1, UnitFacing::Top);
            }
            
            // Generate tiles
            map.tiles.generate(&map.units);

            map
        });

        // Create the battle
        Battle {
            map: map,
            cursor: None,
            keys: Keys::default(),
            selected: 0,
            path: None,
            ui: ui,
            inventory: inventory,
            animations: Animations::new(),
            command_queue: CommandQueue::new(),
            controller: Controller::Player,
        }
    }

    // Handle keypresses
    pub fn handle_key(&mut self, settings: &Settings, key: VirtualKeyCode, pressed: bool) -> bool {
        // Respond to key presses on the score screen            
        if self.ui.menu(0).active {
            if pressed {
                if let VirtualKeyCode::Return = key {
                    return false;
                }
            }

            return true;
        }

        // If the escape key was pressed, create an autosave and quit the game
        if key == VirtualKeyCode::Escape && pressed {
            return false;
        }

        // Respond to key presses when the text input is open
        if self.ui.text_input(0).active && pressed {
            if key == VirtualKeyCode::Return {
                let filename = self.ui.text_input(0).text();

                if let Some(save) = self.map.save(filename, settings) {
                    self.ui.text_display(1).append(&format!("Saved to '{}'", save.display()));
                } else {
                    self.ui.text_display(1).append("Failed to save game");
                }

                self.ui.text_input(0).toggle();
            } else {
                self.ui.text_input(0).handle_key(key);
            }

            return true;
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
                VirtualKeyCode::Return => {
                    let index = self.inventory.menu(active).selection;

                    if let Some(unit) = self.map.units.get_mut(self.selected) {
                        // Was the item transferred?
                        let transferred = if active == 0 {
                            unit.drop_item(&mut self.map.tiles, index)
                        } else {
                            unit.pick_up_item(&mut self.map.tiles, index)
                        };

                        if transferred {
                            self.inventory.menu(active).fit_selection();
                        }
                    }
                },
                // Use an item
                VirtualKeyCode::E => {
                    if active == 0 {
                        let index = self.inventory.menu(active).selection;

                        if let Some(unit) = self.map.units.get_mut(self.selected) {
                            if unit.use_item(index) {
                                self.inventory.menu(active).fit_selection();
                            }
                        }
                    }
                },
                VirtualKeyCode::T => {
                    if let Some((id, x, y, throw, empty)) = self.selected().map(|unit| (unit.id, unit.x, unit.y, unit.tag.throw_distance(), unit.inventory.is_empty())) {
                        if !empty && active == 0 {
                            if let Some((cursor_x, cursor_y)) = self.cursor {
                                if distance_under(x, y, cursor_x, cursor_y, throw) {
                                    self.command_queue.push(ThrowItemCommand::new(id, self.inventory.menu(active).selection, cursor_x, cursor_y));
                                    self.inventory.menu(active).fit_selection();
                                }
                            }
                        }
                    }
                },
                _ => {}
            }

            return true;
        }
    
        // Respond to keys normally!
        match key {
            VirtualKeyCode::Up    | VirtualKeyCode::W => self.keys.up = pressed,
            VirtualKeyCode::Down  | VirtualKeyCode::S => self.keys.down = pressed,
            VirtualKeyCode::Left  | VirtualKeyCode::A => self.keys.left = pressed,
            VirtualKeyCode::Right | VirtualKeyCode::D => self.keys.right = pressed,
            VirtualKeyCode::O => self.keys.zoom_out = pressed,
            VirtualKeyCode::P => self.keys.zoom_in = pressed,
            VirtualKeyCode::LControl | VirtualKeyCode::RControl => self.keys.force_fire = pressed,
            VirtualKeyCode::I if pressed => self.inventory.toggle(),
            _ => {}
        }

        true
    }

    // Update the battle
    pub fn update(&mut self, ctx: &mut Context, dt: f32) {
        // Change camera variables if a key is being pressed
        if self.keys.up       { self.map.camera.y += Camera::SPEED * dt; }
        if self.keys.down     { self.map.camera.y -= Camera::SPEED * dt; }
        if self.keys.left     { self.map.camera.x -= Camera::SPEED * dt; }
        if self.keys.right    { self.map.camera.x += Camera::SPEED * dt; }
        if self.keys.zoom_out { self.map.camera.zoom(-Camera::ZOOM_SPEED * dt); }
        if self.keys.zoom_in  { self.map.camera.zoom( Camera::ZOOM_SPEED * dt); }

        // If the controller is the AI and the command queue and animations are empty, make a ai move
        // If that returns false, switch control back to the player
        if self.controller == Controller::AI && self.command_queue.is_empty() && self.animations.is_empty() {
            let more_moves = ai::make_move(&self.map, &mut self.command_queue);
            
            if !more_moves {
                self.controller = Controller::Player;
                // Update the turn
                self.map.turn += 1;
                // Log to the text display
                self.ui.text_display(1).append(&format!("Turn {} started", self.map.turn));

                // Get the number of alive units on both sides
                let player_count = self.map.units.count(&UnitSide::Player);
                let ai_count = self.map.units.count(&UnitSide::AI);

                // If one side has lost all their units, Set the score screen menu
                if player_count == 0 || ai_count == 0 {
                    self.ui.menu(0).active = true;
                    self.ui.menu(0).selection = 3;
                    self.ui.menu(0).list = vec![
                        item!("Skirmish Ended", false),
                        item!("Units lost: {}", self.map.units.total_player_units - player_count, false),
                        item!("Units killed: {}", self.map.units.total_ai_units - ai_count, false),
                        item!("Close")
                    ];
                }
            }
        }

        // Update the command queue (if there are no animations in progress)
        self.command_queue.update(&mut self.map, &mut self.animations, &mut self.ui.text_display(1));
        // Update the animations
        self.animations.update(ctx, dt);
    }

    // Draw both the map and the UI
    pub fn draw(&mut self, ctx: &mut Context) {
        // If the score screen menu is active, draw it
        if self.ui.menu(0).active {
            self.ui.menu(0).render(ctx);
        // Otherwise draw the battle and UI
        } else {
            draw_battle(ctx, self);
            self.draw_ui(ctx);
        }
    }

    // Set and draw the UI
    fn draw_ui(&mut self, ctx: &mut Context) {
        // Get a string of info about the selected unit
        let selected = match self.selected() {
            Some(unit) => format!(
                "Name: {}, Moves: {}, Health: {}, Weapon: {}",
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
                // Collect the unit's items into a vec
                let items: Vec<MenuItem> = unit.inventory.iter()
                    .map(|item| item!(item))
                    .collect();

                // Collect the items on the ground into a vec
                let ground: Vec<MenuItem> = self.map.tiles.at(unit.x, unit.y).items.iter()
                    .map(|item| item!(item))
                    .collect();
                
                (
                    format!(
                        "{}\n{} - {} kg\nCarry Capacity: {}/{} kg",
                        unit.name, unit.weapon, unit.weapon.tag.weight(), unit.carrying(), unit.tag.capacity()
                    ),
                    items,
                    ground
                )
            });

            // Set the inventory UI
            if let Some((unit_string, items, ground)) = info {
                self.inventory.text_display(0).text = unit_string;
                self.inventory.text_display(1).text = "Ground".into();
                self.inventory.menu(0).list = vec_or_default!(items, vec![item!("No items")]);
                self.inventory.menu(1).list = vec_or_default!(ground, vec![item!("No items")]);
            }
        }
        
        // Draw the UI
        self.ui.draw(ctx);
        self.inventory.draw(ctx);
    }

    // Move the cursor on the screen
    pub fn move_cursor(&mut self, x: f32, y: f32) {
        // Get the position where the cursor should be
        let (x, y) = tile_under_cursor(x, y, &self.map.camera);

        // Set cursor position if it is on the map and visible
        self.cursor = if x < self.map.tiles.cols && y < self.map.tiles.rows {
            Some((x, y))
        } else {
            None
        }
    }

    fn perform_actions(&mut self, x: usize, y: usize) {
        match self.map.units.at(x, y) {
            Some(unit) => {
                self.path = None;

                match unit.side {
                    // Select a unit
                    UnitSide::Player => self.selected = unit.id,
                    // Fire on a unit
                    UnitSide::AI => self.command_queue.push(FireCommand::new(self.selected, x, y))
                }
            },
            // Force fire on a tile
            None if self.keys.force_fire => self.command_queue.push(FireCommand::new(self.selected, x, y)),
            // Move the current unit
            None => if let Some(unit) = self.map.units.get(self.selected) {
                // return if the target location is taken
                if self.map.taken(x, y) {
                    self.path = None;
                    return;
                }

                self.path = match self.path {
                    Some(ref path) if path[path.len() - 1].at(x, y) => {
                        self.command_queue.push(WalkCommand::new(unit, &self.map, path.clone()));
                        None
                    },
                    _ => pathfind(unit, x, y, &self.map).map(|(path, _)| path)
                }
            }
        }
    }

    // Respond to mouse presses
    pub fn mouse_button(&mut self, button: MouseButton, mouse: (f32, f32), ctx: &Context) {
        // Don't do anything if it's the AI's turn or if there is a command in progress
        if self.controller == Controller::AI || !self.command_queue.is_empty() {
            return;
        }

        match button {
            MouseButton::Left => match self.ui.clicked(ctx, mouse) {
                // End the turn
                Some(0) => self.end_turn(),
                // Toggle the inventory
                Some(1) => self.inventory.toggle(),
                // Toggle the save game input
                Some(2) => self.ui.text_input(0).toggle(),
                // Or select/deselect a unit
                _ => if let Some((x, y)) = self.cursor {
                    self.perform_actions(x, y)
                }
            },
            MouseButton::Right => {
                if let Some((x, y)) = self.cursor {
                    if let Some(unit) = self.map.units.get_mut(self.selected) {
                        self.command_queue.push(TurnCommand::new(self.selected, UnitFacing::from_points(unit.x, unit.y, x, y)));
                    }
                }
            }
            _ => {}
        }
    }

    // Get a reference to the unit that is selected
    fn selected(&self) -> Option<&Unit> {
        self.map.units.get(self.selected)
    }

    // Work out if the cursor is on an ai unit
    fn cursor_active(&self) -> bool {
        self.keys.force_fire ||
        self.cursor.map(|(x, y)| self.map.units.on_side(x, y, &UnitSide::AI)).unwrap_or(false)
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

#[test]
fn battle_operations() {
    let skirmish_settings = SkirmishSettings::default();

    let mut battle = Battle::new(&skirmish_settings, None);

    // It should be on the first turn and it should be the player's turn

    assert_eq!(battle.map.turn, 1);
    assert_eq!(battle.controller, Controller::Player);

    // The cols and rows are equal
    assert_eq!(battle.map.tiles.cols, skirmish_settings.cols);
    assert_eq!(battle.map.tiles.rows, skirmish_settings.rows);

    // The unit counts are equal
    assert_eq!(battle.map.units.count(&UnitSide::Player) as usize, skirmish_settings.player_units);
    assert_eq!(battle.map.units.count(&UnitSide::AI) as usize, skirmish_settings.ai_units);

    // The unit types are correct

    assert!(battle.map.units.iter()
        .filter(|unit| unit.side == UnitSide::Player)
        .all(|unit| unit.tag == skirmish_settings.player_unit_type));

    assert!(battle.map.units.iter()
        .filter(|unit| unit.side == UnitSide::AI)
        .all(|unit| unit.tag == skirmish_settings.ai_unit_type));

    {
        // The first unit should be a player unit at (0, 0)

        let unit = battle.map.units.get_mut(0).unwrap();

        assert_eq!(unit.side, UnitSide::Player);
        assert_eq!(unit.tag, skirmish_settings.player_unit_type);

        assert_eq!((unit.x, unit.y), (0, 0));
    }
}