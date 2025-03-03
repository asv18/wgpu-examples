use wgpu::util::DeviceExt;
use winit::{event::{ElementState, KeyEvent, WindowEvent}, keyboard::{KeyCode, PhysicalKey}, window::Window};

use super::vertex::{Vertex, INDICES, VERTICES};

pub struct State<'a> {
    pub surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    clear_color: wgpu::Color,
    window: &'a Window,
    render_pipeline: wgpu::RenderPipeline,
    //
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_vertices: u32,
    num_indices: u32,
    //
    // for challenge 4
    // challenge_vertex_buffer: wgpu::Buffer,
    // challenge_index_buffer: wgpu::Buffer,
    // challenge_num_vertices: u32,
    // challenge_num_indices: u32,
    // selected_polygon: bool,
    // for challenge 3
    // render_pipelines: Vec<wgpu::RenderPipeline>,
    // selected_pipeline: usize,
}

impl<'a> State<'a> {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &'a Window) -> State<'a> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch="wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch="wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web, we'll have to disable some.
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None,
                memory_hints: Default::default(),
            },
            None, // Trace path
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            desired_maximum_frame_latency: 2,
            view_formats: vec![],
        };

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        // alternatively:
        // let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"), // specifies the entry point function in shader.wgsl
                buffers: &[
                    Vertex::desc(),
                ], // tells wgpu what types of vertices we want to pass from the wgsl file
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState { // stores color data
                module: &shader,
                entry_point: Some("fs_main"), // entry point for fragment
                targets: &[Some(wgpu::ColorTargetState { // tells wgpu what color outputs it should set up
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // specifies each entry into the list is a triangle
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // tells wgpu whether or not triangles are facing forwards or backwards; Ccw specifies triangles are forwards if their vertices are drawn ccw; cw follows similarly but with cw
                cull_mode: Some(wgpu::Face::Back), // culls triangles that are not facing forwards
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1, // how many sample pipelines we need to specify
                mask: !0, // specifies which samples should be active
                alpha_to_coverage_enabled: false, // anti-aliasing stuff
            },
            multiview: None, // indicates how many array layers the render attachments can have
            cache: None, // allows wgpu to cache shader compilation data
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("challenge_3.wgsl").into()),
        });

        let render_pipeline_2 = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"), // specifies the entry point function in shader.wgsl
                buffers: &[], // tells wgpu what types of vertices we want to pass from the wgsl file
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState { // stores color data
                module: &shader,
                entry_point: Some("fs_main"), // entry point for fragment
                targets: &[Some(wgpu::ColorTargetState { // tells wgpu what color outputs it should set up
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // specifies each entry into the list is a triangle
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // tells wgpu whether or not triangles are facing forwards or backwards; Ccw specifies triangles are forwards if their vertices are drawn ccw; cw follows similarly but with cw
                cull_mode: Some(wgpu::Face::Back), // culls triangles that are not facing forwards
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1, // how many sample pipelines we need to specify
                mask: !0, // specifies which samples should be active
                alpha_to_coverage_enabled: false, // anti-aliasing stuff
            },
            multiview: None, // indicates how many array layers the render attachments can have
            cache: None, // allows wgpu to cache shader compilation data
        });

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        
        let num_indices = INDICES.len() as u32;

        let num_vertices = VERTICES.len() as u32;

        Self {
            surface,
            device,
            queue,
            config,
            size,
            clear_color: wgpu::Color::RED,
            window,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_vertices,
            num_indices,
            // challenge_vertex_buffer,
            // challenge_index_buffer,
            // challenge_num_vertices,
            // challenge_num_indices,
            // selected_polygon: false,
            // render_pipelines: vec![render_pipeline_1, render_pipeline_2],
            // selected_pipeline: 0,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            // WindowEvent::KeyboardInput {
            //     event:
            //         KeyEvent {
            //             state,
            //             physical_key: PhysicalKey::Code(KeyCode::Space),
            //             ..
            //         },
            //     ..
            // } => {
            //     self.selected_pipeline = if *state == ElementState::Released { 0 } else { 1};
            //     true
            // },
            // WindowEvent::KeyboardInput {
            //     event:
            //         KeyEvent {
            //             state,
            //             physical_key: PhysicalKey::Code(KeyCode::Space),
            //             ..
            //         },
            //     ..
            // } => {
            //     self.selected_polygon = *state != ElementState::Released;
            //     true
            // },
            // WindowEvent::CursorMoved { device_id, position } => {
            //     self.clear_color = wgpu::Color {
            //         r: position.x as f64 / self.size.width as f64,
            //         g: position.y as f64 / self.size.height as f64,
            //         b: position.x as f64 / self.size.width as f64 * position.y as f64 / self.size.height as f64,
            //         a: 1.0,
            //     };

            //     true
            // },
            _ => false,
        }
    }

    pub fn update(&mut self) {

    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline); // 2.
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);

            render_pass.draw(0..self.num_vertices, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
