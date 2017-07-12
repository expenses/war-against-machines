// A drawer struct for drawing the map and battle items

use battle::Battle;
use battle::units::UnitSide;
use battle::tiles::Visibility;
use battle::animations::Animation;
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

// A simple camera for what the user is looking at
pub struct Camera {
    pub x: f32,
    pub y: f32,
    zoom: f32
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
    pub fn zoom(&mut self, amount: f32) {
        self.camera.zoom += amount * self.camera.zoom;

        if self.camera.zoom > ZOOM_MAX { self.camera.zoom = ZOOM_MAX; }
        if self.camera.zoom < ZOOM_MIN { self.camera.zoom = ZOOM_MIN; }
    }

    fn draw_location(&self, ctx: &Context, x: f32, y: f32) -> Option<(f32, f32)> {
        let (max_x, max_y) = (ctx.width / 2.0, ctx.height / 2.0);
        let (tile_width, tile_height) = (TILE_WIDTH / 2.0 * self.camera.zoom, TILE_HEIGHT / 2.0 * self.camera.zoom);

        let (x, y) = from_map_coords(x, y);

        let x = (x - self.camera.x) * tile_width;
        let y = (y - self.camera.y) * tile_height;

        if  x > -max_x - tile_width &&
            y > -max_y - tile_height &&
            x < max_x + tile_width &&
            y < max_y + tile_height * 2.0 {
            Some((x, y))
        } else {
            None
        }
    }

    fn draw_if_visible(&self, image: &Image, x: usize, y: usize, ctx: &mut Context) {
        if let Some((x, y)) = self.draw_location(ctx, x as f32, y as f32) {
            ctx.render(image, x, y, self.camera.zoom);
        }
    }

    pub fn draw_tile(&self, x: usize, y: usize, ctx: &mut Context, battle: &Battle) {
        // Get the tile
        let tile = battle.map.tiles.at(x, y);

        // If the tile is invisible, return
        if !tile.visible() {
            return;
        }

        // If the tile is on the screen, draw it
        if let Some((screen_x, screen_y)) = self.draw_location(ctx, x as f32, y as f32) {
            // Draw the tile base
            if tile.player_visibility != Visibility::Foggy {
                ctx.render(&tile.base, screen_x, screen_y, self.camera.zoom);
            } else {
                ctx.render_with_overlay(&tile.base, screen_x, screen_y, self.camera.zoom, colours::FOGGY);
            }

            // Draw the tile decoration
            if let Some(ref obstacle) = tile.obstacle {
                if tile.player_visibility != Visibility::Foggy {
                    ctx.render(obstacle, screen_x, screen_y, self.camera.zoom);
                } else {
                    ctx.render_with_overlay(obstacle, screen_x, screen_y, self.camera.zoom, colours::FOGGY);
                }
            }

            // Draw the cursor if it isn't on an ai unit and or a unit isn't selected
            if !battle.cursor_on_ai_unit() || battle.selected.is_none() {
                if let Some((cursor_x, cursor_y)) = battle.cursor.position {
                    if cursor_x == x && cursor_y == y {
                        // Determine the cursor colour
                        let image = if !tile.walkable() {
                            Image::CursorUnwalkable
                        } else if battle.map.units.at(x, y).is_some() {
                            Image::CursorUnit
                        } else {
                            Image::Cursor
                        };

                        ctx.render(&image, screen_x, screen_y, self.camera.zoom);
                    }
                }
            }

            if tile.player_visibility != Visibility::Foggy {
                for item in &tile.items {
                    ctx.render(&item.image, screen_x, screen_y, self.camera.zoom);
                }

                // Draw a unit at the position
                if let Some(unit) = battle.map.units.at(x, y) {
                    // Draw the cursor to show that the unit is selected
                    if let Some(selected) = battle.selected {
                        if selected == unit.id {
                            ctx.render(&Image::CursorUnit, screen_x, screen_y, self.camera.zoom);
                        }
                    }

                    ctx.render(&unit.image, screen_x, screen_y, self.camera.zoom);
                }
            }
        }
    }

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

        // Draw the path
        if let Some(ref points) = battle.path {
            if let Some(unit) = map.units.get(battle.selected.unwrap()) {
                let mut total_cost = 0;

                // Draw the path tiles
                for point in points {
                    total_cost += point.cost;

                    if let Some((x, y)) = self.draw_location(ctx, point.x as f32, point.y as f32) {
                        // Render the path tile

                        let image = if total_cost > unit.moves {
                            Image::PathUnreachable
                        } else if total_cost + unit.weapon.info().cost > unit.moves {
                            Image::PathCannotFire
                        } else {
                            Image::Path
                        };

                        ctx.render(&image, x, y, self.camera.zoom);
                    }
                }

                total_cost = 0;

                for point in points {
                    total_cost += point.cost;

                    if let Some((x, y)) = self.draw_location(ctx, point.x as f32, point.y as f32) {
                        // Render the path cost
                        ctx.render_text(&format!("{}", total_cost), x, y, colours::WHITE);
                    }
                }
            }
        }

        // Draw the firing crosshair if the cursor is on an ai unit and a unit is selected
        if battle.cursor_on_ai_unit() && battle.selected.is_some() {
            if let Some((x, y)) = battle.cursor.position {
                if let Some((screen_x, screen_y)) = self.draw_location(ctx, x as f32, y as f32) {
                    // Draw the crosshair
                    ctx.render(&Image::CursorCrosshair, screen_x, screen_y, self.camera.zoom);

                    // Draw the chance-to-hit if a player unit is selected and an ai unit is at the cursor position
                    if let Some((firing, target)) = battle.selected.and_then(|firing|
                        map.units.get(firing).and_then(|firing|
                            map.units.at(x, y).map(|target|
                                (firing, target)
                            )
                        )
                    ) {
                        if firing.side == UnitSide::Player && target.side == UnitSide::AI {
                            // Get the chance to hit
                            let hit_chance = firing.chance_to_hit(target.x, target.y) * firing.weapon.info().hit_modifier;

                            // Render it!
                            ctx.render_text(
                                &format!("{:0.3}%", hit_chance * 100.0),
                                screen_x, screen_y + TILE_HEIGHT * self.camera.zoom, colours::WHITE
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
                if let Some((x, y)) = self.draw_location(ctx, bullet.x, bullet.y) {
                    ctx.render_with_rotation(
                        &bullet.image(),
                        x, y, self.camera.zoom, convert_rotation(bullet.direction)
                    );
                }
            }
        }
    }

    // Work out which tile is under the cursor
    pub fn tile_under_cursor(&self, x: f32, y: f32) -> (usize, usize) {
        // Work out the position of the mouse on the screen relative to the camera
        let x = x  / TILE_WIDTH  / self.camera.zoom + self.camera.x / 2.0;
        let y = -y / TILE_HEIGHT / self.camera.zoom - self.camera.y / 2.0;

        // Account for the images being square
        let y = y - 0.5;

        // Convert to map coordinates
        let (x, y) = to_map_coords(x, y);

        // And then to usize
        (x.round() as usize, y.round() as usize)
    }
}