use glutin::EventsLoop;
use rodio;
use rodio::{Decoder, Source};

use std::rc::Rc;
use std::io::Cursor;

mod renderer;

use colours;
use settings::Settings;
use resources::{ImageSource, Image, SoundEffect};
use self::renderer::{Renderer, Properties}; 

// The size of a tile
const CHARACTER_GAP: f32 = 1.0;
use constants::FONT_HEIGHT;

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
    pub font_size: f32,
}

impl Context {
    pub fn new(event_loop: &EventsLoop, title: String, width: u32, height: u32, tileset: &[u8], audio: [&[u8]; 3]) -> Context {
        let renderer = Renderer::new(event_loop, tileset, title, width, height);

        // Create the renderer!
        Context {
            renderer,
            width: width as f32,
            height: height as f32,
            font_size: 2.0,
            volume: 100,
            audio: [load_audio(audio[0]), load_audio(audio[1]), load_audio(audio[2])]
        }
    }

    // Resize the renderer window
    pub fn resize(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
        // Update the view
        self.renderer.resize(width, height);
    }

    pub fn render_text(&mut self, string: &str, mut x: f32, y: f32, colour: [f32; 4]) {
        x -= self.font_width(string) / 2.0;

        let scale = self.font_size;

        for character in string.chars() {
            x += (character.width() + CHARACTER_GAP) / 2.0 * scale;
            
            self.renderer.render(Properties {
                src: character.source(),
                dest: [x, y, character.width() * scale, character.height() * scale],
                rotation: 0.0,
                overlay_colour: colour
            });

            x += (character.width() + CHARACTER_GAP) / 2.0 * scale;
        }
    }

    pub fn render(&mut self, image: &Image, x: f32, y: f32, scale: f32) {
        self.renderer.render(Properties {
            src: image.source(),
            dest: [x, y, image.width() * scale, image.height() * scale],
            rotation: 0.0,
            overlay_colour: colours::ALPHA
        });
    }

    pub fn render_with_overlay(&mut self, image: &Image, x: f32, y: f32, scale: f32, colour: [f32; 4]) {
        self.renderer.render(Properties {
            src: image.source(),
            dest: [x, y, image.width() * scale, image.height() * scale],
            rotation: 0.0,
            overlay_colour: colour
        });
    }

    pub fn render_with_rotation(&mut self, image: &Image, x: f32, y: f32, scale: f32, rotation: f32) {
        self.renderer.render(Properties {
            src: image.source(),
            dest: [x, y, image.width() * scale, image.height() * scale],
            rotation: rotation,
            overlay_colour: colours::ALPHA
        });
    }

    pub fn font_width(&self, string: &str) -> f32 {
        string.chars().fold(0.0, |total, character| total + (character.width() + CHARACTER_GAP) * self.font_size)
    }

    pub fn font_height(&self) -> f32 {
        FONT_HEIGHT * self.font_size
    }

    // Flush the renderer and swap buffers
    pub fn flush(&mut self) {
        self.renderer.flush();
    }

    // Clear the renderer
    pub fn clear(&mut self) {
        self.renderer.clear(colours::BLACK);
    }

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