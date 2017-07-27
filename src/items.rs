// The different items in the game

use std::fmt;

use resources::Image;
use weapons::WeaponType;

pub const BANDAGE_HEAL: i16 = 25;

// The type of an item
#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum Item {
    Scrap,
    Bandages,
    Rifle(u8),
    MachineGun(u8),
    PlasmaRifle(u8),
    RifleClip(u8),
    MachineGunClip(u8),
    PlasmaClip(u8),
    SquaddieCorpse,
    MachineCorpse,
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - {} kg", match *self {
            Item::Scrap                => "Scrap".into(),
            Item::Bandages             => "Bandages".into(),
            Item::Rifle(ammo)          => format!("Rifle ({}/{})", ammo, self.capacity()),
            Item::MachineGun(ammo)     => format!("Machine Gun ({}/{})", ammo, self.capacity()),
            Item::PlasmaRifle(ammo)    => format!("Plasma Rifle ({}/{})", ammo, self.capacity()),
            Item::RifleClip(ammo)      => format!("Rifle Clip ({}/{})", ammo, self.capacity()),
            Item::MachineGunClip(ammo) => format!("Machine Gun Clip ({}/{})", ammo, self.capacity()),
            Item::PlasmaClip(ammo)     => format!("Plasma Clip ({}/{})", ammo, self.capacity()),
            Item::SquaddieCorpse       => "Squaddie Corpse".into(),
            Item::MachineCorpse        => "Machine Corpse".into(),
        }, self.weight())
    }
}

impl Item {
    // Get the item's weight
    pub fn weight(&self) -> f32 {
        match *self {
            Item::Scrap          => 5.0,
            Item::Bandages       => 1.0,
            Item::Rifle(_)       => 4.0,
            Item::MachineGun(_)  => 6.0,
            Item::PlasmaRifle(_) => 5.5,
            Item::SquaddieCorpse => 60.0,
            Item::MachineCorpse  => 150.0,
            Item::RifleClip(_) | Item::MachineGunClip(_) | Item::PlasmaClip(_) => 0.5,
        }
    }

    // Get the item's image
    pub fn image(&self) -> Image {
        match *self {
            Item::Scrap => Image::Scrap,
            Item::Bandages => Image::Bandages,
            Item::SquaddieCorpse => Image::SquaddieCorpse,
            Item::MachineCorpse => Image::MachineCorpse,
            Item::RifleClip(_) | Item::MachineGunClip(_) | Item::PlasmaClip(_) => Image::AmmoClip,
            _ => Image::Weapon
        }
    }

    // The item's bullet capacity (if it has one)
    fn capacity(&self) -> u8 {
        match *self {
            Item::Rifle(_) | Item::RifleClip(_) => WeaponType::Rifle.capacity(),
            Item::MachineGun(_) | Item::MachineGunClip(_) => WeaponType::MachineGun.capacity(),
            Item::PlasmaRifle(_) | Item::PlasmaClip(_) => WeaponType::PlasmaRifle.capacity(),
            _ => 0
        }
    }
}