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
const BULLET_SPEED: f32 = 30.0;
const WALK_SPEED: f32 = 5.0;

// A pretty simple walk animation
pub struct Walk {
    status: f32,
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
            status: 0.0
        }
    }

    // Move the animation a step, and return if its still going
    // If not, move the unit
    fn step(&mut self, map: &mut Map, ctx: &Context, dt: f32) -> bool {
        self.status += WALK_SPEED * dt;
        
        let still_going = self.status <= 1.0;

        if !still_going {
            match map.units.get_mut(self.unit_id) {
                Some(unit) => {
                    unit.move_to(self.x, self.y, self.cost);
                    ctx.play_sound(SoundEffect::Walk);
                }
                _ => return true
            }

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
    pub weapon_type: WeaponType,
    left: bool,
    above: bool,
    target_id: u8,
    target_x: f32,
    target_y: f32,
    will_hit: bool,
    lethal: bool,
    started: bool
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
            let mut rng = rand::thread_rng();
            direction += Range::new(-0.2, 0.2).ind_sample(&mut rng);
        }

        Bullet {
           x, y, direction, target_x, target_y, will_hit, lethal,
           target_id: target.id,
           // Work out if the bullet started to the left/right and above/below the target
           left: x < target_x,
           above: y < target_y,
           // Get the type of the firing weapon
           weapon_type: unit.weapon.tag,
           // The bullet hasn't started moving
           started: false
        }
    }
    
    // Get the image of the bullet
    pub fn image(&self) -> Image {
        self.weapon_type.bullet()
    }

    // Move the bullet a step and work out if its still going or not
    fn step(&mut self, map: &mut Map, ctx: &Context, dt: f32) -> bool {
        // If the bullet hasn't started moving, play its sound effect
        if !self.started {
            ctx.play_sound(self.weapon_type.fire_sound());
            self.started = true;
        }

        // Move the bullet
        self.x += self.direction.cos() * BULLET_SPEED * dt;
        self.y += self.direction.sin() * BULLET_SPEED * dt;

        // If the bullet won't hit the target or hasn't hit the target
        let still_going = (!self.will_hit || (
            self.left  == (self.x < self.target_x) &&
            self.above == (self.y < self.target_y)
        )) &&
        // And if the bullet is within a certain margin of the map
        self.x >= -MARGIN && self.x <= map.tiles.cols as f32 + MARGIN &&
        self.y >= -MARGIN && self.y <= map.tiles.rows as f32 + MARGIN;

        // If the bullet is finished and is lethal, kill the target unit
        if !still_going && self.lethal {
            map.units.kill(&mut map.tiles, self.target_id);
        }

        still_going
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