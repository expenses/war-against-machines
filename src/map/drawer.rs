use sdl2::render::{Texture, Canvas};
use sdl2::rect::Rect;
use sdl2::video::Window;

use map::map::Map;
use Resources;
use context::Context;
use units::chance_to_hit;

const TILE_WIDTH: u32 = 48;
const TILE_HEIGHT: u32 = 24;
const TILE_IMAGE_SIZE: u32 = 48;
const DEFAULT_ZOOM: f32 = 2.0;

fn from_map_coords(x: f32, y: f32) -> (f32, f32) {
    (x - y, x + y)
}

fn to_map_coords(x: f32, y: f32) -> (f32, f32) {
    (y + x, y - x)
}

pub struct CanvasTexture<'a> {
    canvas: &'a mut Canvas<Window>,
    width: u32,
    height: u32,
    camera: &'a Camera
}

impl<'a> CanvasTexture<'a> {
    fn new(canvas: &'a mut Canvas<Window>, width: u32, height: u32, camera: &'a Camera) -> CanvasTexture<'a> {
        CanvasTexture {
            canvas, width, height, camera
        }
    }

    fn clear(&mut self) {
        self.canvas.clear();
    }

    fn draw(&mut self, image: &Texture, x: i32, y: i32) {
        let query = image.query();

        self.canvas.copy(image, None, Some(Rect::new(x, y, query.width, query.height))).unwrap();
    }

    fn draw_tile(&mut self, image: &Texture, x: usize, y: usize) {
        let (x, y) = self.draw_location(x as f32, y as f32);

        if self.on_screen(x, y) {
            self.draw(image, x, y);
        }
    }

    fn draw_with_rotation(&mut self, image: &Texture, x: i32, y: i32, angle: f32) {
        let query = image.query();

        self.canvas.copy_ex(image, None, Some(Rect::new(x, y, query.width, query.height)), angle as f64, None, false, false).unwrap();
    }

    fn on_screen(&self, x: i32, y: i32) -> bool {
        let min = (-(TILE_IMAGE_SIZE as f32) * self.camera.zoom) as i32;
        let max_x = ((self.width  + TILE_IMAGE_SIZE / 2) as f32 * self.camera.zoom) as i32;
        let max_y = ((self.height + TILE_IMAGE_SIZE / 2) as f32 * self.camera.zoom) as i32;

        x > min && x < max_x && y > min && y < max_y
    }

    fn draw_location(&self, x: f32, y: f32) -> (i32, i32) {
        let (x, y) = from_map_coords(x, y);
        let (tile_width, tile_height) = (TILE_WIDTH as f32, TILE_HEIGHT as f32);

        let x = (x * tile_width  - (self.camera.x * tile_width  - self.width  as f32)) / 2.0;
        let y = (y * tile_height - (self.camera.y * tile_height - self.height as f32)) / 2.0;
        
        (x as i32, y as i32)
    }
}

pub struct Camera {
    pub x: f32,
    pub y: f32,
    pub zoom: f32
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

pub struct Drawer {
    pub camera: Camera,
}

impl Drawer {
    pub fn new() -> Drawer {
        Drawer {
            camera: Camera::new()
        }
    }

    pub fn zoom(&mut self, amount: f32) {
        self.camera.zoom += amount * self.camera.zoom;

        if self.camera.zoom > 10.0 { self.camera.zoom = 10.0; }
        if self.camera.zoom < 1.0 { self.camera.zoom = 1.0; }
    }

    pub fn draw_to_canvas(&self, canvas: &mut CanvasTexture, resources: &Resources, map: &Map) {
        for x in 0 .. map.tiles.cols {
            for y in 0 .. map.tiles.rows {
                let tile = map.tiles.tile_at(x, y);
                let (screen_x, screen_y) = canvas.draw_location(x as f32, y as f32);

                if canvas.on_screen(screen_x, screen_y) {
                    canvas.draw(resources.image(tile.base.as_str()), screen_x, screen_y);

                    match tile.decoration {
                        Some(ref decoration) => canvas.draw(resources.image(decoration.as_str()), screen_x, screen_y),
                        _ => {}
                    }

                    if !map.cursor.fire {
                        match map.cursor.position {
                            Some((cursor_x, cursor_y)) => {
                                if cursor_x == x && cursor_y == y {
                                    let image = if !tile.walkable {
                                        "cursor_unwalkable"
                                    } else if map.squaddie_at(x, y).is_some() {
                                        "cursor_unit"
                                    } else {
                                        "cursor"
                                    };

                                    canvas.draw(resources.image(image), screen_x, screen_y);
                                }
                            },
                            _ => {}
                        }
                    }

                    match map.squaddie_at(x, y) {
                        Some((index, squaddie)) => {
                            match map.selected {
                                Some(selected) => if selected == index {
                                    canvas.draw(resources.image("cursor_unit"), screen_x, screen_y);
                                },
                                _ => {}
                            }

                            canvas.draw(resources.image(squaddie.image().as_str()), screen_x, screen_y);
                        },
                        _ => {}
                    }

                    match map.enemy_at(x, y) {
                        Some((_, enemy)) => canvas.draw(resources.image(enemy.image().as_str()), screen_x, screen_y),
                        _ => {}
                    }
                }
            }
        }

        canvas.draw_tile(resources.image("edge_left_corner"), 0, map.tiles.rows);
        canvas.draw_tile(resources.image("edge_corner"), map.tiles.cols, map.tiles.rows);
        canvas.draw_tile(resources.image("edge_right_corner"), map.tiles.cols, 0);

        for x in 1..map.tiles.cols {
            canvas.draw_tile(resources.image("edge_left"), x, map.tiles.rows);
        }

        for y in 1..map.tiles.rows {
            canvas.draw_tile(resources.image("edge_right"), map.tiles.cols, y);
        }

        // Draw path
        match map.path {
            Some(ref points) => {
                let squaddie = &map.squaddies[map.selected.unwrap()];

                for point in points {
                    let (x, y) = canvas.draw_location(point.x as f32, point.y as f32);

                    if canvas.on_screen(x, y) {
                        let image = if point.cost > squaddie.moves {
                            "path_unreachable"
                        } else if point.cost + squaddie.weapon.cost > squaddie.moves {
                            "path_no_weapon"
                        } else {
                            "path"
                        };

                        let cost = resources.render("main", format!("{}", point.cost).as_str());
                        let center = (TILE_WIDTH as f32 - cost.query().width as f32) / 2.0;

                        canvas.draw(&cost, x + center as i32, y);
                        canvas.draw(resources.image(image), x, y);
                    }
                }
            }
            _ => {}
        }

        if map.cursor.fire {
            match map.cursor.position {
                Some((x, y)) => {
                    let (screen_x, screen_y) = canvas.draw_location(x as f32, y as f32);

                    if canvas.on_screen(screen_x, screen_y) {
                        canvas.draw(resources.image("cursor_crosshair"), screen_x, screen_y);

                        match map.selected.and_then(|i| map.enemy_at(x, y).map(|(_, enemy)| (i, enemy))) {
                            Some((i, enemy)) => {
                                let squaddie = &map.squaddies[i];

                                let hit_chance = format!("{:0.3}%", chance_to_hit(squaddie, enemy) * 100.0);

                                let chance = resources.render("main", hit_chance.as_str());

                                let center = (TILE_WIDTH as f32 - chance.query().width as f32) / 2.0;

                                canvas.draw(&chance, screen_x + center as i32, screen_y - TILE_HEIGHT as i32);
                            }
                            _ => {}
                        }
                    }                    
                },
                _ => {}
            }
        }

        for bullet in &map.bullets {
            let (x, y) = canvas.draw_location(bullet.x, bullet.y);
            if canvas.on_screen(x, y) {
                canvas.draw_with_rotation(resources.image("bullet"), x, y, bullet.direction.to_degrees() + 45.0);
            }
        }
    }

    pub fn draw_map(&self, ctx: &mut Context, resources: &Resources, map: &Map) {
        let (width, height) = (ctx.width(), ctx.height());

        let mut texture = resources.create_texture(width, height);

        ctx.canvas.with_texture_canvas(&mut texture, |canvas| {
            let mut canvas = CanvasTexture::new(canvas, width, height, &self.camera);
            canvas.clear();

            self.draw_to_canvas(&mut canvas, resources, map);
        }).unwrap(); 

        let (center_x, center_y) = (width as f32 / 2.0, height as f32 / 2.0);

        ctx.draw(&texture, center_x - center_x * self.camera.zoom, center_y - center_y * self.camera.zoom, self.camera.zoom);
    }

    pub fn tile_under_cursor(&self, ctx: &mut Context, x: f32, y: f32) -> (usize, usize) {
        // Get the center of the window
        let center_x = ctx.width()  as f32 / 2.0;
        let center_y = ctx.height() as f32 / 2.0;

        let x = (x - center_x) / TILE_WIDTH as f32  / self.camera.zoom + self.camera.x / 2.0;
        let y = (y - center_y) / TILE_HEIGHT as f32 / self.camera.zoom + self.camera.y / 2.0;

        let (x, y) = (x - 0.5, y - 0.5);

        // Account for the images being square
        let y = y - 1.0;

        // Convert to map coordinates
        let (x, y) = to_map_coords(x, y);

        // And then to usize
        (x.round() as usize, y.round() as usize)
    }
}