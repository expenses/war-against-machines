//! The different items in the game

/// The type of an item
#[derive(Copy, Clone)]
pub enum ItemType {
    _Scrap,
    _Weapon,
    SquaddieCorpse,
    MachineCorpse,
    Skeleton
}

/// An item with a weight value
pub struct Item {
    pub tag: ItemType,
    pub weight: usize,
    pub image: String
}

impl Item {
    /// Create a new item
    pub fn new(tag: ItemType) -> Item {
        let (weight, image) = match tag {
            ItemType::_Scrap => (5, "scrap"),
            ItemType::_Weapon => (4, "weapon"),
            ItemType::SquaddieCorpse => (6, "squaddie_corpse"),
            ItemType::MachineCorpse => (8, "machine_corpse"),
            ItemType::Skeleton => (4, "skeleton")
        };

        Item {
            tag, weight,
            image: image.into()
        }
    }
}