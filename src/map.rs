use pathfinding;

use std::cmp::{min, max};

use ggez::graphics;
use ggez::graphics::{Point, Image, DrawParam};
use ggez::Context;
use ggez::event::{Keycode, MouseButton};

use tiles::Tiles;
use images;
use Resources;
use {WINDOW_WIDTH, WINDOW_HEIGHT};
use units::{Squaddie, Enemy};
use ui::{UI, Button, TextDisplay};

const CAMERA_SPEED: f32 = 0.2;
const CAMERA_ZOOM_SPEED: f32 = 0.02;
const DEFAULT_ZOOM: f32 = 2.0;

const TILE_ROWS: usize = 20;
const TILE_COLS: usize = 20;

const TILE_WIDTH: f32 = 48.0;
const TILE_HEIGHT: f32 = 24.0;
const TILE_IMAGE_SIZE: f32 = 48.0;

const WALK_COST: usize = 2;
const WALK_DIAGONAL_COST: usize = 3;

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

pub struct Map {
    camera: Camera,
    cursor: Cursor,
    keys: [bool; 6],
    tiles: Tiles,
    squaddies: Vec<Squaddie>,
    enemies: Vec<Enemy>,
    selected: Option<usize>,
    path: Option<Vec<PathPoint>>,
    turn: u16,
    ui: UI
}

impl Map {
    pub fn new(resources: &Resources) -> Map {
        let scale = 2.0;

        let mut ui = UI::new();
        let end_turn_image = &resources.images[images::END_TURN_BUTTON];
        ui.add_button(Button::new(
            images::END_TURN_BUTTON,
            WINDOW_WIDTH as f32 - end_turn_image.width() as f32 * scale,
            WINDOW_HEIGHT as f32 - end_turn_image.height() as f32 * scale,
            scale,
            resources
        ));
        ui.add_button(Button::new(
            images::FIRE_BUTTON,
            WINDOW_WIDTH as f32 - end_turn_image.width() as f32 * scale * 2.0,
            WINDOW_HEIGHT as f32 - end_turn_image.height() as f32 * scale,
            scale,
            resources
        ));
        ui.add_text_display(TextDisplay::new(0.0, 0.0));


        Map {
            camera: Camera::new(),
            cursor: Cursor::new(),
            keys: [false; 6],
            tiles: Tiles::new(TILE_COLS, TILE_ROWS),
            squaddies: Vec::new(),
            enemies: Vec::new(),
            selected: None,
            path: None,
            turn: 1,
            ui
        }
    }

    pub fn start(&mut self) {
        // Generate tiles
        self.tiles.generate();

        // Add squaddies
        for x in 0..3 {
            self.squaddies.push(Squaddie::new(x, 0));
        }

        // Add enemies
        for y in TILE_ROWS - 3 .. TILE_ROWS {
            self.enemies.push(Enemy::new(TILE_ROWS - 1, y));
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
    }

    fn draw_at_scale(&self, ctx: &mut Context, image: &Image, x: f32, y: f32, scale: f32) {
        graphics::draw_ex(
            ctx,
            image,
            DrawParam {
                dest: Point::new(x, y),
                scale: Point::new(scale, scale),
                ..Default::default()
            }
        ).unwrap();
    }

    fn draw_image(&self, ctx: &mut Context, image: &Image, x: f32, y: f32) {
        let (x, y) = from_map_coords(x, y);
        let width = WINDOW_WIDTH as f32;
        let height = WINDOW_HEIGHT as f32;

        let x = (x - self.camera.x) * TILE_WIDTH  / 2.0 * self.camera.zoom + (width  / 2.0 - self.camera.x * self.camera.zoom);
        let y = (y - self.camera.y) * TILE_HEIGHT / 2.0 * self.camera.zoom + (height / 2.0 - self.camera.y * self.camera.zoom);

        let min_x = -TILE_IMAGE_SIZE / 2.0 * self.camera.zoom;
        let min_y = -TILE_IMAGE_SIZE / 2.0 * self.camera.zoom;
        let max_x = width  + TILE_IMAGE_SIZE / 2.0 * self.camera.zoom;
        let max_y = height + TILE_IMAGE_SIZE / 2.0 * self.camera.zoom;

        if x > min_x && x < max_x && y > min_y && y < max_y {
            self.draw_at_scale(ctx, image, x, y, self.camera.zoom);
        }   
    }

    pub fn draw(&mut self, ctx: &mut Context, resources: &Resources) {
        for x in 0 .. self.tiles.cols {
            for y in 0 .. self.tiles.rows {
                let tile = self.tiles.tile_at(x, y);
                let (x, y) = (x as f32, y as f32);

                self.draw_image(ctx, &resources.images[tile.image], x, y);

                match tile.decoration {
                    Some(decoration) => self.draw_image(ctx, &resources.images[decoration], x, y),
                    _ => {}
                }
            }
        }

        // Draw the edges

        self.draw_image(ctx, &resources.images[images::EDGE_LEFT_CORNER], 0.0, TILE_ROWS as f32);
        self.draw_image(ctx, &resources.images[images::EDGE_CORNER], TILE_COLS as f32, TILE_ROWS as f32);
        self.draw_image(ctx, &resources.images[images::EDGE_RIGHT_CORNER], TILE_COLS as f32, 0.0);

        for x in 1..TILE_COLS {
            self.draw_image(ctx, &resources.images[images::EDGE_LEFT], x as f32, TILE_ROWS as f32);
        }

        for y in 1..TILE_ROWS {
            self.draw_image(ctx, &resources.images[images::EDGE_RIGHT], TILE_COLS as f32, y as f32);
        }

        // Draw cursor
        match self.cursor.position {
            Some((x, y)) => {
                let tile = self.tiles.tile_at(x, y);

                let image = if !tile.walkable {
                    images::CURSOR_UNWALKABLE
                } else if self.squaddie_at(x, y).is_some() {
                    images::CURSOR_UNIT
                } else {
                    images::CURSOR
                };

                self.draw_image(ctx, &resources.images[image], x as f32, y as f32);
            }
            None => {}
        }

        // Draw path
        match self.path {
            Some(ref points) => {
                for point in points {
                    self.draw_image(ctx, &resources.images[images::PATH], point.x as f32, point.y as f32);
                }
            }
            None => {}
        }

        // Draw squaddies
        for squaddie in &self.squaddies {
            self.draw_image(ctx, &resources.images[squaddie.image], squaddie.x as f32, squaddie.y as f32);
        }

        // Draw enemies
        for enemy in &self.enemies {
            self.draw_image(ctx, &resources.images[enemy.image], enemy.x as f32, enemy.y as f32);
        }

        if self.cursor.fire {
            match self.cursor.position {
                Some((x, y)) => self.draw_image(ctx, &resources.images[images::CROSSHAIR], x as f32, y as f32),
                None => {}
            }
        }

        self.draw_ui(ctx, resources);
    }

    pub fn draw_ui(&mut self, ctx: &mut Context, resources: &Resources) {
        let selected = match self.selected {
            Some(i) => {
                let squaddie = &self.squaddies[i];
                format!("(Name: {}, ID: {}, X: {}, Y: {}, Moves: {})", squaddie.name, i, squaddie.x, squaddie.y, squaddie.moves)
            },
            None => String::from("~")
        };

        self.ui.set_text(0, format!("Turn: {}, Selected: {}", self.turn, selected));

        self.ui.draw(ctx, resources);
    }

    pub fn move_cursor(&mut self, x: f32, y: f32) {
        let center_x = WINDOW_WIDTH  as f32 / 2.0;
        let center_y = WINDOW_HEIGHT as f32 / 2.0;

        let x = (x - center_x + self.camera.x * self.camera.zoom) / TILE_WIDTH  / self.camera.zoom + self.camera.x / 2.0;
        let y = (y - center_y + self.camera.y * self.camera.zoom) / TILE_HEIGHT / self.camera.zoom + self.camera.y / 2.0;

        // Account for the images being square
        let y = y - 0.5;

        let (x, y) = to_map_coords(x, y);

        let x = x.round() as usize;
        let y = y.round() as usize;

        if x < TILE_COLS && y < TILE_ROWS {
            self.cursor.position = Some((x, y));
        } else {
            self.cursor.position = None;
        }
    }

    pub fn mouse_button(&mut self, button: MouseButton, x: f32, y: f32) {
        match button {
            MouseButton::Left => match self.ui.clicked(x, y) {
                Some(0) => self.end_turn(),
                Some(1) => self.cursor.fire = !self.cursor.fire,
                None => match self.cursor.position {
                    Some((x, y)) => {
                        if self.cursor.fire && self.selected.is_some() {
                            match self.enemy_at(x, y) {
                                Some(enemy) => {
                                    let squaddie = &self.squaddies[self.selected.unwrap()];
                                    squaddie.fire_at(enemy);
                                },
                                None => {}
                            }
                        } else {
                            self.selected = self.squaddie_at(x, y);
                        }
                    },
                    None => {}
                },
                _ => {}
            },
            MouseButton::Right => match self.cursor.position.and_then(|(x, y)| self.selected.map(|selected| (x, y, selected))) {
                Some((x, y, selected)) => {
                    if self.taken(x, y) { return; }

                    let start = PathPoint::from(&self.squaddies[selected]);
                    let end = PathPoint::new(x, y);

                    let path = pathfinding::astar(
                        &start,
                        |point| point.neighbours(&self),
                        |point| point.cost(&end),
                        |point| *point == end
                    );

                    if path.is_none() {
                        self.path = None;
                        return;
                    }

                    let (points, cost) = path.unwrap();

                    let same_path = match self.path {
                        Some(ref path) => {
                            let path_end = &path[path.len() - 1];
                            *path_end == end
                        }
                        None => false
                    };

                    let squaddie = &mut self.squaddies[selected];

                    if same_path && squaddie.moves >= cost {
                        squaddie.x = x;
                        squaddie.y = y;
                        squaddie.moves -= cost;
                        self.path = None;
                    } else {
                        self.path = Some(points);
                    }
                }
                None => {}
            },
            _ => {}
        }
    }

    fn squaddie_at(&self, x: usize, y: usize) -> Option<usize> {
        for (i, squaddie) in self.squaddies.iter().enumerate() {
            if squaddie.x == x && squaddie.y == y {
                return Some(i);
            }
        }

        None
    }

    fn enemy_at(&self, x: usize, y: usize) -> Option<&Enemy> {
        self.enemies.iter().find(|enemy| enemy.x == x && enemy.y == y)
    }

    fn taken(&self, x: usize, y: usize) -> bool {
        !self.tiles.tile_at(x, y).walkable ||
        self.squaddie_at(x, y).is_some() ||
        self.enemies.iter().any(|enemy| enemy.x == x && enemy.y == y)
    }

    /// End the current turn
    fn end_turn(&mut self) {
        for squaddie in &mut self.squaddies {
            squaddie.moves = squaddie.max_moves;
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
struct PathPoint {
    x: usize,
    y: usize
}

impl PathPoint {
    fn new(x: usize, y: usize) -> PathPoint {
        PathPoint {
            x,
            y
        }
    }

    fn from(squaddie: &Squaddie) -> PathPoint {
        PathPoint {
            x: squaddie.x,
            y: squaddie.y
        }
    }

    fn cost(&self, point: &PathPoint) -> usize {
        if self.x == point.x || self.y == point.y { WALK_COST } else { WALK_DIAGONAL_COST }
    }

    fn neighbours(&self, map: &Map) -> Vec<(PathPoint, usize)> {
        let mut neighbours = Vec::new();

        let min_x = max(0, self.x as i32 - 1) as usize;
        let min_y = max(0, self.y as i32 - 1) as usize;

        let max_x = min(map.tiles.cols - 1, self.x + 1);
        let max_y = min(map.tiles.rows - 1, self.y + 1);

        for x in min_x .. max_x + 1 {
            for y in min_y .. max_y + 1 {
                if !map.taken(x, y) {
                    let point = PathPoint::new(x, y);
                    let cost = self.cost(&point);
                    neighbours.push((point, cost));
                }
            }
        }

        return neighbours;
    }
}