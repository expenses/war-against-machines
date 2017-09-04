use gfx;
use gfx::{Encoder, PipelineState};
use gfx::traits::FactoryExt;
use gfx::format::DepthStencil;
use gfx::Device as GfxDevice;
use gfx::Factory as GfxFactory;
use gfx::handle::{DepthStencilView, ShaderResourceView};
use gfx::texture::{SamplerInfo, FilterMethod, WrapMode, Kind, AaMode};
use gfx_window_glutin;
use gfx_device_gl;
use glutin;
use glutin::{GlContext, EventsLoop};
use image::{load_from_memory_with_format, ImageFormat};

use settings::Settings;

// A square of vertices
const SQUARE: &[Vertex] = &[
    Vertex { pos: [1.0, -1.0],  uv: [1.0, 1.0]},
    Vertex { pos: [-1.0, -1.0], uv: [0.0, 1.0]},
    Vertex { pos: [-1.0, 1.0],  uv: [0.0, 0.0]},
    Vertex { pos: [1.0, 1.0],   uv: [1.0, 0.0]},
];
// The indices that hold the square together
const INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];

// Definitions for stuff
type ColorFormat = gfx::format::Srgba8;
type DepthFormat = gfx::format::DepthStencil;
// Type definitions for opengl stuff (so I can change the backend easily)
type Resources = gfx_device_gl::Resources;
type Factory = gfx_device_gl::Factory;
type CommandBuffer = gfx_device_gl::CommandBuffer;
type Device = gfx_device_gl::Device;
type Texture = ShaderResourceView<Resources, [f32; 4]>;

// Load a texture from memory and return it and its dimensions
fn load_texture(factory: &mut Factory, bytes: &[u8]) -> ([f32; 2], Texture) {
    let img = load_from_memory_with_format(bytes, ImageFormat::PNG).unwrap().to_rgba();
    let (width, height) = img.dimensions();
    let kind = Kind::D2(width as u16, height as u16, AaMode::Single);
    let texture = factory.create_texture_immutable_u8::<ColorFormat>(kind, &[&img]).unwrap().1;
    
    ([width as f32, height as f32], texture)
}

// Define the rendering stuff
gfx_defines! {
    // The input vertex
    vertex Vertex {
        pos: [f32; 2] = "in_pos",
        uv: [f32; 2] = "in_uv",
    }

    // Constants for rendering
    constant Constants {
        tileset: [f32; 2] = "constant_tileset",
    }

    // Global settings
    constant Global {
        resolution: [f32; 2] = "global_resolution",
    }

    // Settings for the current image
    constant Properties {
        src: [f32; 4] = "prop_src",
        overlay_colour: [f32; 4] = "prop_overlay_colour",
        dest: [f32; 2] = "prop_dest",
        rotation: f32 = "prop_rotation",
        scale: f32 = "prop_scale",
    }
    
    // The pipeline
    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        properties: gfx::ConstantBuffer<Properties> = "Properties",
        global: gfx::ConstantBuffer<Global> = "Global",
        constants: gfx::ConstantBuffer<Constants> = "Constants",
        sampler: gfx::TextureSampler<[f32; 4]> = "sampler",
        out: gfx::BlendTarget<ColorFormat> = ("target", gfx::state::MASK_ALL, gfx::preset::blend::ALPHA),
    }
}

pub struct Renderer {
    encoder: Encoder<Resources, CommandBuffer>,
    data: pipe::Data<Resources>,
    pso: PipelineState<Resources, pipe::Meta>,
    slice: gfx::Slice<Resources>,
    window: glutin::GlWindow,
    device: Device,
    depth: DepthStencilView<Resources, DepthStencil>,
}

impl Renderer {
    pub fn new(event_loop: &EventsLoop, tileset: &[u8], title: String, settings: &Settings) -> Renderer {
        // Get the with and the height of the window from settings
        let (width, height) = (settings.window_width, settings.window_height);

        // Build the window
        let mut builder = glutin::WindowBuilder::new()
            .with_title(title)
            .with_dimensions(width, height);

        // Set the window to be fullscreen if that's set in settings
        if settings.fullscreen {
            builder = builder.with_fullscreen(glutin::get_primary_monitor());
        }

        // Create the GL context
        let context = glutin::ContextBuilder::new()
            .with_gl(glutin::GL_CORE)
            .with_vsync(true);

        // Initialise the gfx-glutin connection
        let (window, device, mut factory, colour, depth) =
            gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder, context, event_loop);

        // Create the Pipeline State Object, loading in the shaders
        let pso = factory.create_pipeline_simple(
            include_bytes!("shaders/rect_150.glslv"),
            include_bytes!("shaders/rect_150.glslf"),
            pipe::new()
        ).unwrap();

        // Get the vertex buffer and slice
        let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(SQUARE, INDICES);

        // Load the texture
        let (tileset_size, tileset) = load_texture(&mut factory, tileset);
        // Create a Nearest-Neighbor sampler (with basic mipmapping)
        let sampler = factory.create_sampler(SamplerInfo::new(FilterMethod::Mipmap, WrapMode::Clamp));

        // Create the pipeline data
        let data = pipe::Data {
            vbuf: vertex_buffer,
            sampler: (tileset, sampler),
            properties: factory.create_constant_buffer(1),
            global: factory.create_constant_buffer(1),
            constants: factory.create_constant_buffer(1),
            out: colour
        };

        // Create a encoder to abstract the command buffer
        let encoder = factory.create_command_buffer().into();

        // Create the renderer!
        let mut renderer = Renderer {
            encoder, data, pso, slice, window, device, depth
        };

        // Set the constants buffer
        renderer.encoder.update_constant_buffer(
            &renderer.data.constants,
            &Constants {
                tileset: tileset_size
            }
        );

        // Set the global buffer
        renderer.encoder.update_constant_buffer(
            &renderer.data.global,
            &Global {
                resolution: [width as f32, height as f32]
            }
        );

        renderer
    }

    // Resize the renderer window
    pub fn resize(&mut self, width: u32, height: u32) {
        // Update the global buffer
        self.encoder.update_constant_buffer(&self.data.global, &Global {
            resolution: [width as f32, height as f32]
        });
        // Resize the gl context
        self.window.resize(width, height);
        // Update the view
        gfx_window_glutin::update_views(&self.window, &mut self.data.out, &mut self.depth);
    }

    // Render a image from a set of properties
    pub fn render(&mut self, properties: Properties) {
        // Update the properties buffer
        self.encoder.update_constant_buffer(&self.data.properties, &properties);
        // Draw the image
        self.encoder.draw(&self.slice, &self.pso, &self.data);
    }

    // Flush the renderer and swap buffers
    pub fn flush(&mut self) {
        // Flush the device
        self.encoder.flush(&mut self.device);
        // Swap buffers
        self.window.swap_buffers().unwrap();
        // Clean up
        self.device.cleanup();
    }

    // Clear the renderer
    pub fn clear(&mut self, colour: [f32; 4]) {
        self.encoder.clear(&self.data.out, colour);
    }
}

#[test]
// Getting an off-screen OpenGL context is tricky on windows and linux
// so only enable this test on OSX
#[cfg(target_os = "macos")]
fn compile_shaders() {
    use std::os::raw::c_void;

    // Create the headless context and make it the current one

    let headless = glutin::HeadlessRendererBuilder::new(640, 480)
        .build()
        .unwrap();

    unsafe {
        headless.make_current().unwrap();
    };

    // Get the factory

    let (_, mut factory) = gfx_device_gl::create(
        |s| headless.get_proc_address(s) as *const c_void
    );

    // Test creating the PSO

    factory.create_pipeline_simple(
        include_bytes!("shaders/rect_150.glslv"),
        include_bytes!("shaders/rect_150.glslf"),
        pipe::new()
    ).unwrap_or_else(|error| panic!("{}", error));
}