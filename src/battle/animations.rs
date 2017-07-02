// Unit and game animations

use rand;
use rand::distributions::{IndependentSample, Range};
use odds::vec::VecExt;

use battle::map::Map;
use battle::units::Unit;
use Resources;

const MARGIN: f32 = 5.0;
const BULLET_SPEED: f32 = 0.5;
const WALK_SPEED: f32 = 0.1;

// A pretty simple walk animation
pub struct Walk {
    status: f32,
    unit_id: usize,
    x: usize,
    y: usize,
    cost: usize
}

impl Walk {
    // Create a new walk animation
    pub fn new(unit_id: usize, x: usize, y: usize, cost: usize) -> Walk {
        Walk {
            unit_id, x, y, cost,
            status: 0.0
        }
    }

    // Move the animation a step, and return if its still going
    // If not, move the unit
    fn step(&mut self, map: &mut Map, resources: &Resources) -> bool {
        self.status += WALK_SPEED;
        
        let still_going = self.status <= 1.0;

        if !still_going {
            match map.units.get_mut(self.unit_id) {
                Some(unit) => {
                    unit.move_to(self.x, self.y, self.cost);
                    resources.play_audio("walk");
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
    pub image: &'static str,
    left: bool,
    above: bool,
    target_id: usize,
    target_x: f32,
    target_y: f32,
    will_hit: bool,
    lethal: bool,
    started: bool
}

impl Bullet {
    // Create a new bullet based of the firing unit and the target unit
    pub fn new(target_id: usize, unit: &Unit, target: &Unit, will_hit: bool, lethal: bool) -> Bullet {
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
           x, y, direction, target_id, target_x, target_y, will_hit, lethal,
           // Work out if the bullet started to the left/right and above/below the target
           left: x < target_x,
           above: y < target_y,
           // Get the image of the bullet
           image: unit.weapon.bullet(),
           // The bullet hasn't started moving
           started: false
        }
    }
    
    // Move the bullet a step and work out if its still going or not
    fn step(&mut self, map: &mut Map, resources: &Resources) -> bool {
        // If the bullet hasn't started moving, play its sound effect
        if !self.started {
            resources.play_audio("plasma");
            self.started = true;
        }

        // Move the bullet
        self.x += self.direction.cos() * BULLET_SPEED;
        self.y += self.direction.sin() * BULLET_SPEED;

        // Work out if the bullet is currently traveling or has reached the destination

        // If the bullet won't hit the target or hasn't hit the target
        let still_going = (!self.will_hit || (
            self.left  == (self.x < self.target_x as f32) &&
            self.above == (self.y < self.target_y as f32)
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
    fn update(&mut self, map: &mut Map, resources: &Resources);
}

impl UpdateAnimations for Animations {
    // Update all of the animations, keeping only those that are still going
    fn update(&mut self, map: &mut Map, resources: &Resources) {
        self.retain_mut(|mut animation| match *animation {
            Animation::Walk(ref mut walk) => walk.step(map, resources),
            Animation::Bullet(ref mut bullet) => bullet.step(map, resources)
        });
    }
}