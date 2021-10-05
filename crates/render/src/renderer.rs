use std::collections::HashMap;

use app::{EventReader, Events, ParRunnable, SystemBuilder};
use wgpu::util::DeviceExt;
use window_plugin::{
    winit::window::{Window, WindowId},
    WindowClosed, WindowCreated, WindowManager, WindowResized,
};

use crate::{buffer::Vertex, camera::Camera, surface::SurfaceInfo, texture::Texture, RenderStage};

pub struct Renderer {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub initialized: bool,

    surface_infos: HashMap<WindowId, SurfaceInfo>,
    //
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    indices_buffer: wgpu::Buffer,
    texture_bind_group: wgpu::BindGroup,
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
                bind_group_layouts: &[&texture_bind_group_layout, &Camera::layout(&device)],
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
        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
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
            surface_infos: Default::default(),
            //
            render_pipeline,
            vertex_buffer,
            indices_buffer,
            texture_bind_group,
        }
    }

    pub fn create_surface(&mut self, window: &Window) {
        let window_id = window.id();
        if self.surface_infos.contains_key(&window_id) {
            panic!("Duplicated create surface for a window",);
        }

        let surface_info = SurfaceInfo::new(window, &self.instance, &self.device);
        self.surface_infos.insert(window_id, surface_info);
    }

    pub fn remove_surface(&mut self, window_id: &WindowId) {
        self.surface_infos.remove(window_id);
    }

    pub fn resize(&mut self, window_id: &WindowId, width: u32, height: u32) {
        if width > 0 && height > 0 {
            if let Some(surface_info) = self.surface_infos.get_mut(window_id) {
                surface_info.resize(&self.device, &self.queue, width, height);
            }
        }
    }

    pub fn udpate(&mut self) {}

    pub fn render(&mut self) {
        for (_, surface_info) in self.surface_infos.iter() {
            match surface_info.render(
                &self.device,
                &self.queue,
                &self.render_pipeline,
                &self.texture_bind_group,
                &self.vertex_buffer,
                &self.indices_buffer,
            ) {
                Ok(_) => {}
                Err(wgpu::SurfaceError::OutOfMemory) => println!("Out of memory"),
                Err(err) => eprintln!("Render error: {:?}", err),
            }
        }
    }
}

pub(crate) fn update_renderer_sys() -> impl ParRunnable {
    SystemBuilder::new()
        .on_stage(RenderStage::Render)
        .write_resource::<Renderer>()
        .build(|_, _, renderer, _| {
            renderer.udpate();
            renderer.render();
        })
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
