// The different items in the game

use std::fmt;

use resources::Image;

// The type of an item
#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum ItemType {
    Scrap,
    Weapon,
    SquaddieCorpse,
    MachineCorpse,
}

// An item with a weight value and image
#[derive(Clone, Serialize, Deserialize)]
pub struct Item {
    pub tag: ItemType,
    pub weight: u8,
    pub image: Image
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - weight {}", match self.tag {
            ItemType::Scrap =>          "Scrap",
            ItemType::Weapon =>         "Weapon",
            ItemType::SquaddieCorpse => "Squaddie Corpse",
            ItemType::MachineCorpse =>  "Machine Corpse",
        }, self.weight)
    }
}

impl Item {
    // Create a new item
    pub fn new(tag: ItemType) -> Item {
        let (weight, image) = match tag {
            ItemType::Scrap =>          (5, Image::Scrap),
            ItemType::Weapon =>         (4, Image::Weapon),
            ItemType::SquaddieCorpse => (6, Image::SquaddieCorpse),
            ItemType::MachineCorpse =>  (8, Image::MachineCorpse),
        };

        Item {
            tag, weight, image
        }
    }
}