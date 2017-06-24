use sdl2;
use sdl2::rect::Rect;
use sdl2::video::{WindowContext, Window};
use sdl2::render::{Texture, TextureCreator};

// A Context struct for holding the sdl context, the canvas and a bool for whether the game should be running
pub struct Context {
    sdl_context: sdl2::Sdl,
    pub canvas: sdl2::render::Canvas<Window>,
    pub running: bool,
}

impl Context {
    pub fn new(title: &str, width: u32, height: u32) -> Context {
        // Initialise sdl and get the video subsystem
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        // Build the window, making it resizable
        let window = video_subsystem
            .window(title, width, height)
            .resizable()
            .build()
            .unwrap();
    
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

    // Get the size of the canvas
    pub fn size(&self) -> (u32, u32) {
        self.canvas.output_size().unwrap()
    }

    // Get the width of the canvas
    pub fn width(&self) -> u32 {
        self.size().0
    }

    // Get the height of the canvas
    pub fn height(&self) -> u32 {
        self.size().1
    }

    // 'Quit' the game
    pub fn quit(&mut self) {
        self.running = false;
    }
}
