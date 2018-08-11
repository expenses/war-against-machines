use glium;
use glium::*;
use glutin::*;
use glutin::dpi::*;
use image::{load_from_memory_with_format, ImageFormat};
use glium::texture::*;
use glium::index::PrimitiveType::*;
use glium::uniforms::*;
use settings::Settings;

const VERT: &str = include_str!("shaders/shader.vert");
const FRAG: &str = include_str!("shaders/shader.frag");

#[derive(Copy, Clone)]
struct Vertex {
    in_pos: [f32; 2],
    in_uv: [f32; 2]
}

implement_vertex!(Vertex, in_pos, in_uv);

// A square of vertices
const SQUARE: &[Vertex] = &[
    Vertex { in_pos: [1.0, -1.0],  in_uv: [1.0, 1.0]},
    Vertex { in_pos: [-1.0, -1.0], in_uv: [0.0, 1.0]},
    Vertex { in_pos: [-1.0, 1.0],  in_uv: [0.0, 0.0]},
    Vertex { in_pos: [1.0, 1.0],   in_uv: [1.0, 0.0]},
];
// The indices that hold the square together
const INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];


// Load a texture from memory and return it and its dimensions
fn load_texture(display: &Display, bytes: &[u8]) -> ([f32; 2], SrgbTexture2d) {
    let img = load_from_memory_with_format(bytes, ImageFormat::PNG).unwrap().to_rgba();
    let (width, height) = img.dimensions();
    let image = RawImage2d::from_raw_rgba_reversed(&img.into_raw(), (width, height));
    let texture = SrgbTexture2d::new(display, image).unwrap();
    
    ([width as f32, height as f32], texture)
}

#[derive(Default)]
pub struct Properties {
    pub src: [f32; 4],
    pub overlay_colour: [f32; 4],
    pub dest: [f32; 2],
    pub rotation: f32,
    pub scale: f32
}

struct Uniforms {
    screen_resolution: [f32; 2],
    tileset_size: [f32; 2],
    texture: SrgbTexture2d
}

pub struct Renderer {
    uniforms: Uniforms,
    target: Frame,
    display: Display,
    vertex_buffer: VertexBuffer<Vertex>,
    indices: IndexBuffer<u16>,
    program: Program
}

impl Renderer {
    pub fn new(event_loop: &EventsLoop, tileset: &[u8], title: String, settings: &Settings) -> Renderer {
        // Get the with and the height of the window from settings
        let (width, height) = (settings.window_width, settings.window_height);

        // Build the window
        let mut builder = WindowBuilder::new()
            .with_title(title)
            .with_dimensions(LogicalSize::new(f64::from(width), f64::from(height)));

        // Set the window to be fullscreen if that's set in settings
        if settings.fullscreen {
            builder = builder.with_fullscreen(Some(event_loop.get_primary_monitor()));
        }

        // Create the context and display
        let context = ContextBuilder::new().with_vsync(true);
        let display = Display::new(builder, context, &event_loop).unwrap();

        // Create the buffers and program
        let vertex_buffer = VertexBuffer::new(&display, SQUARE).unwrap();
        let indices = IndexBuffer::new(&display, TrianglesList, INDICES).unwrap();
        let program = Program::from_source(&display, VERT, FRAG, None).unwrap();

        // Load the texture
        let (tileset_size, tileset) = load_texture(&display, tileset);
        
        Renderer {
            // Setup uniforms
            uniforms: Uniforms {
                tileset_size,
                screen_resolution: [width as f32, height as f32],
                texture: tileset
            },
            // Setup the draw target
            target: display.draw(),
            display,
            vertex_buffer,
            indices,
            program
        }
    }

    // Set the internal screen resolution uniform
    pub fn resize(&mut self, width: u32, height: u32) {
        self.uniforms.screen_resolution = [width as f32, height as f32];
    }

    // Render a image from a set of properties
    pub fn render(&mut self, properties: Properties) {
        // Create the samper and set it to do nearest neighbour resizing of the image for nice pixelation
        let sampler = glium::uniforms::Sampler::new(&self.uniforms.texture)
            .minify_filter(MinifySamplerFilter::Nearest)
            .magnify_filter(MagnifySamplerFilter::Nearest);

        // Setup the uniforms
        // TODO: See if properties can be supplied as a whole struct
        let uniforms = uniform! {
            tileset_size: self.uniforms.tileset_size,
            screen_resolution: self.uniforms.screen_resolution,
            prop_src: properties.src,
            prop_dest: properties.dest,
            prop_overlay_colour: properties.overlay_colour,
            prop_rotation: properties.rotation,
            prop_scale: properties.scale,
            sampler: sampler
        };

        // Make sure alpha blending is enabled
        let draw_params = DrawParameters {
            blend: Blend::alpha_blending(),
            .. Default::default()
        };

        // Draw to the target
        self.target.draw(&self.vertex_buffer, &self.indices, &self.program, &uniforms, &draw_params).unwrap();
    }

    // Finish the current draw target (swapping it onto the screen) and setup a new one
    pub fn flush(&mut self) {
        self.target.set_finish().unwrap();
        self.target = self.display.draw();
    }

    // Clear the target
    pub fn clear(&mut self, colour: [f32; 4]) {
        self.target.clear_color(colour[0], colour[1], colour[2], colour[3]);
    }
}

// When the renderer is dropped the current target needs to finish or a panic occurs
impl Drop for Renderer {
    fn drop(&mut self) {
        self.target.set_finish().unwrap();
    }
}

#[test]
// Getting an off-screen OpenGL context is tricky on windows and linux
// so only enable this test on OSX
#[cfg(target_os = "macos")]
fn compile_shaders() {
    // Create the headless context and renderer
    let context = HeadlessRendererBuilder::new(640, 480).build().unwrap();
    let display = HeadlessRenderer::new(context).unwrap();
    // Try create the program
    Program::from_source(&display, VERT, FRAG, None).unwrap();
}