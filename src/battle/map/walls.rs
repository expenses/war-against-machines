use resources::Image;

// Walls in-between tiles

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum WallSide {
    Left,
    Top
}

// The type of wall
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum WallType {
    Ruin1,
    Ruin2
}

impl WallType {
    // Get the left image
    pub fn left_image(&self) -> Image {
        match *self {
            WallType::Ruin1 => Image::Ruin1Left,
            WallType::Ruin2 => Image::Ruin2Left
        }
    }

    // Get the top image
    pub fn top_image(&self) -> Image {
        match *self {
            WallType::Ruin1 => Image::Ruin1Top,
            WallType::Ruin2 => Image::Ruin2Top
        }
    }
    
    // How much damage the wall can take before it breaks
    pub fn health(&self) -> i16 {
        50
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Wall {
    pub tag: WallType,
    pub health: i16
}

impl Wall {
    pub fn new(tag: WallType) -> Wall {
        Wall {
            health: tag.health(),
            tag
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Walls {
    pub left: Option<Wall>,
    pub top: Option<Wall>
}

impl Walls {
    pub fn new() -> Walls {
        Walls {
            left: None,
            top: None
        }
    }

    pub fn set_left(&mut self, tag: WallType) {
        if self.left.is_none() {
            self.left = Some(Wall::new(tag));
        }
    }

    pub fn set_top(&mut self, tag: WallType) {
        if self.top.is_none() {
            self.top = Some(Wall::new(tag));
        }
    }
}