//! The different weapons in the game

use std::fmt;

/// The type of weapon
pub enum WeaponType {
    Rifle,
    MachineGun,
    PlasmaRifle,
}

/// The struct for a weapon
pub struct Weapon {
    pub tag: WeaponType,
    pub cost: usize,
    pub damage: i16,
}

impl fmt::Display for Weapon {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self.tag {
            WeaponType::Rifle => "Rifle",
            WeaponType::MachineGun => "Machine Gun",
            WeaponType::PlasmaRifle => "Plasma Rifle"
        })
    }
}

impl Weapon {
    /// Create a new weapon based of the weapon type
    pub fn new(tag: WeaponType) -> Weapon {
        let (cost, damage) = match tag {
            WeaponType::Rifle       => (10, 40),
            WeaponType::MachineGun  => (5,  20),
            WeaponType::PlasmaRifle => (8,  60)
        };

        Weapon {
            tag, cost, damage
        }
    }
}