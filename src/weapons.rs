
use units::Unit;

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
    _target_x: f32,
    _target_y: f32,
    pub direction: f32,
    _will_hit: bool
}

impl Bullet {
    pub fn new(fired_by: &Unit, target: &Unit, _will_hit: bool) -> Bullet {
        let x = fired_by.x as f32;
        let y = fired_by.y as f32;
        let target_x = target.x as f32;
        let target_y = target.y as f32;
        let direction = (target_y - y).atan2(target_x - x);

        Bullet {
           x, y, direction,
           _target_x: target_x,
           _target_y: target_y,
           _will_hit
        }
    }

    pub fn travel(&mut self) {        
        self.x += self.direction.cos();
        self.y += self.direction.sin();
    }
}