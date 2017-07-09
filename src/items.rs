// The different items in the game

use std::fmt;

use resources::SetImage;

// The type of an item
#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum ItemType {
    Scrap,
    Weapon,
    SquaddieCorpse,
    MachineCorpse,
    Skeleton
}

// An item with a weight value and image
#[derive(Serialize, Deserialize)]
pub struct Item {
    pub tag: ItemType,
    pub weight: u8,
    pub image: SetImage
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - weight {}", match self.tag {
            ItemType::Scrap =>          "Scrap",
            ItemType::Weapon =>         "Weapon",
            ItemType::SquaddieCorpse => "Squaddie Corpse",
            ItemType::MachineCorpse =>  "Machine Corpse",
            ItemType::Skeleton =>       "Skeleton"
        }, self.weight)
    }
}

impl Item {
    // Create a new item
    pub fn new(tag: ItemType) -> Item {
        let (weight, image) = match tag {
            ItemType::Scrap =>          (5, SetImage::Scrap),
            ItemType::Weapon =>         (4, SetImage::Weapon),
            ItemType::SquaddieCorpse => (6, SetImage::SquaddieCorpse),
            ItemType::MachineCorpse =>  (8, SetImage::MachineCorpse),
            ItemType::Skeleton =>       (4, SetImage::Skeleton)
        };

        Item {
            tag, weight, image
        }
    }
}