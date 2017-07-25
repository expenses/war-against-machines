// A drawer struct for drawing the map and battle items

use super::Battle;
use super::units::UnitSide;
use super::tiles::Visibility;
use super::animations::Animation;
use resources::Image;
use utils::{clamp_float, convert_rotation};
use context::Context;
use colours;

const TILE_WIDTH: f32 = 48.0;
const TILE_HEIGHT: f32 = 24.0;

const DEFAULT_ZOOM: f32 = 2.0;
const ZOOM_MAX: f32 = 10.0;
const ZOOM_MIN: f32 = 1.0;

// Convert coordinates from isometric
pub fn from_map_coords(x: f32, y: f32) -> (f32, f32) {
    (x - y, -(x + y))
}

// Convert coordinates to isometric
pub fn to_map_coords(x: f32, y: f32) -> (f32, f32) {
    (y + x, y - x)
}

// The drawing struct that contains the camera position
pub struct Drawer {
    pub x: f32,
    pub y: f32,
    zoom: f32
}

impl Drawer {
    // Create a new Drawer
    pub fn new() -> Drawer {
        Drawer {
            x: 0.0, y: 0.0, zoom: DEFAULT_ZOOM
        }
    }

    // Zoom in the camera by a particular amount, checking if it's zoomed in/out too far
    pub fn zoom(&mut self, amount: f32) {
        self.zoom += amount * self.zoom;

        if self.zoom > ZOOM_MAX { self.zoom = ZOOM_MAX; }
        if self.zoom < ZOOM_MIN { self.zoom = ZOOM_MIN; }
    }

    // If a tile is visible, get it's location on the screen
    fn draw_location(&self, ctx: &Context, x: f32, y: f32) -> Option<[f32; 2]> {
        // Get the maximum x and y values (given that (0, 0) is at the center)
        let (max_x, max_y) = (ctx.width / 2.0, ctx.height / 2.0);
        // The x and y difference of a tile compared to another tile on the same row/col
        let (x_size, y_size) = (TILE_WIDTH / 2.0 * self.zoom, TILE_HEIGHT / 2.0 * self.zoom);

        // Convert the location from the map coords to the screen coords
        let (x, y) = from_map_coords(x, y);

        // Get the correct position
        let x = (x - self.x) * x_size;
        let y = (y - self.y) * y_size;

        // Check if the tile is onscreen
        if  x > -max_x - x_size && y > -max_y - y_size &&
            x < max_x  + x_size && y < max_y  + y_size * 2.0 {
            
            Some([x, y])
        } else {
            None
        }
    }

    // If a tile is visible, draw an image
    fn draw_if_visible(&self, image: &Image, x: usize, y: usize, ctx: &mut Context) {
        if let Some(dest) = self.draw_location(ctx, x as f32, y as f32) {
            ctx.render(image, dest, self.zoom);
        }
    }

    // Draw all the elements of a particular map tile
    pub fn draw_tile(&self, x: usize, y: usize, ctx: &mut Context, battle: &Battle) {
        // Get the tile
        let tile = battle.map.tiles.at(x, y);

        // If the tile is invisible, return
        if !tile.visible() {
            return;
        }

        // If the tile is on the screen, draw it
        if let Some(dest) = self.draw_location(ctx, x as f32, y as f32) {
            // Get the colour overlay
            let overlay = if tile.player_visibility == Visibility::Foggy {
                colours::FOGGY
            } else {
                colours::ALPHA
            };

            // Draw the tile base
            ctx.render_with_overlay(&tile.base, dest, self.zoom, overlay);

            // Draw the cursor if it isn't on an ai unit and or a unit isn't selected
            if !battle.cursor_on_ai_unit() || battle.selected.is_none() {
                if let Some((cursor_x, cursor_y)) = battle.cursor.position {
                    if cursor_x == x && cursor_y == y {
                        // Determine the cursor type
                        let image = if !tile.walkable() {
                            Image::CursorUnwalkable
                        } else if battle.map.units.at(x, y).is_some() {
                            Image::CursorUnit
                        } else {
                            Image::Cursor
                        };

                        ctx.render(&image, dest, self.zoom);
                    }
                }
            }

            if tile.player_visibility != Visibility::Foggy {
                // Draw the tile decoration
                if let Some(ref decoration) = tile.decoration {
                    ctx.render_with_overlay(decoration, dest, self.zoom, overlay);
                }

                for item in &tile.items {
                    ctx.render(&item.image(), dest, self.zoom);
                }

                // Draw a unit at the position
                if let Some(unit) = battle.map.units.at(x, y) {
                    // Draw the cursor to show that the unit is selected
                    if let Some(selected) = battle.selected {
                        if selected == unit.id {
                            ctx.render(&Image::CursorUnit, dest, self.zoom);
                        }
                    }

                    ctx.render(&unit.image, dest, self.zoom);
                }
            }

            // Draw the tile obstacle
            if let Some(ref obstacle) = tile.obstacle {
                ctx.render_with_overlay(obstacle, dest, self.zoom, overlay);
            }
        }
    }

    // Draw the whole battle
    pub fn draw_battle(&self, ctx: &mut Context, battle: &Battle) {
        let map = &battle.map;
        
        // Draw all the tiles
        for x in 0 .. map.tiles.cols {
            for y in 0 .. map.tiles.rows {
                self.draw_tile(x, y, ctx, battle);
            }
        }

        // Draw the edge edges if visible

        for x in 0 .. map.tiles.cols {
            if map.tiles.at(x, map.tiles.rows - 1).visible() {
                self.draw_if_visible(&Image::LeftEdge, x + 1, map.tiles.rows, ctx);
            }
        }

        for y in 0 .. map.tiles.rows {
            if map.tiles.at(map.tiles.cols - 1, y).visible() {
                self.draw_if_visible(&Image::RightEdge, map.tiles.cols, y + 1, ctx);
            }
        }

        // Draw the path if there is one
        if let Some(ref points) = battle.path {
            if let Some(unit) = battle.selected() {
                let mut total_cost = 0;

                // Draw the path tiles
                for point in points {
                    total_cost += point.cost;

                    if let Some(dest) = self.draw_location(ctx, point.x as f32, point.y as f32) {
                        // Render the path tile

                        let image = if total_cost > unit.moves {
                            Image::PathUnreachable
                        } else if total_cost + unit.weapon.info().cost > unit.moves {
                            Image::PathCannotFire
                        } else {
                            Image::Path
                        };

                        ctx.render(&image, dest, self.zoom);
                    }
                }

                // Draw the path costs

                total_cost = 0;

                for point in points {
                    total_cost += point.cost;

                    if let Some(dest) = self.draw_location(ctx, point.x as f32, point.y as f32) {
                        // Render the path cost
                        ctx.render_text(&total_cost.to_string(), dest[0], dest[1], colours::WHITE);
                    }
                }
            }
        }

        // Draw the firing crosshair if the cursor is on an ai unit and a unit is selected
        if battle.cursor_on_ai_unit() && battle.selected.is_some() {
            if let Some((x, y)) = battle.cursor.position {
                if let Some(dest) = self.draw_location(ctx, x as f32, y as f32) {
                    // Draw the crosshair
                    ctx.render(&Image::CursorCrosshair, dest, self.zoom);

                    // Draw the chance-to-hit if a player unit is selected and an ai unit is at the cursor position
                    if let Some((firing, target)) = battle.selected().and_then(|firing|
                        map.units.at(x, y).map(|target|
                            (firing, target)
                        )
                    ) {
                        if firing.side == UnitSide::Player && target.side == UnitSide::AI {
                            // Get the chance to hit
                            let hit_chance = firing.chance_to_hit(target.x, target.y) * firing.weapon.info().hit_modifier;

                            // Render it!
                            ctx.render_text(
                                &format!("{:0.3}%", hit_chance * 100.0),
                                dest[0], dest[1] + TILE_HEIGHT * self.zoom, colours::WHITE
                            );                            
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
                if let Some(dest) = self.draw_location(ctx, bullet.x, bullet.y) {
                    ctx.render_with_rotation(
                        &bullet.image(), dest, self.zoom, convert_rotation(bullet.direction)
                    );
                }
            }
        }
    }

    // Work out which tile is under the cursor
    pub fn tile_under_cursor(&self, x: f32, y: f32) -> (usize, usize) {
        // Work out the position of the mouse on the screen relative to the camera
        let x = x  / TILE_WIDTH  / self.zoom + self.x / 2.0;
        let y = -y / TILE_HEIGHT / self.zoom - self.y / 2.0;

        // Account for the images being square
        let y = y - 0.5;

        // Convert to map coordinates
        let (x, y) = to_map_coords(x, y);

        // And then to usize
        (x.round() as usize, y.round() as usize)
    }
}