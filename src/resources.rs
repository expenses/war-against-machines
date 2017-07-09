use image;
use image::ImageFormat;
use opengl_graphics::{Texture, TextureSettings, Filter, GlGraphics};
use graphics::color::gamma_srgb_to_linear;
use graphics::math::Matrix2d;
use graphics::image::Image;
use graphics::draw_state::{DrawState, Blend};
use graphics::Transformed;
use rodio;
use rodio::{Source, Decoder};
use settings::Settings;
use traits::Dimensions;

use std::io::Cursor;
use std::rc::Rc;

const TILE: f64 = 48.0;
const FONT_Y: f64 = TILE * 9.5;
const FONT_HEIGHT: f64 = 8.0;
const CHARACTER_GAP: f64 = 1.0;
const DRAW_STATE: DrawState = DrawState {
    blend: Some(Blend::Alpha),
    stencil: None,
    scissor: None
};

// include_bytes! but prepends the resources directory
macro_rules! bytes {
    ($file: expr) => (
        include_bytes!(concat!("../resources/", $file))
    )
}

// Scale up a tile position for the 48 by 48 tileset
macro_rules! tiles {
    ($x: expr, $y: expr, $width: expr, $height: expr) => (
        [TILE * $x as f64, TILE * $y as f64, TILE * $width as f64, TILE * $height as f64]
    )
}

// Simplify writing the coordinates for a character in the tileset
macro_rules! char_loc {
    ($x: expr, $width: expr) => (
        [$x as f64, FONT_Y, $width as f64, FONT_HEIGHT]
    )
}

// A trait for mapping an image to its position in the tileset
trait ImageSource {
    fn source(&self) -> [f64; 4];
}

// An image in the tileset
#[derive(Serialize, Deserialize, Copy, Clone)]
pub enum SetImage {
    Base1,
    Base2,
    Fog,
    
    Ruin1,
    Ruin2,
    Ruin3,

    Squaddie,
    Machine,

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
    Skeleton,
    Scrap,
    Weapon,

    Cursor,
    CursorUnit,
    CursorUnwalkable,
    CursorCrosshair,

    Path,
    PathCannotFire,
    PathUnreachable,

    EndTurnButton,
    InventoryButton,
    ChangeFireModeButton,

    LeftEdge,
    RightEdge,

    Title
}

impl ImageSource for SetImage {
    // Map the image to it's location in the tileset
    fn source(&self) -> [f64; 4] {
        match *self {
            SetImage::Base1 => tiles!(0, 0, 1, 1),
            SetImage::Base2 => tiles!(1, 0, 1, 1),
            SetImage::Fog => tiles!(2, 0, 1, 1),
            
            SetImage::Ruin1 => tiles!(0, 1, 1, 1),
            SetImage::Ruin2 => tiles!(1, 1, 1, 1),
            SetImage::Ruin3 => tiles!(2, 1, 1, 1),

            SetImage::Squaddie => tiles!(0, 2, 1, 1),
            SetImage::Machine => tiles!(1, 2, 1, 1),

            SetImage::PitTop => tiles!(3, 0, 1, 1),
            SetImage::PitLeft => tiles!(4, 0, 1, 1),
            SetImage::PitRight => tiles!(5, 0, 1, 1),
            SetImage::PitBottom => tiles!(6, 0, 1, 1),
            SetImage::PitCenter => tiles!(7, 0, 1, 1),
            SetImage::PitTL => tiles!(3, 1, 1, 1),
            SetImage::PitTR => tiles!(4, 1, 1, 1),
            SetImage::PitBL => tiles!(5, 1, 1, 1),
            SetImage::PitBR => tiles!(6, 1, 1, 1),

            SetImage::RegularBullet => tiles!(0, 3, 1, 1),
            SetImage::PlasmaBullet => tiles!(1, 3, 1, 1),

            SetImage::SquaddieCorpse => tiles!(0, 4, 1, 1),
            SetImage::MachineCorpse => tiles!(1, 4, 1, 1),
            SetImage::Skeleton => tiles!(2, 4, 1, 1),
            SetImage::Scrap => tiles!(3, 4, 1, 1),
            SetImage::Weapon => tiles!(4, 4, 1, 1),

            SetImage::Cursor => tiles!(0, 5, 1, 1),
            SetImage::CursorUnit => tiles!(1, 5, 1, 1),
            SetImage::CursorUnwalkable => tiles!(2, 5, 1, 1),
            SetImage::CursorCrosshair => tiles!(3, 5, 1, 1),

            SetImage::Path => tiles!(0, 6, 1, 1),
            SetImage::PathCannotFire => tiles!(1, 6, 1, 1),
            SetImage::PathUnreachable => tiles!(2, 6, 1, 1),

            SetImage::LeftEdge => tiles!(0, 7, 1, 1),
            SetImage::RightEdge => tiles!(1, 7, 1, 1),

            SetImage::Title => tiles!(0, 8, 10, 1),
            
            SetImage::EndTurnButton => tiles!(0, 9, 1, 0.5),
            SetImage::InventoryButton => tiles!(1, 9, 1, 0.5),
            SetImage::ChangeFireModeButton => tiles!(2, 9, 1, 0.5),
        }
    }
}

impl Dimensions for SetImage {
    // Get the width of the image
    fn width(&self) -> f64 {
        self.source()[2]
    }

    // Get the height of the image
    fn height(&self) -> f64 {
        self.source()[3]
    }
}

impl ImageSource for char {
    // Map a character to its position in the tileset (oh boy...)
    fn source(&self) -> [f64; 4] {
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
            'U' => char_loc!(95, 3),
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
            ' ' => char_loc!(316, 4),
            _ => char_loc!(321, 4),
        }
    }
}

impl Dimensions for char {
    // get the width of the character (with padding)
    fn width(&self) -> f64 {
        self.source()[2] + CHARACTER_GAP
    }

    // Get the height of the character
    fn height(&self) -> f64 {
        FONT_HEIGHT
    }
}

// Load a png image and perform the sRGB -> linear conversion
fn load_texture(bytes: &[u8], texture_settings: &TextureSettings) -> Texture {
    let mut image = image::load_from_memory_with_format(bytes, ImageFormat::PNG).unwrap().to_rgba();

    for pixel in image.pixels_mut() {
        let converted = gamma_srgb_to_linear([
            pixel[0] as f32 / 255.0,

            pixel[1] as f32 / 255.0,
            pixel[2] as f32 / 255.0,
            pixel[3] as f32 / 255.0
        ]);

        pixel.data = [
            (converted[0] * 255.0).round() as u8,
            (converted[1] * 255.0).round() as u8,
            (converted[2] * 255.0).round() as u8,
            (converted[3] * 255.0).round() as u8,
        ];
    }

    Texture::from_image(&image, texture_settings)
}

// Use reference-counting to avoid cloning the source each time
type Audio = Rc<Vec<u8>>;

// Load a piece of audio
fn load_audio(bytes: &[u8]) -> Audio {
    Rc::new(bytes.to_vec())
}

// A sound effect
pub enum SoundEffect {
    Walk,
    RegularShot,
    PlasmaShot,
}

// A struct to hold resources for the game such as images and fonts
pub struct Resources {
    tileset: Texture,
    font_size: f64,
    sounds: [Audio; 3],
    volume: u8
}

impl Resources {
    // Create the Resource with a tileset, font and audio
    pub fn new(tileset: &[u8], font_size: f64, sounds: [&[u8]; 3]) -> Resources { 
        let settings = TextureSettings::new().filter(Filter::Nearest);

        Resources {
            font_size,
            tileset: load_texture(tileset, &settings),
            sounds: [
                load_audio(sounds[0]),
                load_audio(sounds[1]),
                load_audio(sounds[2])
            ],
            volume: 100
        }
    }

    // Render an image
    pub fn render(&self, image: &SetImage, transform: Matrix2d, gl: &mut GlGraphics) {
        Image::new()
            .src_rect(image.source())
            .draw(&self.tileset, &DRAW_STATE, transform, gl);
    }

    // Render an image with a particular rotation
    pub fn render_with_rotation(&self, image: &SetImage, rotation: f64, transform: Matrix2d, gl: &mut GlGraphics) {
        // Get the center of the image
        let (center_x, center_y) = (image.width() / 2.0, image.height() / 2.0);
        // Calculate the radius of the containing circle of the image
        let radius = center_x.hypot(center_y);
        // Use offset of -45' (because the top left corner is the origin)
        let offset = -45_f64.to_radians();

        let transform = transform
            // Translate the image so that the center remains in the right place regardless of orientation
            .trans((rotation + offset).sin() * radius + center_x, (rotation + offset).cos() * -radius + center_y)
            .rot_rad(rotation);

        self.render(image, transform, gl);
    }

    // Get the width of a string of text rendered with the font
    pub fn font_width(&mut self, string: &str) -> f64 {
        string.chars().fold(0.0, |total, character| total + character.width() * self.font_size)
    }

    // Get the height of the font
    pub fn font_height(&self) -> f64 {
        FONT_HEIGHT * self.font_size
    }

    // Render a string of text with a colour and transformation
    pub fn render_text(&mut self, string: &str, transform: Matrix2d, gl: &mut GlGraphics) {
        let mut width = 0.0;

        for character in string.chars() {
            Image::new()
                .src_rect(character.source())
                .draw(&self.tileset, &DRAW_STATE, transform.scale(self.font_size, self.font_size).trans(width, 0.0), gl);
            width += character.width();
        }
    }

    // Set the volume
    pub fn set(&mut self, settings: &Settings) {
        self.volume = settings.volume;
    }

    // Play a sound effect
    pub fn play_sound(&self, sound: SoundEffect) {
        // Get the sound effect
        let sound = match sound {
            SoundEffect::Walk => self.sounds[0].as_ref(),
            SoundEffect::RegularShot => self.sounds[1].as_ref(),
            SoundEffect::PlasmaShot => self.sounds[2].as_ref()
        };

        // Clone the reference and wrap it in a cursor
        let cursor = Cursor::new(sound.clone());
        // Decode the audio
        let decoder = Decoder::new(cursor).unwrap();
        // Play it!
        let endpoint = rodio::get_default_endpoint().unwrap();        
        rodio::play_raw(&endpoint, decoder.convert_samples().amplify(self.volume as f32 / 100.0));
    }
}