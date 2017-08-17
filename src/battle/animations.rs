// Unit and game animations

use rand;
use rand::distributions::{IndependentSample, Range};
use odds::vec::VecExt;

use super::map::Map;
use super::units::Unit;
use resources::{Image, SoundEffect};
use weapons::WeaponType;
use context::Context;
use utils::{direction, clamp};

const MARGIN: f32 = 5.0;
// Bullets travel 30 tiles a second
const BULLET_SPEED: f32 = 30.0;
// The minimum length of time for a bullet animation is a quarter of a second
const MIN_BULLET_TIME: f32 = 0.25;
// Units move 5 tiles a second
const WALK_TIME: f32 = 1.0 / 5.0;

pub struct AnimationStatus {
    status: f32,
    finished: bool
}

impl AnimationStatus {
    fn new() -> AnimationStatus {
        AnimationStatus {
            status: 0.0,
            finished: false
        }
    }

    fn increment(&mut self, value: f32) {
        self.status += value;
    }

    fn past(&self, value: f32) -> bool {
        self.status >= value
    }

    fn at_start(&self) -> bool {
        self.status == 0.0
    }

    pub fn in_progress(&self) -> bool {
        !self.at_start() && !self.finished
    }
}

// A pretty simple walk animation
pub struct Walk {
    status: AnimationStatus
}

impl Walk {
    // Create a new walk animation
    pub fn new() -> Walk {
        Walk {
            status: AnimationStatus::new()
        }
    }

    // Move the animation a step, and return if its still going
    fn step(&mut self, ctx: &Context, dt: f32) -> bool {
        if self.status.at_start() {
            ctx.play_sound(SoundEffect::Walk);
        }
        
        self.status.increment(dt);

        !self.status.past(WALK_TIME)
    }
}

// A bullet animation for drawing on the screen
pub struct Bullet {
    pub x: f32,
    pub y: f32,
    pub direction: f32,
    pub status: AnimationStatus,
    weapon_type: WeaponType,
    target_x: f32,
    target_y: f32,
}

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

impl Bullet {
    // Create a new bullet based of the firing unit and the target unit
    pub fn new(unit: &Unit, target_x: usize, target_y: usize, will_hit: bool, map: &Map) -> Bullet {
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

        Bullet {
           x, y, direction, target_x, target_y,
           // Get the type of the firing weapon
           weapon_type: unit.weapon.tag,
           // The bullet hasn't started moving
           status: AnimationStatus::new()
        }
    }
    
    // Get the image of the bullet
    pub fn image(&self) -> Image {
        self.weapon_type.bullet()
    }

    // Move the bullet a step and work out if its still going or not
    fn step(&mut self, ctx: &Context, dt: f32) -> bool {
        // If the bullet hasn't started moving, play its sound effect
        if self.status.at_start() {
            ctx.play_sound(self.weapon_type.fire_sound());
        }

        // Increment the time
        self.status.increment(dt);

        // If the bullet is still moving
        if self.status.in_progress() {
            // Work out if the bullet started to the left/right and above/below the target
            let left = self.x < self.target_x;
            let above = self.y < self.target_y;

            // Move the bullet
            self.x += self.direction.cos() * BULLET_SPEED * dt;
            self.y += self.direction.sin() * BULLET_SPEED * dt;
            
            // Set if the bullet is past the target
            self.status.finished = left != (self.x < self.target_x) || above != (self.y < self.target_y);
        }

        // If the bullet hasn't finished or isn't past the minimum time
        !(self.status.finished && self.status.past(MIN_BULLET_TIME))
    }
}

// An animation enum to hold the different types of animations
pub enum Animation {
    Walk(Walk),
    Bullet(Bullet),
}

// A type for holding the animations
pub type Animations = Vec<Animation>;

// A trait for updating the animations
pub trait UpdateAnimations {
    fn update(&mut self, ctx: &Context, dt: f32);
}

impl UpdateAnimations for Animations {
    // Update all of the animations, keeping only those that are still going
    fn update(&mut self, ctx: &Context, dt: f32) {
        self.retain_mut(|animation| match *animation {
            Animation::Walk(ref mut walk) => walk.step(ctx, dt),
            Animation::Bullet(ref mut bullet) => bullet.step(ctx, dt)
        });
    }
}

// test extrapolation
#[test]
fn extrapolation_tests() {
    let map = Map::new(20, 10);

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