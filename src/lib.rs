//! FRUG is intended to provide a similar abstraction layer over graphics programming as to how SDL does for C++, meaning that it should provide developers enough control and flexibility to implement their own architectures & design patterns, yet simplifying the process of working with graphics so developers won't have to worry about implementing all the repetitive tasks related to getting things to the screen.
//! 
//! FRUG aims to include the following features (unchecked items are the ones still under development):
//! - [x] Window management
//! - [ ]  Loading & rendering textures
//! - [ ]  Rotating textures
//! - [ ]  Scaling textures
//! - [ ]  Alpha blending for textures
//! - [ ]  Choosing a specific backend (aka. Direct X, Metal, Vulkan, etc.)
//! - [ ]  Writing and using custom shaders
//! - [ ]  Handle window state events
//! - [ ]  Handle Mouse input
//! - [ ]  Handle Keyboard input
//! - [ ]  Playing audio
//! - [ ]  Configure audio


use wgpu::util::DeviceExt;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{EventLoop, ControlFlow},
    window::Window
};

/// Vertex struct
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3]
}

/// Implementation of Vertex methods
impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout { 
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress, 
            step_mode: wgpu::VertexStepMode::Vertex, 
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3
                }
            ] 
        }
    }
}


// - - - - - TEST! - - - - -
// We should remove this in the future so we can create these in frug usage.
const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.0868241, 0.49240386, 0.0], color: [0.5, 0.0, 0.5] },
    Vertex { position: [-0.49513406, 0.06958647, 0.0], color: [0.5, 0.0, 0.5] },
    Vertex { position: [-0.21918549, -0.44939706, 0.0], color: [0.5, 0.0, 0.5] },
    Vertex { position: [0.35966998, -0.3473291, 0.0], color: [0.5, 0.0, 0.5] },
    Vertex { position: [0.44147372, 0.2347359, 0.0], color: [0.5, 0.0, 0.5] },
];

const INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4
];
// - - - - - TEST! - - - - -

/// The Frug instance.
/// Contains the surface in which we draw, the device we're using, the queue, the surface configuration, surface size, window, background color, and render pipeline.
pub struct FrugInstance {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    background_color: wgpu::Color,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32
}

/// Implementation of FrugInstance methods
impl FrugInstance {
    /// Creates a new instance of FrugInstance, instantiating the window, configuration, and the surface to draw in.
    async fn new_instance(window_title: &str, event_loop: &EventLoop<()>) -> Self {
        // Enable wgpu logging
        env_logger::init();

        // Setup
        let window = Window::new(&event_loop).unwrap();
        window.set_title(window_title);
        let size = window.inner_size();
        let background_color = wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());

        let surface = unsafe { 
            instance.create_surface(&window)
        }.unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false
            }
        ).await.expect("Failed to find an appropiate adapter.");

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default()
            }, None).await.expect("Failed to create device.");

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .filter(|f| f.describe().srgb)
            .next()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[]
        });

        // our render pipeline
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState { 
                module: &shader, 
                entry_point: "vs_main", 
                buffers: &[Vertex::desc()] 
            },
            fragment: Some(wgpu::FragmentState { 
                module: &shader, 
                entry_point: "fs_main", 
                targets: &[Some(wgpu::ColorTargetState { 
                    format: config.format, 
                    blend: Some(wgpu::BlendState::REPLACE), 
                    write_mask: wgpu::ColorWrites::ALL 
                })]
            }),
            primitive: wgpu::PrimitiveState { 
                topology: wgpu::PrimitiveTopology::TriangleList, 
                strip_index_format: None, 
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back), 
                unclipped_depth: false, 
                polygon_mode: wgpu::PolygonMode::Fill, 
                conservative: false 
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState { 
                count: 1, 
                mask: !0, 
                alpha_to_coverage_enabled: false
            },
            multiview: None
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX
        });

        let num_indices = INDICES.len() as u32;

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            background_color,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices
        }
    }

    /// Resize the canvas for our window given a new defined size.
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    /// Render
    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder")
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { 
                label: Some("Render Pass"), 
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view, 
                    resolve_target: None, 
                    ops: wgpu::Operations { 
                        load: wgpu::LoadOp::Clear(self.background_color), 
                        store: true
                    }
                })], 
                depth_stencil_attachment: None
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// Sets new background color.
    /// 
    /// Receives a wgpu color (you can create one using the `frug::create_color` method).
    /// 
    /// # Example
    /// ```
    /// let new_color = frug::create_color(0.2, 0.3, 0.4, 1.0);
    /// my_frug_instance.set_background_color(new_color);
    /// ```
    pub fn set_background_color(&mut self, color: wgpu::Color) {
        self.background_color = color;
    }
}

/// Starts running your project.
/// 
/// Should receive a string which will be the title for the window created. It should also receive a loop which will be the main loop for your game/app.
/// * `window_title (&str)`         - The title for your window.
/// * `window_loop (static Fn())`   - The loop you want to execute with each frame.
/// 
/// # Example:
/// 
/// ```
/// let my_loop = || {
///     // your code
/// };
/// frug::run("My Game", my_loop);
/// ```
pub fn run<F: 'static + Fn()>(window_title: &str, window_loop: F) {
    // setup
    let event_loop = EventLoop::new();
    let mut frug_instance = pollster::block_on( FrugInstance::new_instance(window_title, &event_loop));

    // Run the loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Act on events
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } 
            // Window events
            if window_id == frug_instance.window.id() => match event {
                // Close
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                },

                // Resize
                WindowEvent::Resized(physical_size) => {
                    frug_instance.resize(*physical_size);
                },
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    frug_instance.resize(**new_inner_size);
                }
                _ => ()
            }
            Event::RedrawRequested(window_id) if window_id == frug_instance.window.id() => {
                // frug_instance.update();
                match frug_instance.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => frug_instance.resize(frug_instance.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                frug_instance.window.request_redraw();
            }
            _ => (),
        }

        window_loop();
    });
}

/// Creates a color.
/// Should receive in range from 0.0 - 1.0 the red, green, blue, and alpha channels.
/// * `red (f64)`   - The red channel.
/// * `green (f64)`   - The green channel.
/// * `blue (f64)`   - The blue channel.
/// * `alpha (f64)`   - The alpha channel.
/// 
/// # Example:
/// 
/// ```
/// frug::create_color(0.1, 0.2, 0.3, 1.0);
/// ```
pub fn create_color(red: f64, green: f64, blue: f64, alpha: f64) -> wgpu::Color {
    wgpu::Color { r: red, g: green, b: blue, a: alpha }
}

// EOF