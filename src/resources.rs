use sdl2::render::{Texture, TextureCreator};
use sdl2::video::WindowContext;
use sdl2::ttf::{Sdl2TtfContext, Font};
use sdl2::pixels::{PixelFormatEnum, Color};
use sdl2::rwops::RWops;
use sdl2::image::ImageRWops;
use sdl2::mixer::{Chunk, LoaderRWops};
use sdl2::mixer::Channel;

use std::collections::HashMap;

// Load a resource into a SDL2 RWops struct at compile time
macro_rules! rw_ops {
    ($file:expr) => (
        RWops::from_bytes(include_bytes!(concat!("../resources/", $file))).unwrap()
    )
}

// A struct to hold resources for the game such as images and fonts
pub struct Resources<'a> {
    texture_creator: &'a TextureCreator<WindowContext>,
    images: HashMap<&'static str, Texture<'a>>,
    font_context: &'a Sdl2TtfContext,
    fonts: HashMap<&'static str, Font<'a, 'a>>,
    audio: HashMap<&'static str, Chunk>
}

impl<'a> Resources<'a> {
    // Create a new resource struct with a texture creator, font context and directory string
    pub fn new(texture_creator: &'a TextureCreator<WindowContext>,
           font_context: &'a Sdl2TtfContext) -> Resources<'a> {        
        Resources {
            texture_creator,
            images: HashMap::new(),
            font_context,
            fonts: HashMap::new(),
            audio: HashMap::new()
        }
    }

    // Load an image into the images hashmap from a RWops of a png
    pub fn load_image(&mut self, name: &'static str, rw_ops: RWops) {
        self.images.insert(name, self.texture_creator.create_texture_from_surface(
            rw_ops.load_png().unwrap()
        ).unwrap());
    }

    // Get an image from the hashmap or panic
    pub fn image(&self, name: &str) -> &Texture {
        self.images.get(name).expect(&format!("Image '{}' could not be found.", name))
    }

    // Create a new texture using the texture creator
    pub fn create_texture(&self, width: u32, height: u32) -> Texture {
        self.texture_creator.create_texture_target(PixelFormatEnum::ARGB8888, width, height).unwrap()
    }

    // Load a font into the fonts hashmap from a RWops of a font
    pub fn load_font(&mut self, name: &'static str, rw_ops: RWops<'a>, size: u16) {
        self.fonts.insert(name, self.font_context.load_font_from_rwops(rw_ops, size).unwrap());
    }

    // Render a string of text using a font
    pub fn render(&self, font: &str, text: &str, colour: Color) -> Texture {
        // Render the text into a surface in a solid colour
        let rendered = self.fonts[font].render(text).solid(colour).unwrap();

        // Create a texture from that surface
        self.texture_creator.create_texture_from_surface(rendered).unwrap()
    }

    pub fn load_audio(&mut self, name: &'static str, rw_ops: RWops) {
        self.audio.insert(name, rw_ops.load_wav().unwrap());
    }

    pub fn play_audio(&self, name: &str) {
        self.audio.get(name).and_then(|audio| Channel::all().play(audio, 0).ok()).unwrap();
    }
}