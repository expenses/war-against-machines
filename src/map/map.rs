use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use context::Context;

use map::tiles::Tiles;
use Resources;
use units::{Unit, UnitType, UnitSide};
use ui::{UI, Button, TextDisplay, VerticalAlignment, HorizontalAlignment};
use map::paths::{pathfind, PathPoint};
use weapons::Bullet;
// use items::ItemType;

use map::drawer::Drawer;

const CAMERA_SPEED: f32 = 0.2;
const CAMERA_ZOOM_SPEED: f32 = 0.02;

pub struct Cursor {
    pub position: Option<(usize, usize)>,
    pub fire: bool
}

impl Cursor {
    fn new() -> Cursor {
        Cursor {
            position: None,
            fire: false
        }
    }
}

pub struct Map {
    pub tiles: Tiles,
    drawer: Drawer,
    pub cursor: Cursor,
    keys: [bool; 6],
    pub squaddies: Vec<Unit>,
    pub enemies: Vec<Unit>,
    pub selected: Option<usize>,
    pub path: Option<Vec<PathPoint>>,
    turn: u16,
    ui: UI,
    pub bullets: Vec<Bullet>
}

impl Map {
    pub fn new(resources: &Resources) -> Map {
        let scale = 2.0;
        let width = resources.image("end_turn_button").query().width as f32 * scale;

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
            TextDisplay::new(5.0, 5.0)
        ];

        Map {
            tiles: Tiles::new(),
            drawer: Drawer::new(),
            cursor: Cursor::new(),
            keys: [false; 6],
            squaddies: Vec::new(),
            enemies: Vec::new(),
            selected: None,
            path: None,
            turn: 1,
            ui: ui,
            bullets: Vec::new()
        }
    }

    pub fn start(&mut self, cols: usize, rows: usize) {
        // Generate tiles
        self.tiles.generate(cols, rows);

        // Add squaddies
        for x in 0..3 {
            self.squaddies.push(Unit::new(UnitType::Squaddie, UnitSide::Friendly, x, 0));
        }

        // Add enemies
        for y in self.tiles.cols - 3 .. self.tiles.cols {
            self.enemies.push(Unit::new(UnitType::Robot, UnitSide::Enemy, y, self.tiles.rows - 1));
        }
    }

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
        if self.keys[0] { self.drawer.camera.y -= CAMERA_SPEED; }
        if self.keys[1] { self.drawer.camera.y += CAMERA_SPEED; }
        if self.keys[2] { self.drawer.camera.x -= CAMERA_SPEED; }
        if self.keys[3] { self.drawer.camera.x += CAMERA_SPEED; }
        if self.keys[4] { self.drawer.zoom(-CAMERA_ZOOM_SPEED) }
        if self.keys[5] { self.drawer.zoom(CAMERA_ZOOM_SPEED) }

        let cols = self.tiles.cols;
        let rows = self.tiles.rows;
        let mut update = false;

        // Update all the bullet positions
        for bullet in &mut self.bullets {
            bullet.travel();
            
            // Check if a bullet has stopped traveling
            update = update || !bullet.traveling(cols, rows);
        }

        // Only keep bullets that are still traveling
        self.bullets.retain(|bullet| bullet.traveling(cols, rows));

        // If a bullet has stopped traveling, update all the units
        if update {
            self.update_all_units();
        }

    }

    pub fn update_all_units(&mut self) {
        for squaddie in &mut self.squaddies {
            squaddie.update();
        }

        for enemy in &mut self.enemies {
            enemy.update();
        }
    }

    pub fn draw(&mut self, ctx: &mut Context, resources: &Resources) {
        self.drawer.draw_map(ctx, resources, &self);
        self.draw_ui(ctx, resources);
    }

    pub fn draw_ui(&mut self, ctx: &mut Context, resources: &Resources) {
        let selected = match self.selected {
            Some(i) => {
                let squaddie = &self.squaddies[i];
                format!(
                    "(Name: {}, ID: {}, X: {}, Y: {}, Moves: {}, Weapon: {})",
                    squaddie.name, i, squaddie.x, squaddie.y, squaddie.moves, squaddie.weapon.name()
                )
            },
            _ => "~".into()
        };

        self.ui.set_text(0, format!("Turn: {}, Selected: {}", self.turn, selected));

        self.ui.draw(ctx, resources);
    }

    pub fn move_cursor(&mut self, ctx: &mut Context, x: f32, y: f32) {
        let (x, y) = self.drawer.tile_under_cursor(ctx, x, y);

        // Set cursor position
        self.cursor.position = if x < self.tiles.cols && y < self.tiles.rows {
            Some((x, y))
        } else {
            None
        }
    }

    pub fn mouse_button(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        match button {
            MouseButton::Left => match self.ui.clicked(ctx, x, y) {
                Some(0) => self.end_turn(),
                Some(1) => {
                    self.cursor.fire = !self.cursor.fire;
                    self.path = None;
                },
                _ => match self.cursor.position {
                    Some((x, y)) => {
                        self.path = None;

                        if self.cursor.fire && self.selected.is_some() {
                            match self.enemies.iter_mut().find(|enemy| enemy.x == x && enemy.y == y) {
                                Some(enemy) => {
                                    let squaddie = &mut self.squaddies[self.selected.unwrap()];
                                    let bullets = &mut self.bullets;
                                    squaddie.fire_at(enemy, bullets);
                                }
                                _ => {}
                            }
                        } else if !self.cursor.fire {
                            self.selected = self.squaddie_at(x, y).map(|(i, _)| i);
                        }
                    },
                    _ => {}
                }
            },
            MouseButton::Right => match self.cursor.position.and_then(|(x, y)| self.selected.map(|selected| (x, y, selected))) {
                Some((x, y, selected)) => {
                    if self.cursor.fire {
                        return;
                    } else if self.taken(x, y) {
                        self.path = None;
                        return;
                    }

                    let start = PathPoint::from(&self.squaddies[selected]);
                    let end = PathPoint::new(x, y);

                    let (points, cost) = match pathfind(&start, &end, &self) {
                        Some((points, cost)) => (points, cost),
                        _ => {
                            self.path = None;
                            return;
                        }
                    };

                    let same_path = match self.path {
                        Some(ref path) => path[path.len() - 1].at(&end),
                        _ => false
                    };

                    let squaddie = &mut self.squaddies[selected];

                    self.path = if same_path && squaddie.move_to(x, y, cost) {
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

    pub fn squaddie_at(&self, x: usize, y: usize) -> Option<(usize, &Unit)> {
        self.squaddies.iter().enumerate().find(|&(_, squaddie)| squaddie.x == x && squaddie.y == y)
    }

    pub fn enemy_at(&self, x: usize, y: usize) -> Option<(usize, &Unit)> {
        self.enemies.iter().enumerate().find(|&(_, enemy)| enemy.x == x && enemy.y == y)
    }

    pub fn taken(&self, x: usize, y: usize) -> bool {
        !self.tiles.tile_at(x, y).walkable ||
        self.squaddie_at(x, y).is_some() ||
        self.enemy_at(x, y).is_some()
    }

    // End the current turn
    fn end_turn(&mut self) {
        for squaddie in &mut self.squaddies {
            squaddie.moves = squaddie.max_moves;
        }
        self.turn += 1;
    }
}
