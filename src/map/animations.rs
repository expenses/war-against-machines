use rand;
use rand::distributions::{IndependentSample, Range};

use map::units::Unit;

const MARGIN: f32 = 5.0;

// A bullet for drawing on the screen
pub struct Bullet {
    pub x: f32,
    pub y: f32,
    pub direction: f32,
    left: bool,
    above: bool,
    target_x: f32,
    target_y: f32,
    will_hit: bool
}

impl Bullet {
    // Create a new bullet based of the firing unit and the target unit
    pub fn new(fired_by: &Unit, target: &Unit, will_hit: bool) -> Bullet {
        let x = fired_by.x as f32;
        let y = fired_by.y as f32;
        let target_x = target.x as f32;
        let target_y = target.y as f32;
        // Calculate the direction of the bullet
        let mut direction = (target_y - y).atan2(target_x - x);

        // If the bullet won't hit the target, change the direction slightly
        if !will_hit {
            let mut rng = rand::thread_rng();
            direction += Range::new(-0.1, 0.1).ind_sample(&mut rng);
        }

        // Work out if the bullet started to the left/right and above/below the target
        let left = x < target_x;
        let above = y < target_y;

        Bullet {
           x, y, direction,
           left, above,
           target_x, target_y,
           will_hit
        }
    }

    // Move the bullet
    pub fn travel(&mut self) {        
        self.x += self.direction.cos();
        self.y += self.direction.sin();
    }

    // Work out if the bullet is currently traveling or has reached the destination
    pub fn traveling(&self, cols: usize, rows: usize) -> bool {
        // If the bullet won't hit the target or hasn't hit the target
        (!self.will_hit || (
            self.left == (self.x < self.target_x) &&
            self.above == (self.y < self.target_y)
        )) &&
        // And if the bullet is within a certain margin of the map
        self.x > -MARGIN && self.x < cols as f32 + MARGIN &&
        self.y > -MARGIN && self.y < rows as f32 + MARGIN
    }
}

pub struct AnimationQueue {
    bullets: Vec<Bullet>
}

impl AnimationQueue {
    pub fn new() -> AnimationQueue {
        AnimationQueue {
            bullets: Vec::new()
        }
    }

    pub fn first(&self) -> Option<&Bullet> {
        self.bullets.first()
    }

    pub fn update(&mut self, cols: usize, rows: usize) -> bool {
        let next = match self.bullets.first_mut() {
            Some(bullet) => {
                bullet.travel();
                
                !bullet.traveling(cols, rows)
            }
            _ => false
        };

        if next {
            self.bullets.remove(0);
        }

        next
    }

    pub fn push(&mut self, bullet: Bullet) {
        self.bullets.push(bullet);
    }

    pub fn empty(&self) -> bool {
        self.bullets.is_empty()
    }
}