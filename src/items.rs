pub enum ItemType {
    Scrap,
    Weapon
}

pub struct Item {
    pub tag: ItemType,
    pub weight: usize
}

impl Item {
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