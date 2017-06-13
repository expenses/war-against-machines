extern crate sdl2;

use sdl2::rect::Rect;
use sdl2::video::{WindowContext, Window};
use sdl2::render::{Texture, TextureCreator};

pub struct Context {
    sdl_context: sdl2::Sdl,
    canvas: sdl2::render::Canvas<Window>,
    pub running: bool,
}

impl Context {
    pub fn new(title: &str, width: u32, height: u32) -> Context {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window(title, width, height)
            .opengl()
            .resizable()
            .build()
            .unwrap();
    
        let canvas = window
            .into_canvas()
            .present_vsync()
            .accelerated()
            .build()
            .unwrap();

        Context {
            sdl_context, canvas,
            running: true
        }
    }

    pub fn texture_creator(&self) -> TextureCreator<WindowContext> {
        self.canvas.texture_creator()
    }

    pub fn event_pump(&self) -> sdl2::EventPump {
        self.sdl_context.event_pump().unwrap()
    }

    pub fn draw_with_rotation(&mut self, image: &Texture, x: f32, y: f32, scale: f32, rotation: f64) {
        let query = image.query();
        let (width, height) = (query.width as f32 * scale, query.height as f32 * scale);
        let rect = Rect::new(x as i32, y as i32, width as u32, height as u32);

        self.canvas.copy_ex(image, None, Some(rect), rotation, None, false, false).unwrap();
    }

    pub fn draw(&mut self, image: &Texture, x: f32, y: f32, scale: f32) {
        let query = image.query();
        let (width, height) = (query.width as f32 * scale, query.height as f32 * scale);
        let rect = Rect::new(x as i32, y as i32, width as u32, height as u32);

        self.canvas.copy(image, None, Some(rect)).unwrap();
    }

    pub fn clear(&mut self) {
        self.canvas.clear();
    }

    pub fn present(&mut self) {
        self.canvas.present()
    }

    pub fn size(&self) -> (f32, f32) {
        let (width, height) = self.canvas.output_size().unwrap();
        (width as f32, height as f32)
    }

    pub fn width(&self) -> f32 {
        self.size().0
    }

    pub fn height(&self) -> f32 {
        self.size().1
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}
