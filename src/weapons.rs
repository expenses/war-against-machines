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

const RIFLE_MODES: &[FiringMode] = &[FiringMode::SingleShot, FiringMode::AimedShot];
const MACHINE_GUN_MODES: &[FiringMode] = &[FiringMode::SingleShot, FiringMode::SemiAuto, FiringMode::FullAuto];
const PLASMA_RIFLE_MODES: &[FiringMode] = &[FiringMode::SingleShot, FiringMode::AimedShot, FiringMode::SemiAuto];

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

    pub fn modes(&self) -> &[FiringMode] {
        match *self {
            WeaponType::Rifle => RIFLE_MODES,
            WeaponType::MachineGun => MACHINE_GUN_MODES,
            WeaponType::PlasmaRifle => PLASMA_RIFLE_MODES
        }
    }

    pub fn base_cost(&self) -> u16 {
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

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum FiringMode {
    SingleShot,
    AimedShot,
    SemiAuto,
    FullAuto
}

impl fmt::Display for FiringMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            FiringMode::SingleShot => "Single Shot",
            FiringMode::AimedShot => "Aimed Shot",
            FiringMode::SemiAuto => "Semi Auto",
            FiringMode::FullAuto => "Full Auto"
        })
    }
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
    pub mode: usize,
    pub ammo: u8
}

impl fmt::Display for Weapon {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let info = self.info();

        write!(
            f, "{} ({}/{}) - {} (Hit modifier: {}, Cost: {}, Bullets: {}",
            self.tag, self.ammo, self.tag.capacity(), self.mode(), info.hit_modifier, info.cost, info.bullets
        )
    }
}

impl Weapon {
    // Create a new weapon based of the weapon type
    pub fn new(tag: WeaponType, ammo: u8) -> Weapon {
        Weapon {
            tag, ammo,
            mode: 0
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
        self.ammo >= self.info().bullets
    }

    // Can the weapon be reloaded with a given amount of bullets
    pub fn can_reload(&self, ammo: u8) -> bool {
        ammo > 0 && (self.tag.capacity() - self.ammo) >= ammo
    }

    // The current firing mode of the weapon
    fn mode(&self) -> FiringMode {
        self.tag.modes()[self.mode]
    }

    // Change the fire mode
    pub fn change_mode(&mut self) {
        self.mode = (self.mode + 1) % self.tag.modes().len()
    }

    // get the firing cost
    fn cost(&self, modifier: f32) -> u16 {
        (self.tag.base_cost() as f32 * modifier).ceil() as u16
    }

    // Get the hit modifier, the firing cost and the bullets fired
    pub fn info(&self) -> FiringModeInfo {
        match self.mode() {
            FiringMode::SingleShot => FiringModeInfo {
                hit_modifier: 1.0, 
                cost: self.cost(1.0),
                bullets: 1
            },
            FiringMode::AimedShot  => FiringModeInfo {
                hit_modifier: 1.5,
                cost: self.cost(2.0),
                bullets: 1
            },
            FiringMode::SemiAuto   => FiringModeInfo {
                hit_modifier: 0.75,
                cost: self.cost(1.5),
                bullets: 3
            },
            FiringMode::FullAuto   => FiringModeInfo {
                hit_modifier: 0.66,
                cost: self.cost(2.5),
                bullets: 6
            }
        }
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