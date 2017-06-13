use sdl2::render::Texture;

use map::map::Map;
use Resources;
use context::Context;

const TILE_WIDTH: f32 = 48.0;
const TILE_HEIGHT: f32 = 24.0;
// const TILE_IMAGE_SIZE: f32 = 48.0;
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

    fn draw_tile(&self, ctx: &mut Context, image: &Texture, x: usize, y: usize) {
        let (x, y) = self.draw_location(ctx, x as f32, y as f32);

        //if self.tile_onscreen(ctx, x, y) {
        ctx.draw(image, x, y, self.camera.zoom);
        //}
    }

    fn tile_onscreen(&self, _ctx: &Context, _x: f32, _y: f32) -> bool {
        // let min = -TILE_IMAGE_SIZE / 2.0 * self.camera.zoom;
        // let max_x = ctx.width()  + TILE_IMAGE_SIZE / 2.0 * self.camera.zoom;
        // let max_y = ctx.height() + TILE_IMAGE_SIZE / 2.0 * self.camera.zoom;

        true
        //x > min && x < max_x && y > min && y < max_y
    }

    fn draw_location(&self, ctx: &Context, x: f32, y: f32) -> (f32, f32) {
        let (x, y) = from_map_coords(x, y);

        let x = (x - self.camera.x) * TILE_WIDTH  / 2.0 * self.camera.zoom + (ctx.width()  / 2.0 - self.camera.x * self.camera.zoom);
        let y = (y - self.camera.y) * TILE_HEIGHT / 2.0 * self.camera.zoom + (ctx.height() / 2.0 - self.camera.y * self.camera.zoom);

        (x, y)
    }

    pub fn draw_map(&self, ctx: &mut Context, resources: &Resources, map: &Map) {
        for x in 0 .. map.tiles.cols {
            for y in 0 .. map.tiles.rows {
                let tile = map.tiles.tile_at(x, y);
                let (screen_x, screen_y) = self.draw_location(ctx, x as f32, y as f32);

                if self.tile_onscreen(ctx, screen_x, screen_y) {
                    ctx.draw(resources.image(tile.base.as_str()), screen_x, screen_y, self.camera.zoom);

                    match tile.decoration {
                        Some(ref decoration) => ctx.draw(resources.image(decoration.as_str()), screen_x, screen_y, self.camera.zoom),
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

                                    ctx.draw(resources.image(image), screen_x, screen_y, self.camera.zoom);
                                }
                            },
                            _ => {}
                        }
                    }

                    match map.squaddie_at(x, y) {
                        Some((index, squaddie)) => {
                            match map.selected {
                                Some(selected) => if selected == index {
                                    ctx.draw(resources.image("cursor_unit"), screen_x, screen_y, self.camera.zoom);
                                },
                                None => {}
                            }
                            ctx.draw(resources.image(squaddie.image().as_str()), screen_x, screen_y, self.camera.zoom);
                        },
                        _ => {}
                    }

                    match map.enemy_at(x, y) {
                        Some((_, enemy)) => ctx.draw(resources.image(enemy.image().as_str()), screen_x, screen_y, self.camera.zoom),
                        _ => {}
                    }

                    if map.cursor.fire {
                        match map.cursor.position {
                            Some((cursor_x, cursor_y)) => if cursor_x == x && cursor_y == y {
                                ctx.draw(resources.image("cursor_crosshair"), screen_x, screen_y, self.camera.zoom);
                            },
                            None => {}
                        }
                    }
                }
            }
        }

        self.draw_tile(ctx, resources.image("edge_left_corner"), 0, map.tiles.rows);
        self.draw_tile(ctx, resources.image("edge_corner"), map.tiles.cols, map.tiles.rows);
        self.draw_tile(ctx, resources.image("edge_right_corner"), map.tiles.cols, 0);

        for x in 1..map.tiles.cols {
            self.draw_tile(ctx, resources.image("edge_left"), x, map.tiles.rows);
        }

        for y in 1..map.tiles.rows {
            self.draw_tile(ctx, resources.image("edge_right"), map.tiles.cols, y);
        }

        // Draw path
        match map.path {
            Some(ref points) => {
                let squaddie = &map.squaddies[map.selected.unwrap()];

                for point in points {
                    let (x, y) = self.draw_location(ctx, point.x as f32, point.y as f32);

                    if self.tile_onscreen(ctx, x, y) {
                        let image = if point.cost > squaddie.moves {
                            "path_unreachable"
                        } else if point.cost + squaddie.weapon.cost > squaddie.moves {
                            "path_no_weapon"
                        } else {
                            "path"
                        };

                        let cost = resources.render("main", format!("{}", point.cost).as_str());
                        let center = cost.query().width as f32 / 2.0;

                        ctx.draw(&cost, x + center, y, self.camera.zoom);
                        ctx.draw(resources.image(image), x, y, self.camera.zoom);
                    }
                }
            }
            _ => {}
        }

        for bullet in &map.bullets {
            let (x, y) = self.draw_location(ctx, bullet.x, bullet.y);
            if self.tile_onscreen(ctx, x, y) {
                ctx.draw_with_rotation(resources.image("bullet"), x, y, self.camera.zoom, (bullet.direction.to_degrees() + 45.0) as f64);
            }
        }
    }

    pub fn tile_under_cursor(&self, ctx: &mut Context, x: f32, y: f32) -> (usize, usize) {
        // Get the center of the window
        let center_x = ctx.width()  / 2.0;
        let center_y = ctx.height() / 2.0;

        // Convert the points to their locations on the map
        // This involves finding the points relative to the center of the screen and the camera
        // Then scaling them down to the proper locations and finally offsetting by half the camera position
        let x = (x - center_x + self.camera.x * self.camera.zoom) / TILE_WIDTH  / self.camera.zoom + self.camera.x / 2.0;
        let y = (y - center_y + self.camera.y * self.camera.zoom) / TILE_HEIGHT / self.camera.zoom + self.camera.y / 2.0;

        let (x, y) = (x - 0.5, y - 0.5);

        // Account for the images being square
        let y = y - 1.0;

        // Convert to map coordinates
        let (x, y) = to_map_coords(x, y);

        // And then to usize
        (x.round() as usize, y.round() as usize)
    }
}