// The different weapons in the game

use std::fmt;

// The type of weapon
pub enum WeaponType {
    Rifle,
    MachineGun,
    PlasmaRifle,
}

pub enum FiringMode {
    SingleShot,
    AimedShot,
    SemiAuto,
    FullAuto
}

// The struct for a weapon
pub struct Weapon {
    pub tag: WeaponType,
    pub cost: usize,
    pub damage: i16,
    pub mode: usize,
    pub modes: Vec<FiringMode>
}

impl fmt::Display for Weapon {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - {}", match self.tag {
            WeaponType::Rifle => "Rifle",
            WeaponType::MachineGun => "Machine Gun",
            WeaponType::PlasmaRifle => "Plasma Rifle"
        }, match self.modes[self.mode] {
            FiringMode::SingleShot => "Single Shot",
            FiringMode::AimedShot => "Aimed Shot",
            FiringMode::SemiAuto => "Semi Auto",
            FiringMode::FullAuto => "Full Auto"
        })
    }
}

impl Weapon {
    // Create a new weapon based of the weapon type
    pub fn new(tag: WeaponType) -> Weapon {
        match tag {
            WeaponType::Rifle => Weapon {
                tag,
                cost: 10,
                damage: 40,
                mode: 0,
                modes: vec![FiringMode::SingleShot, FiringMode::AimedShot, FiringMode::SemiAuto] 
            },
            WeaponType::MachineGun => Weapon {
                tag,
                cost: 5,
                damage: 40,
                mode: 0,
                modes: vec![FiringMode::SingleShot, FiringMode::SemiAuto, FiringMode::FullAuto]
            },
            WeaponType::PlasmaRifle => Weapon {
                tag,
                cost: 8,
                damage: 60,
                mode: 0,
                modes: vec![FiringMode::SingleShot, FiringMode::AimedShot, FiringMode::SemiAuto] 
            }
        }
    }
}