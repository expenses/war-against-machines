pub enum WeaponType {
    Rifle,
    MachineGun,
    PlasmaRifle,
}

pub struct Weapon {
    tag: WeaponType,
    capacity: usize,
    accuracy: f32,
    damage: u8,
    ammo: usize,
}

impl Weapon {
    pub fn new(tag: WeaponType) -> Weapon {
        let (capacity, accuracy, damage) = match tag {
            WeaponType::Rifle       => (24, 0.5, 5),
            WeaponType::MachineGun  => (48, 0.3, 3),
            WeaponType::PlasmaRifle => (18, 0.7, 10)
        };

        Weapon {
            tag, capacity, accuracy, damage,
            ammo: capacity
        }
    }

    pub fn fire(&mut self) {
        self.ammo -= 1;
    }

    pub fn reload(&mut self) {
        self.ammo = self.capacity;
    }
}