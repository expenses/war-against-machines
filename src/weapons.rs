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