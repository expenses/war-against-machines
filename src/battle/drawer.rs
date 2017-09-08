// A drawer struct for drawing the map and battle items

use super::Battle;
use super::tiles::{Visibility, Obstacle};
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

pub const CAMERA_SPEED: f32 = 10.0;
pub const CAMERA_ZOOM_SPEED: f32 = 1.0;

// Convert coordinates from isometric
pub fn from_map_coords(x: f32, y: f32) -> (f32, f32) {
    (x - y, -(x + y))
}

// Convert coordinates to isometric
pub fn to_map_coords(x: f32, y: f32) -> (f32, f32) {
    (y + x, y - x)
}

// A camera for drawing the map with
#[derive(Serialize, Deserialize)]
pub struct Camera {
    pub x: f32,
    pub y: f32,
    zoom: f32
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            x: 0.0,
            y: 0.0,
            zoom: DEFAULT_ZOOM
        }
    }

    // Zoom in the camera by a particular amount, checking if it's zoomed in/out too far
    pub fn zoom(&mut self, amount: f32) {
        self.zoom += amount * self.zoom;

        if self.zoom > ZOOM_MAX { self.zoom = ZOOM_MAX; }
        if self.zoom < ZOOM_MIN { self.zoom = ZOOM_MIN; }
    }
}

// If a tile is visible, get it's location on the screen
fn draw_location(ctx: &Context, camera: &Camera, x: f32, y: f32) -> Option<[f32; 2]> {
    // Get the maximum x and y values (given that (0, 0) is at the center)
    let (max_x, max_y) = (ctx.width / 2.0, ctx.height / 2.0);
    // The x and y difference of a tile compared to another tile on the same row/col
    let (x_size, y_size) = (TILE_WIDTH / 2.0 * camera.zoom, TILE_HEIGHT / 2.0 * camera.zoom);

    // Convert the location from the map coords to the screen coords
    let (x, y) = from_map_coords(x, y);

    // Get the correct position
    let x = (x - camera.x) * x_size;
    let y = (y - camera.y) * y_size;

    // Check if the tile is onscreen
    if x > -max_x - x_size && y > -max_y - y_size * 2.0 &&
        x < max_x  + x_size && y < max_y  + y_size * 2.0 {    
        Some([x, y])
    } else {
        None
    }
}

// If a tile is visible, draw an image
fn draw_if_visible(image: &Image, x: usize, y: usize, ctx: &mut Context, camera: &Camera) {
    if let Some(dest) = draw_location(ctx, camera, x as f32, y as f32) {
        ctx.render(image, dest, camera.zoom);
    }
}

// Draw all the elements of a particular map tile
fn draw_tile(x: usize, y: usize, ctx: &mut Context, battle: &Battle) {
    let tiles = &battle.map.tiles;
    let camera = &battle.map.camera;

    // Get the tile
    let tile = tiles.at(x, y);

    // If the tile is on the screen, draw it
    if let Some(dest) = draw_location(ctx, camera, x as f32, y as f32) {
        // Get the colour overlay
        let overlay = tile.player_visibility.colour();

        // Draw the tile base if visible
        if tile.visible() {
            ctx.render_with_overlay(&tile.base, dest, camera.zoom, overlay);
        }

        // Draw the left wall if visible
        if let Some(ref wall) = tile.walls.left {
            let visibility = tiles.left_wall_visibility(x, y);

            if visibility != Visibility::Invisible {
                ctx.render_with_overlay(&wall.tag.left_image(), dest, camera.zoom, visibility.colour());
            }
        }

        // Draw the right wall if visible
        if let Some(ref wall) = tile.walls.top {
            let visibility = tiles.top_wall_visibility(x, y);

            if visibility != Visibility::Invisible {
                ctx.render_with_overlay(&wall.tag.top_image(), dest, camera.zoom, visibility.colour());
            }
        }

        // If the tile is visible, draw the rest
        if tile.visible() {
            // If the tile has a pit on it, draw it
            if let Obstacle::Pit(ref image) = tile.obstacle {
                ctx.render_with_overlay(image, dest, camera.zoom, overlay);
            }

            // Draw the cursor if it isn't on an ai unit and or a unit isn't selected
            if !battle.cursor_active() || battle.selected.is_none() {
                if let Some((cursor_x, cursor_y)) = battle.cursor {
                    if cursor_x == x && cursor_y == y {
                        // Determine the cursor type
                        // Grey if the tile is foggy
                        let colour = if !tile.player_visibility.is_visible() {
                            colours::GREY
                        // Red if the tile has an obstacle
                        } else if !tile.obstacle.is_empty() {
                            colours::RED
                        // Orange if it has a unit
                        } else if battle.map.units.at(x, y).is_some() {
                            colours::ORANGE
                        // Yellow by default
                        } else {
                            colours::YELLOW
                        };

                        ctx.render_with_overlay(&Image::Cursor, dest, camera.zoom, colour);
                    }
                }
            }

            // Draw items that should only be shown on visible tiles
            if tile.player_visibility.is_visible() {
                // Draw the tile decoration
                if let Some(ref decoration) = tile.decoration {
                    ctx.render_with_overlay(decoration, dest, camera.zoom, overlay);
                }

                for item in &tile.items {
                    ctx.render_with_overlay(&item.image(), dest, camera.zoom, overlay);
                }

                // Draw a unit at the position
                if let Some(unit) = battle.map.units.at(x, y) {
                    // Draw the cursor to show that the unit is selected
                    if let Some(selected) = battle.selected {
                        if selected == unit.id {
                            ctx.render_with_overlay(&Image::Cursor, dest, camera.zoom, colours::ORANGE);
                        }
                    }

                    ctx.render_with_overlay(&unit.tag.image(), dest, camera.zoom, overlay);
                }
            }

            // If the tile has an obstacle on it, draw it
            if let Obstacle::Object(ref image) = tile.obstacle {
                ctx.render_with_overlay(image, dest, camera.zoom, overlay);
            }
        }
    }
}

// Draw the whole battle
pub fn draw_battle(ctx: &mut Context, battle: &Battle) {
    let map = &battle.map;
    let camera = &battle.map.camera;
    
    // Draw all the tiles
    for x in 0 .. map.tiles.cols {
        for y in 0 .. map.tiles.rows {
            draw_tile(x, y, ctx, battle);
        }
    }

    // Draw the edge edges if visible

    for x in 0 .. map.tiles.cols {
        if map.tiles.at(x, map.tiles.rows - 1).visible() {
            draw_if_visible(&Image::LeftEdge, x + 1, map.tiles.rows, ctx, camera);
        }
    }

    for y in 0 .. map.tiles.rows {
        if map.tiles.at(map.tiles.cols - 1, y).visible() {
            draw_if_visible(&Image::RightEdge, map.tiles.cols, y + 1, ctx, camera);
        }
    }

    // Draw the path if there is one
    if let Some(ref points) = battle.path {
        if let Some(unit) = battle.selected() {
            let mut total_cost = 0;

            // Draw the path tiles
            for point in points {
                total_cost += point.cost;

                if let Some(dest) = draw_location(ctx, camera, point.x as f32, point.y as f32) {
                    // Render the path tile

                    let colour = if total_cost > unit.moves {
                        colours::RED
                    } else if total_cost + unit.weapon.tag.cost() > unit.moves {
                        colours::ORANGE
                    } else {
                        colours::WHITE
                    };

                    ctx.render_with_overlay(&Image::Path, dest, camera.zoom, colour);
                }
            }

            // Draw the path costs

            total_cost = 0;

            for point in points {
                total_cost += point.cost;

                if let Some(dest) = draw_location(ctx, camera, point.x as f32, point.y as f32) {
                    // Render the path cost
                    ctx.render_text(&total_cost.to_string(), dest[0], dest[1], colours::WHITE);
                }
            }
        }
    }

    // Draw the firing crosshair if the cursor is on an ai unit and a unit is selected
    if battle.cursor_active() {
        if let Some(firing) = battle.selected() {
            if let Some((x, y)) = battle.cursor {
                if let Some(dest) = draw_location(ctx, camera, x as f32, y as f32) {
                    // Draw the crosshair
                    ctx.render(&Image::CursorCrosshair, dest, camera.zoom);

                    let colour = if !firing.weapon.can_fire() {
                        colours::RED
                    } else if map.tiles.line_of_fire(firing.x, firing.y, x, y).is_some() {
                        colours::ORANGE
                    } else {
                        colours::WHITE
                    };

                    // Draw the chance-to-hit
                    ctx.render_text(
                        &format!("{:0.3}%", firing.chance_to_hit(x, y) * 100.0),
                        dest[0], dest[1] + TILE_HEIGHT * camera.zoom,
                        colour
                    );
                }
            }
        }
    }

    // Draw all the bullets in the animation queue
    for bullet in battle.animations.iter().filter_map(|animation| match *animation {
        Animation::Bullet(ref bullet) if bullet.status.in_progress() => Some(bullet),
        _ => None
    }) {
        // Calculate if the nearest tile to the bullet is visible
        let visible = map.tiles.at(
            clamp_float(bullet.x, 0, map.tiles.cols - 1),
            clamp_float(bullet.y, 0, map.tiles.rows - 1)
        ).player_visibility.is_visible();

        // If the bullet is visable and on screen, draw it with the right rotation
        if visible {
            if let Some(dest) = draw_location(ctx, camera, bullet.x, bullet.y) {
                ctx.render_with_rotation(
                    &bullet.image(), dest, camera.zoom, convert_rotation(bullet.direction)
                );
            }
        }
    }
}

// Work out which tile is under the cursor
pub fn tile_under_cursor(x: f32, y: f32, camera: &Camera) -> (usize, usize) {
    // Work out the position of the mouse on the screen relative to the camera
    let x = x  / TILE_WIDTH  / camera.zoom + camera.x / 2.0;
    let y = -y / TILE_HEIGHT / camera.zoom - camera.y / 2.0;

    // Account for the images being square
    let y = y - 0.5;

    // Convert to map coordinates
    let (x, y) = to_map_coords(x, y);

    // And then to usize
    (x.round() as usize, y.round() as usize)
}

#[test]
fn default_camera_pos() {
    // If the cursor is in the center of the screen and the camera is
    // the default, the tile under the cursor should be at (0, 0)
    assert_eq!(tile_under_cursor(0.0, -1.0, &Camera::new()), (0, 0));
}