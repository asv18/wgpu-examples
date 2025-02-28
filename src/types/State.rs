use winit::window::Window;
use winit::event::*;

pub struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    window: &'a Window,
    clear_color: wgpu::Color,
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
        // Instance is the first thing created when using wgpu;
        // it is used to create Adapters and Surfaces

        let surface = instance.create_surface(window).unwrap();
        // part of the window that we draw to
        // winit's Window possesses the HasRawWindowHandle trait, making it a suitable target

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(), // Two variants: LowPower and HighPerformance; each pick an adapter suited for its description
                compatible_surface: Some(&surface), // Tells wgpu to find an adapter that works with the supplied surface
                force_fallback_adapter: false, // forces wgpu to pick an adapter that will work on all hardware
            },
        ).await.unwrap();
        // The adapter is a handle for the actual graphics card.
        // It retrieves information about the graphics cards e.g.
        // its name and what backend the adapter uses
        // The adapter is used to create a Device and Queue

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            required_features: wgpu::Features::empty(),
            // because WebGL does not support all of wgpu's features,
            // some features need to be disabled
            required_limits: if cfg!(target_arch = "wasm32") {
                wgpu::Limits::downlevel_webgl2_defaults()
            } else {
                wgpu::Limits::default()
            },
            label: None,
            memory_hints: Default::default(),
        }, None).await.unwrap();
        // can view list of supported features using adapter.features() or device.features()
        // memory_hints field provides adapter with a preferred memory allocation strategy if supported

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        // defining here the config for our surface, which defines how
        // the surface creates its underlying SurfaceTextures (see render)
        // usage - describes how SurfaceTextures will be used
        // width and height in pixels of a Surface Texture are also defined
        // present_mode determines how to sync the surface with the display

        // view_formats is a list of TextureFormats that can be used when
        // creating TextureViews

        // if the surface is an sRGB color space, then you can create a texture
        // view that uses a linear color space

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
            clear_color: wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.width;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.clear_color = wgpu::Color {
                    r: (position.x as f64 / self.size.width as f64) % 255.0,
                    g: (position.y as f64 / self.size.height as f64) % 255.0,
                    b: ((position.x * position.y) / self.size.height as f64) % 255.0,
                    a: 1.0,
                };

                true
            },
            _ => false,
        }
    }

    pub fn update(&mut self) {
        // todo!()
        // nothing to update for now
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        // this will wait for the surface to provide a new SurfaceTexture to render

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        // creates a TextureView with default settings to control how the render code
        // interacts with the texture

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        // creates the actual commands to send to the GPU

        // the block is needed as we borrow encoder mutably 
        // and cannot call encoder.finish() until we release
        // the mutable borrow
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                // describes where to write colors to
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
        }
    
        // submit will accept anything that implements IntoIter
        // tells wgpu to finish the command buffer and submit it
        // to the GPU's render queue
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    
        Ok(())
    }
}
