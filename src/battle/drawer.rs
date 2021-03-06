// A drawer struct for drawing the map and battle items

use super::map::*;
use super::responses::*;
use super::Battle;
use colours;
use context::Context;
use resources::Image;
use utils::convert_rotation;

const TILE_WIDTH: f32 = 48.0;
const TILE_HEIGHT: f32 = 24.0;

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
    zoom: f32,
}

impl Camera {
    const SPEED: f32 = 10.0;
    const ZOOM_SPEED: f32 = 1.0;
    const DEFAULT_ZOOM: f32 = 2.0;
    const ZOOM_MAX: f32 = 10.0;
    const ZOOM_MIN: f32 = 1.0;

    pub fn new() -> Camera {
        Camera {
            x: 0.0,
            y: 0.0,
            zoom: Self::DEFAULT_ZOOM,
        }
    }

    pub fn move_x(&mut self, step: f32, map: &Map) {
        let (x, y) = self.map_location();
        let width = map.tiles.width() as f32;
        let height = map.tiles.height() as f32;

        if step > 0.0 {
            if x < width && y > 0.0 {
                self.x += step * Self::SPEED;
            }
        } else if x > 0.0 && y < height {
            self.x += step * Self::SPEED;
        }
    }

    pub fn move_y(&mut self, step: f32, map: &Map) {
        let (x, y) = self.map_location();
        let width = map.tiles.width() as f32;
        let height = map.tiles.height() as f32;

        if step > 0.0 {
            if x > 0.0 && y > 0.0 {
                self.y += step * Self::SPEED;
            }
        } else if x < width && y < height {
            self.y += step * Self::SPEED;
        }
    }

    // Zoom in the camera by a particular amount, checking if it's zoomed in/out too far
    pub fn zoom(&mut self, amount: f32) {
        self.zoom += amount * self.zoom * Self::ZOOM_SPEED;

        if self.zoom > Self::ZOOM_MAX {
            self.zoom = Self::ZOOM_MAX;
        }
        if self.zoom < Self::ZOOM_MIN {
            self.zoom = Self::ZOOM_MIN;
        }
    }

    pub fn set_to(&mut self, x: usize, y: usize) {
        let (x, y) = (x as f32, y as f32);
        let (x, y) = from_map_coords(x, y);
        self.x = x;
        self.y = y + 0.5;
    }

    pub fn map_location(&self) -> (f32, f32) {
        let x = self.x / 2.0;
        let y = -self.y / 2.0 - 0.5;
        to_map_coords(x, y)
    }

    pub fn tile_under_cursor(
        &self,
        mut mouse_x: f32,
        mut mouse_y: f32,
        width: f32,
        height: f32,
    ) -> (usize, usize) {
        mouse_x -= width / 2.0;
        mouse_y -= height / 2.0;

        // Work out the position of the mouse on the screen relative to the camera
        let x = mouse_x / TILE_WIDTH / self.zoom + self.x / 2.0;
        let y = mouse_y / TILE_HEIGHT / self.zoom - self.y / 2.0;

        // Account for the images being square
        let y = y - 0.5;

        // Convert to map coordinates
        let (x, y) = to_map_coords(x, y);

        // And then to usize
        (x.round() as usize, y.round() as usize)
    }
}

// If a tile is visible, get it's location on the screen
fn draw_location(ctx: &Context, camera: &Camera, x: f32, y: f32) -> Option<[f32; 2]> {
    // Get the maximum x and y values (given that (0, 0) is at the center)
    let (max_x, max_y) = (ctx.width, ctx.height);
    // The x and y difference of a tile compared to another tile on the same row/col
    let (x_size, y_size) = (
        TILE_WIDTH / 2.0 * camera.zoom,
        TILE_HEIGHT / 2.0 * camera.zoom,
    );

    // Convert the location from the map coords to the screen coords
    let (x, y) = from_map_coords(x, y);

    // Get the correct position
    let x = (x - camera.x) * x_size + ctx.width / 2.0;
    let y = -(y - camera.y) * y_size + ctx.height / 2.0;

    // Check if the tile is onscreen
    if x > -x_size && y > -y_size * 2.0 && x < max_x + x_size && y < max_y + y_size * 2.0 {
        Some([x, y])
    } else {
        None
    }
}

// Draw all the elements of a particular map tile
fn draw_tile(x: usize, y: usize, ctx: &mut Context, battle: &Battle) {
    let camera = &battle.camera;
    let debugging = battle.visual_debugging;
    let map = &battle.client.map;
    let responses = battle.client.responses();
    let side = battle.client.side;
    let tiles = &map.tiles;
    let light = map.light;

    // Get the tile
    let visibility = tiles.visibility_at(x, y, side);
    let overlay = visibility.colour(light, debugging);
    let tile = tiles.at(x, y);

    // If the tile is on the screen, draw it
    if let Some(dest) = draw_location(ctx, camera, x as f32, y as f32) {
        ctx.render_with_overlay(tile.base, dest, camera.zoom, overlay);

        // Draw the left wall
        if let Some(ref wall) = tile.walls.left {
            let visibility = tiles.left_wall_visibility(x, y, side);
            ctx.render_with_overlay(
                wall.tag.left_image(),
                dest,
                camera.zoom,
                visibility.colour(light, debugging),
            );
        }

        // Draw the right wall
        if let Some(ref wall) = tile.walls.top {
            let visibility = tiles.top_wall_visibility(x, y, side);
            ctx.render_with_overlay(
                wall.tag.top_image(),
                dest,
                camera.zoom,
                visibility.colour(light, debugging),
            );
        }

        if let Obstacle::Pit(image) = tile.obstacle {
            ctx.render_with_overlay(image, dest, camera.zoom, overlay);
        }

        // Draw the cursor if it isn't on an ai unit and or a unit isn't selected
        if !battle.cursor_active() {
            if let Some((cursor_x, cursor_y)) = battle.cursor {
                if cursor_x == x && cursor_y == y {
                    // Determine the cursor type
                    // Grey if the tile is not visible
                    let colour = if !visibility.is_visible() {
                        colours::GREY
                    // Red if the tile has an obstacle
                    } else if !tile.obstacle.is_empty() {
                        colours::RED
                    // Orange if it has a unit
                    } else if battle.client.map.units.at(x, y).is_some() {
                        colours::ORANGE
                    // Yellow by default
                    } else {
                        colours::YELLOW
                    };

                    ctx.render_with_overlay(Image::Cursor, dest, camera.zoom, colour);
                }
            }
        }

        // Draw the tile decoration
        if let Some(decoration) = tile.decoration {
            ctx.render_with_overlay(decoration, dest, camera.zoom, overlay);
        }

        // Draw the tile items
        for item in &tile.items {
            ctx.render_with_overlay(item.image(), dest, camera.zoom, overlay);
        }

        // Draw a unit at the position
        if let Some(unit) = battle.client.map.units.at(x, y) {
            // Draw the cursor to show that the unit is selected
            if battle.selected == Some(unit.id) {
                ctx.render_with_overlay(Image::Cursor, dest, camera.zoom, colours::ORANGE);
            }

            unit.render(ctx, dest, camera.zoom, overlay);
        }
        // If the tile has an obstacle on it, draw it
        if let Obstacle::Object(image) = tile.obstacle {
            ctx.render_with_overlay(image, dest, camera.zoom, overlay);
        }

        // Draw explosions on the tile
        responses
            .iter()
            .filter_map(Response::as_explosion)
            .filter(|explosion| explosion.x() == x && explosion.y() == y)
            .for_each(|explosion| ctx.render(explosion.image(), dest, camera.zoom));
    }
}

pub fn draw_map(ctx: &mut Context, battle: &Battle) {
    let camera = &battle.camera;
    let debugging = battle.visual_debugging;
    let map = &battle.client.map;
    let side = battle.client.side;
    let width = map.tiles.width();
    let height = map.tiles.height();
    let light = map.light;

    // Draw all the tiles
    for (x, y) in map.tiles.iter() {
        draw_tile(x, y, ctx, battle);
    }

    // Draw the edge edges

    for x in 0..width {
        let visibility = map.tiles.visibility_at(x, height - 1, side);

        if let Some(dest) = draw_location(ctx, camera, (x + 1) as f32, height as f32) {
            ctx.render_with_overlay(
                Image::LeftEdge,
                dest,
                camera.zoom,
                visibility.colour(light, debugging),
            );
        }
    }

    for y in 0..height {
        let visibility = map.tiles.visibility_at(width - 1, y, side);

        if let Some(dest) = draw_location(ctx, camera, width as f32, (y + 1) as f32) {
            ctx.render_with_overlay(
                Image::RightEdge,
                dest,
                camera.zoom,
                visibility.colour(light, debugging),
            );
        }
    }
}

// Draw the whole battle
pub fn draw_battle(ctx: &mut Context, battle: &Battle) {
    let camera = &battle.camera;
    let map = &battle.client.map;
    let responses = &battle.client.responses();
    let side = battle.client.side;

    draw_map(ctx, battle);

    // Draw the path if there is one
    if let Some(ref points) = battle.path {
        if let Some(unit) = battle.selected() {
            let mut unit_moves = i32::from(unit.moves);

            // Draw the path tiles
            for point in points {
                unit_moves -= i32::from(point.cost);

                if let Some(dest) = draw_location(ctx, camera, point.x as f32, point.y as f32) {
                    // Render the path tile

                    let colour = if unit_moves < 0 {
                        colours::RED
                    } else if unit_moves < i32::from(unit.weapon.tag.cost()) {
                        colours::ORANGE
                    } else {
                        colours::WHITE
                    };

                    ctx.render_with_overlay(Image::Path, dest, camera.zoom, colour);
                }
            }

            // Draw the path costs

            unit_moves = i32::from(unit.moves);

            for point in points {
                unit_moves -= i32::from(point.cost);

                if unit_moves >= 0 {
                    if let Some(dest) = draw_location(ctx, camera, point.x as f32, point.y as f32) {
                        // Render the path cost
                        ctx.render_text(&unit_moves.to_string(), dest[0], dest[1], colours::WHITE);
                    }
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
                    ctx.render(Image::CursorCrosshair, dest, camera.zoom);

                    let colour = if map.tiles.visibility_at(x, y, side).is_invisible() {
                        colours::GREY
                    } else if !firing.weapon.can_fire() {
                        colours::RED
                    } else if map.tiles.line_of_fire(firing.x, firing.y, x, y).is_some() {
                        colours::ORANGE
                    } else {
                        colours::WHITE
                    };

                    // Draw the chance-to-hit
                    ctx.render_text(
                        &format!("{:0.3}%", firing.chance_to_hit(x, y) * 100.0),
                        dest[0],
                        dest[1] + TILE_HEIGHT * camera.zoom,
                        colour,
                    );
                }
            }
        }
    }

    // Draw all the visible bullets in the response queue
    responses
        .iter()
        .filter_map(Response::as_bullet)
        .for_each(|bullet| {
            // If the bullet is on screen, draw it with the right rotation
            if let Some(dest) = draw_location(ctx, camera, bullet.x(), bullet.y()) {
                ctx.render_with_rotation(
                    bullet.image(),
                    dest,
                    camera.zoom,
                    convert_rotation(bullet.direction()),
                );
            }
        });

    responses
        .iter()
        .filter_map(Response::as_thrown_item)
        .for_each(|thrown_item| {
            if let Some(dest) = draw_location(ctx, camera, thrown_item.x(), thrown_item.y()) {
                ctx.render(
                    thrown_item.image(),
                    [
                        dest[0],
                        dest[1] - thrown_item.height() * camera.zoom * TILE_HEIGHT,
                    ],
                    camera.zoom,
                );
            }
        });
}

#[test]
fn default_camera_pos() {
    // If the cursor is in the center of the screen and the camera is
    // the default, the tile under the cursor should be at (0, 0)
    assert_eq!(
        Camera::new().tile_under_cursor(501.0, 501.0, 1000.0, 1000.0),
        (0, 0)
    );
}
