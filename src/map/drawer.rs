use ggez::graphics::{Point, DrawParam, Text, Drawable};
use ggez::graphics;
use ggez::Context;

use std::f32::consts::PI;

use map::map::Map;
use Resources;
use images;

const TILE_WIDTH: f32 = 48.0;
const TILE_HEIGHT: f32 = 24.0;
const TILE_IMAGE_SIZE: f32 = 48.0;
const DEFAULT_ZOOM: f32 = 2.0;

fn from_map_coords(x: f32, y: f32) -> (f32, f32) {
    (x - y, x + y)
}

fn to_map_coords(x: f32, y: f32) -> (f32, f32) {
    (y + x, y - x)
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

    fn draw_tile(&self, ctx: &mut Context, image: &Drawable, x: usize, y: usize) {
        let (x, y) = self.draw_location(ctx, x as f32, y as f32);

        if self.tile_onscreen(ctx, x, y) {
            self.draw_scaled(ctx, image, x, y);
        }
    }

    fn draw_scaled_with_rotation(&self, ctx: &mut Context, image: &Drawable, x: f32, y: f32, rotation: f32) {
         graphics::draw_ex(
            ctx,
            image,
            DrawParam {
                dest: Point::new(x, y),
                scale: Point::new(self.camera.zoom, self.camera.zoom),
                rotation,
                ..Default::default()
            }
        ).unwrap();
    }

    fn draw_scaled(&self, ctx: &mut Context, image: &Drawable, x: f32, y: f32) {
        self.draw_scaled_with_rotation(ctx, image, x, y, 0.0);
    }

    fn tile_onscreen(&self, ctx: &Context, x: f32, y: f32) -> bool {
        let min = -TILE_IMAGE_SIZE / 2.0 * self.camera.zoom;
        let max_x = ctx.conf.window_width  as f32 + TILE_IMAGE_SIZE / 2.0 * self.camera.zoom;
        let max_y = ctx.conf.window_height as f32 + TILE_IMAGE_SIZE / 2.0 * self.camera.zoom;

        x > min && x < max_x && y > min && y < max_y
    }

    fn draw_location(&self, ctx: &Context, x: f32, y: f32) -> (f32, f32) {
        let (x, y) = from_map_coords(x, y);

        let width  = ctx.conf.window_width as f32;
        let height = ctx.conf.window_height as f32;

        let x = (x - self.camera.x) * TILE_WIDTH  / 2.0 * self.camera.zoom + (width  / 2.0 - self.camera.x * self.camera.zoom);
        let y = (y - self.camera.y) * TILE_HEIGHT / 2.0 * self.camera.zoom + (height / 2.0 - self.camera.y * self.camera.zoom);

        (x, y)
    }

    pub fn draw_map(&self, ctx: &mut Context, resources: &Resources, map: &Map) {
        for x in 0 .. map.tiles.cols {
            for y in 0 .. map.tiles.rows {
                let tile = map.tiles.tile_at(x, y);
                let (screen_x, screen_y) = self.draw_location(ctx, x as f32, y as f32);

                if self.tile_onscreen(ctx, screen_x, screen_y) {
                    self.draw_scaled(ctx, &resources.images[tile.base], screen_x, screen_y);

                    match tile.decoration {
                        Some(decoration) => self.draw_scaled(ctx, &resources.images[decoration], screen_x, screen_y),
                        _ => {}
                    }

                    if !map.cursor.fire {
                        match map.cursor.position {
                            Some((cursor_x, cursor_y)) => {
                                if cursor_x == x && cursor_y == y {
                                    let image = if !tile.walkable {
                                        images::CURSOR_UNWALKABLE
                                    } else if map.squaddie_at(x, y).is_some() {
                                        images::CURSOR_UNIT
                                    } else {
                                        images::CURSOR
                                    };

                                    self.draw_scaled(ctx, &resources.images[image], screen_x, screen_y);
                                }
                            },
                            _ => {}
                        }
                    }

                    match map.squaddie_at(x, y) {
                        Some((index, squaddie)) => {
                            match map.selected {
                                Some(selected) => if selected == index {
                                    self.draw_scaled(ctx, &resources.images[images::CURSOR_UNIT], screen_x, screen_y);
                                },
                                None => {}
                            }
                            self.draw_scaled(ctx, &resources.images[squaddie.image()], screen_x, screen_y)
                        },
                        _ => {}
                    }

                    match map.enemy_at(x, y) {
                        Some((_, enemy)) => self.draw_scaled(ctx, &resources.images[enemy.image()], screen_x, screen_y),
                        _ => {}
                    }

                    if map.cursor.fire {
                        match map.cursor.position {
                            Some((cursor_x, cursor_y)) => if cursor_x == x && cursor_y == y {
                                self.draw_scaled(ctx, &resources.images[images::CURSOR_CROSSHAIR], screen_x, screen_y);
                            },
                            None => {}
                        }
                    }
                }
            }
        }

        self.draw_tile(ctx, &resources.images[images::EDGE_LEFT_CORNER], 0, map.tiles.rows);
        self.draw_tile(ctx, &resources.images[images::EDGE_CORNER], map.tiles.cols, map.tiles.rows);
        self.draw_tile(ctx, &resources.images[images::EDGE_RIGHT_CORNER], map.tiles.cols, 0);

        for x in 1..map.tiles.cols {
            self.draw_tile(ctx, &resources.images[images::EDGE_LEFT], x, map.tiles.rows);
        }

        for y in 1..map.tiles.rows {
            self.draw_tile(ctx, &resources.images[images::EDGE_RIGHT], map.tiles.cols, y);
        }

        // Draw path
        match map.path {
            Some(ref points) => {
                let squaddie = &map.squaddies[map.selected.unwrap()];

                for point in points {
                    let (x, y) = self.draw_location(ctx, point.x as f32, point.y as f32);

                    if self.tile_onscreen(ctx, x, y) {
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

        for bullet in &map.bullets {
            let (x, y) = self.draw_location(ctx, bullet.x, bullet.y);
            if self.tile_onscreen(ctx, x, y) {
                self.draw_scaled_with_rotation(ctx, &resources.images[images::BULLET], x, y, bullet.direction + PI/4.0);
            }
        }
    }

    pub fn tile_under_cursor(&self, ctx: &mut Context, x: f32, y: f32) -> (usize, usize) {
        // Get the center of the window
        let center_x = ctx.conf.window_width  as f32 / 2.0;
        let center_y = ctx.conf.window_height as f32 / 2.0;

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
        (x.round() as usize, y.round() as usize)
    }
}