use rand;

use ggez::graphics;
use ggez::graphics::{Point, Image, DrawParam, Font, Text};
use ggez::Context;
use ggez::event::{Keycode, MouseButton};

use images;
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

struct Tile {
    image: usize,
    decoration: Option<usize>
}

impl Tile {
    fn new(image: usize) -> Tile {
        Tile {
            image: image,
            decoration: None
        }
    }
}

impl Copy for Tile {}
impl Clone for Tile {
    fn clone(&self) -> Tile {
        *self
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

struct Cursor {
    position: Option<(usize, usize)>
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

pub struct Map {
    camera: Camera,
    cursor: Cursor,
    keys: [bool; 6],
    tiles: [[Tile; TILE_ROWS]; TILE_COLS],
    squaddies: Vec<Squaddie>,
    enemies: Vec<Enemy>,
    selected: Option<usize>
}

impl Map {
    pub fn new() -> Map {
        Map {
            camera: Camera::new(),
            cursor: Cursor {position: None},
            keys: [false; 6],
            tiles: [[Tile::new(images::MUD); TILE_ROWS]; TILE_COLS],
            squaddies: Vec::new(),
            enemies: Vec::new(),
            selected: None
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

        for x in 0..3 {
            self.squaddies.push(Squaddie::new(x, 0));
        }

        for y in (TILE_ROWS - 3).. TILE_ROWS {
            self.enemies.push(Enemy::new(TILE_ROWS, y));
        }
    }

    pub fn handle_key(&mut self, key: Keycode, pressed: bool) {
        match key {
            Keycode::Up =>      self.keys[0] = pressed,
            Keycode::Down =>    self.keys[1] = pressed,
            Keycode::Left =>    self.keys[2] = pressed,
            Keycode::Right =>   self.keys[3] = pressed,
            Keycode::O =>       self.keys[4] = pressed,
            Keycode::P =>       self.keys[5] = pressed,
            _ => {}
        };
    }

    pub fn update(&mut self) {
        if self.keys[0] { self.camera.y -= CAMERA_SPEED; }
        if self.keys[1] { self.camera.y += CAMERA_SPEED; }
        if self.keys[2] { self.camera.x -= CAMERA_SPEED; }
        if self.keys[3] { self.camera.x += CAMERA_SPEED; }
        if self.keys[4] { self.camera.zoom -= CAMERA_ZOOM_SPEED * self.camera.zoom}
        if self.keys[5] { self.camera.zoom += CAMERA_ZOOM_SPEED * self.camera.zoom}

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

    pub fn draw(&mut self, ctx: &mut Context, images: &Vec<Image>, font: &Font) {
        for x in 0..TILE_COLS {
            for y in 0..TILE_ROWS {
                let tile = self.tiles[x][y];
                let x = x as f32;
                let y = y as f32;

                self.draw_image(ctx, &images[tile.image], x, y);

                match tile.decoration {
                    Some(decoration) => self.draw_image(ctx, &images[decoration], x, y),
                    _ => {}
                }
            }
        }

        self.draw_image(ctx, &images[images::EDGE_LEFT_CORNER], 0.0, TILE_ROWS as f32);

        for x in 1..TILE_COLS {
            self.draw_image(ctx, &images[images::EDGE_LEFT], x as f32, TILE_ROWS as f32);
        }

        self.draw_image(ctx, &images[images::EDGE_CORNER], TILE_COLS as f32, TILE_ROWS as f32);

        for y in 1..TILE_ROWS {
            self.draw_image(ctx, &images[images::EDGE_RIGHT], TILE_COLS as f32, y as f32);
        }

        self.draw_image(ctx, &images[images::EDGE_RIGHT_CORNER], TILE_COLS as f32, 0.0);

        match self.cursor.position {
            Some((x, y)) => {
                let image = if self.tiles[x][y].decoration.is_some() {images::CURSOR_SELECTED} else {images::CURSOR};
                self.draw_image(ctx, &images[image], x as f32, y as f32);
            },
            None => {}
        }

        for squaddie in &self.squaddies {
            self.draw_image(ctx, &images[squaddie.sprite], squaddie.x as f32, squaddie.y as f32);
        }

        for enemy in &self.enemies {
            self.draw_image(ctx, &images[enemy.sprite], enemy.x as f32, enemy.y as f32);
        }

        let text = match self.selected {
            Some(i) => &self.squaddies[i].name,
            None => "None"
        };

        let rendered = Text::new(ctx, format!("Selected: {}", text).as_str(), font).unwrap();

        graphics::draw(ctx, &rendered, Point{x: rendered.width() as f32, y: rendered.height() as f32}, 0.0).unwrap();
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
            MouseButton::Right => {
                if self.cursor.position.is_some() && self.selected.is_some() {
                    let (x, y) = self.cursor.position.unwrap();
                    let ref mut squaddie = self.squaddies[self.selected.unwrap()];
                    squaddie.x = x;
                    squaddie.y = y;
                }
            }
            _ => {}
        }
    }
}