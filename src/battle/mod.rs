// A battle in the game

pub mod units;
pub mod map;
mod drawer;
mod paths;
mod responses;
mod ai;
mod commands;
mod networking;
mod messages;
mod ui;

use *;

use self::drawer::*;
use self::paths::*;
use self::units::*;
use self::networking::*;
use self::map::*;
use self::ui::*;
use super::error::*;

use context::*;
use settings::*;

// todo: review all code & comments

pub enum KeyResponse {
    GameOver,
    Continue,
    OpenMenu
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
    camera: Camera,
    cursor: Option<(usize, usize)>,
    selected: Option<u8>,
    path: Option<Vec<PathPoint>>,
    client: Client,
    server: Option<ThreadHandle>,
    ai: Option<ThreadHandle>,
    keys: Keys,
    interface: Interface,
    visual_debugging: bool
}

impl Battle {
    fn new(client: Client, server: Option<ThreadHandle>, ai: Option<ThreadHandle>) -> Self {
        let mut camera = Camera::new();
        if let Some(unit) = client.map.units.iter().find(|unit| unit.side == client.side) {
            camera.set_to(unit.x, unit.y);
        }

        Self {
            client, server, ai, camera,
            cursor: None,
            keys: Keys::default(),
            selected: None,
            path: None,
            interface: Interface::new(),
            visual_debugging: false
        }
    }

    // Create a new Battle
    pub fn new_vs_ai(map: Either<SkirmishSettings, &Path>, settings: Settings) -> Result<Self> {
        let (client, ai, server) = singleplayer(map, settings)?;
        Ok(Self::new(client, Some(server), Some(ai)))
    }

    pub fn new_multiplayer_host(addr: &str, map: Either<SkirmishSettings, &Path>, settings: Settings) -> Result<Self> {
        let (client, server) = multiplayer(addr, map, settings)?;
        Ok(Self::new(client, Some(server), None))
    }

    pub fn new_multiplayer_connect(addr: &str) -> Result<Self> {
        let client = Client::new_from_addr(addr)?;
        Ok(Self::new(client, None, None))
    }

    // Handle keypresses
    pub fn handle_key(&mut self, key: VirtualKeyCode, pressed: bool) -> KeyResponse {
        // Respond to key presses on the score screen            
        if self.interface.game_over_screen_active() {
            if pressed {
                if let VirtualKeyCode::Return = key {
                    return KeyResponse::GameOver;
                }
            }

            return KeyResponse::Continue;
        }

        // If the escape key was pressed, open the menu
        if key == VirtualKeyCode::Escape && pressed {
            return KeyResponse::OpenMenu;
        }

        // Respond to key presses when the text input is open
        if pressed {
            let handled = self.interface.try_handle_save_game_keypress(key, &self.client);
            if handled {
                return KeyResponse::Continue;
            }
        }

        // Respond to key presses when the inventory is open
        if pressed {
            if let Some(selected) = self.selected {
                let handled = self.interface.try_handle_inventory_keypress(key, &self.client, selected, &self.cursor);
                if handled {
                    return KeyResponse::Continue;
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
            VirtualKeyCode::Grave if pressed => self.visual_debugging = !self.visual_debugging,
            _ => {}
        }

        KeyResponse::Continue
    }

    // Update the battle
    pub fn update(&mut self, ctx: &mut Context, dt: f32) {
        // Move the camera
        if self.keys.up       { self.camera.move_y(dt, &self.client.map); }
        if self.keys.down     { self.camera.move_y(-dt, &self.client.map); }
        if self.keys.left     { self.camera.move_x(-dt, &self.client.map); }
        if self.keys.right    { self.camera.move_x(dt, &self.client.map); }
        if self.keys.zoom_out { self.camera.zoom(-dt); }
        if self.keys.zoom_in  { self.camera.zoom(dt); }

        self.client.recv();
        self.client.process_responses(dt, ctx, &mut self.interface, &mut self.camera);

        if self.interface.game_over_screen_active() {
            if let Some(server) = self.server.take() {
                server.join().unwrap().unwrap();
            }

            if let Some(ai) = self.ai.take() {
                ai.join().unwrap().unwrap();
            }
        }
    }

    // Draw both the map and the UI
    pub fn draw(&mut self, ctx: &mut Context) {
        // If the score screen menu is active, draw it
        if self.interface.game_over_screen_active() {
            self.interface.draw_game_over_screen(ctx);
        // Otherwise draw the battle and UI
        } else {
            draw_battle(ctx, self);
            self.interface.draw(ctx, self.selected, &self.client.map, self.ai.is_some());
        }
    }

    // Move the cursor on the screen
    pub fn move_cursor(&mut self, x: f32, y: f32) {
        // Get the position where the cursor should be
        let (x, y) = self.camera.tile_under_cursor(x, y);

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
                            self.client.walk(unit.id, path);
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
        self.client.our_turn() && self.client.responses().is_empty()
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

    let mut battle = Battle::new_vs_ai(Left(skirmish_settings.clone()), settings).unwrap();

    {
        let map = &battle.client.map;
        assert_eq!(map.side, Side::PlayerA);
        assert_eq!(map.turn(), 1);

        // The cols and rows are equal
        assert_eq!(map.tiles.width(), skirmish_settings.cols);
        assert_eq!(map.tiles.height(), skirmish_settings.rows);

        // The player unit counts are equal
        assert_eq!(map.units.count(Side::PlayerA) as usize, skirmish_settings.player_units);
        
        // No AI units should be in the map, because the server shouldn't have sent info for them as they aren't visible
        assert_eq!(map.units.count(Side::PlayerB) as usize, 0);

        // The unit types are correct

        assert!(map.units.iter()
            .filter(|unit| unit.side == Side::PlayerA)
            .all(|unit| unit.tag == skirmish_settings.player_unit_type));
    }

    // The first unit should be a player unit at (0, 0)

    let unit = battle.client.map.units.get_mut(0).unwrap();

    assert_eq!(unit.side, Side::PlayerA);
    assert_eq!(unit.tag, skirmish_settings.player_unit_type);

    assert_eq!((unit.x, unit.y), (0, 0));
}