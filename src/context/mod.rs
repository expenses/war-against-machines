use glutin::EventsLoop;
use rodio;
use rodio::{Decoder, Source};

use std::rc::Rc;
use std::io::Cursor;

mod renderer;

use colours;
use settings::Settings;
use resources::{ImageSource, Image, SoundEffect, CHARACTER_GAP};
use self::renderer::{Renderer, Properties}; 

// Use reference-counting to avoid cloning the source each time
type Audio = Rc<Vec<u8>>;

// Load a piece of audio
fn load_audio(bytes: &[u8]) -> Audio {
    Rc::new(bytes.to_vec())
}

pub struct Context {
    pub settings: Settings,
    pub width: f32,
    pub height: f32,
    renderer: Renderer,
    audio: [Audio; 3]
}

impl Context {
    // Create a new context
    pub fn new(event_loop: &EventsLoop, settings: Settings, title: String, width: u32, height: u32, tileset: &[u8], audio: [&[u8]; 3]) -> Context {
        let renderer = Renderer::new(event_loop, tileset, title, width, height);

        // Create the context!
        Context {
            renderer,
            settings,
            width: width as f32,
            height: height as f32,
            audio: [load_audio(audio[0]), load_audio(audio[1]), load_audio(audio[2])]
        }
    }

    // Resize the context
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width as f32;
        self.height = height as f32;
        // resize the renderer
        self.renderer.resize(width, height);
    }

    // Render text
    pub fn render_text(&mut self, string: &str, mut x: f32, y: f32, colour: [f32; 4]) {
        // Center the text on its width
        x = (x - self.settings.font_width(string) / 2.0).floor();
        
        // If the ui scale is odd, offset by 0.5
        if self.settings.ui_scale % 2 == 1 {
            x += 0.5;
        }

        let scale = self.settings.ui_scale();

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

    // Flush the renderer
    pub fn flush(&mut self) {
        self.renderer.flush();
    }

    // Clear the renderer
    pub fn clear(&mut self) {
        self.renderer.clear(colours::BLACK);
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
        rodio::play_raw(&endpoint, decoder.convert_samples().amplify(f32::from(self.settings.volume) / 100.0));
    }
}