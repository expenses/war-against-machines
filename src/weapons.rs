// The different weapons in the game

use std::fmt;

use resources::{Image, SoundEffect};

// The type of weapon
#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum WeaponType {
    Rifle,
    MachineGun,
    PlasmaRifle
}

impl WeaponType {
    // Get the corresponding bullet image
    pub fn bullet(&self) -> Image {
        match *self {
            WeaponType::PlasmaRifle => Image::PlasmaBullet,
            _ => Image::RegularBullet
        }
    }

    // Get the corresponding fire sound
    pub fn fire_sound(&self) -> SoundEffect {
        match *self {
            WeaponType::PlasmaRifle => SoundEffect::PlasmaShot,
            _ => SoundEffect::RegularShot
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum FiringMode {
    SingleShot,
    AimedShot,
    SemiAuto,
    FullAuto
}

pub struct FiringModeInfo {
    pub hit_modifier: f32,
    pub cost: u16,
    pub bullets: u8
}

// The struct for a weapon
#[derive(Serialize, Deserialize)]
pub struct Weapon {
    pub tag: WeaponType,
    base_cost: u16,
    base_bullets: u8,
    pub damage: i16,
    pub mode: usize,
    pub modes: Vec<FiringMode>
}

impl fmt::Display for Weapon {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let info = self.info();

        write!(f, "{} - {} (Hit modifier: {}, Cost: {}, Bullets: {}", match self.tag {
            WeaponType::Rifle => "Rifle",
            WeaponType::MachineGun => "Machine Gun",
            WeaponType::PlasmaRifle => "Plasma Rifle"
        }, match self.modes[self.mode] {
            FiringMode::SingleShot => "Single Shot",
            FiringMode::AimedShot => "Aimed Shot",
            FiringMode::SemiAuto => "Semi Auto",
            FiringMode::FullAuto => "Full Auto"
        }, info.hit_modifier, info.cost, info.bullets)
    }
}

impl Weapon {
    // Create a new weapon based of the weapon type
    pub fn new(tag: WeaponType) -> Weapon {
        match tag {
            WeaponType::Rifle => Weapon {
                tag,
                base_cost: 10,
                damage: 40,
                base_bullets: 1,
                mode: 0,
                modes: vec![FiringMode::SingleShot, FiringMode::AimedShot] 
            },
            WeaponType::MachineGun => Weapon {
                tag,
                base_cost: 5,
                damage: 20,
                base_bullets: 1,
                mode: 0,
                modes: vec![FiringMode::SingleShot, FiringMode::SemiAuto, FiringMode::FullAuto]
            },
            WeaponType::PlasmaRifle => Weapon {
                tag,
                base_cost: 8,
                damage: 60,
                base_bullets: 1,
                mode: 0,
                modes: vec![FiringMode::SingleShot, FiringMode::AimedShot, FiringMode::SemiAuto] 
            }
        }
    }

    // Change the fire mode
    pub fn change_mode(&mut self) {
        self.mode = (self.mode + 1) % self.modes.len()
    }

    // get the firing cost
    fn cost(&self, modifier: f32) -> u16 {
        (self.base_cost as f32 * modifier).ceil() as u16
    }

    // Get the hit modifier, the firing cost and the bullets fired
    pub fn info(&self) -> FiringModeInfo {
        match self.modes[self.mode] {
            FiringMode::SingleShot => FiringModeInfo {
                hit_modifier: 1.0, 
                cost: self.cost(1.0),
                bullets: self.base_bullets
            },
            FiringMode::AimedShot  => FiringModeInfo {
                hit_modifier: 1.5,
                cost: self.cost(2.0),
                bullets: self.base_bullets
            },
            FiringMode::SemiAuto   => FiringModeInfo {
                hit_modifier: 0.75,
                cost: self.cost(1.5),
                bullets: self.base_bullets * 3
            },
            FiringMode::FullAuto   => FiringModeInfo {
                hit_modifier: 0.66,
                cost: self.cost(2.5),
                bullets: self.base_bullets * 6
            }
        }
    }
}