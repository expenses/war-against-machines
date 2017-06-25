//! The different weapons in the game

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
    pub name: String
}

impl Weapon {
    /// Create a new weapon based of the weapon type
    pub fn new(tag: WeaponType) -> Weapon {
        let (cost, damage, name) = match tag {
            WeaponType::Rifle       => (10, 40, "Rifle"),
            WeaponType::MachineGun  => (5,  20, "Machine Gun"),
            WeaponType::PlasmaRifle => (8,  60, "Plasma Rifle")
        };

        Weapon {
            tag, cost, damage,
            name: name.into()
        }
    }
}