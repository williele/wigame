use app::Handle;

use crate::{BindGroupDescriptor, MultisampleState, PrimitiveTopology, Shader};

#[derive(Clone, Debug, Default)]
pub struct PipelineLayout {
    pub bind_groups: Vec<BindGroupDescriptor>,
}

#[derive(Debug, Clone)]
pub struct VertexState {
    pub shader: Handle<Shader>,
    pub entry_point: String,
}

#[derive(Debug, Clone)]
pub struct FragmentState {
    pub shader: Handle<Shader>,
    pub entry_point: String,
}

#[derive(Debug, Clone)]
pub struct RenderPipelineDescriptor {
    pub layout: Option<PipelineLayout>,
    pub vertex: VertexState,
    pub fragment: Option<FragmentState>,
    pub primitive: PrimitiveTopology,
    pub multisample: MultisampleState,
}
