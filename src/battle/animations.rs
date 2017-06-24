use rand;
use rand::distributions::{IndependentSample, Range};
use odds::vec::VecExt;

use std::slice::Iter;

use battle::map::Map;
use battle::units::{UnitType, Units};
use weapons::WeaponType;
use items::{Item, ItemType};

const MARGIN: f32 = 5.0;
const BULLET_SPEED: f32 = 0.5;
const WALK_SPEED: f32 = 0.1;

pub struct Walk {
    status: f32,
    unit_id: usize,
    x: usize,
    y: usize,
    cost: usize
}

impl Walk {
    pub fn new(unit_id: usize, x: usize, y: usize, cost: usize) -> Walk {
        Walk {
            unit_id, x, y, cost,
            status: 0.0
        }
    }

    fn step(&mut self, map: &mut Map) -> bool {
        self.status += WALK_SPEED;
        
        let still_going = self.status <= 1.0;

        if !still_going {
            match map.units.get_mut(self.unit_id) {
                Some(unit) => unit.move_to(self.x, self.y, self.cost),
                _ => return true
            }

            map.tiles.update_visibility(&map.units);
        }

        still_going
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

    fn step(&mut self, map: &mut Map) -> bool {
        self.status += WALK_SPEED;
        let still_going = self.status <= 1.0;

        if !still_going {
            let (x, y) = match map.units.get(self.unit_id) {
                Some(unit) => (unit.x, unit.y),
                _ => return true
            };

            let corpse = Item::new(match map.units.get(self.unit_id).map(|unit| unit.tag) {
                Some(UnitType::Squaddie) => ItemType::SquaddieCorpse,
                Some(UnitType::Machine) => ItemType::MachineCorpse,
                _ => return true
            });

            map.units.kill(self.unit_id);
            map.tiles.drop(x, y, corpse);
            map.tiles.update_visibility(&map.units);
        }

        still_going
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
        let unit = units.get(unit_id).unwrap();
        let target = units.get(target_id).unwrap();

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
    fn step(&mut self, map: &Map) -> bool {
        self.x += self.direction.cos() * BULLET_SPEED;
        self.y += self.direction.sin() * BULLET_SPEED;

        // Work out if the bullet is currently traveling or has reached the destination

        // If the bullet won't hit the target or hasn't hit the target
        (!self.will_hit || (
            self.left  == (self.x < self.target_x as f32) &&
            self.above == (self.y < self.target_y as f32)
        )) &&
        // And if the bullet is within a certain margin of the map
        self.x >= -MARGIN && self.x <= map.tiles.cols as f32 + MARGIN &&
        self.y >= -MARGIN && self.y <= map.tiles.rows as f32 + MARGIN
    }
}

pub enum Animation {
    Walk(Walk),
    Bullet(Bullet),
    Dying(Dying)
}

pub struct Animations {
    animations: Vec<Animation>
}

impl Animations {
    pub fn new() -> Animations {
        Animations {
            animations: Vec::new()
        }
    }

    pub fn iter(&self) -> Iter<Animation> {
        self.animations.iter()
    }

    pub fn update(&mut self, map: &mut Map) {
        self.animations.retain_mut(|mut animation| match animation {
            &mut Animation::Walk(ref mut walk) => walk.step(map),
            &mut Animation::Bullet(ref mut bullet) => bullet.step(map),
            &mut Animation::Dying(ref mut dying) => dying.step(map)
        });
    }

    pub fn push(&mut self, animation: Animation) {
        self.animations.push(animation);
    }

    pub fn is_empty(&self) -> bool {
        self.animations.is_empty()
    }
}