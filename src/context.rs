// A Context struct for containing the SDL2 context and the canvas

use sdl2;
use sdl2::ttf;
use sdl2::ttf::Sdl2TtfContext;
use sdl2::mixer;
use sdl2::mixer::{INIT_OGG, AUDIO_S16LSB, DEFAULT_CHANNELS};
use sdl2::rect::Rect;
use sdl2::video::{WindowContext, Window, FullscreenType};
use sdl2::render::{Texture, TextureCreator};

use settings::Settings;
use TITLE;

// Audio settings
const FREQUENCY: i32 = 44100;
const CHUNK_SIZE: i32 = 1024;
const ALLOCATED_CHANNELS: i32 = 16;

// Contains the SDL2 context, the canvas and a bool for whether the game should be running
pub struct Context {
    sdl_context: sdl2::Sdl,
    pub canvas: sdl2::render::Canvas<Window>,
    pub running: bool,
}

impl Context {
    // Create a new Context with a title, width and height
    pub fn new(width: u32, height: u32) -> Context {
        // Initialise sdl and get the video subsystem
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        // Set up the audio mixer
        sdl_context.audio().unwrap();
        mixer::init(INIT_OGG).unwrap();
        mixer::open_audio(FREQUENCY, AUDIO_S16LSB, DEFAULT_CHANNELS, CHUNK_SIZE).unwrap();
        mixer::allocate_channels(ALLOCATED_CHANNELS);

        // Build the window, making it resizable
        let mut window_builder = video_subsystem.window(TITLE, width, height);
        window_builder.resizable();

        let window = window_builder.build().unwrap();

        // Create the canvas from the window with vsync
        let canvas = window
            .into_canvas()
            .present_vsync()
            .build()
            .unwrap();

        // Return the context
        Context {
            sdl_context, canvas,
            running: true
        }
    }

    // Simple alias for making a texture creator from the canvas
    pub fn texture_creator(&self) -> TextureCreator<WindowContext> {
        self.canvas.texture_creator()
    }

    pub fn font_context(&self) -> Sdl2TtfContext {
        ttf::init().unwrap()
    }

    // Simple alias for the event pump
    pub fn event_pump(&self) -> sdl2::EventPump {
        self.sdl_context.event_pump().unwrap()
    }

    // Draw an image at x, y, and a particular scale
    pub fn draw(&mut self, image: &Texture, x: f32, y: f32, scale: f32) {
        // Scale up/down the image
        let query = image.query();
        let (width, height) = (query.width as f32 * scale, query.height as f32 * scale);
        // Create the rect
        let rect = Rect::new(x as i32, y as i32, width as u32, height as u32);
        // Copy the image onto the canvas
        self.canvas.copy(image, None, Some(rect)).unwrap();
    }

    // Clear the canvas
    pub fn clear(&mut self) {
        self.canvas.clear();
    }

    // Present the canvas
    pub fn present(&mut self) {
        self.canvas.present()
    }

    // Get the width of the canvas
    pub fn get_width(&self) -> u32 {
        self.canvas.window().size().0
    }

    // Get the height of the canvas
    pub fn get_height(&self) -> u32 {
        self.canvas.window().size().1
    }

    pub fn set(&mut self, settings: &Settings) {
        let window = self.canvas.window_mut();

        window.set_size(settings.width, settings.height).unwrap();
        window.set_fullscreen(match settings.fullscreen {
            true => FullscreenType::True,
            false => FullscreenType::Off
        }).unwrap();
        mixer::Channel::all().set_volume(settings.volume);
    }

    // 'Quit' the game by setting self.running to false
    pub fn quit(&mut self) {
        self.running = false;
    }
}
