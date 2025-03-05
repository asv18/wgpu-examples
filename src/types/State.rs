use wgpu::{
    util::DeviceExt, BindGroupLayout, Color, Device, PipelineLayout,
    RenderPipeline, ShaderModuleDescriptor, SurfaceConfiguration
};

use winit::{event::WindowEvent, window::Window};

use crate::types::texture;

use super::{
    camera_types::{camera::Camera, camera_controller::CameraController, camera_uniform::CameraUniform},
    polygon_buffer::PolygonBuffer,
    vertex_types::{textured_vertex::*, Vertex}
};

pub struct State<'a> {
    pub surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    clear_color: wgpu::Color,
    window: &'a Window,
    render_pipeline: wgpu::RenderPipeline,
    polygon_buffer: PolygonBuffer<TexturedVertex>,
    diffuse_bind_group: wgpu::BindGroup,
    _diffuse_texture: texture::Texture,
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_controller: CameraController,
    //
    // for challenge 6
    // camera_staging: CameraStaging,
    //
    // for challenge 5
    // challenge_diffuse_bind_group: wgpu::BindGroup,
    // challenge_diffuse_texture: texture::Texture,
    // selected_image: bool,
    //
    // for challenge 4
    // challenge_vertex_buffer: wgpu::Buffer,
    // challenge_index_buffer: wgpu::Buffer,
    // challenge_num_vertices: u32,
    // challenge_num_indices: u32,
    // selected_polygon: bool,
    //
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

        // surface.configure(&device, &config);

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    // This should match the filterable field of the
                    // corresponding Texture entry above.
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        let diffuse_bytes = include_bytes!("resources/challenge_image.jpeg");
        let (
            diffuse_bind_group,
            diffuse_texture
        ) = Self::generate_texture(diffuse_bytes, "resources/challenge_image.jpeg", &texture_bind_group_layout, &device, &queue);

        // let challenge_diffuse_bytes = include_bytes!("resources/challenge_image.jpeg");
        // let (
        //     challenge_diffuse_bind_group,
        //     challenge_diffuse_texture
        // ) = Self::generate_texture(challenge_diffuse_bytes, "resources/challenge_image.jpeg", &texture_bind_group_layout, &device, &queue);
        


        let camera = Camera {
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });



        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &texture_bind_group_layout,
                &camera_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = Self::generate_render_pipeline::<TexturedVertex>(
            wgpu::include_wgsl!("resources/camera_shader.wgsl"),
            // alternatively:
            // wgpu::ShaderModuleDescriptor {
            //     label: Some("Shader"),
            //     source: wgpu::ShaderSource::Wgsl("resources/shader.wgsl"),
            // }
            &render_pipeline_layout,
            &device,
            &config,
        );
        
        // let (vertices, indices) = ColoredVertex::generate_polygon(5, 0.5);
        // let challenge_render_pipeline = Self::generate_render_pipeline(include_str!("resources/challenge_3.wgsl").into(), &render_pipeline_layout, &device, &config);
        let camera_controller = CameraController::new(0.2);

        let polygon_buffer = PolygonBuffer::new(&device, VERTICES, INDICES);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            clear_color: Color { r: 0.0, g: 0.5, b: 0.5, a: 1.0, },
            window,
            render_pipeline,
            polygon_buffer,
            diffuse_bind_group,
            _diffuse_texture: diffuse_texture,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_controller,
            // challenge_diffuse_bind_group,
            // challenge_diffuse_texture,
            // selected_image: false,
            // challenge_vertex_buffer,
            // challenge_index_buffer,
            // challenge_num_vertices,
            // challenge_num_indices,
            // selected_polygon: false,
            // render_pipelines: vec![render_pipeline_1, render_pipeline_2],
            // selected_pipeline: 0,
        }
    }

    fn generate_texture(diffuse_bytes: &[u8], label: &str, texture_bind_group_layout: &BindGroupLayout, device: &Device, queue: &wgpu::Queue) -> (wgpu::BindGroup, texture::Texture) {
        let diffuse_texture = texture::Texture::from_bytes(&device, &queue, diffuse_bytes, label).unwrap();

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        (diffuse_bind_group, diffuse_texture)
    }

    fn generate_render_pipeline<T: Vertex>(source: ShaderModuleDescriptor, layout: &PipelineLayout, device: &Device, config: &SurfaceConfiguration) -> RenderPipeline {
        let shader = device.create_shader_module(source);

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"), // specifies the entry point function in shader.wgsl
                buffers: &[
                    T::desc(),
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
        })
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

            self.camera.aspect = self.config.width as f32 / self.config.height as f32;
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event)

        // match event {
            // WindowEvent::KeyboardInput {
            //     event:
            //         KeyEvent {
            //             state,
            //             physical_key: PhysicalKey::Code(KeyCode::Space),
            //             ..
            //         },
            //     ..
            // } => {
            //     // self.selected_pipeline = if *state == ElementState::Released { 0 } else { 1};
            //     // self.selected_polygon = *state != ElementState::Released;
            //     // self.selected_image = *state != ElementState::Released;

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
            // _ => false,
        // }
    }

    pub fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
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

            render_pass.set_pipeline(&self.render_pipeline);

            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

            render_pass.set_vertex_buffer(0, self.polygon_buffer.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.polygon_buffer.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            render_pass.draw_indexed(0..self.polygon_buffer.num_indices, 0, 0..1);

            // render_pass.draw(0..self.polygon_buffer.num_vertices, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
