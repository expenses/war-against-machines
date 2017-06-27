//! The different items in the game

use std::fmt;

/// The type of an item
#[derive(Copy, Clone)]
pub enum ItemType {
    Scrap,
    Weapon,
    SquaddieCorpse,
    MachineCorpse,
    Skeleton
}

/// An item with a weight value
pub struct Item {
    pub tag: ItemType,
    pub weight: usize,
    pub image: &'static str
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - weight {}", match self.tag {
            ItemType::Scrap => "Scrap",
            ItemType::Weapon => "Weapon",
            ItemType::SquaddieCorpse => "Squaddie Corpse",
            ItemType::MachineCorpse => "Machine Corpse",
            ItemType::Skeleton => "Skeleton"
        }, self.weight)
    }
}

impl Item {
    /// Create a new item
    pub fn new(tag: ItemType) -> Item {
        let (weight, image) = match tag {
            ItemType::Scrap => (5, "scrap"),
            ItemType::Weapon => (4, "weapon"),
            ItemType::SquaddieCorpse => (6, "squaddie_corpse"),
            ItemType::MachineCorpse => (8, "machine_corpse"),
            ItemType::Skeleton => (4, "skeleton")
        };

        Item {
            tag, weight, image
        }
    }
}