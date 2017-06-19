use sdl2::render::{Texture, Canvas};
use sdl2::rect::Rect;
use sdl2::video::Window;

use map::map::Map;
use map::units::UnitSide;
use map::tiles::Visibility;
use Resources;
use context::Context;
use utils::{bound_f, chance_to_hit, convert_rotation};

const TILE_WIDTH: u32 = 48;
const TILE_HEIGHT: u32 = 24;
const TILE_IMAGE_SIZE: u32 = 48;

const DEFAULT_ZOOM: f32 = 2.0;
const ZOOM_MAX: f32 = 10.0;
const ZOOM_MIN: f32 = 1.0;

// Convert coordinates from isometric
fn from_map_coords(x: f32, y: f32) -> (f32, f32) {
    (x - y, x + y)
}

// Convert coordinates to isometric
fn to_map_coords(x: f32, y: f32) -> (f32, f32) {
    (y + x, y - x)
}

// A struct for writing to a canvas
pub struct CanvasTexture<'a> {
    canvas: &'a mut Canvas<Window>,
    width: u32,
    height: u32,
    camera: &'a Camera
}

impl<'a> CanvasTexture<'a> {
    // Create a new CanvasTexture
    fn new(canvas: &'a mut Canvas<Window>, width: u32, height: u32, camera: &'a Camera) -> CanvasTexture<'a> {
        CanvasTexture {
            canvas, width, height, camera
        }
    }

    // Clear the canvas
    fn clear(&mut self) {
        self.canvas.clear();
    }

    // Draw a texture on the canvas at (x, y) at the same size as the image
    fn draw(&mut self, image: &Texture, x: i32, y: i32) {
        let query = image.query();

        self.canvas.copy(image, None, Some(Rect::new(x, y, query.width, query.height))).unwrap();
    }

    // Draw a tile at the correct location if it is on screen
    fn draw_tile(&mut self, image: &Texture, x: usize, y: usize) {
        let (x, y) = self.draw_location(x as f32, y as f32);

        if self.on_screen(x, y) {
            self.draw(image, x, y);
        }
    }

    // Draw a texture with a particular rotation
    fn draw_with_rotation(&mut self, image: &Texture, x: i32, y: i32, angle: f64) {
        let query = image.query();

        self.canvas.copy_ex(image, None, Some(Rect::new(x, y, query.width, query.height)),
                            angle, None, false, false).unwrap();
    }

    // Calculate if a tile is on screen
    fn on_screen(&self, x: i32, y: i32) -> bool {
        let min = (-(TILE_IMAGE_SIZE as f32) * self.camera.zoom) as i32;
        let max_x = ((self.width  + TILE_IMAGE_SIZE / 2) as f32 * self.camera.zoom) as i32;
        let max_y = ((self.height + TILE_IMAGE_SIZE / 2) as f32 * self.camera.zoom) as i32;

        x > min && x < max_x && y > min && y < max_y
    }

    // Calculate the correct position to draw a tile on the screen
    fn draw_location(&self, x: f32, y: f32) -> (i32, i32) {
        let (x, y) = from_map_coords(x, y);
        let (tile_width, tile_height) = (TILE_WIDTH as f32, TILE_HEIGHT as f32);

        let x = (x * tile_width  - (self.camera.x * tile_width  - self.width  as f32)) / 2.0;
        let y = (y * tile_height - (self.camera.y * tile_height - self.height as f32)) / 2.0;
        
        (x as i32, y as i32)
    }
}

// A simple camera for what the user is looking at
pub struct Camera {
    pub x: f32,
    pub y: f32,
    pub zoom: f32
}

// The drawer object
pub struct Drawer {
    pub camera: Camera,
}

impl Drawer {
    // Create a new Drawer
    pub fn new() -> Drawer {
        Drawer {
            camera: Camera { x: 0.0, y: 0.0, zoom: DEFAULT_ZOOM }
        }
    }

    // Zoom in the camera by a particular amount, checking if it's zoomed in/out too far
    pub fn zoom(&mut self, amount: f32) {
        self.camera.zoom += amount * self.camera.zoom;

        if self.camera.zoom > ZOOM_MAX { self.camera.zoom = ZOOM_MAX; }
        if self.camera.zoom < ZOOM_MIN { self.camera.zoom = ZOOM_MIN; }
    }

    // Draw the map onto a CanvasTexture
    pub fn draw_to_canvas(&self, canvas: &mut CanvasTexture, resources: &Resources, map: &Map) {
        // Loop through tiles
        for x in 0 .. map.tiles.cols {
            for y in 0 .. map.tiles.rows {
                // Get the tile
                let tile = map.tiles.tile_at(x, y);
                // Get the position of the tile in the screen
                let (screen_x, screen_y) = canvas.draw_location(x as f32, y as f32);

                if tile.visible() && canvas.on_screen(screen_x, screen_y) {
                    // Draw the tile base
                    canvas.draw(resources.image(&tile.base), screen_x, screen_y);

                    // Draw the tile decoration
                    match tile.decoration {
                        Some(ref decoration) => canvas.draw(resources.image(&decoration), screen_x, screen_y),
                        _ => {}
                    }

                    if tile.visibility != Visibility::Foggy {
                        // Draw the cursor if it's not in fire mode
                        if !map.cursor.fire {
                            match map.cursor.position {
                                Some((cursor_x, cursor_y)) => {
                                    if cursor_x == x && cursor_y == y {
                                        // Determine the cursor colour
                                        let image = if !tile.walkable {
                                            "cursor_unwalkable"
                                        } else if map.units.at(x, y).is_some() {
                                            "cursor_unit"
                                        } else {
                                            "cursor"
                                        };

                                        canvas.draw(resources.image(&image.into()), screen_x, screen_y);
                                    }
                                },
                                _ => {}
                            }
                        }

                        // Draw a squaddie at the position
                        match map.units.at(x, y) {
                            Some((index, unit)) => {
                                // Draw the cursor to show that the unit is selected
                                match map.selected {
                                    Some(selected) => if selected == index {
                                        canvas.draw(resources.image(&"cursor_unit".into()), screen_x, screen_y);
                                    },
                                    _ => {}
                                }

                                canvas.draw(resources.image(&unit.image), screen_x, screen_y);
                            },
                            _ => {}
                        }
                    } else {
                        canvas.draw(resources.image(&"fog".into()), screen_x, screen_y);
                    }
                }
            }
        }

        // Draw the edge corners if visible

        if map.tiles.tile_at(0, map.tiles.rows - 1).visible() {
            canvas.draw_tile(resources.image(&"edge_left_corner".into()), 0, map.tiles.rows);
        }
        
        if map.tiles.tile_at(map.tiles.cols - 1, map.tiles.rows - 1).visible() {
            canvas.draw_tile(resources.image(&"edge_corner".into()), map.tiles.cols, map.tiles.rows);
        }

        if map.tiles.tile_at(map.tiles.cols - 1, 0).visible() {
            canvas.draw_tile(resources.image(&"edge_right_corner".into()), map.tiles.cols, 0);
        }

        // Draw the edges

        for x in 1 .. map.tiles.cols {
            if map.tiles.tile_at(x, map.tiles.rows - 1).visible() {
                canvas.draw_tile(resources.image(&"edge_left".into()), x, map.tiles.rows);
            }
        }

        for y in 1 .. map.tiles.rows {
            if map.tiles.tile_at(map.tiles.cols - 1, y).visible() {
                canvas.draw_tile(resources.image(&"edge_right".into()), map.tiles.cols, y);
            }
        }

        // Draw the path
        match map.path {
            Some(ref points) => {
                let mut total_cost = 0;

                // Get the squaddie the path if for
                let unit = map.units.get(map.selected.unwrap());

                for point in points {
                    total_cost += point.cost;

                    let (x, y) = canvas.draw_location(point.x as f32, point.y as f32);

                    if canvas.on_screen(x, y) {
                        // Get the image for the path
                        let image = if total_cost > unit.moves {
                            "path_unreachable"
                        } else if total_cost + unit.weapon.cost > unit.moves {
                            "path_no_weapon"
                        } else {
                            "path"
                        };

                        // Rendet the path cost
                        let cost = resources.render("main", &format!("{}", total_cost));
                        let center = (TILE_WIDTH as f32 - cost.query().width as f32) / 2.0;

                        canvas.draw(&cost, x + center as i32, y);
                        canvas.draw(resources.image(&image.into()), x, y);
                    }
                }
            }
            _ => {}
        }

        // Draw the fire crosshair
        if map.cursor.fire {
            match map.cursor.position {
                Some((x, y)) => {
                    let (screen_x, screen_y) = canvas.draw_location(x as f32, y as f32);

                    if canvas.on_screen(screen_x, screen_y) {
                        // Draw the crosshair
                        canvas.draw(resources.image(&"cursor_crosshair".into()), screen_x, screen_y);

                        // Draw the chance-to-hit if a squaddie is selected and an enemy is at the cursor position
                        match map.selected.and_then(|selected| map.units.at_i(x, y).map(|target| (selected, target))) {
                            Some((selected, target)) => {
                                let firing = map.units.get(selected);
                                let target = map.units.get(target);

                                if firing.side == UnitSide::Friendly && target.side == UnitSide::Enemy {
                                    // Get the chance to hit as a percentage
                                    let hit_chance = chance_to_hit(firing.x, firing.y, target.x, target.y) * 100.0;

                                    // Render it and draw it at the center
                                    let chance = resources.render("main", &format!("{:0.3}%", hit_chance));
                                    let center = (TILE_WIDTH as f32 - chance.query().width as f32) / 2.0;
                                    canvas.draw(&chance, screen_x + center as i32, screen_y - TILE_HEIGHT as i32);
                                }
                            }
                            _ => {}
                        }
                    }                    
                },
                _ => {}
            }
        }

        // If a bullet is the first item in the animation queue, draw it
        match map.animation_queue.first() {
            Some(bullet) => {
                // Calculate if the nearest tile to the bullet is visible
                let visible = map.tiles.tile_at(
                    bound_f(bullet.x.round(), 0, map.tiles.cols - 1),
                    bound_f(bullet.y.round(), 0, map.tiles.rows - 1)
                ).visible();
                // Get the drawing location of the bullet
                let (x, y) = canvas.draw_location(bullet.x, bullet.y);

                // If the bullet is visable and on screen, draw it with the right rotation
                if visible && canvas.on_screen(x, y) {
                    canvas.draw_with_rotation(resources.image(&bullet.image), x, y, convert_rotation(bullet.direction));
                }
            }
            _ => {}
        }
    }

    pub fn draw_map(&self, ctx: &mut Context, resources: &Resources, map: &Map) {
        // Get the width and height of the screen
        let (width, height) = (ctx.width(), ctx.height());

        // Create a texture to render into
        let mut texture = resources.create_texture(width, height);

        // As I had problems with seams between textures before,
        // the strategy to render the map is to render it into the texture
        // and _then_ scale it to the screen, so here we use the canvas as a texture
        // and wrap it in a CanvasTexture object.
        ctx.canvas.with_texture_canvas(&mut texture, |canvas| {
            let mut canvas = CanvasTexture::new(canvas, width, height, &self.camera);
            // Clear the canvas
            canvas.clear();

            // And draw to it
            self.draw_to_canvas(&mut canvas, resources, map);
        }).unwrap(); 

        // Work out the center of the screen
        let (center_x, center_y) = (width as f32 / 2.0, height as f32 / 2.0);

        // Draw the map texture onto the screen at the correct location
        ctx.draw(&texture, center_x - center_x * self.camera.zoom, center_y - center_y * self.camera.zoom, self.camera.zoom);
    }

    pub fn tile_under_cursor(&self, ctx: &mut Context, x: f32, y: f32) -> (usize, usize) {
        // Get the center of the window
        let center_x = ctx.width()  as f32 / 2.0;
        let center_y = ctx.height() as f32 / 2.0;

        // Work out the position of the mouse on the screen relative to the camera
        let x = (x - center_x) / TILE_WIDTH as f32  / self.camera.zoom + self.camera.x / 2.0 - 0.5;
        let y = (y - center_y) / TILE_HEIGHT as f32 / self.camera.zoom + self.camera.y / 2.0 - 0.5;

        // Account for the images being square
        let y = y - 1.0;

        // Convert to map coordinates
        let (x, y) = to_map_coords(x, y);

        // And then to usize
        (x.round() as usize, y.round() as usize)
    }
}