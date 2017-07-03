use std::collections::HashMap;

use image;
use opengl_graphics::{Texture, TextureSettings, Filter};
use opengl_graphics::glyph_cache::GlyphCache;
use graphics::color::gamma_srgb_to_linear;

fn load_texture(filename: &str, texture_settings: &TextureSettings) -> Texture {
    let mut image = image::open(filename).unwrap().to_rgba();

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

// A struct to hold resources for the game such as images and fonts
pub struct Resources {
    tileset: Texture,
    texture_settings: TextureSettings,
    images: HashMap<&'static str, Texture>,
    pub font: GlyphCache<'static>
}

impl Resources {
    // Create a new resource struct with a texture creator, font context and directory string
    pub fn new(tileset: &str, font: &str) -> Resources {  
        let texture_settings = TextureSettings::new().filter(Filter::Nearest);

        let tileset = load_texture(tileset, &texture_settings);

        Resources {
            tileset, texture_settings,
            images: HashMap::new(),
            font: GlyphCache::new(font).unwrap()
        }
    }

    // Load an image into the images hashmap from a RWops of a png
    pub fn load_image(&mut self, name: &'static str, filename: &str) {
        self.images.insert(name, load_texture(filename, &self.texture_settings));
    }

    // Get an image from the hashmap or panic
    pub fn image(&self, name: &str) -> &Texture {
        self.images.get(name).expect(&format!("Image '{}' could not be found.", name))
    }
}