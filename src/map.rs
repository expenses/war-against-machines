use rand;
use rand::Rng;
use pathfinding;

use std::cmp::{min, max};

use ggez::graphics;
use ggez::graphics::{Point, Image, DrawParam, Text};
use ggez::Context;
use ggez::event::{Keycode, MouseButton};

use images;
use Resources;
use {WINDOW_WIDTH, WINDOW_HEIGHT};
use units::{Squaddie, Enemy};

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

#[derive(Copy, Clone)]
struct Tile {
    image: usize,
    decoration: Option<usize>,
    walkable: bool
}

impl Tile {
    fn new(image: usize) -> Tile {
        Tile {
            image,
            decoration: None,
            walkable: true
        }
    }

    fn set_pit(&mut self, decoration: usize) {
        self.decoration = Some(decoration);
        self.walkable = false;        
    }
}

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
    position: Option<(usize, usize)>
}

pub struct Map {
    camera: Camera,
    cursor: Cursor,
    keys: [bool; 6],
    tiles: [[Tile; TILE_ROWS]; TILE_COLS],
    squaddies: Vec<Squaddie>,
    enemies: Vec<Enemy>,
    selected: Option<usize>,
    path: Option<Vec<PathPoint>>,
    turn: u16
}

impl Map {
    pub fn new() -> Map {
        Map {
            camera: Camera::new(),
            cursor: Cursor { position: None },
            keys: [false; 6],
            tiles: [[Tile::new(images::MUD); TILE_ROWS]; TILE_COLS],
            squaddies: Vec::new(),
            enemies: Vec::new(),
            selected: None,
            path: None,
            turn: 1
        }
    }

    pub fn generate(&mut self) {
        for x in 0..TILE_COLS {
            for y in 0..TILE_ROWS {
                if rand::random::<f32>() > 0.975 {
                    self.tiles[x][y].decoration = Some(images::SKULL);
                } else if rand::random::<f32>() > 0.975 {
                    self.tiles[x][y].decoration = Some(images::MUD_POOL);
                }
            }
        }

        // Generate pit position and size
        let mut rng = rand::thread_rng();
        let pit_width = rng.gen_range(1, 4);
        let pit_height = rng.gen_range(1, 4);
        let pit_x = rng.gen_range(1, TILE_COLS - 1 - pit_width);
        let pit_y = rng.gen_range(1, TILE_ROWS - 1 - pit_height);

        // Add pit corners
        self.tiles[pit_x][pit_y].set_pit(images::PIT_TOP);
        self.tiles[pit_x][pit_y + pit_height].set_pit(images::PIT_LEFT);
        self.tiles[pit_x + pit_width][pit_y].set_pit(images::PIT_RIGHT);
        self.tiles[pit_x + pit_width][pit_y + pit_height].set_pit(images::PIT_BOTTOM);

        // Add pit edges and center

        for x in pit_x + 1 .. pit_x + pit_width {
            self.tiles[x][pit_y].set_pit(images::PIT_TR);
            self.tiles[x][pit_y + pit_height].set_pit(images::PIT_BL);

            for y in pit_y + 1 .. pit_y + pit_height {
                self.tiles[x][y].set_pit(images::PIT_CENTER);
            }
        }

        for y in pit_y + 1 .. pit_y + pit_height {
             self.tiles[pit_x][y].set_pit(images::PIT_TL);
             self.tiles[pit_x + pit_width][y].set_pit(images::PIT_BR);
        }

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
            Keycode::Up     => self.keys[0] = pressed,
            Keycode::Down   => self.keys[1] = pressed,
            Keycode::Left   => self.keys[2] = pressed,
            Keycode::Right  => self.keys[3] = pressed,
            Keycode::O      => self.keys[4] = pressed,
            Keycode::P      => self.keys[5] = pressed,
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

    pub fn draw_image(&self, ctx: &mut Context, image: &Image, x: f32, y: f32) {
        let (x, y) = from_map_coords(x, y);
        let width = WINDOW_WIDTH as f32;
        let height = WINDOW_HEIGHT as f32;

        let point = Point {
            x: (x - self.camera.x) * TILE_WIDTH  / 2.0 * self.camera.zoom + (width  / 2.0 - self.camera.x * self.camera.zoom),
            y: (y - self.camera.y) * TILE_HEIGHT / 2.0 * self.camera.zoom + (height / 2.0 - self.camera.y * self.camera.zoom)
        };

        let min_x = -TILE_IMAGE_SIZE / 2.0 * self.camera.zoom;
        let min_y = -TILE_IMAGE_SIZE / 2.0 * self.camera.zoom;
        let max_x = width  + TILE_IMAGE_SIZE / 2.0 * self.camera.zoom;
        let max_y = height + TILE_IMAGE_SIZE / 2.0 * self.camera.zoom;

        if point.x > min_x && point.x < max_x && point.y > min_y && point.y < max_y {
            graphics::draw_ex(
                ctx,
                image,
                DrawParam {
                    dest: point,
                    scale: Point {x: self.camera.zoom, y: self.camera.zoom},
                    ..Default::default()
                }
            ).unwrap();
        }   
    }

    pub fn draw(&mut self, ctx: &mut Context, resources: &Resources) {
        for x in 0..TILE_COLS {
            for y in 0..TILE_ROWS {
                let tile = self.tiles[x][y];
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
                let tile = self.tiles[x][y];

                let image = if !tile.walkable {
                    images::CURSOR_UNWALKABLE
                } else if self.squaddie_at(x, y) {
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

        // Print info

        let selected = match self.selected {
            Some(i) => {
                let squaddie = &self.squaddies[i];
                format!("(Name: {}, ID: {}, X: {}, Y: {}, Moves: {})", squaddie.name, i, squaddie.x, squaddie.y, squaddie.moves)
            },
            None => String::from("~")
        };

        let rendered = Text::new(ctx, format!("Turn: {}, Selected: {}", self.turn, selected).as_str(), &resources.font).unwrap();

        let point = Point {
            x: rendered.width() as f32 / 2.0 + 5.0,
            y: rendered.height() as f32
        };

        graphics::draw(ctx, &rendered, point, 0.0).unwrap();
    }

    pub fn move_cursor(&mut self, x: i32, y: i32) {
        let (x, y) = (x as f32, y as f32);
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

    pub fn mouse_button(&mut self, button: MouseButton, _x: i32, _y: i32) {
        match button {
            MouseButton::Left => match self.cursor.position {
                Some((x, y)) => {
                     for (i, squaddie) in self.squaddies.iter().enumerate() {
                        if squaddie.x == x && squaddie.y == y {
                            self.selected = Some(i);
                            break;
                        }
                        self.selected = None;
                    }
                },
                None => {}
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

                    if path.is_none() { return; }

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

    fn squaddie_at(&self, x: usize, y: usize) -> bool {
        self.squaddies.iter().any(|squaddie| squaddie.x == x && squaddie.y == y)
    }

    fn taken(&self, x: usize, y: usize) -> bool {
        !self.tiles[x][y].walkable ||
        self.squaddie_at(x, y) ||
        self.enemies.iter().any(|enemy| enemy.x == x && enemy.y == y)
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

        let max_x = min(TILE_COLS - 1, self.x + 1);
        let max_y = min(TILE_ROWS - 1, self.y + 1);

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