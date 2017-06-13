use rand;
use units::Unit;
use rand::distributions::{IndependentSample, Range};

pub enum WeaponType {
    Rifle,
    MachineGun,
    PlasmaRifle,
    // None
}

/*
pub enum FireMode {
    SingleShot,
    AimedShot,
    SemiAuto,
    FullAuto
}
*/

pub struct Weapon {
    tag: WeaponType,
    pub cost: usize,
    pub damage: u8,
    // modes: Vec<FireMode>
}

impl Weapon {
    pub fn new(tag: WeaponType) -> Weapon {
        let (cost, damage) = match tag {
            WeaponType::Rifle       => (5,  25), // vec![FireMode::SingleShot, FireMode::AimedShot]),
            WeaponType::MachineGun  => (10, 30), // vec![FireMode::SingleShot, FireMode::SemiAuto, FireMode::FullAuto]),
            WeaponType::PlasmaRifle => (15, 75)  // vec![FireMode::SingleShot, FireMode::AimedShot, FireMode::SemiAuto]),
        };

        Weapon {
            tag, cost, damage
        }
    }

    pub fn name(&self) -> &str {
        match self.tag {
            WeaponType::Rifle       => "Rifle",
            WeaponType::MachineGun  => "Machine Gun",
            WeaponType::PlasmaRifle => "Plasma Rifle",
        }
    }
}

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
    pub fn new(fired_by: &Unit, target: &Unit, will_hit: bool) -> Bullet {
        let x = fired_by.x as f32;
        let y = fired_by.y as f32;
        let target_x = target.x as f32;
        let target_y = target.y as f32;
        let mut direction = (target_y - y).atan2(target_x - x);

        if !will_hit {
            let mut rng = rand::thread_rng();
            direction += Range::new(-0.1, 0.1).ind_sample(&mut rng);
        }

        let left = x < target_x;
        let above = y < target_y;

        Bullet {
           x, y, direction,
           left, above,
           target_x, target_y,
           will_hit
        }
    }

    pub fn travel(&mut self) {        
        self.x += self.direction.cos();
        self.y += self.direction.sin();
    }

    pub fn traveling(&self) -> bool {
        !self.will_hit || (self.left == (self.x < self.target_x) && self.above == (self.y < self.target_y))
    }

    pub fn on_map(&self, cols: usize, rows: usize) -> bool {
        self.x > -5.0 && self.x < cols as f32 + 5.0 &&
        self.y > -5.0 && self.y < rows as f32 + 5.0
    }
}