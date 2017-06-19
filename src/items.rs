// The type of an item
pub enum ItemType {
    Scrap,
    Weapon
}

// An item with a weight value
pub struct Item {
    pub tag: ItemType,
    pub weight: usize
}

impl Item {
    // Create a new item
    pub fn new(tag: ItemType) -> Item {
        let weight = match tag {
            ItemType::Scrap => 5,
            ItemType::Weapon => 4
        };

        Item {
            tag, weight
        }
    }
}