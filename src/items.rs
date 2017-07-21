// The different items in the game

use std::fmt;

use resources::Image;

// The type of an item
#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum Item {
    Scrap,
    Rifle,
    MachineGun,
    PlasmaRifle,
    SquaddieCorpse,
    MachineCorpse,
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - weight {}", match *self {
            Item::Scrap          => "Scrap",
            Item::Rifle          => "Rifle",
            Item::MachineGun     => "Machine Gun",
            Item::PlasmaRifle    => "Plasma Rifle",
            Item::SquaddieCorpse => "Squaddie Corpse",
            Item::MachineCorpse  =>  "Machine Corpse",
        }, self.weight())
    }
}

impl Item {
    // Get the item's weight
    pub fn weight(&self) -> f32 {
        match *self {
            Item::Scrap          => 5.0,
            Item::Rifle          => 3.0,
            Item::MachineGun     => 3.5,
            Item::PlasmaRifle    => 3.75,
            Item::SquaddieCorpse => 6.0,
            Item::MachineCorpse  => 8.0,
        }
    }

    // Get the item's image
    pub fn image(&self) -> Image {
        match *self {
            Item::Scrap => Image::Scrap,
            Item::SquaddieCorpse => Image::SquaddieCorpse,
            Item::MachineCorpse => Image::MachineCorpse,
            _ => Image::Weapon
        }
    }
}