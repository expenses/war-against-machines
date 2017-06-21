use rand;
use rand::distributions::{IndependentSample, Range};

use map::tiles::Tiles;
use map::units::Units;
use weapons::WeaponType;

const MARGIN: f32 = 5.0;
const BULLET_SPEED: f32 = 0.5;
const WALK_SPEED: f32 = 0.1;

pub struct Walk {
    status: f32
}

impl Walk {
    pub fn new() -> Walk {
        Walk {
            status: 0.0
        }
    }

    fn step(&mut self) -> bool {
        self.status += WALK_SPEED;
        
        self.status > 1.0
    }
}

pub struct Dying {
    unit_id: usize,
    status: f32
}

impl Dying {
    pub fn new(unit_id: usize) -> Dying {
        Dying {
            unit_id,
            status: 0.0
        }
    }

    fn step(&mut self, units: &mut Units) -> bool {
        self.status += WALK_SPEED;
        let finished = self.status > 1.0;

        if finished {
            units.get_mut(self.unit_id).update();
        }

        finished
    }
}

// A bullet for drawing on the screen
pub struct Bullet {
    pub x: f32,
    pub y: f32,
    pub direction: f32,
    pub image: String,
    left: bool,
    above: bool,
    target_x: f32,
    target_y: f32,
    will_hit: bool
}

impl Bullet {
    // Create a new bullet based of the firing unit and the target unit
    pub fn new(unit_id: usize, target_id: usize, will_hit: bool, units: &Units) -> Bullet {
        let unit = units.get(unit_id);
        let target = units.get(target_id);

        let x = unit.x as f32;
        let y = unit.y as f32;
        let target_x = target.x as f32;
        let target_y = target.y as f32;
        // Calculate the direction of the bullet
        let mut direction = (target_y - y).atan2(target_x - x);

        let image = match unit.weapon.tag {
            WeaponType::Rifle => "rifle_round",
            WeaponType::MachineGun => "machine_gun_round",
            WeaponType::PlasmaRifle => "plasma_round"
        }.into();

        // If the bullet won't hit the target, change the direction slightly
        if !will_hit {
            let mut rng = rand::thread_rng();
            direction += Range::new(-0.1, 0.1).ind_sample(&mut rng);
        }

        // Work out if the bullet started to the left/right and above/below the target
        let left = x < target_x;
        let above = y < target_y;

        Bullet {
           x, y, direction, image, left, above, target_x, target_y, will_hit
        }
    }
    
    // Move the bullet
    fn step(&mut self, tiles: &mut Tiles) -> bool {        
        self.x += self.direction.cos() * BULLET_SPEED;
        self.y += self.direction.sin() * BULLET_SPEED;

        // Work out if the bullet is currently traveling or has reached the destination

        // If the bullet won't hit the target or hasn't hit the target
        (self.will_hit && (
            self.left  != (self.x < self.target_x as f32) ||
            self.above != (self.y < self.target_y as f32)
        )) ||
        // And if the bullet is within a certain margin of the map
        self.x < -MARGIN || self.x > tiles.cols as f32 + MARGIN ||
        self.y < -MARGIN || self.y > tiles.rows as f32 + MARGIN
    }
}

pub enum Animation {
    Walk(Walk),
    Bullet(Bullet),
    Dying(Dying)
}

pub struct AnimationQueue {
    animations: Vec<Animation>
}

impl AnimationQueue {
    pub fn new() -> AnimationQueue {
        AnimationQueue {
            animations: Vec::new()
        }
    }

    pub fn first_bullet(&self) -> Option<&Bullet> {
        match self.animations.first() {
            Some(&Animation::Bullet(ref bullet)) => Some(bullet),
            _ => None
        }
    }

    pub fn update(&mut self, tiles: &mut Tiles, units: &mut Units) {
        let finished = match self.animations.first_mut() {
            Some(&mut Animation::Walk(ref mut walk)) => walk.step(),
            Some(&mut Animation::Bullet(ref mut bullet)) => bullet.step(tiles),
            Some(&mut Animation::Dying(ref mut dying)) => dying.step(units),
            _ => false
        };

        if finished {
            self.animations.remove(0);
        }
    }

    pub fn push(&mut self, animation: Animation) {
        self.animations.push(animation);
    }

    pub fn empty(&self) -> bool {
        self.animations.is_empty()
    }
}