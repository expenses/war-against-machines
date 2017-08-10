// The different weapons in the game

use std::fmt;

use resources::{Image, SoundEffect};
use items::Item;

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

    pub fn cost(&self) -> u16 {
        match *self {
            WeaponType::Rifle => 10,
            WeaponType::MachineGun => 5,
            WeaponType::PlasmaRifle => 8
        }
    }

    pub fn damage(&self) -> i16 {
        match *self {
            WeaponType::Rifle => 40,
            WeaponType::MachineGun => 20,
            WeaponType::PlasmaRifle => 60
        }
    }

    pub fn capacity(&self) -> u8 {
        match *self {
            WeaponType::Rifle => 6,
            WeaponType::MachineGun => 10,
            WeaponType::PlasmaRifle => 8
        }
    }

    pub fn weight(&self) -> f32 {
        match *self {
            WeaponType::Rifle => Item::Rifle(0).weight(),
            WeaponType::MachineGun => Item::MachineGun(0).weight(),
            WeaponType::PlasmaRifle => Item::PlasmaRifle(0).weight()
        }
    }
}

impl fmt::Display for WeaponType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            WeaponType::Rifle => "Rifle",
            WeaponType::MachineGun => "Machine Gun",
            WeaponType::PlasmaRifle => "Plasma Rifle"
        })
    }
}

// The struct for a weapon
#[derive(Serialize, Deserialize)]
pub struct Weapon {
    pub tag: WeaponType,
    pub ammo: u8
}

impl fmt::Display for Weapon {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({}/{})", self.tag, self.ammo, self.tag.capacity())
    }
}

impl Weapon {
    // Create a new weapon based of the weapon type
    pub fn new(tag: WeaponType, ammo: u8) -> Weapon {
        Weapon {
            tag, ammo
        }
    }

    // Use up a piece of ammo in the weapon
    pub fn fire(&mut self) {
        if self.ammo != 0 {
            self.ammo -= 1;
        }
    }

    // Can the weapon be fired with the current firing mode
    pub fn can_fire(&self) -> bool {
        self.ammo > 0
    }

    // Can the weapon be reloaded with a given amount of bullets
    pub fn reload(&mut self, ammo: u8) -> bool {
        let can_reload = ammo > 0 && (self.tag.capacity() - self.ammo) >= ammo;

        if can_reload {
            self.ammo += ammo
        }

        can_reload
    }

    // The corresponding item to the weapon
    pub fn to_item(&self) -> Item {
        match self.tag {
            WeaponType::Rifle => Item::Rifle(self.ammo),
            WeaponType::MachineGun => Item::MachineGun(self.ammo),
            WeaponType::PlasmaRifle => Item::PlasmaRifle(self.ammo)
        }
    }
}