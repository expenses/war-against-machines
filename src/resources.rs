use image;
use opengl_graphics::{Texture, TextureSettings, Filter, GlGraphics};
use opengl_graphics::glyph_cache::GlyphCache;
use graphics::character::CharacterCache;
use graphics::color::gamma_srgb_to_linear;
use graphics::types::Color;
use graphics::math::Matrix2d;
use graphics::image::Image;
use graphics::draw_state::{DrawState, Blend};
use graphics::text::Text;
use graphics::Transformed;

use rodio;
use rodio::Source;

use std::io::Cursor;
use std::rc::Rc;

const TILE: f64 = 48.0;
const DRAW_STATE: DrawState = DrawState {
    blend: Some(Blend::Alpha),
    stencil: None,
    scissor: None
};

macro_rules! bytes {
    ($file: expr) => (
        include_bytes!(concat!("../resources/", $file))
    )
}

macro_rules! rect {
    ($x: expr, $y: expr, $width: expr, $height: expr) => (
        [TILE * $x as f64, TILE * $y as f64, TILE * $width as f64, TILE * $height as f64]
    )
}

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

impl SetImage {
    fn source(&self) -> [f64; 4] {
        match *self {
            SetImage::Base1 => rect!(0, 0, 1, 1),
            SetImage::Base2 => rect!(1, 0, 1, 1),
            SetImage::Fog => rect!(2, 0, 1, 1),
            
            SetImage::Ruin1 => rect!(0, 1, 1, 1),
            SetImage::Ruin2 => rect!(1, 1, 1, 1),
            SetImage::Ruin3 => rect!(2, 1, 1, 1),

            SetImage::Squaddie => rect!(0, 2, 1, 1),
            SetImage::Machine => rect!(1, 2, 1, 1),

            SetImage::PitTop => rect!(3, 0, 1, 1),
            SetImage::PitLeft => rect!(4, 0, 1, 1),
            SetImage::PitRight => rect!(5, 0, 1, 1),
            SetImage::PitBottom => rect!(6, 0, 1, 1),
            SetImage::PitCenter => rect!(7, 0, 1, 1),
            SetImage::PitTL => rect!(3, 1, 1, 1),
            SetImage::PitTR => rect!(4, 1, 1, 1),
            SetImage::PitBL => rect!(5, 1, 1, 1),
            SetImage::PitBR => rect!(6, 1, 1, 1),

            SetImage::PlasmaBullet => rect!(0, 3, 1, 1),
            SetImage::RegularBullet => rect!(1, 3, 1, 1),

            SetImage::SquaddieCorpse => rect!(0, 4, 1, 1),
            SetImage::MachineCorpse => rect!(1, 4, 1, 1),
            SetImage::Skeleton => rect!(2, 4, 1, 1),
            SetImage::Scrap => rect!(3, 4, 1, 1),
            SetImage::Weapon => rect!(4, 4, 1, 1),

            SetImage::Cursor => rect!(0, 5, 1, 1),
            SetImage::CursorUnit => rect!(1, 5, 1, 1),
            SetImage::CursorUnwalkable => rect!(2, 5, 1, 1),
            SetImage::CursorCrosshair => rect!(3, 5, 1, 1),

            SetImage::Path => rect!(0, 6, 1, 1),
            SetImage::PathCannotFire => rect!(1, 6, 1, 1),
            SetImage::PathUnreachable => rect!(2, 6, 1, 1),

            SetImage::LeftEdge => rect!(0, 7, 1, 1),
            SetImage::RightEdge => rect!(1, 7, 1, 1),

            SetImage::Title => rect!(0, 8, 10, 1),
            
            SetImage::EndTurnButton => rect!(0, 9, 1, 0.5),
            SetImage::InventoryButton => rect!(0, 9.5, 1, 0.5),
            SetImage::ChangeFireModeButton => rect!(1, 9, 1, 0.5)
        }
    }

    pub fn width(&self) -> f64 {
        self.source()[2]
    }

    pub fn height(&self) -> f64 {
        self.source()[3]
    }
}

fn load_texture(bytes: &[u8], texture_settings: &TextureSettings) -> Texture {
    let mut image = image::load_from_memory(bytes).unwrap().to_rgba();

    for pixel in image.pixels_mut() {
        let converted = gamma_srgb_to_linear([
            pixel[0] as f32 / 255.0,
            pixel[1] as f32 / 255.0,
            pixel[2] as f32 / 255.0,
            pixel[3] as f32 / 255.0
        ]);

        pixel.data = [
            (converted[0] * 255.0) as u8,
            (converted[1] * 255.0) as u8,
            (converted[2] * 255.0) as u8,
            (converted[3] * 255.0) as u8,
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

pub enum SoundEffect {
    Plasma,
    Walk,
}

// A struct to hold resources for the game such as images and fonts
pub struct Resources {
    tileset: Texture,
    font: GlyphCache<'static>,
    font_size: u32,
    audio: [Audio; 2]
}

impl Resources {
    // Create a new resource struct with a texture creator, font context and directory string
    pub fn new(tileset: &[u8], font: &'static [u8], font_size: u32, audio: [&[u8]; 2]) -> Resources {  
        let tileset = load_texture(tileset, &TextureSettings::new().filter(Filter::Nearest));

        Resources {
            tileset, font_size,
            font: GlyphCache::from_bytes(font).unwrap(),
            audio: [load_audio(audio[0]), load_audio(audio[1])]
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
        self.font.width(self.font_size, string)
    }

    // Get the height of the font
    pub fn font_height(&self) -> f64 {
        self.font_size as f64
    }

    // Render a string of text with a colour and transformation
    pub fn render_text(&mut self, string: &str, colour: Color, transform: Matrix2d, gl: &mut GlGraphics) {
        Text::new_color(colour, self.font_size)
            .draw(string, &mut self.font, &DRAW_STATE, transform, gl);
    }

    pub fn play_sound(&self, sound: SoundEffect) {
        let endpoint = rodio::get_default_endpoint().unwrap();

        let sound = match sound {
            SoundEffect::Plasma => self.audio[0].as_ref(),
            SoundEffect::Walk => self.audio[1].as_ref()
        };

        // Clone the reference and wrap it in a cursor
        let cursor = Cursor::new(sound.clone());
        // Decode the audio
        let decoder = rodio::Decoder::new(cursor).unwrap();
        // Play it!
        rodio::play_raw(&endpoint, decoder.convert_samples());
    }
}