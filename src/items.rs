// The different items in the game

use std::fmt;

use resources::Image;

// The type of an item
#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum Item {
    Scrap,
    Weapon,
    SquaddieCorpse,
    MachineCorpse,
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - weight {}", match *self {
            Item::Scrap =>          "Scrap",
            Item::Weapon =>         "Weapon",
            Item::SquaddieCorpse => "Squaddie Corpse",
            Item::MachineCorpse =>  "Machine Corpse",
        }, self.weight())
    }
}

impl Item {
    pub fn weight(&self) -> u8 {
        match *self {
            Item::Scrap =>          5,
            Item::Weapon =>         4,
            Item::SquaddieCorpse => 6,
            Item::MachineCorpse =>  8,
        }
    }

    pub fn image(&self) -> Image {
        match *self {
            Item::Scrap => Image::Scrap,
            Item::Weapon => Image::Weapon,
            Item::SquaddieCorpse => Image::SquaddieCorpse,
            Item::MachineCorpse => Image::MachineCorpse
        }
    }
}