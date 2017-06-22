// The type of an item
#[derive(Copy, Clone)]
pub enum ItemType {
    Scrap,
    Weapon,
    SquaddieCorpse,
    MachineCorpse
}

// An item with a weight value
pub struct Item {
    pub tag: ItemType,
    pub weight: usize,
    pub image: String
}

impl Item {
    // Create a new item
    pub fn new(tag: ItemType) -> Item {
        let (weight, image) = match tag {
            ItemType::Scrap => (5, "scrap"),
            ItemType::Weapon => (4, "weapon"),
            ItemType::SquaddieCorpse => (6, "squaddie_corpse"),
            ItemType::MachineCorpse => (8, "machine_corpse")
        };

        Item {
            tag, weight,
            image: image.into()
        }
    }
}