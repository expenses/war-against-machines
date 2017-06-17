// The type of weapon
pub enum WeaponType {
    Rifle,
    MachineGun,
    PlasmaRifle,
}

// The struct for a weapon
pub struct Weapon {
    pub tag: WeaponType,
    pub cost: usize,
    pub damage: i16,
    pub name: String
}

impl Weapon {
    // Create a new weapon based off the weapon type
    pub fn new(tag: WeaponType) -> Weapon {
        let (cost, damage, name) = match tag {
            WeaponType::Rifle       => (5,  25, "Rifle"),
            WeaponType::MachineGun  => (10, 30, "Machine Gun"),
            WeaponType::PlasmaRifle => (15, 75, "Plasma Rifle")
        };

        Weapon {
            tag, cost, damage,
            name: name.into()
        }
    }
}