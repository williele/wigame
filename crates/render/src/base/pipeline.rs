use app::Handle;

use crate::{BindGroupDescriptor, MultisampleState, Shader, VertexBufferLayout};

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
pub struct Pipeline {
    pub buffers: Vec<VertexBufferLayout>,
    pub vertex: VertexState,
    pub fragment: Option<FragmentState>,
    pub multisample: MultisampleState,
}
