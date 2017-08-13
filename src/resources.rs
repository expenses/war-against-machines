// Image and text resources

use glutin::VirtualKeyCode;

const TILE: f32 = 48.0;
const FONT_Y: f32 = TILE * 8.5;
pub const FONT_HEIGHT: f32 = 8.0;

// include_bytes! but prepends the resources directory
macro_rules! bytes {
    ($file: expr) => (
        include_bytes!(concat!("../resources/", $file))
    )
}

// Scale up a tile position for the 48 by 48 tileset
macro_rules! tiles {
    ($x: expr, $y: expr, $width: expr, $height: expr) => (
        [$x as f32 * TILE, $y as f32 * TILE, $width as f32 * TILE, $height as f32 * TILE]
    )
}

// Simplify writing the coordinates for a character in the tileset
macro_rules! char_loc {
    ($x: expr, $width: expr) => (
        [$x as f32, FONT_Y, $width as f32, FONT_HEIGHT]
    )
}

// A trait for mapping an image to its position in the tileset
pub trait ImageSource {
    fn source(&self) -> [f32; 4];
    fn width(&self) -> f32;
    fn height(&self) -> f32;
}

// An image in the tileset
#[derive(Serialize, Deserialize, Copy, Clone, Debug, Eq, PartialEq)]
pub enum Image {
    Base1,
    Base2,
    
    ObjectRebar,
    ObjectRubble,

    Squaddie,
    Machine,

    Ruin1Left,
    Ruin1Top,
    Ruin2Left,
    Ruin2Top,

    PitTop,
    PitLeft,
    PitRight,
    PitBottom,
    PitCenter,
    PitTL,
    PitTR,
    PitBL,
    PitBR,

    PlasmaBullet,
    RegularBullet,

    SquaddieCorpse,
    MachineCorpse,
    Scrap,
    Weapon,
    AmmoClip,
    Bandages,

    Cursor,
    CursorCrosshair,
    Path,

    EndTurnButton,
    InventoryButton,
    SaveGameButton,

    LeftEdge,
    RightEdge,
    Skeleton,
    SkeletonCracked,
    Rubble,

    Title
}

impl ImageSource for Image {
    // Map the image to it's location in the tileset
    fn source(&self) -> [f32; 4] {
        match *self {
            Image::Base1 => tiles!(0, 0, 1, 1),
            Image::Base2 => tiles!(1, 0, 1, 1),
            
            Image::ObjectRebar => tiles!(0, 1, 1, 1),
            Image::ObjectRubble => tiles!(1, 1, 1, 1),

            Image::Squaddie => tiles!(0, 2, 1, 1),
            Image::Machine => tiles!(1, 2, 1, 1),

            Image::Ruin1Left => tiles!(2, 2, 1, 1),
            Image::Ruin1Top => tiles!(3, 2, 1, 1),
            Image::Ruin2Left => tiles!(4, 2, 1, 1),
            Image::Ruin2Top => tiles!(5, 2, 1, 1),

            Image::PitTop => tiles!(2, 0, 1, 1),
            Image::PitLeft => tiles!(3, 0, 1, 1),
            Image::PitRight => tiles!(4, 0, 1, 1),
            Image::PitBottom => tiles!(5, 0, 1, 1),
            Image::PitCenter => tiles!(6, 0, 1, 1),
            Image::PitTL => tiles!(2, 1, 1, 1),
            Image::PitTR => tiles!(3, 1, 1, 1),
            Image::PitBL => tiles!(4, 1, 1, 1),
            Image::PitBR => tiles!(5, 1, 1, 1),

            Image::RegularBullet => tiles!(0, 3, 1, 1),
            Image::PlasmaBullet => tiles!(1, 3, 1, 1),

            Image::SquaddieCorpse => tiles!(0, 4, 1, 1),
            Image::MachineCorpse => tiles!(1, 4, 1, 1),
            Image::Scrap => tiles!(2, 4, 1, 1),
            Image::Weapon => tiles!(3, 4, 1, 1),
            Image::AmmoClip => tiles!(4, 4, 1, 1),
            Image::Bandages => tiles!(5, 4, 1, 1),

            Image::Cursor => tiles!(0, 5, 1, 1),
            Image::CursorCrosshair => tiles!(1, 5, 1, 1),
            Image::Path => tiles!(2, 5, 1, 1),

            Image::LeftEdge => tiles!(0, 6, 1, 1),
            Image::RightEdge => tiles!(1, 6, 1, 1),
            Image::Skeleton => tiles!(2, 6, 1, 1),
            Image::SkeletonCracked => tiles!(3, 6, 1, 1),
            Image::Rubble => tiles!(4, 6, 1, 1),

            Image::Title => tiles!(0, 7, 10, 1),
            
            Image::EndTurnButton => tiles!(0, 8, 1, 0.5),
            Image::InventoryButton => tiles!(1, 8, 1, 0.5),
            Image::SaveGameButton => tiles!(2, 8, 1, 0.5),
        }
    }

    fn width(&self) -> f32 {
        self.source()[2]
    }

    fn height(&self) -> f32 {
        self.source()[3]
    }
}

impl ImageSource for char {
    // Map a character to its position in the tileset (oh boy...)
    fn source(&self) -> [f32; 4] {
        match *self {
            'A' => char_loc!(0, 4),
            'B' => char_loc!(5, 4),
            'C' => char_loc!(10, 3),
            'D' => char_loc!(14, 4),
            'E' => char_loc!(19, 3),
            'F' => char_loc!(23, 3),
            'G' => char_loc!(27, 4),
            'H' => char_loc!(32, 4),
            'I' => char_loc!(37, 3),
            'J' => char_loc!(41, 4),
            'K' => char_loc!(46, 4),
            'L' => char_loc!(51, 3),
            'M' => char_loc!(55, 5),
            'N' => char_loc!(61, 4),
            'O' => char_loc!(66, 4),
            'P' => char_loc!(71, 4),
            'Q' => char_loc!(76, 4),
            'R' => char_loc!(81, 4),
            'S' => char_loc!(86, 4),
            'T' => char_loc!(91, 3),
            'U' => char_loc!(95, 4),
            'V' => char_loc!(100, 4),
            'W' => char_loc!(105, 5),
            'X' => char_loc!(111, 4),
            'Y' => char_loc!(116, 4),
            'Z' => char_loc!(121, 3),
            'a' => char_loc!(125, 4),
            'b' => char_loc!(130, 4),
            'c' => char_loc!(135, 3),
            'd' => char_loc!(139, 4),
            'e' => char_loc!(144, 4),
            'f' => char_loc!(149, 3),
            'g' => char_loc!(153, 4),
            'h' => char_loc!(158, 4),
            'i' => char_loc!(163, 1),
            'j' => char_loc!(165, 2),
            'k' => char_loc!(168, 4),
            'l' => char_loc!(173, 1),
            'm' => char_loc!(175, 5),
            'n' => char_loc!(181, 4),
            'o' => char_loc!(186, 4),
            'p' => char_loc!(191, 4),
            'q' => char_loc!(196, 4),
            'r' => char_loc!(201, 3),
            's' => char_loc!(205, 4),
            't' => char_loc!(210, 3),
            'u' => char_loc!(214, 4),
            'v' => char_loc!(219, 4),
            'w' => char_loc!(224, 5),
            'x' => char_loc!(230, 3),
            'y' => char_loc!(234, 4),
            'z' => char_loc!(239, 4),
            '1' => char_loc!(244, 2),
            '2' => char_loc!(247, 4),
            '3' => char_loc!(252, 4),
            '4' => char_loc!(257, 4),
            '5' => char_loc!(262, 4),
            '6' => char_loc!(267, 4),
            '7' => char_loc!(272, 4),
            '8' => char_loc!(277, 4),
            '9' => char_loc!(282, 4),
            '0' => char_loc!(287, 4),
            '>' => char_loc!(292, 2),
            '(' => char_loc!(295, 2),
            ')' => char_loc!(298, 2),
            '-' => char_loc!(301, 3),
            ':' => char_loc!(305, 1),
            ',' => char_loc!(307, 2),
            '.' => char_loc!(310, 1),
            '%' => char_loc!(312, 3),
            '!' => char_loc!(316, 1),
            '/' => char_loc!(318, 3),
            '\'' => char_loc!(322, 1),
            ' ' => char_loc!(324, 4),
            _ => char_loc!(329, 4),
        }
    }
    
    // get the width of the character
    fn width(&self) -> f32 {
        self.source()[2]
    }

    // Get the height of the character
    fn height(&self) -> f32 {
        self.source()[3]
    }
}

// A sound effect
pub enum SoundEffect {
    Walk,
    RegularShot,
    PlasmaShot,
}

pub trait ToChar {
    fn to_char(&self) -> char;
}

impl ToChar for VirtualKeyCode {
    fn to_char(&self) -> char {
        match *self {
            VirtualKeyCode::A => 'a',
            VirtualKeyCode::B => 'b',
            VirtualKeyCode::C => 'c',
            VirtualKeyCode::D => 'd',
            VirtualKeyCode::E => 'e',
            VirtualKeyCode::F => 'f',
            VirtualKeyCode::G => 'g',
            VirtualKeyCode::H => 'h',
            VirtualKeyCode::I => 'i',
            VirtualKeyCode::J => 'j',
            VirtualKeyCode::K => 'k',
            VirtualKeyCode::L => 'l',
            VirtualKeyCode::M => 'm',
            VirtualKeyCode::N => 'n',
            VirtualKeyCode::O => 'o',
            VirtualKeyCode::P => 'p',
            VirtualKeyCode::Q => 'q',
            VirtualKeyCode::R => 'r',
            VirtualKeyCode::S => 's',
            VirtualKeyCode::T => 't',
            VirtualKeyCode::U => 'u',
            VirtualKeyCode::V => 'v',
            VirtualKeyCode::W => 'w',
            VirtualKeyCode::X => 'x',
            VirtualKeyCode::Y => 'y',
            VirtualKeyCode::Z => 'z',
            VirtualKeyCode::Key1 => '1',
            VirtualKeyCode::Key2 => '2',
            VirtualKeyCode::Key3 => '3',
            VirtualKeyCode::Key4 => '4',
            VirtualKeyCode::Key5 => '5',
            VirtualKeyCode::Key6 => '6',
            VirtualKeyCode::Key7 => '7',
            VirtualKeyCode::Key8 => '8',
            VirtualKeyCode::Key9 => '9',
            VirtualKeyCode::Key0 => '0',            
            VirtualKeyCode::Minus => '-',
            VirtualKeyCode::Period => '.',
            VirtualKeyCode::Space => ' ',
            _ => '�'
        }
    }
}