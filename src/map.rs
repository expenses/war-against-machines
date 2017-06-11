use ggez::graphics;
use ggez::graphics::{Point, DrawParam, Text, Drawable};
use ggez::Context;
use ggez::event::{Keycode, MouseButton};

use tiles::Tiles;
use images;
use Resources;
use {WINDOW_WIDTH, WINDOW_HEIGHT};
use units::{Unit, UnitType, UnitSide};
use ui::{UI, Button, TextDisplay};
use paths::{pathfind, PathPoint};
// use items::ItemType;

const CAMERA_SPEED: f32 = 0.2;
const CAMERA_ZOOM_SPEED: f32 = 0.02;
const DEFAULT_ZOOM: f32 = 2.0;

const TILE_WIDTH: f32 = 48.0;
const TILE_HEIGHT: f32 = 24.0;
const TILE_IMAGE_SIZE: f32 = 48.0;

fn from_map_coords(x: f32, y: f32) -> (f32, f32) {
    (x - y, x + y)
}

fn to_map_coords(x: f32, y: f32) -> (f32, f32) {
    (y + x, y - x)
}

struct Camera {
    x: f32,
    y: f32,
    zoom: f32
}

impl Camera {
    fn new() -> Camera {
        Camera {
            x: 0.0,
            y: 0.0,
            zoom: DEFAULT_ZOOM
        }
    }
}

struct Cursor {
    position: Option<(usize, usize)>,
    fire: bool
}

impl Cursor {
    fn new() -> Cursor {
        Cursor {
            position: None,
            fire: false
        }
    }
}

/*
struct Fog {
    x: f32,
    y: f32,
    image: usize
}

impl Fog {
    fn new(x: f32, y: f32) -> Fog {
        Fog {
            x, y,
            image: images::FOG
        }
    }
}
*/

pub struct Map {
    pub tiles: Tiles,
    camera: Camera,
    cursor: Cursor,
    keys: [bool; 6],
    squaddies: Vec<Unit>,
    enemies: Vec<Unit>,
    selected: Option<usize>,
    path: Option<Vec<PathPoint>>,
    turn: u16,
    ui: UI,
    // fog: Vec<Fog>
}

impl Map {
    pub fn new(resources: &Resources) -> Map {
        let scale = 2.0;
        let button_image = &resources.images[images::END_TURN_BUTTON];
        let (width, height) = (button_image.width() as f32 * scale, button_image.height() as f32 * scale);

        let mut ui = UI::new();

        ui.buttons = vec![
            Button::new(
                images::END_TURN_BUTTON,
                WINDOW_WIDTH as f32 - width,
                WINDOW_HEIGHT as f32 - height,
                scale,
                resources
            ),
            Button::new(
                images::FIRE_BUTTON,
                WINDOW_WIDTH as f32 - width * 2.0,
                WINDOW_HEIGHT as f32 - height,
                scale,
                resources
            )
        ];
        ui.text_displays = vec![
            TextDisplay::new(5.0, 5.0)
        ];

        Map {
            tiles: Tiles::new(),
            camera: Camera::new(),
            cursor: Cursor::new(),
            keys: [false; 6],
            squaddies: Vec::new(),
            enemies: Vec::new(),
            selected: None,
            path: None,
            turn: 1,
            ui: ui,
            // fog: vec![Fog::new(-2.0, 3.0), Fog::new(-4.0, 2.0)]
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

    pub fn handle_key(&mut self, key: Keycode, pressed: bool) {
        match key {
            Keycode::Up    | Keycode::W => self.keys[0] = pressed,
            Keycode::Down  | Keycode::S => self.keys[1] = pressed,
            Keycode::Left  | Keycode::A => self.keys[2] = pressed,
            Keycode::Right | Keycode::D => self.keys[3] = pressed,
            Keycode::O                  => self.keys[4] = pressed,
            Keycode::P                  => self.keys[5] = pressed,
            _ => {}
        };
    }

    pub fn update(&mut self) {
        if self.keys[0] { self.camera.y -= CAMERA_SPEED; }
        if self.keys[1] { self.camera.y += CAMERA_SPEED; }
        if self.keys[2] { self.camera.x -= CAMERA_SPEED; }
        if self.keys[3] { self.camera.x += CAMERA_SPEED; }
        if self.keys[4] { self.camera.zoom -= CAMERA_ZOOM_SPEED * self.camera.zoom }
        if self.keys[5] { self.camera.zoom += CAMERA_ZOOM_SPEED * self.camera.zoom }

        if self.camera.zoom > 10.0 { self.camera.zoom = 10.0; }
        if self.camera.zoom < 1.0 { self.camera.zoom = 1.0; }

        /*for fog_cloud in &mut self.fog {
            fog_cloud.x += 0.01;
        }*/
    }

    fn draw_scaled(&self, ctx: &mut Context, image: &Drawable, x: f32, y: f32) {
        graphics::draw_ex(
            ctx,
            image,
            DrawParam {
                dest: Point::new(x, y),
                scale: Point::new(self.camera.zoom, self.camera.zoom),
                ..Default::default()
            }
        ).unwrap();
    }

    fn draw_tile(&self, ctx: &mut Context, image: &Drawable, x: usize, y: usize) {
        let (x, y) = self.draw_location(x as f32, y as f32);

        if self.tile_onscreen(x, y) {
            self.draw_scaled(ctx, image, x, y);
        }
    }

    pub fn tile_onscreen(&self, x: f32, y: f32) -> bool {
        let min = -TILE_IMAGE_SIZE / 2.0 * self.camera.zoom;
        let max_x = WINDOW_WIDTH as f32  + TILE_IMAGE_SIZE / 2.0 * self.camera.zoom;
        let max_y = WINDOW_HEIGHT as f32 + TILE_IMAGE_SIZE / 2.0 * self.camera.zoom;

        x > min && x < max_x && y > min && y < max_y
    }

    pub fn draw_location(&self, x: f32, y: f32) -> (f32, f32) {
        let (x, y) = from_map_coords(x, y);

        let width = WINDOW_WIDTH as f32;
        let height = WINDOW_HEIGHT as f32;

        let x = (x - self.camera.x) * TILE_WIDTH  / 2.0 * self.camera.zoom + (width  / 2.0 - self.camera.x * self.camera.zoom);
        let y = (y - self.camera.y) * TILE_HEIGHT / 2.0 * self.camera.zoom + (height / 2.0 - self.camera.y * self.camera.zoom);

        (x, y)
    }

    pub fn draw(&mut self, ctx: &mut Context, resources: &Resources) {
        for x in 0 .. self.tiles.cols {
            for y in 0 .. self.tiles.rows {
                let tile = self.tiles.tile_at(x, y);
                let (screen_x, screen_y) = self.draw_location(x as f32, y as f32);

                if self.tile_onscreen(screen_x, screen_y) {
                    self.draw_scaled(ctx, &resources.images[tile.base], screen_x, screen_y);

                    match tile.decoration {
                        Some(decoration) => self.draw_scaled(ctx, &resources.images[decoration], screen_x, screen_y),
                        _ => {}
                    }

                    match self.cursor.position {
                        Some((cursor_x, cursor_y)) => {
                            if cursor_x == x && cursor_y == y {
                                let image = if self.cursor.fire {
                                    images::CURSOR_CROSSHAIR
                                } else if !tile.walkable {
                                    images::CURSOR_UNWALKABLE
                                } else if self.squaddie_at(x, y).is_some() {
                                    images::CURSOR_UNIT
                                } else {
                                    images::CURSOR
                                };

                                self.draw_scaled(ctx, &resources.images[image], screen_x, screen_y);
                            }
                        },
                        _ => {}
                    }

                    match self.squaddie_at(x, y) {
                        Some((_, squaddie)) => self.draw_scaled(ctx, &resources.images[squaddie.image()], screen_x, screen_y),
                        _ => {}
                    }

                    match self.enemy_at(x, y) {
                        Some((_, enemy)) => self.draw_scaled(ctx, &resources.images[enemy.image()], screen_x, screen_y),
                        _ => {}
                    }
                }
            }
        }

        // Draw the edges

        self.draw_tile(ctx, &resources.images[images::EDGE_LEFT_CORNER], 0, self.tiles.rows);
        self.draw_tile(ctx, &resources.images[images::EDGE_CORNER], self.tiles.cols, self.tiles.rows);
        self.draw_tile(ctx, &resources.images[images::EDGE_RIGHT_CORNER], self.tiles.cols, 0);

        for x in 1..self.tiles.cols {
            self.draw_tile(ctx, &resources.images[images::EDGE_LEFT], x, self.tiles.rows);
        }

        for y in 1..self.tiles.rows {
            self.draw_tile(ctx, &resources.images[images::EDGE_RIGHT], self.tiles.cols, y);
        }

        // Draw path
        match self.path {
            Some(ref points) => {
                let squaddie = &self.squaddies[self.selected.unwrap()];

                for point in points {
                    let (x, y) = self.draw_location(point.x as f32, point.y as f32);

                    if self.tile_onscreen(x, y) {
                        let cost = Text::new(ctx, format!("{}", point.cost).as_str(), &resources.font).unwrap();
                        let image = if point.cost > squaddie.moves {
                            images::PATH_UNREACHABLE
                        } else if point.cost + squaddie.weapon.cost > squaddie.moves {
                            images::PATH_NO_WEAPON
                        } else {
                            images::PATH_DEFAULT
                        };

                        self.draw_scaled(ctx, &cost, x, y);
                        self.draw_scaled(ctx, &resources.images[image], x, y);
                    }
                }
            }
            _ => {}
        }

        /*for fog_cloud in &self.fog {
            let (x, y) = to_map_coords(fog_cloud.x, fog_cloud.y);
            let (x, y) = self.draw_location(x, y);

            self.draw_scaled(ctx, &resources.images[fog_cloud.image], x, y);
        }*/

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
            _ => String::from("~")
        };

        self.ui.set_text(0, format!("Turn: {}, Selected: {}", self.turn, selected));

        self.ui.draw(ctx, resources);
    }

    pub fn move_cursor(&mut self, x: f32, y: f32) {
        // Get the center of the window
        let center_x = WINDOW_WIDTH  as f32 / 2.0;
        let center_y = WINDOW_HEIGHT as f32 / 2.0;

        // Convert the points to their locations on the map
        // This involves finding the points relative to the center of the screen and the camera
        // Then scaling them down to the proper locations and finally offsetting by half the camera position
        let x = (x - center_x + self.camera.x * self.camera.zoom) / TILE_WIDTH  / self.camera.zoom + self.camera.x / 2.0;
        let y = (y - center_y + self.camera.y * self.camera.zoom) / TILE_HEIGHT / self.camera.zoom + self.camera.y / 2.0;

        // Account for the images being square
        let y = y - 0.5;

        // Convert to map coordinates
        let (x, y) = to_map_coords(x, y);

        // And then to usize
        let (x, y) = (x.round() as usize, y.round() as usize);

        // Set cursor position
        self.cursor.position = if x < self.tiles.cols && y < self.tiles.rows {
            Some((x, y))
        } else {
            None
        }
    }

    pub fn mouse_button(&mut self, button: MouseButton, x: f32, y: f32) {
        match button {
            MouseButton::Left => match self.ui.clicked(x, y) {
                Some(0) => self.end_turn(),
                Some(1) => self.cursor.fire = !self.cursor.fire,
                _ => match self.cursor.position {
                    Some((x, y)) => {
                        self.path = None;

                        if self.cursor.fire && self.selected.is_some() {
                            match self.enemies.iter_mut().find(|enemy| enemy.x == x && enemy.y == y) {
                                Some(enemy) => {
                                    let squaddie = &mut self.squaddies[self.selected.unwrap()];
                                    squaddie.fire_at(enemy);
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
                    if self.taken(x, y) {
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

    fn squaddie_at(&self, x: usize, y: usize) -> Option<(usize, &Unit)> {
        self.squaddies.iter().enumerate().find(|&(_, squaddie)| squaddie.x == x && squaddie.y == y)
    }

    fn enemy_at(&self, x: usize, y: usize) -> Option<(usize, &Unit)> {
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
