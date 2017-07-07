// A drawer struct for drawing the map and battle items

use battle::Battle;
use battle::units::UnitSide;
use battle::tiles::Visibility;
use battle::animations::Animation;
use colours;
use Resources;
use utils::{clamp_float, convert_rotation};
use traits::Dimensions;

use graphics::{Context, Transformed};
use graphics::math::Matrix2d;
use opengl_graphics::GlGraphics;
use resources::SetImage;
use WindowSize;

const TILE_WIDTH: f64 = 48.0;
const TILE_HEIGHT: f64 = 24.0;
const TILE_IMAGE_SIZE: f64 = 48.0;

const DEFAULT_ZOOM: f64 = 2.0;
const ZOOM_MAX: f64 = 10.0;
const ZOOM_MIN: f64 = 1.0;

// Convert coordinates from isometric
pub fn from_map_coords(x: f64, y: f64) -> (f64, f64) {
    (x - y, x + y)
}

// Convert coordinates to isometric
pub fn to_map_coords(x: f64, y: f64) -> (f64, f64) {
    (y + x, y - x)
}

// A simple camera for what the user is looking at
pub struct Camera {
    pub x: f64,
    pub y: f64,
    zoom: f64
}

// The drawer struct
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
    pub fn zoom(&mut self, amount: f64) {
        self.camera.zoom += amount * self.camera.zoom;

        if self.camera.zoom > ZOOM_MAX { self.camera.zoom = ZOOM_MAX; }
        if self.camera.zoom < ZOOM_MIN { self.camera.zoom = ZOOM_MIN; }
    }

    fn draw_location(&self, ctx: &Context, x: f64, y: f64) -> Option<(f64, f64)> {
        let (width, height) = (ctx.width(), ctx.height());
        let (x, y) = from_map_coords(x, y);

        let x = (x - self.camera.x) * TILE_WIDTH  / 2.0 * self.camera.zoom + (ctx.width()  / 2.0 - self.camera.x * self.camera.zoom);
        let y = (y - self.camera.y) * TILE_HEIGHT / 2.0 * self.camera.zoom + (ctx.height() / 2.0 - self.camera.y * self.camera.zoom);

        if x > -TILE_IMAGE_SIZE * self.camera.zoom && y > -TILE_IMAGE_SIZE * 2.0 * self.camera.zoom && x < width && y < height {
            Some((x, y))
        } else {
            None
        }
    }

    fn draw_if_visible(&self, image: &SetImage, x: usize, y: usize, ctx: &Context, gl: &mut GlGraphics, resources: &mut Resources) {
        if let Some((x, y)) = self.draw_location(ctx, x as f64, y as f64) {
            resources.render(image, self.transformation(ctx, x, y), gl);
        }
    }

    fn transformation(&self, ctx: &Context, x: f64, y: f64) -> Matrix2d {
        ctx.transform.trans(x, y).scale(self.camera.zoom, self.camera.zoom)
    }

    pub fn draw_tile(&self, x: usize, y: usize, ctx: &Context, gl: &mut GlGraphics, resources: &mut Resources, battle: &Battle) {
        // Get the tile
        let tile = battle.map.tiles.at(x, y);

        // If the tile is invisible, return
        if !tile.visible() {
            return;
        }

        // If the tile is on the screen, draw it
        if let Some((screen_x, screen_y)) = self.draw_location(ctx, x as f64, y as f64) {
            let transformation = self.transformation(ctx, screen_x, screen_y);

            // Draw the tile base
            resources.render(&tile.base, transformation, gl);

            // Draw the tile decoration
            if let Some(ref obstacle) = tile.obstacle {
                resources.render(obstacle, transformation, gl);
            }

            // Draw the cursor if it isn't on an ai unit and or a unit isn't selected
            if !battle.cursor_on_ai_unit() || battle.selected.is_none() {
                if let Some((cursor_x, cursor_y)) = battle.cursor.position {
                    if cursor_x == x && cursor_y == y {
                        // Determine the cursor colour
                        let image = if !tile.walkable() {
                            SetImage::CursorUnwalkable
                        } else if battle.map.units.at(x, y).is_some() {
                            SetImage::CursorUnit
                        } else {
                            SetImage::Cursor
                        };

                        resources.render(&image, transformation, gl);
                    }
                }
            }

            if tile.player_visibility != Visibility::Foggy {
                for item in &tile.items {
                    resources.render(&item.image, transformation, gl);
                }

                // Draw a unit at the position
                if let Some(unit) = battle.map.units.at(x, y) {
                    // Draw the cursor to show that the unit is selected
                    if let Some(selected) = battle.selected {
                        if selected == unit.id {
                            resources.render(&SetImage::CursorUnit, transformation, gl);
                        }
                    }

                    resources.render(&unit.image, transformation, gl);
                }
            } else {
                resources.render(&SetImage::Fog, transformation, gl);
            }
        }
    }

    pub fn draw_battle(&self, ctx: &Context, gl: &mut GlGraphics, resources: &mut Resources, battle: &Battle) {
        let map = &battle.map;

        // Draw all the tiles
        for x in 0 .. map.tiles.cols {
            for y in 0 .. map.tiles.rows {
                self.draw_tile(x, y, ctx, gl, resources, battle);
            }
        }

        // Draw the edge edges if visible

        for x in 0 .. map.tiles.cols {
            if map.tiles.at(x, map.tiles.rows - 1).visible() {
                self.draw_if_visible(&SetImage::LeftEdge, x + 1, map.tiles.rows, ctx, gl, resources);
            }
        }

        for y in 0 .. map.tiles.rows {
            if map.tiles.at(map.tiles.cols - 1, y).visible() {
                self.draw_if_visible(&SetImage::RightEdge, map.tiles.cols, y + 1, ctx, gl, resources);
            }
        }

        // Draw the path
        if let Some(ref points) = battle.path {
            if let Some(unit) = map.units.get(battle.selected.unwrap()) {
                let mut total_cost = 0;

                // Draw the path tiles
                for point in points {
                    total_cost += point.cost;

                    if let Some((x, y)) = self.draw_location(ctx, point.x as f64, point.y as f64) {
                        // Render the tile

                        let image = if total_cost > unit.moves {
                            SetImage::PathUnreachable
                        } else if total_cost + unit.weapon.info().cost > unit.moves {
                            SetImage::PathCannotFire
                        } else {
                            SetImage::Path
                        };

                        resources.render(&image, self.transformation(ctx, x, y), gl);
                    }
                }

                total_cost = 0;

                for point in points {
                    total_cost += point.cost;

                    if let Some((x, y)) = self.draw_location(ctx, point.x as f64, point.y as f64) {
                        let cost = format!("{}", total_cost);

                        let offset_x = (TILE_WIDTH - resources.font_width(&cost)) / 2.0 * self.camera.zoom;
                        let offset_y = TILE_WIDTH / 2.0 * self.camera.zoom;

                        // Render the path cost
                        resources.render_text(&cost, colours::OFF_WHITE, self.transformation(ctx, x + offset_x, y + offset_y), gl);
                    }
                }
            }
        }

        // Draw the firing crosshair if the cursor is on an ai unit and a unit is selected
        if battle.cursor_on_ai_unit() && battle.selected.is_some() {
            if let Some((x, y)) = battle.cursor.position {
                if let Some((screen_x, screen_y)) = self.draw_location(ctx, x as f64, y as f64) {
                    // Draw the crosshair
                    resources.render(&SetImage::CursorCrosshair, self.transformation(ctx, screen_x, screen_y), gl);

                    // Draw the chance-to-hit if a player unit is selected and an ai unit is at the cursor position
                    if let Some((firing, target)) = battle.selected.and_then(|firing|
                        map.units.get(firing).and_then(|firing|
                            map.units.at(x, y).map(|target|
                                (firing, target)
                            )
                        )
                    ) {
                        if firing.side == UnitSide::Player && target.side == UnitSide::AI {
                            // Get the chance to hit as a percentage
                            let hit_modifier = firing.weapon.info().hit_modifier;
                            let hit_chance = firing.chance_to_hit(target.x, target.y) * hit_modifier * 100.0;

                            // Render it and draw it at the center

                            let hit_chance = format!("{:0.3}%", hit_chance);

                            let offset_x = (TILE_WIDTH - resources.font_width(&hit_chance)) / 2.0 * self.camera.zoom;

                            resources.render_text(&hit_chance, colours::WHITE, self.transformation(ctx, screen_x + offset_x, screen_y), gl);                            
                        }
                    }
                }
            }
        }

        // Draw all the bullets in the animation queue
        for bullet in battle.animations.iter().filter_map(|animation| match *animation {
            Animation::Bullet(ref bullet) => Some(bullet),
            _ => None
        }) {
            // Calculate if the nearest tile to the bullet is visible
            let visible = map.tiles.at(
                clamp_float(bullet.x, 0, map.tiles.cols - 1),
                clamp_float(bullet.y, 0, map.tiles.rows - 1)
            ).player_visibility == Visibility::Visible;

            // If the bullet is visable and on screen, draw it with the right rotation
            if visible {
                if let Some((x, y)) = self.draw_location(ctx, bullet.x, bullet.y) {
                    resources.render_with_rotation(
                        &bullet.image(),
                        convert_rotation(bullet.direction),
                        self.transformation(ctx, x, y),
                        gl
                    );
                }
            }
        }
    }

    // Work out which tile is under the cursor
    pub fn tile_under_cursor(&self, x: f64, y: f64, window_size: &WindowSize) -> (usize, usize) {
        // Get the center of the window
        let center_x = window_size.width  / 2.0;
        let center_y = window_size.height / 2.0;

        // Work out the position of the mouse on the screen relative to the camera
        let x = (x - center_x + self.camera.x * self.camera.zoom) / TILE_WIDTH  / self.camera.zoom + self.camera.x / 2.0 - 0.5;
        let y = (y - center_y + self.camera.y * self.camera.zoom) / TILE_HEIGHT / self.camera.zoom + self.camera.y / 2.0 - 0.5;

        // Account for the images being square
        let y = y - 1.0;

        // Convert to map coordinates
        let (x, y) = to_map_coords(x, y);

        // And then to usize
        (x.round() as usize, y.round() as usize)
    }
}