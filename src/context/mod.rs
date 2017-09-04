use glutin::EventsLoop;

mod renderer;
mod audio;

use colours;
use settings::Settings;
use resources::{ImageSource, Image, SoundEffect, CHARACTER_GAP};
use self::renderer::{Renderer, Properties}; 
use self::audio::Player;

pub struct Context {
    pub settings: Settings,
    pub width: f32,
    pub height: f32,
    renderer: Renderer,
    player: Player
}

impl Context {
    // Create a new context
    pub fn new(event_loop: &EventsLoop, settings: Settings, title: String, tileset: &[u8], audio: [&[u8]; 3]) -> Context {
        Context {
            renderer: Renderer::new(event_loop, tileset, title, &settings),
            width: settings.window_width as f32,
            height: settings.window_height as f32,
            settings,
            player: Player::new(audio)
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
    pub fn play_sound(&mut self, sound: SoundEffect) {
        // Get the corresponding sound index
        let index = match sound {
            SoundEffect::Walk => 0,
            SoundEffect::RegularShot => 1,
            SoundEffect::PlasmaShot => 2
        };

        // Play it
        self.player.play(index, f32::from(self.settings.volume) / 100.0);
    }
}