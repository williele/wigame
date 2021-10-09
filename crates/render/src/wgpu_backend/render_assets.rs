use std::{borrow::Cow, collections::HashMap, sync::Arc};

use app::{AssetEvent, Assets, EventReader, Events, Handle, ParRunnable, SystemBuilder};

use crate::{wgpu_backend::DEFAULT_COLOR_TARGET, Pipeline, RenderStage, Shader};

use super::renderer::Renderer;

#[derive(Debug)]
pub(crate) struct RenderAssets {
    device: Arc<wgpu::Device>,
    shaders: HashMap<Handle<Shader>, wgpu::ShaderModule>,
    pipelines: HashMap<Handle<Pipeline>, wgpu::RenderPipeline>,
}

impl From<&Renderer> for RenderAssets {
    fn from(renderer: &Renderer) -> Self {
        RenderAssets::new(renderer.device.clone())
    }
}

impl RenderAssets {
    pub fn new(device: Arc<wgpu::Device>) -> Self {
        Self {
            device,
            shaders: Default::default(),
            pipelines: Default::default(),
        }
    }

    pub fn get_shader(&self, handle: &Handle<Shader>) -> Option<&wgpu::ShaderModule> {
        self.shaders.get(handle)
    }

    fn add_shader(&mut self, handle: &Handle<Shader>, shader: &Shader) {
        println!("add shader");
        let wgpu_shader = self
            .device
            .create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::from(&shader.source)),
            });
        self.shaders.insert(handle.clone_weak(), wgpu_shader);
    }

    fn remove_shader(&mut self, handle: &Handle<Shader>) {
        println!("remove shader");
        self.shaders.remove(handle);
    }

    pub fn get_pipeline(&self, handle: &Handle<Pipeline>) -> Option<&wgpu::RenderPipeline> {
        self.pipelines.get(handle)
    }

    fn add_pipeline(&mut self, handle: &Handle<Pipeline>, pipeline: &Pipeline) {
        println!("add render pipeline");
        let buffers = pipeline
            .buffers
            .iter()
            .map(|layout| wgpu::VertexBufferLayout {
                array_stride: layout.array_stride as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                }],
            })
            .collect::<Vec<_>>();

        let vertex_state = wgpu::VertexState {
            module: self
                .get_shader(&pipeline.vertex.shader)
                .expect("Missing shader"),
            entry_point: pipeline.vertex.entry_point.as_str(),
            buffers: &buffers,
        };

        let fragment_state = pipeline.fragment.as_ref().map(|state| wgpu::FragmentState {
            module: self.get_shader(&state.shader).expect("Missing shader"),
            entry_point: state.entry_point.as_str(),
            targets: &[DEFAULT_COLOR_TARGET],
        });

        let render_pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: None,
                vertex: vertex_state,
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
                fragment: fragment_state,
            });
        self.pipelines.insert(handle.clone_weak(), render_pipeline);
    }

    fn remove_pipeline(&mut self, handle: &Handle<Pipeline>) {
        println!("remove render pipeline");
        self.pipelines.remove(handle);
    }
}

pub(crate) fn sync_shader_assets_sys() -> impl ParRunnable {
    let mut reader = EventReader::<AssetEvent<Shader>>::default();

    SystemBuilder::new()
        .on_stage(RenderStage::PostRender)
        .read_resource::<Events<AssetEvent<Shader>>>()
        .read_resource::<Assets<Shader>>()
        .write_resource::<RenderAssets>()
        .build(move |_, _, (events, assets, render_assets), _| {
            for event in reader.iter(events) {
                match event {
                    AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                        if let Some(shader) = assets.get(handle) {
                            render_assets.add_shader(handle, shader);
                        }
                    }
                    AssetEvent::Removed { handle } => render_assets.remove_shader(handle),
                }
            }
        })
}

pub(crate) fn sync_render_pipeline_assets_sys() -> impl ParRunnable {
    let mut reader = EventReader::<AssetEvent<Pipeline>>::default();

    SystemBuilder::new()
        .on_stage(RenderStage::PostRender)
        .read_resource::<Events<AssetEvent<Pipeline>>>()
        .read_resource::<Assets<Pipeline>>()
        .write_resource::<RenderAssets>()
        .build(move |_, _, (events, assets, render_assets), _| {
            for event in reader.iter(events) {
                match event {
                    AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                        if let Some(render_pipeline) = assets.get(handle) {
                            render_assets.add_pipeline(handle, render_pipeline);
                        }
                    }
                    AssetEvent::Removed { handle } => render_assets.remove_pipeline(handle),
                }
            }
        })
}
