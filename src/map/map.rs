use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;

use map::tiles::Tiles;
use map::drawer::Drawer;
use map::paths::{pathfind, PathPoint};
use map::animations::AnimationQueue;
use map::commands::{CommandQueue, FireCommand, WalkCommand};
use map::units::{Unit, UnitType, UnitSide, Units};
use map::ai;
use context::Context;
use Resources;
use ui::{UI, Button, TextDisplay, VerticalAlignment, HorizontalAlignment};

const CAMERA_SPEED: f32 = 0.2;
const CAMERA_ZOOM_SPEED: f32 = 0.02;

// A cursor on the map with a possible position
pub struct Cursor {
    pub position: Option<(usize, usize)>,
    pub fire: bool
}

// The Map struct
pub struct Map {
    pub tiles: Tiles,
    drawer: Drawer,
    pub cursor: Cursor,
    keys: [bool; 6],
    pub units: Units,
    pub selected: Option<usize>,
    pub path: Option<Vec<PathPoint>>,
    turn: u16,
    ui: UI,
    pub animation_queue: AnimationQueue,
    pub command_queue: CommandQueue
}

impl Map {
    // Create a new name, using the resources to get the size of UI buttons
    pub fn new(resources: &Resources) -> Map {
        let scale = 2.0;
        let width = resources.image(&"end_turn_button".into()).query().width as f32 * scale;

        // Create the UI and add the buttons and text display

        let mut ui = UI::new();

        ui.buttons = vec![
            Button::new(
                "end_turn_button".into(),
                0.0,
                0.0,
                scale,
                resources,
                VerticalAlignment::Right,
                HorizontalAlignment::Bottom
            ),
            Button::new(
                "fire_button".into(),
                -width,
                0.0,
                scale,
                resources,
                VerticalAlignment::Right,
                HorizontalAlignment::Bottom
            )
        ];

        ui.text_displays = vec![
            TextDisplay::new(0.0, 0.0, VerticalAlignment::Middle, HorizontalAlignment::Top)
        ];

        Map {
            tiles: Tiles::new(),
            drawer: Drawer::new(),
            cursor: Cursor { position: None, fire: false },
            keys: [false; 6],
            units: Units::new(),
            selected: None,
            path: None,
            turn: 1,
            ui: ui,
            animation_queue: AnimationQueue::new(),
            command_queue: CommandQueue::new()
        }
    }

    // Start up the map
    pub fn start(&mut self, cols: usize, rows: usize) {
        // Generate tiles
        self.tiles.generate(cols, rows);

        // Add squaddies
        for x in 0 .. 3 {
            self.units.push(Unit::new(UnitType::Squaddie, UnitSide::Friendly, x, 0));
        }

        // Add enemies
        for y in self.tiles.cols - 3 .. self.tiles.cols {
            self.units.push(Unit::new(UnitType::Squaddie, UnitSide::Enemy, y, self.tiles.rows - 1));
        }

        self.tiles.update_visibility(&self.units);
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

    pub fn update(&mut self) {
        // Change camera variables if a key is being pressed
        if self.keys[0] { self.drawer.camera.y -= CAMERA_SPEED; }
        if self.keys[1] { self.drawer.camera.y += CAMERA_SPEED; }
        if self.keys[2] { self.drawer.camera.x -= CAMERA_SPEED; }
        if self.keys[3] { self.drawer.camera.x += CAMERA_SPEED; }
        if self.keys[4] { self.drawer.zoom(-CAMERA_ZOOM_SPEED) }
        if self.keys[5] { self.drawer.zoom(CAMERA_ZOOM_SPEED) }

        if self.animation_queue.empty() {
            self.command_queue.update(&mut self.units, &mut self.tiles, &mut self.animation_queue);
        }

        // Update the animation queue
        self.animation_queue.update(&mut self.tiles, &mut self.units);
    }

    // Draw both the map and the UI
    pub fn draw(&mut self, ctx: &mut Context, resources: &Resources) {
        self.drawer.draw_map(ctx, resources, &self);
        self.draw_ui(ctx, resources);
    }

    // Draw the UI
    pub fn draw_ui(&mut self, ctx: &mut Context, resources: &Resources) {
        // Get  string of info about the selected unit
        let selected = match self.selected {
            Some(i) => {
                let unit = self.units.get(i);

                if unit.side == UnitSide::Friendly {
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
        self.ui.set_text(0, format!("Turn: {}, Selected: {}", self.turn, selected));

        // Draw the UI
        self.ui.draw(ctx, resources);
    }

    // Move the cursor on the screen
    pub fn move_cursor(&mut self, ctx: &mut Context, x: f32, y: f32) {
        // Get the position where the cursor should be
        let (x, y) = self.drawer.tile_under_cursor(ctx, x, y);

        // Set cursor position if it is on the map and visible
        self.cursor.position = if x < self.tiles.cols && y < self.tiles.rows && self.tiles.tile_at(x, y).visible() {
            Some((x, y))
        } else {
            None
        }
    }

    // Respond to mouse presses
    pub fn mouse_button(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        match button {
            MouseButton::Left => match self.ui.clicked(ctx, x, y) {
                // Respond to UI clicks
                Some(0) => self.end_turn(),
                Some(1) => {
                    self.cursor.fire = !self.cursor.fire;
                    self.path = None;
                },
                // Or check if cursor position
                _ => match self.cursor.position {
                    Some((x, y)) => {
                        self.path = None;

                        // If the cursor is in fire mode and a squaddie is selected and an enemy is under the cursor
                        if self.cursor.fire && self.selected.is_some() {
                            match self.selected.and_then(|selected| self.units.at_i(x, y).map(|target| (selected, target))) {
                                Some((selected_id, target_id)) => {
                                    if self.units.get(selected_id).side != UnitSide::Friendly {
                                        return;
                                    }

                                    self.command_queue.add_fire(FireCommand::new(selected_id, target_id));
                                }
                                _ => {}
                            }
                        // Otherwise select the squaddie under the cursor (or none)
                        } else if !self.cursor.fire {
                            self.selected = self.units.at_i(x, y);
                        }
                    },
                    _ => {}
                }
            },
            // Check if the cursor has a position and a unit is selected
            MouseButton::Right => match self.cursor.position.and_then(|(x, y)| self.selected.map(|selected| (x, y, selected))) {
                Some((x, y, selected)) => {
                    // Do nothing if fire mode is on or if the unit isn't friendly
                    if self.cursor.fire || self.units.get(selected).side != UnitSide::Friendly {
                        return;
                    // Or the the target location is selected
                    } else if self.taken(x, y) {
                        self.path = None;
                        return;
                    }

                    // Pathfind to get the path points and the cost
                    let points = match pathfind(self.units.get(selected), x, y, &self) {
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

                    // If the paths are the same and the squaddie can move to the destination, get rid of the path
                    self.path = if same_path {
                        self.command_queue.add_walk(WalkCommand::new(selected, points));
                        None
                    } else {
                        Some(points)
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    // Is a tile taken up by an obstacle or unit?
    pub fn taken(&self, x: usize, y: usize) -> bool {
        !self.tiles.tile_at(x, y).walkable ||
        self.units.at(x, y).is_some()
    }

    // End the current turn
    fn end_turn(&mut self) {
        for unit in self.units.iter_mut() {
            unit.moves = unit.max_moves;
        }

        ai::take_turn(self);

        self.turn += 1;
    }
}