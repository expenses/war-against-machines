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
use glutin::EventsLoop;
use image;

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

// Load a texture
fn load_texture(factory: &mut Factory, bytes: &[u8]) -> ([f32; 2], Texture) {
    let img = image::load_from_memory(bytes).unwrap().to_rgba();
    let (width, height) = img.dimensions();
    let kind = Kind::D2(width as u16, height as u16, AaMode::Single);
    let texture = factory.create_texture_immutable_u8::<ColorFormat>(kind, &[&img]).unwrap().1;
    
    ([width as f32, height as f32], texture)
}

// Define the rendering stuff
gfx_defines! {
    vertex Vertex {
        pos: [f32; 2] = "in_pos",
        uv: [f32; 2] = "in_uv",
    }

    constant Constants {
        tileset: [f32; 2] = "constant_tileset",
    }

    constant Global {
        resolution: [f32; 2] = "global_resolution",
    }

    constant Properties {
        src: [f32; 4] = "prop_src",
        dest: [f32; 4] = "prop_dest",
        overlay_colour: [f32; 4] = "prop_overlay_colour",
        rotation: f32 = "prop_rotation",
    }
    
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
    window: glutin::Window,
    device: Device,
    depth: DepthStencilView<Resources, DepthStencil>,
}

impl Renderer {
    pub fn new(event_loop: &EventsLoop, tileset: &[u8], title: String, width: u32, height: u32) -> Renderer {
        // Set the GL version and profile (this helps speed things up)
        let gl = glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 2));
        let gl_profile = glutin::GlProfile::Core;

        // Build the window
        let builder = glutin::WindowBuilder::new()
            .with_title(title)
            .with_dimensions(width, height)
            .with_gl(gl)
            .with_gl_profile(gl_profile)
            .with_vsync();

        // Initialise the gfx-glutin connection
        let (window, device, mut factory, colour, depth) =
            gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder, event_loop);

        // Create a encoder to abstract the command buffer
        let encoder = factory.create_command_buffer().into();

        // Create the pipeline, loading in the shaders
        let pso = factory.create_pipeline_simple(
            include_bytes!("shaders/rect_150.glslv"),
            include_bytes!("shaders/rect_150.glslf"),
            pipe::new()
        ).unwrap();

        // Get the vertex buffer and slice
        let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(SQUARE, INDICES);

        // Load the texture
        let (tileset_size, tileset) = load_texture(&mut factory, tileset);
        // Create a Nearest-Neighbor sampler 
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

        // Create the renderer!
        let mut renderer = Renderer {
            encoder, data, pso, slice, window, device, depth
        };

        renderer.encoder.update_constant_buffer(
            &renderer.data.constants,
            &Constants {tileset: tileset_size}
        );

        renderer.encoder.update_constant_buffer(
            &renderer.data.global,
            &Global {resolution: [width as f32, height as f32]}
        );

        renderer
    }

    // Resize the renderer window
    pub fn resize(&mut self, width: f32, height: f32) {
        self.encoder.update_constant_buffer(&self.data.global, &Global {resolution: [width, height]});
        // Update the view
        gfx_window_glutin::update_views(&self.window, &mut self.data.out, &mut self.depth);
    }

    pub fn render(&mut self, properties: Properties) {
        self.encoder.update_constant_buffer(&self.data.properties, &properties);
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