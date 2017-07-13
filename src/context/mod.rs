use glutin::EventsLoop;
use rodio;
use rodio::{Decoder, Source};

use std::rc::Rc;
use std::io::Cursor;

mod renderer;

use colours;
use settings::Settings;
use resources::{ImageSource, Image, SoundEffect, FONT_HEIGHT};
use self::renderer::{Renderer, Properties}; 

// The size of a tile
const CHARACTER_GAP: f32 = 1.0;

// Use reference-counting to avoid cloning the source each time
type Audio = Rc<Vec<u8>>;

fn load_audio(bytes: &[u8]) -> Audio {
    Rc::new(bytes.to_vec())
}

pub struct Context {
    renderer: Renderer,
    volume: u8,
    audio: [Audio; 3],
    pub width: f32,
    pub height: f32,
    pub ui_scale: f32,
}

impl Context {
    pub fn new(event_loop: &EventsLoop, title: String, width: u32, height: u32, tileset: &[u8], audio: [&[u8]; 3]) -> Context {
        let renderer = Renderer::new(event_loop, tileset, title, width, height);

        // Create the context!
        Context {
            renderer,
            width: width as f32,
            height: height as f32,
            ui_scale: 2.0,
            volume: 100,
            audio: [load_audio(audio[0]), load_audio(audio[1]), load_audio(audio[2])]
        }
    }

    // Resize the context
    pub fn resize(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
        // resize the renderer
        self.renderer.resize(width, height);
    }

    // Render text
    pub fn render_text(&mut self, string: &str, mut x: f32, y: f32, colour: [f32; 4]) {
        // Center the text on its width
        x -= self.font_width(string) / 2.0;

        // get the scale to render the text at
        let scale = self.ui_scale;

        // Render each character
        for character in string.chars() {
            // Offset the character by its width
            x += (character.width() + CHARACTER_GAP) / 2.0 * scale;
            
            // Render the character
            self.renderer.render(Properties {
                src: character.source(),
                dest: [x, y],
                rotation: 0.0,
                overlay_colour: colour,
                scale: scale
            });

            // Move to the start of the next character
            x += (character.width() + CHARACTER_GAP) / 2.0 * scale;
        }
    }

    // Render an image
    pub fn render(&mut self, image: &Image, dest: [f32; 2], scale: f32) {
        self.renderer.render(Properties {
            src: image.source(),
            dest, scale,
            rotation: 0.0,
            overlay_colour: colours::ALPHA,
        });
    }

    // Render an image with a colour overlay
    pub fn render_with_overlay(&mut self, image: &Image, dest: [f32; 2], scale: f32, overlay_colour: [f32; 4]) {
        self.renderer.render(Properties {
            src: image.source(),
            dest, scale, overlay_colour,
            rotation: 0.0,
        });
    }

    // Render an image with a particular rotation
    pub fn render_with_rotation(&mut self, image: &Image, dest: [f32; 2], scale: f32, rotation: f32) {
        self.renderer.render(Properties {
            src: image.source(),
            dest, scale, rotation,
            overlay_colour: colours::ALPHA,
        });
    }

    // Get the width that a string would be rendered at
    pub fn font_width(&self, string: &str) -> f32 {
        string.chars().fold(0.0, |total, character| total + (character.width() + CHARACTER_GAP) * self.ui_scale)
    }

    // Get the height of rendered text
    pub fn font_height(&self) -> f32 {
        FONT_HEIGHT * self.ui_scale
    }

    // Flush the renderer
    pub fn flush(&mut self) {
        self.renderer.flush();
    }

    // Clear the renderer
    pub fn clear(&mut self) {
        self.renderer.clear(colours::BLACK);
    }

    // Set the context from settings
    pub fn set(&mut self, settings: &Settings) {
        self.volume = settings.volume;
    }

    // Play a sound effect
    pub fn play_sound(&self, sound: SoundEffect) {
        // Get the sound effect
        let sound = match sound {
            SoundEffect::Walk => self.audio[0].as_ref(),
            SoundEffect::RegularShot => self.audio[1].as_ref(),
            SoundEffect::PlasmaShot => self.audio[2].as_ref()
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