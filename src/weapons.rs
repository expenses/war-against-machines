// The type of weapon
pub enum WeaponType {
    Rifle,
    MachineGun,
    PlasmaRifle,
}

// The struct for a weapon
pub struct Weapon {
    tag: WeaponType,
    pub cost: usize,
    pub damage: i16,
}

impl Weapon {
    // Create a new weapon based off the weapon type
    pub fn new(tag: WeaponType) -> Weapon {
        let (cost, damage) = match tag {
            WeaponType::Rifle       => (5,  25),
            WeaponType::MachineGun  => (10, 30),
            WeaponType::PlasmaRifle => (15, 75)
        };

        Weapon {
            tag, cost, damage
        }
    }

    // The name of the weapon
    pub fn name(&self) -> &str {
        match self.tag {
            WeaponType::Rifle       => "Rifle",
            WeaponType::MachineGun  => "Machine Gun",
            WeaponType::PlasmaRifle => "Plasma Rifle",
        }
    }
}