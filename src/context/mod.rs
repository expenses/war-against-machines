mod audio;
mod renderer;

use self::audio::Player;
use self::renderer::{Properties, Renderer};
use colours;
use glutin::*;
use pedot::*;
use resources::*;
use runic::*;
use settings::Settings;

pub struct Context {
    pub settings: Settings,
    pub width: f32,
    pub height: f32,
    renderer: Renderer,
    player: Player,
    font: CachedFont<'static>,
    pub gui: Gui,
}

impl Context {
    pub const UI_SCALE: f32 = 2.0;
    pub const FONT_SCALE: f32 = 13.0;
    pub const FONT_HEIGHT: f32 = Self::UI_SCALE * Self::FONT_SCALE;

    // Create a new context
    pub fn new(event_loop: &EventsLoop, settings: Settings) -> Self {
        let renderer = Renderer::new(event_loop, &settings);

        let (width, height) = (settings.window_width as f32, settings.window_height as f32);

        Self {
            width,
            height,
            settings,
            player: Player::new(),
            font: CachedFont::from_bytes(FONT, &renderer.display).unwrap(),
            gui: Gui::new(width, height),
            renderer,
        }
    }

    // Resize the context
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width as f32;
        self.height = height as f32;
        // resize the renderer
        self.renderer.resize(width, height);
    }

    pub fn font_width(&self, text: &str) -> f32 {
        self.font
            .rendered_width(text, Self::FONT_HEIGHT, true, &self.renderer.display)
    }

    // Render text
    pub fn render_text(&mut self, string: &str, mut x: f32, mut y: f32, colour: [f32; 4]) {
        // Correct for screen position
        y -= Self::FONT_SCALE;

        // Center the text on its width
        let width = self.font_width(string);
        x -= width / 2.0;

        self.font
            .render_pixelated(
                string,
                [x, y],
                Self::FONT_SCALE,
                Self::UI_SCALE,
                colour,
                &mut self.renderer.target,
                &self.renderer.display,
                &self.renderer.text_program,
            )
            .unwrap();
    }

    // Render an image
    pub fn render(&mut self, image: Image, dest: [f32; 2], scale: f32) {
        self.renderer.render(Properties {
            src: image.source(),
            dest,
            scale,
            rotation: 0.0,
            overlay_colour: colours::ALPHA,
        });
    }

    // Render an image with a colour overlay
    pub fn render_with_overlay(
        &mut self,
        image: Image,
        dest: [f32; 2],
        scale: f32,
        overlay_colour: [f32; 4],
    ) {
        self.renderer.render(Properties {
            src: image.source(),
            dest,
            scale,
            overlay_colour,
            rotation: 0.0,
        });
    }

    // Render an image with a particular rotation
    pub fn render_with_rotation(
        &mut self,
        image: Image,
        dest: [f32; 2],
        scale: f32,
        rotation: f32,
    ) {
        self.renderer.render(Properties {
            src: image.source(),
            dest,
            scale,
            rotation,
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

    pub fn update_gui(&mut self, event: &WindowEvent) {
        self.gui.update(event);
    }

    pub fn clear_gui(&mut self) {
        self.gui.clear();
    }

    // Play a sound effect
    pub fn play_sound(&mut self, sound: &SoundEffect) {
        // Get the corresponding sound index
        let index = match *sound {
            SoundEffect::Walk => 0,
            SoundEffect::RegularShot => 1,
            SoundEffect::PlasmaShot => 2,
        };

        // Play it
        self.player
            .play(index, f32::from(self.settings.volume) / 100.0);
    }
}
