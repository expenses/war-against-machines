// Unit and game animations

// Turn off clippy warning for floating point comparisons
// and `new` functons not returning self because I'm lazy
#![cfg_attr(feature = "cargo-clippy", allow(float_cmp, new_ret_no_self))]

use rand;
use rand::distributions::{IndependentSample, Range};
use odds::vec::VecExt;

use super::map::Map;
use super::units::Unit;
use resources::{Image, SoundEffect};
use weapons::WeaponType;
use context::Context;
use utils::{direction, clamp, clamp_float, distance, lerp};

use std::f32::consts::PI;

const MARGIN: f32 = 5.0;
// Bullets travel 30 tiles a second
const BULLET_SPEED: f32 = 30.0;
// The minimum length of time for a bullet animation is a quarter of a second
const MIN_BULLET_TIME: f32 = 0.25;
// Units move 5 tiles a second
const WALK_TIME: f32 = 1.0 / 5.0;
// Items can be thrown 7.5 tiles a second
const THROW_ITEM_TIME: f32 = 7.5;
// And reach a peak height of 1 tile
const THROW_ITEM_HEIGHT: f32 = 1.0;
const THROW_MIN: f32 = 5.0;

const EXPLOSION_DURATION: f32 = 0.25;
const EXPLOSION_SPEED: f32 = 10.0;

// Extrapolate two points on the map to get the point at which a bullet 
// would go off the map
fn extrapolate(x_1: f32, y_1: f32, x_2: f32, y_2: f32, map: &Map) -> (f32, f32) {
    // Get the min and max edges
    let min_x = -MARGIN;
    let min_y = -MARGIN;
    let max_x = map.tiles.cols as f32 + MARGIN;
    let max_y = map.tiles.rows as f32 + MARGIN;
    
    // get the relevant edges
    let relevant_x = if x_2 > x_1 {max_x} else {min_x};
    let relevant_y = if y_2 > y_1 {max_y} else {min_y};

    // If the line is straight just change the x or y coord
    if x_2 == x_1 {
        (x_1, relevant_y)
    } else if y_2 == y_1 {
        (relevant_x, y_1)
    } else {(
        // Extrapolate the values by the difference to an edge and clamp
        clamp(x_2 + ((x_2 - x_1) / (y_2 - y_1)) * (relevant_y - y_2), min_x, max_x),
        clamp(y_2 + ((y_2 - y_1) / (x_2 - x_1)) * (relevant_x - x_2), min_y, max_y)
    )}
}

// Calculate if the nearest tile to the an item is visible
fn visible(x: f32, y: f32, map: &Map) -> bool {
    map.tiles.at(
        clamp_float(x, 0, map.tiles.cols - 1),
        clamp_float(y, 0, map.tiles.rows - 1)
    ).player_visibility.is_visible()
}

// A pretty simple walk animation
pub struct Walk {
    time: f32
}

impl Walk {
    // Create a new walk animation
    pub fn new() -> Animation {
        Animation::Walk(Walk {
            time: 0.0
        })
    }

    // Move the animation a step, and return if its still going
    fn step(&mut self, ctx: &mut Context, dt: f32) -> bool {
        if self.time == 0.0 {
            ctx.play_sound(SoundEffect::Walk);
        }
        
        self.time += dt;
        self.time < WALK_TIME
    }
}

// A bullet animation for drawing on the screen
pub struct Bullet {
    pub x: f32,
    pub y: f32,
    pub direction: f32,
    time: f32,
    finished: bool,
    weapon_type: WeaponType,
    target_x: f32,
    target_y: f32,
}

impl Bullet {
    // Create a new bullet based of the firing unit and the target unit
    pub fn new(unit: &Unit, target_x: usize, target_y: usize, will_hit: bool, map: &Map) -> Animation {
        let x = unit.x as f32;
        let y = unit.y as f32;
        let mut target_x = target_x as f32;
        let mut target_y = target_y as f32;

        // Calculate the direction of the bullet
        let mut direction = direction(x, y, target_x, target_y);

        // If the bullet won't hit the target, change the direction slightly
        if !will_hit {
            direction += Range::new(-0.2, 0.2).ind_sample(&mut rand::thread_rng());
            
            let (x, y) = extrapolate(x, y, x + direction.cos(), y + direction.sin(), map);

            target_x = x;
            target_y = y;
        }

        Animation::Bullet(Bullet {
           x, y, direction, target_x, target_y,
           // Get the type of the firing weapon
           weapon_type: unit.weapon.tag,
           // The bullet hasn't started moving
           time: 0.0,
           finished: false
        })
    }
    
    // Get the image of the bullet
    pub fn image(&self) -> Image {
        self.weapon_type.bullet()
    }

    // Calculate if the nearest tile to the bullet is visible
    pub fn visible(&self, map: &Map) -> bool {
        visible(self.x, self.y, map)
    }

    // Move the bullet a step and work out if its still going or not
    fn step(&mut self, ctx: &mut Context, dt: f32) -> bool {
        // If the bullet hasn't started moving, play its sound effect
        if self.time == 0.0 {
            ctx.play_sound(self.weapon_type.fire_sound());
        }

        // Increment the time
        self.time += dt;

        // If the bullet is still moving
        if !self.finished {
            // Work out if the bullet started to the left/right and above/below the target
            let left = self.x < self.target_x;
            let above = self.y < self.target_y;

            // Move the bullet
            self.x += self.direction.cos() * BULLET_SPEED * dt;
            self.y += self.direction.sin() * BULLET_SPEED * dt;
            
            // Set if the bullet is past the target
            self.finished = left != (self.x < self.target_x) || above != (self.y < self.target_y);
        }

        // If the bullet hasn't finished or is under the minimum time
        !self.finished || self.time < MIN_BULLET_TIME
    }
}

pub struct ThrowItem {
    pub image: Image,
    pub height: f32,
    start_x: f32,
    start_y: f32,
    progress: f32,
    increment: f32,
    end_x: f32,
    end_y: f32
}

impl ThrowItem {
    pub fn new(image: Image, start_x: usize, start_y: usize, end_x: usize, end_y: usize) -> Animation {        
        Animation::ThrowItem(ThrowItem {
            image,
            start_x: start_x as f32,
            start_y: start_y as f32,
            end_x: end_x as f32,
            end_y: end_y as f32,
            increment: THROW_ITEM_TIME / distance(start_x, start_y, end_x, end_y).min(THROW_MIN),
            progress: 0.0,
            height: 0.0
        })
    }

    // Interpolate the x position
    pub fn x(&self) -> f32 {
        lerp(self.start_x, self.end_x, self.progress)
    }

    // Interpolate the y position
    pub fn y(&self) -> f32 {
        lerp(self.start_y, self.end_y, self.progress)
    }

    // Return if the item is visible
    pub fn visible(&self, map: &Map) -> bool {
        visible(self.x(), self.y(), map)
    }

    // Update the progress and the height
    fn step(&mut self, dt: f32) -> bool {
        self.progress += self.increment * dt;
        self.height = (self.progress * PI).sin() * THROW_ITEM_HEIGHT;
        self.progress < 1.0
    }
}

pub struct Explosion {
    pub x: usize,
    pub y: usize,
    time: f32
}

impl Explosion {
    pub fn new(x: usize, y: usize, center_x: usize, center_y: usize) -> Animation {
        Animation::Explosion(Explosion {
            x, y,
            time: -distance(x, y, center_x, center_y) / EXPLOSION_SPEED
        })
    }

    fn step(&mut self, dt: f32) -> bool {
        self.time += dt;
        self.time < EXPLOSION_DURATION
    }
}

// An animation enum to hold the different types of animations
pub enum Animation {
    Walk(Walk),
    Bullet(Bullet),
    ThrowItem(ThrowItem),
    Explosion(Explosion)
}

impl Animation {
    // Attempt to get the inner bullet
    pub fn as_bullet(&self) -> Option<&Bullet> {
        match *self {
            Animation::Bullet(ref bullet) if !bullet.finished => Some(bullet),
            _ => None
        }
    }

    // Attempt to get the inner thrown item
    pub fn as_throw_item(&self) -> Option<&ThrowItem> {
        match *self {
            Animation::ThrowItem(ref throw_item) => Some(throw_item),
            _ => None
        }
    }

    pub fn as_explosion(&self) -> Option<&Explosion> {
        match *self {
            Animation::Explosion(ref explosion) if explosion.time > 0.0 => Some(explosion),
            _ => None
        }
    }
}

// A type for holding the animations
pub type Animations = Vec<Animation>;

// A trait for updating the animations
pub trait UpdateAnimations {
    fn update(&mut self, ctx: &mut Context, dt: f32);
}

impl UpdateAnimations for Animations {
    // Update all of the animations, keeping only those that are still going
    fn update(&mut self, ctx: &mut Context, dt: f32) {
        self.retain_mut(|animation| match *animation {
            Animation::Walk(ref mut animation) => animation.step(ctx, dt),
            Animation::Bullet(ref mut animation) => animation.step(ctx, dt),
            Animation::ThrowItem(ref mut animation) => animation.step(dt),
            Animation::Explosion(ref mut animation) => animation.step(dt),
        });
    }
}

// test extrapolation
#[test]
fn extrapolation_tests() {
    let map = Map::new(20, 10, 1.0);

    // Lateral directions
    assert_eq!(extrapolate(1.0, 1.0, 2.0, 1.0, &map), (25.0, 1.0));
    assert_eq!(extrapolate(1.0, 1.0, 1.0, 2.0, &map), (1.0, 15.0));
    assert_eq!(extrapolate(1.0, 1.0, 0.0, 1.0, &map), (-5.0, 1.0));
    assert_eq!(extrapolate(1.0, 1.0, 1.0, 0.0, &map), (1.0, -5.0));

    // Diagonal directions
    assert_eq!(extrapolate(1.0, 1.0, 0.0, 0.0, &map), (-5.0, -5.0));
    assert_eq!(extrapolate(1.0, 1.0, 0.0, 2.0, &map), (-5.0, 7.0));
    assert_eq!(extrapolate(1.0, 1.0, 2.0, 0.0, &map), (7.0, -5.0));
    assert_eq!(extrapolate(1.0, 1.0, 2.0, 2.0, &map), (15.0, 15.0));

    assert_eq!(extrapolate(0.0, 0.0, 2.0, 1.0, &map), (25.0, 12.5));
    assert_eq!(extrapolate(0.0, 0.0, 1.0, 2.0, &map), (7.5, 15.0));
}