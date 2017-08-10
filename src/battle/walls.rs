use resources::Image;

// Walls in-between tiles

// The type of wall
#[derive(Serialize, Deserialize)]
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
}

#[derive(Serialize, Deserialize)]
pub struct Wall {
    pub tag: WallType,
}

impl Wall {
    pub fn new(tag: WallType) -> Wall {
        Wall {
            tag
        }
    }
}

#[derive(Serialize, Deserialize)]
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