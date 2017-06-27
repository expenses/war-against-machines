//! A drawer struct for drawing the map and battle items

use sdl2::render::{Texture, Canvas};
use sdl2::rect::Rect;
use sdl2::video::Window;

use battle::battle::Battle;
use battle::units::UnitSide;
use battle::tiles::Visibility;
use battle::animations::Animation;
use Resources;
use context::Context;
use utils::{bound_float, chance_to_hit, convert_rotation};
use colours::WHITE;

const TILE_WIDTH: u32 = 48;
const TILE_HEIGHT: u32 = 24;
const TILE_IMAGE_SIZE: u32 = 48;

const DEFAULT_ZOOM: f32 = 2.0;
const ZOOM_MAX: f32 = 10.0;
const ZOOM_MIN: f32 = 1.0;

/// Convert coordinates from isometric
pub fn from_map_coords(x: f32, y: f32) -> (f32, f32) {
    (x - y, x + y)
}

/// Convert coordinates to isometric
pub fn to_map_coords(x: f32, y: f32) -> (f32, f32) {
    (y + x, y - x)
}

/// A struct to abstract drawing to a canvas
pub struct CanvasTexture<'a> {
    canvas: &'a mut Canvas<Window>,
    width: u32,
    height: u32,
    camera: &'a Camera
}

impl<'a> CanvasTexture<'a> {
    /// Create a new `CanvasTexture`
    pub fn new(canvas: &'a mut Canvas<Window>, width: u32, height: u32, camera: &'a Camera) -> CanvasTexture<'a> {
        CanvasTexture {
            canvas, width, height, camera
        }
    }

    /// Clear the canvas
    pub fn clear(&mut self) {
        self.canvas.clear();
    }

    /// Draw a texture on the canvas at (x, y) at the same size as the image
    pub fn draw(&mut self, image: &Texture, x: i32, y: i32) {
        let query = image.query();

        self.canvas.copy(image, None, Some(Rect::new(x, y, query.width, query.height))).unwrap();
    }

    /// Draw a tile at the correct location if it is on screen
    pub fn draw_tile(&mut self, image: &Texture, x: usize, y: usize) {
        let (x, y) = self.draw_location(x as f32, y as f32);

        if self.on_screen(x, y) {
            self.draw(image, x, y);
        }
    }

    /// Draw a texture with a particular rotation
    pub fn draw_with_rotation(&mut self, image: &Texture, x: i32, y: i32, angle: f64) {
        let query = image.query();

        self.canvas.copy_ex(image, None, Some(Rect::new(x, y, query.width, query.height)),
                            angle, None, false, false).unwrap();
    }

    /// Calculate if a tile is on screen
    pub fn on_screen(&self, x: i32, y: i32) -> bool {
        let min = (-(TILE_IMAGE_SIZE as f32) * self.camera.zoom) as i32;
        let max_x = ((self.width  + TILE_IMAGE_SIZE / 2) as f32 * self.camera.zoom) as i32;
        let max_y = ((self.height + TILE_IMAGE_SIZE / 2) as f32 * self.camera.zoom) as i32;

        x > min && x < max_x && y > min && y < max_y
    }

    /// Calculate the correct position to draw a tile on the screen
    pub fn draw_location(&self, x: f32, y: f32) -> (i32, i32) {
        let (x, y) = from_map_coords(x, y);
        let (tile_width, tile_height) = (TILE_WIDTH as f32, TILE_HEIGHT as f32);

        let x = (x * tile_width  - (self.camera.x * tile_width  - self.width  as f32)) / 2.0;
        let y = (y * tile_height - (self.camera.y * tile_height - self.height as f32)) / 2.0;
        
        (x as i32, y as i32)
    }
}

/// A simple camera for what the user is looking at
pub struct Camera {
    pub x: f32,
    pub y: f32,
    pub zoom: f32
}

/// The drawer struct
pub struct Drawer {
    pub camera: Camera,
}

impl Drawer {
    /// Create a new `Drawer`
    pub fn new() -> Drawer {
        Drawer {
            camera: Camera { x: 0.0, y: 0.0, zoom: DEFAULT_ZOOM }
        }
    }

    /// Zoom in the camera by a particular amount, checking if it's zoomed in/out too far
    pub fn zoom(&mut self, amount: f32) {
        self.camera.zoom += amount * self.camera.zoom;

        if self.camera.zoom > ZOOM_MAX { self.camera.zoom = ZOOM_MAX; }
        if self.camera.zoom < ZOOM_MIN { self.camera.zoom = ZOOM_MIN; }
    }

    /// Draw the map onto a `CanvasTexture`
    pub fn draw_to_canvas(&self, canvas: &mut CanvasTexture, resources: &Resources, battle: &Battle) {
        let map = &battle.map;

        // Loop through tiles
        for x in 0 .. map.tiles.cols {
            for y in 0 .. map.tiles.rows {
                // Get the tile
                let tile = map.tiles.at(x, y);
                // Get the position of the tile in the screen
                let (screen_x, screen_y) = canvas.draw_location(x as f32, y as f32);

                if tile.visible() && canvas.on_screen(screen_x, screen_y) {
                    // Draw the tile base
                    canvas.draw(resources.image(&tile.base), screen_x, screen_y);

                    // Draw the tile decoration
                    if let Some(ref obstacle) = tile.obstacle {
                        canvas.draw(resources.image(&obstacle), screen_x, screen_y);
                    }

                    // Draw the cursor if it isn't on an ai unit and or a unit isn't selected
                    if !battle.cursor_on_ai_unit() || battle.selected.is_none() {
                        if let Some((cursor_x, cursor_y)) = battle.cursor.position {
                            if cursor_x == x && cursor_y == y {
                                // Determine the cursor colour
                                let image = if !tile.walkable() {
                                    "cursor_unwalkable"
                                } else if map.units.at(x, y).is_some() {
                                    "cursor_unit"
                                } else {
                                    "cursor"
                                };

                                canvas.draw(resources.image(image), screen_x, screen_y);
                            }
                        }
                    }

                    if tile.player_visibility != Visibility::Foggy {
                        for item in &tile.items {
                            canvas.draw(resources.image(&item.image), screen_x, screen_y);
                        }

                        // Draw a unit at the position
                        if let Some((index, unit)) = map.units.at(x, y) {
                            // Draw the cursor to show that the unit is selected
                            if let Some(selected) = battle.selected {
                                if selected == index {
                                    canvas.draw(resources.image("cursor_unit"), screen_x, screen_y);
                                }
                            }

                            canvas.draw(resources.image(&unit.image), screen_x, screen_y);
                        }
                    } else {
                        canvas.draw(resources.image("fog"), screen_x, screen_y);
                    }
                }
            }
        }

        // Draw the edge corners if visible

        if map.tiles.at(0, map.tiles.rows - 1).visible() {
            canvas.draw_tile(resources.image("edge_left_corner"), 0, map.tiles.rows);
        }
        
        if map.tiles.at(map.tiles.cols - 1, map.tiles.rows - 1).visible() {
            canvas.draw_tile(resources.image("edge_corner"), map.tiles.cols, map.tiles.rows);
        }

        if map.tiles.at(map.tiles.cols - 1, 0).visible() {
            canvas.draw_tile(resources.image("edge_right_corner"), map.tiles.cols, 0);
        }

        // Draw the edges

        for x in 1 .. map.tiles.cols {
            if map.tiles.at(x, map.tiles.rows - 1).visible() {
                canvas.draw_tile(resources.image("edge_left"), x, map.tiles.rows);
            }
        }

        for y in 1 .. map.tiles.rows {
            if map.tiles.at(map.tiles.cols - 1, y).visible() {
                canvas.draw_tile(resources.image("edge_right"), map.tiles.cols, y);
            }
        }

        // Draw the path
        if let Some(ref points) = battle.path {
            if let Some(unit) = map.units.get(battle.selected.unwrap()) {
                let mut total_cost = 0;

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
                        let cost = resources.render("main", &format!("{}", total_cost), WHITE);
                        let center = (TILE_WIDTH as f32 - cost.query().width as f32) / 2.0;

                        canvas.draw(&cost, x + center as i32, y);
                        canvas.draw(resources.image(image), x, y);
                    }
                }
            }
        }

        // Draw the firing crosshair if the cursor is on an ai unit and a unit is selected
        if battle.cursor_on_ai_unit() && battle.selected.is_some() {
            if let Some((x, y)) = battle.cursor.position {
                let (screen_x, screen_y) = canvas.draw_location(x as f32, y as f32);

                if canvas.on_screen(screen_x, screen_y) {
                    // Draw the crosshair
                    canvas.draw(resources.image("cursor_crosshair"), screen_x, screen_y);

                    // Draw the chance-to-hit if a player unit is selected and an ai unit is at the cursor position
                    if let Some((firing, target)) = battle.selected.and_then(|firing|
                        map.units.get(firing).and_then(|firing|
                            map.units.at(x, y).map(|(_, target)|
                                (firing, target)
                            )
                        )
                    ) {
                        if firing.side == UnitSide::Player && target.side == UnitSide::AI {
                            // Get the chance to hit as a percentage
                            let hit_chance = chance_to_hit(firing.x, firing.y, target.x, target.y) * 100.0;

                            // Render it and draw it at the center
                            let chance = resources.render("main", &format!("{:0.3}%", hit_chance), WHITE);
                            let center = (TILE_WIDTH as f32 - chance.query().width as f32) / 2.0;
                            canvas.draw(&chance, screen_x + center as i32, screen_y - TILE_HEIGHT as i32);
                        }
                    }
                }
            }
        }

        // If a bullet is the first item in the animation queue, draw it
        for bullet in battle.animations.iter().filter_map(|animation| match animation {
            &Animation::Bullet(ref bullet) => Some(bullet),
            _ => None
        }) {
            // Calculate if the nearest tile to the bullet is visible
            let visible = map.tiles.at(
                bound_float(bullet.x, 0, map.tiles.cols - 1),
                bound_float(bullet.y, 0, map.tiles.rows - 1)
            ).visible();
            // Get the drawing location of the bullet
            let (x, y) = canvas.draw_location(bullet.x, bullet.y);

            // If the bullet is visable and on screen, draw it with the right rotation
            if visible && canvas.on_screen(x, y) {
                canvas.draw_with_rotation(resources.image(&bullet.image), x, y, convert_rotation(bullet.direction));
            }
        }
    }

    /// Draw the battle
    pub fn draw_battle(&self, ctx: &mut Context, resources: &Resources, battle: &Battle) {
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
            self.draw_to_canvas(&mut canvas, resources, battle);
        }).unwrap(); 

        // Work out the center of the screen
        let (center_x, center_y) = (width as f32 / 2.0, height as f32 / 2.0);

        // Draw the map texture onto the screen at the correct location
        ctx.draw(&texture, center_x - center_x * self.camera.zoom, center_y - center_y * self.camera.zoom, self.camera.zoom);
    }

    /// Work out which tile is under the cursor
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