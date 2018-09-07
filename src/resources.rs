// Image and text resources

use *;

const TILE: f32 = 48.0;

// include_bytes! but prepends the resources directory
macro_rules! bytes {
    ($file: expr) => (
        include_bytes!(concat!("../resources/", $file))
    )
}

pub const TILESET: &[u8] = bytes!("tileset.png");

pub const AUDIO: [&[u8]; 3] = [
    bytes!("audio/walk.ogg"),
    bytes!("audio/regular_shot.ogg"),
    bytes!("audio/plasma_shot.ogg")
];

pub const FONT: &[u8] = bytes!("font/TinyUnicode.ttf");

// Scale up a tile position for the 48 by 48 tileset
macro_rules! tiles {
    ($x: expr, $y: expr, $width: expr, $height: expr) => (
        [$x as f32 * TILE, $y as f32 * TILE, $width as f32 * TILE, $height as f32 * TILE]
    )
}

// A trait for mapping an image to its position in the tileset
pub trait ImageSource {
    fn source(&self) -> [f32; 4];
     
    fn width(&self) -> f32 {
        self.source()[2]
    }

    fn height(&self) -> f32 {
        self.source()[3]
    }
}

// An image in the tileset
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
pub enum Image {
    Base1,
    Base2,
    
    ObjectRebar,
    ObjectRubble,

    SquaddieFront,
    SquaddieLeft,
    SquaddieBack,
    SquaddieRight,

    // todo: machine left and right
    MachineFront,
    MachineBack,

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
    Grenade,

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
    Crater,
    Explosion1,
    Explosion2,
    Explosion3,

    Title
}

impl ImageSource for Image {
    // Map the image to it's location in the tileset
    fn source(&self) -> [f32; 4] {
        match *self {
            Image::Base1 => tiles!(0, 0, 1, 1),
            Image::Base2 => tiles!(1, 0, 1, 1),
            
            Image::PitTop => tiles!(2, 0, 1, 1),
            Image::PitLeft => tiles!(3, 0, 1, 1),
            Image::PitRight => tiles!(4, 0, 1, 1),
            Image::PitBottom => tiles!(5, 0, 1, 1),
            Image::PitCenter => tiles!(6, 0, 1, 1),

            Image::ObjectRebar => tiles!(0, 1, 1, 1),
            Image::ObjectRubble => tiles!(1, 1, 1, 1),

            Image::PitTL => tiles!(2, 1, 1, 1),
            Image::PitTR => tiles!(3, 1, 1, 1),
            Image::PitBL => tiles!(4, 1, 1, 1),
            Image::PitBR => tiles!(5, 1, 1, 1),

            Image::SquaddieFront => tiles!(0, 2, 1, 1),
            Image::SquaddieLeft => tiles!(1, 2, 1, 1),
            Image::SquaddieBack => tiles!(2, 2, 1, 1),
            Image::SquaddieRight => tiles!(3, 2, 1, 1),
            Image::MachineFront => tiles!(4, 2, 1, 1),
            Image::MachineBack => tiles!(5, 2, 1, 1),

            Image::Ruin1Left => tiles!(0, 3, 1, 1),
            Image::Ruin1Top => tiles!(1, 3, 1, 1),
            Image::Ruin2Left => tiles!(2, 3, 1, 1),
            Image::Ruin2Top => tiles!(3, 3, 1, 1),
            
            Image::RegularBullet => tiles!(0, 4, 1, 1),
            Image::PlasmaBullet => tiles!(1, 4, 1, 1),

            Image::SquaddieCorpse => tiles!(0, 5, 1, 1),
            Image::MachineCorpse => tiles!(1, 5, 1, 1),
            Image::Scrap => tiles!(2, 5, 1, 1),
            Image::Weapon => tiles!(3, 5, 1, 1),
            Image::AmmoClip => tiles!(4, 5, 1, 1),
            Image::Bandages => tiles!(5, 5, 1, 1),
            Image::Grenade => tiles!(6, 5, 1, 1),

            Image::Cursor => tiles!(0, 6, 1, 1),
            Image::CursorCrosshair => tiles!(1, 6, 1, 1),
            Image::Path => tiles!(2, 6, 1, 1),

            Image::LeftEdge => tiles!(0, 7, 1, 1),
            Image::RightEdge => tiles!(1, 7, 1, 1),
            Image::Skeleton => tiles!(2, 7, 1, 1),
            Image::SkeletonCracked => tiles!(3, 7, 1, 1),
            Image::Rubble => tiles!(4, 7, 1, 1),
            Image::Crater => tiles!(5, 7, 1, 1),
            Image::Explosion1 => tiles!(6, 7, 1, 1),
            Image::Explosion2 => tiles!(7, 7, 1, 1),
            Image::Explosion3 => tiles!(8, 7, 1, 1),

            Image::Title => tiles!(0, 8, 10, 1),
            
            Image::EndTurnButton => tiles!(0, 9, 1, 0.5),
            Image::InventoryButton => tiles!(1, 9, 1, 0.5),
            Image::SaveGameButton => tiles!(2, 9, 1, 0.5),
        }
    }
}

// A sound effect
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
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
            VirtualKeyCode::Semicolon => ':',
            _ => '�'
        }
    }
}
