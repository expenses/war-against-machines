// A battle in the game

pub mod units;
pub mod map;
mod drawer;
mod paths;
mod animations;
//mod ai;
mod commands;
mod networking;
mod messages;
mod ui;

use *;

use std::thread::*;

use self::drawer::*;
use self::paths::*;
use self::units::*;
use self::networking::*;
use self::map::*;
use self::ui::*;

use context::*;
use settings::*;

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
    pub camera: Camera,
    pub client: Client,
    pub cursor: Option<(usize, usize)>,
    pub selected: Option<u8>,
    pub path: Option<Vec<PathPoint>>,
    server: Option<JoinHandle<()>>,
    ai: Option<JoinHandle<()>>,
    keys: Keys,
    interface: Interface
}

impl Battle {
    // Create a new Battle
    pub fn new_singleplayer(map: Either<SkirmishSettings, &Path>, settings: Settings) -> Option<Self> {
        let (client, server) = client_and_server(map, settings)?;

        // Create the battle
        Some(Self {
            client,
            server: Some(server),
            cursor: None,
            ai: None,
            keys: Keys::default(),
            selected: None,
            path: None,
            camera: Camera::new(),
            interface: Interface::new()
        })
    }

    pub fn new_multiplayer_host(addr: &str, map: Either<SkirmishSettings, &Path>, settings: Settings) -> Option<Self> {
        let (client, server) = client_and_multiplayer_server(addr, map, settings)?;

        Some(Self {
            client,
            server: Some(server),
            cursor: None,
            ai: None,
            keys: Keys::default(),
            selected: None,
            path: None,
            camera: Camera::new(),
            interface: Interface::new()
        })
    }

    pub fn new_multiplayer_connect(addr: &str) -> Option<Self> {
        let client = client(addr)?;

        Some(Self {
            client,
            server: None,
            cursor: None,
            ai: None,
            keys: Keys::default(),
            selected: None,
            path: None,
            camera: Camera::new(),
            interface: Interface::new()
        })
    }

    // Handle keypresses
    pub fn handle_key(&mut self, key: VirtualKeyCode, pressed: bool) -> bool {
        // Respond to key presses on the score screen            
        if self.interface.game_over_screen_active() {
            if pressed {
                if let VirtualKeyCode::Return = key {
                    return false;
                }
            }

            return true;
        }

        // If the escape key was pressed, quit the game
        if key == VirtualKeyCode::Escape && pressed {
            return false;
        }

        // Respond to key presses when the text input is open
        if pressed {
            let handled = self.interface.try_handle_save_game_keypress(key, &self.client);
            if handled {
                return true;
            }
        }

        // Respond to key presses when the inventory is open
        if pressed {
            if let Some(selected) = self.selected {
                let handled = self.interface.try_handle_inventory_keypress(key, &self.client, selected, &self.cursor);
                if handled {
                    return true;
                }
            }
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
            VirtualKeyCode::I if pressed => self.interface.toggle_inventory(),
            _ => {}
        }

        true
    }

    // Update the battle
    pub fn update(&mut self, ctx: &mut Context, dt: f32) {
        // Change camera variables if a key is being pressed
        if self.keys.up       { self.camera.y += Camera::SPEED * dt; }
        if self.keys.down     { self.camera.y -= Camera::SPEED * dt; }
        if self.keys.left     { self.camera.x -= Camera::SPEED * dt; }
        if self.keys.right    { self.camera.x += Camera::SPEED * dt; }
        if self.keys.zoom_out { self.camera.zoom(-Camera::ZOOM_SPEED * dt); }
        if self.keys.zoom_in  { self.camera.zoom( Camera::ZOOM_SPEED * dt); }

        self.client.recv();
        self.client.process_animations(dt, ctx, &mut self.interface.get_log());

        // todo: game overs
        /*        
        let stats = ...

        self.ui.menu(0).active = true;
        self.ui.menu(0).selection = 3;
        self.ui.menu(0).list = vec![
            item!("Skirmish Ended", false),
            item!("Units lost: {}", stats.units_lost, false),
            item!("Units killed: {}", stats.units_killed, false),
            item!("Close")
        ];
        }*/
    }

    // Draw both the map and the UI
    pub fn draw(&mut self, ctx: &mut Context) {
        // If the score screen menu is active, draw it
        if self.interface.game_over_screen_active() {
            self.interface.draw_game_over_screen(ctx);
        // Otherwise draw the battle and UI
        } else {
            draw_battle(ctx, self);
            self.interface.draw(ctx, self.selected, &self.client.map);
        }
    }

    // Move the cursor on the screen
    pub fn move_cursor(&mut self, x: f32, y: f32) {
        // Get the position where the cursor should be
        let (x, y) = tile_under_cursor(x, y, &self.camera);

        // Set cursor position if it is on the map and visible
        self.cursor = if x < self.map().tiles.width() && y < self.map().tiles.height() {
            Some((x, y))
        } else {
            None
        }
    }

    fn perform_actions(&mut self, x: usize, y: usize) {
        match self.client.map.units.at(x, y) {
            Some(unit) => {
                self.path = None;

                if unit.side == self.client.side {
                    self.selected = Some(unit.id);
                } else if let Some(selected) = self.selected {
                    self.client.fire(selected, x, y);
                }
            },
            // Force fire on a tile
            None if self.keys.force_fire => if let Some(selected) = self.selected {
                self.client.fire(selected, x, y);
            },
            // Move the current unit
            None => {
                let map = &self.client.map;
                if let Some(unit) = self.selected.and_then(|selected| map.units.get(selected)) {
                    // return if the target location is taken
                    if self.map().taken(x, y) {
                        self.path = None;
                        return;
                    }

                    self.path = match self.path {
                        Some(ref path) if path[path.len() - 1].at(x, y) => {
                            self.client.walk(unit.id, path.clone());
                            None
                        },
                        _ => pathfind(unit, x, y, &self.map()).map(|(path, _)| path)
                    }
                }
            }
        }
    }

    // Respond to mouse presses
    pub fn mouse_button(&mut self, button: MouseButton, mouse: (f32, f32), ctx: &Context) {
        if !self.waiting_for_command() {
            return;
        }

        match button {
            MouseButton::Left => match self.interface.clicked(ctx, mouse) {
                // End the turn
                Some(Button::EndTurn) => self.end_turn(),
                // Toggle the inventory
                Some(Button::Inventory) => self.interface.toggle_inventory(),
                // Toggle the save game input
                Some(Button::SaveGame) => self.interface.toggle_save_game(),
                // Or select/deselect a unit
                _ => if let Some((x, y)) = self.cursor {
                    self.perform_actions(x, y)
                }
            },
            MouseButton::Right => if let Some((x, y)) = self.cursor {
                if let Some(unit) = self.selected() {
                    self.client.turn(unit.id, UnitFacing::from_points(unit.x, unit.y, x, y));
                }
            }
            _ => {}
        }
    }

    fn map(&self) -> &Map {
        &self.client.map
    }

    fn waiting_for_command(&self) -> bool {
        self.map().side == self.client.side && self.client.animations.is_empty()
    }

    // Get a reference to the unit that is selected
    fn selected(&self) -> Option<&Unit> {
        self.selected.and_then(|selected| self.map().units.get(selected))
    }

    // Work out if the cursor is on an ai unit
    fn cursor_active(&self) -> bool {
        self.keys.force_fire ||
        self.cursor.map(|(x, y)| self.map().units.on_side(x, y, self.client.side.enemies())).unwrap_or(false)
    }

    // End the current turn
    fn end_turn(&mut self) {
        if self.waiting_for_command() {
            self.client.end_turn();
        }
    }
}

#[test]
fn battle_operations() {
    let skirmish_settings = SkirmishSettings::default();
    let settings = Settings::default();

    let mut battle = Battle::new_singleplayer(Left(skirmish_settings.clone()), settings).unwrap();

    // It should be on the first turn and it should be the player's turn

    assert_eq!(battle.client.map.turn, 1);
    assert_eq!(battle.client.map.side, Side::PlayerA);

    // The cols and rows are equal
    assert_eq!(battle.client.map.tiles.width(), skirmish_settings.cols);
    assert_eq!(battle.client.map.tiles.height(), skirmish_settings.rows);

    // The player unit counts are equal
    assert_eq!(battle.client.map.units.count(Side::PlayerA) as usize, skirmish_settings.player_units);
    
    // No AI units should be in the map, because the server shouldn't have sent info for them as they aren't visible
    assert_eq!(battle.client.map.units.count(Side::PlayerB) as usize, 0);

    // The unit types are correct

    assert!(battle.client.map.units.iter()
        .filter(|unit| unit.side == Side::PlayerA)
        .all(|unit| unit.tag == skirmish_settings.player_unit_type));

    {
        // The first unit should be a player unit at (0, 0)

        let unit = battle.client.map.units.get_mut(0).unwrap();

        assert_eq!(unit.side, Side::PlayerA);
        assert_eq!(unit.tag, skirmish_settings.player_unit_type);

        assert_eq!((unit.x, unit.y), (0, 0));
    }
}