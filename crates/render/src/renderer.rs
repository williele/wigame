use std::collections::HashMap;

use app::{EventReader, Events, ParRunnable, SystemBuilder};
use wgpu::{util::DeviceExt, RenderPipeline, Surface, SurfaceConfiguration};
use window_plugin::{
    winit::window::{Window, WindowId},
    WindowClosed, WindowCreated, WindowManager, WindowResized,
};

use crate::{buffer::Vertex, texture::Texture, RenderStage};

pub struct Renderer {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub initialized: bool,
    surfaces: HashMap<WindowId, (Surface, SurfaceConfiguration)>,
    //
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    indices_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl Renderer {
    pub async fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: None,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
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
                        ty: wgpu::BindingType::Sampler {
                            comparison: false,
                            filtering: true,
                        },
                        count: None,
                    },
                ],
            });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::all(),
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                clamp_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });

        // buffer

        const VERTICES: &[Vertex] = &[
            Vertex {
                position: [-0.5, -0.5, 0.0],
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                position: [0.5, -0.5, 0.0],
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                position: [0.5, 0.5, 0.0],
                tex_coords: [1.0, 0.0],
            },
            Vertex {
                position: [-0.5, 0.5, 0.0],
                tex_coords: [0.0, 0.0],
            },
        ];

        const INDICES: &[u16] = &[0, 2, 3, 2, 0, 1];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let indices_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let texture =
            Texture::from_bytes(&device, &queue, include_bytes!("wood_texture.png"), None).unwrap();
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
            label: None,
        });

        Renderer {
            instance,
            adapter,
            device,
            queue,
            initialized: false,
            surfaces: Default::default(),
            render_pipeline,
            //
            vertex_buffer,
            indices_buffer,
            bind_group,
        }
    }

    pub fn create_surface(&mut self, window: &Window) {
        let size = window.inner_size();
        let window_id = window.id();

        if self.surfaces.contains_key(&window_id) {
            panic!("Duplicated create surface for a window",);
        }

        let surface = unsafe { self.instance.create_surface(window) };
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Immediate,
        };
        surface.configure(&self.device, &config);
        self.surfaces.insert(window_id, (surface, config));
    }

    pub fn remove_surface(&mut self, window_id: &WindowId) {
        self.surfaces.remove(window_id);
    }

    pub fn resize(&mut self, window_id: &WindowId, width: u32, height: u32) {
        if width > 0 && height > 0 {
            if let Some((surface, config)) = self.surfaces.get_mut(window_id) {
                config.width = width;
                config.height = height;
                surface.configure(&self.device, &config);
            }
        }
    }

    pub fn update(&mut self) {
        for (_, (surface, config)) in self.surfaces.iter_mut() {
            match render_surface(
                surface,
                &mut self.device,
                &mut self.queue,
                &self.render_pipeline,
                &self.vertex_buffer,
                &self.indices_buffer,
                &self.bind_group,
            ) {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => surface.configure(&self.device, &config),
                Err(wgpu::SurfaceError::OutOfMemory) => println!("Out of memory"),
                Err(err) => eprintln!("Render error: {:?}", err),
            }
        }
    }
}

fn render_surface(
    surface: &mut Surface,
    device: &mut wgpu::Device,
    queue: &mut wgpu::Queue,
    render_pipeline: &RenderPipeline,
    vertex_buffer: &wgpu::Buffer,
    indices_buffer: &wgpu::Buffer,
    bind_group: &wgpu::BindGroup,
) -> Result<(), wgpu::SurfaceError> {
    let output = surface.get_current_frame()?.output;
    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(render_pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_index_buffer(indices_buffer.slice(..), wgpu::IndexFormat::Uint16);

        render_pass.draw_indexed(0..6, 0, 0..1);
    }
    queue.submit(std::iter::once(encoder.finish()));
    Ok(())
}

pub(crate) fn update_renderer_sys() -> impl ParRunnable {
    SystemBuilder::new()
        .on_stage(RenderStage::Render)
        .write_resource::<Renderer>()
        .build(|_, _, renderer, _| renderer.update())
}

pub(crate) fn handle_window_created_sys() -> impl ParRunnable {
    let mut event_reader = EventReader::<WindowCreated>::default();

    SystemBuilder::new()
        .on_stage(RenderStage::PostRender)
        .write_resource::<Renderer>()
        .read_resource::<Events<WindowCreated>>()
        .read_resource::<WindowManager>()
        .build(move |_, _, (renderer, events, window_manager), _| {
            for event in event_reader.iter(events) {
                let window = window_manager
                    .get(&event.id)
                    .expect("Created window event but window not found.");
                renderer.create_surface(window);
            }
        })
}

pub(crate) fn handle_window_closed_sys() -> impl ParRunnable {
    let mut event_reader = EventReader::<WindowClosed>::default();

    SystemBuilder::new()
        .on_stage(RenderStage::PostRender)
        .write_resource::<Renderer>()
        .read_resource::<Events<WindowClosed>>()
        .build(move |_, _, (renderer, window_closed_events), _| {
            for event in event_reader.iter(window_closed_events) {
                renderer.remove_surface(&event.id);
            }
        })
}

pub(crate) fn handle_window_resized_sys() -> impl ParRunnable {
    let mut event_reader = EventReader::<WindowResized>::default();

    SystemBuilder::new()
        .on_stage(RenderStage::PostRender)
        .write_resource::<Renderer>()
        .read_resource::<Events<WindowResized>>()
        .build(move |_, _, (renderer, events), _| {
            if let Some(event) = event_reader.iter(events).last() {
                renderer.resize(&event.id, event.width, event.height);
            }
        })
}
