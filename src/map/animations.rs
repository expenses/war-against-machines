use rand;
use rand::distributions::{IndependentSample, Range};

use map::paths::PathPoint;
use map::tiles::Tiles;
use map::units::{Unit, Units};
use weapons::WeaponType;

const MARGIN: f32 = 5.0;
const BULLET_SPEED: f32 = 0.5;
const WALK_SPEED: f32 = 0.1;

trait Animation {
    fn step(&mut self, units: &mut Units, tiles: &mut Tiles) -> bool;
}

pub struct Walk {
    unit: usize,
    progress: f32,
    path: Vec<PathPoint>
}

impl Walk {
    pub fn new(unit: usize, path: Vec<PathPoint>) -> Walk {
        Walk {
            unit, path,
            progress: 0.0
        }
    }
}

impl Animation for Walk {
    fn step(&mut self, units: &mut Units, tiles: &mut Tiles) -> bool {
        self.progress += WALK_SPEED;

        if self.progress > 1.0 {
            {
                let point = &self.path[0];

                if units.at(point.x, point.y).is_some() {
                    return true;
                }

                let unit = units.get_mut(self.unit);
                unit.x = point.x;
                unit.y = point.y;
                unit.moves -= point.cost;
            }

            tiles.update_visibility(units);

            self.progress = self.progress % 1.0;

            self.path.remove(0);
        }

        self.path.len() == 0
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
    target_id: usize,
    target_x: f32,
    target_y: f32,
    lethal: bool,
    will_hit: bool
}

impl Bullet {
    // Create a new bullet based of the firing unit and the target unit
    pub fn new(fired_by: &Unit, target_id: usize, target: &mut Unit, lethal: bool, will_hit: bool) -> Bullet {
        let x = fired_by.x as f32;
        let y = fired_by.y as f32;
        let target_x = target.x as f32;
        let target_y = target.y as f32;
        // Calculate the direction of the bullet
        let mut direction = (target_y - y).atan2(target_x - x);

        let image = match fired_by.weapon.tag {
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
           x, y, direction, image, left, above, target_id, target_x, target_y, lethal, will_hit
        }
    }
}

impl Animation for Bullet {
    // Move the bullet
    fn step(&mut self, units: &mut Units, tiles: &mut Tiles) -> bool {        
        self.x += self.direction.cos() * BULLET_SPEED;
        self.y += self.direction.sin() * BULLET_SPEED;

        // Work out if the bullet is currently traveling or has reached the destination

        // If the bullet won't hit the target or hasn't hit the target
        let finished = (self.will_hit && (
            self.left  != (self.x < self.target_x as f32) ||
            self.above != (self.y < self.target_y as f32)
        )) ||
        // And if the bullet is within a certain margin of the map
        self.x < -MARGIN || self.x > tiles.cols as f32 + MARGIN ||
        self.y < -MARGIN || self.y > tiles.rows as f32 + MARGIN;

        if finished && self.lethal {
            units.get_mut(self.target_id).update();
        }

        finished
    }
}

pub struct AnimationQueue {
    bullets: Vec<Bullet>,
    walks: Vec<Walk>
}

impl AnimationQueue {
    pub fn new() -> AnimationQueue {
        AnimationQueue {
            bullets: Vec::new(),
            walks: Vec::new()
        }
    }

    pub fn first(&self) -> Option<&Bullet> {
        self.bullets.first()
    }

    pub fn update(&mut self, tiles: &mut Tiles, units: &mut Units) {
        if self.bullets.first_mut().map(|bullet| bullet.step(units, tiles)).unwrap_or(false) {
            self.bullets.remove(0);
        }
        
        if self.walks.first_mut().map(|walk| walk.step(units, tiles)).unwrap_or(false) {
            self.walks.remove(0);
        }
    }

    pub fn add_bullet(&mut self, bullet: Bullet) {
        self.bullets.push(bullet);
    }

    pub fn add_walk(&mut self, walk: Walk) {
        self.walks.push(walk);
    }

    pub fn _empty(&self) -> bool {
        self.bullets.is_empty()
    }
}