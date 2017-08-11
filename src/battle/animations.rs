// Unit and game animations

use rand;
use rand::distributions::{IndependentSample, Range};
use odds::vec::VecExt;

use super::map::Map;
use super::units::Unit;
use resources::{Image, SoundEffect};
use weapons::WeaponType;
use context::Context;

const MARGIN: f32 = 5.0;
// Bullets travel 30 tiles a second
const BULLET_SPEED: f32 = 30.0;
// The minimum length of time for a bullet animation is half a second
const MIN_BULLET_TIME: f32 = 0.5;
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
    status: AnimationStatus,
    unit_id: u8,
    x: usize,
    y: usize,
    cost: u16
}

impl Walk {
    // Create a new walk animation
    pub fn new(unit_id: u8, x: usize, y: usize, cost: u16) -> Walk {
        Walk {
            unit_id, x, y, cost,
            status: AnimationStatus::new()
        }
    }

    // Move the animation a step, and return if its still going
    // If not, move the unit
    fn step(&mut self, map: &mut Map, ctx: &Context, dt: f32) -> bool {
        self.status.increment(dt);
        
        let still_going = !self.status.past(WALK_TIME);

        if !still_going {
            match map.units.get_mut(self.unit_id) {
                Some(unit) => {
                    // Move the unit and play a walking sound
                    unit.move_to(self.x, self.y, self.cost);
                    map.tiles.at_mut(self.x, self.y).walk_on();
                    ctx.play_sound(SoundEffect::Walk);
                }
                _ => return true
            }

            // Update the visibility of the tiles
            map.tiles.update_visibility(&map.units);
        }

        still_going
    }
}

// A bullet animation for drawing on the screen
pub struct Bullet {
    pub x: f32,
    pub y: f32,
    pub direction: f32,
    pub status: AnimationStatus,
    weapon_type: WeaponType,
    left: bool,
    above: bool,
    target_id: u8,
    will_hit: bool,
    lethal: bool
}

impl Bullet {
    // Create a new bullet based of the firing unit and the target unit
    pub fn new(unit: &Unit, target: &Unit, will_hit: bool, lethal: bool) -> Bullet {
        let x = unit.x as f32;
        let y = unit.y as f32;
        let target_x = target.x as f32;
        let target_y = target.y as f32;
        // Calculate the direction of the bullet
        let mut direction = (target_y - y).atan2(target_x - x);

        // If the bullet won't hit the target, change the direction slightly
        if !will_hit {
            direction += Range::new(-0.2, 0.2).ind_sample(&mut rand::thread_rng());
        }

        Bullet {
           x, y, direction, will_hit, lethal,
           target_id: target.id,
           // Work out if the bullet started to the left/right and above/below the target
           left: x < target_x,
           above: y < target_y,
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
    fn step(&mut self, map: &mut Map, ctx: &Context, dt: f32) -> bool {
        // If the bullet hasn't started moving, play its sound effect
        if self.status.at_start() {
            ctx.play_sound(self.weapon_type.fire_sound());
        }

        // Increment the time
        self.status.increment(dt);

        // If the bullet is still moving
        if self.status.in_progress() {
            // Move the bullet
            self.x += self.direction.cos() * BULLET_SPEED * dt;
            self.y += self.direction.sin() * BULLET_SPEED * dt;

            // Get the target x and y position
            let (target_x, target_y) = match map.units.get(self.target_id) {
                Some(target) => (target.x as f32, target.y as f32),
                _ => return false
            };

            // Calculate if the bullet has hit the target (if it's going to)
            let hit_target = self.will_hit && (
                self.left  != (self.x < target_x) ||
                self.above != (self.y < target_y)
            );

            // And if the bullet is within a certain margin of the map
            let within_margins = 
                self.x >= -MARGIN && self.x <= map.tiles.cols as f32 + MARGIN &&
                self.y >= -MARGIN && self.y <= map.tiles.rows as f32 + MARGIN;

            // Set if the bullet is finished
            self.status.finished = hit_target || !within_margins;

            // If the bullet is finished and is lethal, kill the target unit
            if self.status.finished && self.lethal {
                map.units.kill(&mut map.tiles, self.target_id);
            }
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
    fn update(&mut self, map: &mut Map, ctx: &Context, dt: f32);
}

impl UpdateAnimations for Animations {
    // Update all of the animations, keeping only those that are still going
    fn update(&mut self, map: &mut Map, ctx: &Context, dt: f32) {
        self.retain_mut(|mut animation| match *animation {
            Animation::Walk(ref mut walk) => walk.step(map, ctx, dt),
            Animation::Bullet(ref mut bullet) => bullet.step(map, ctx, dt)
        });
    }
}