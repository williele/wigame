use crate::{ShaderStages, TextureSampleType, TextureViewDimension, VertexFormat};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindGroupDescriptor {
    pub bindings: Vec<BindDescriptor>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BindDescriptor {
    pub binding: u32,
    pub ty: BindType,
    pub visibility: ShaderStages,
}

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub enum BindType {
    Uniform {
        has_dynamic_offset: bool,
    },
    StorageBuffer {
        has_dynamic_offset: bool,
        readonly: bool,
    },
    Sampler {
        filtering: bool,
        comparison: bool,
    },
    Texture {
        multisampled: bool,
        view_dimension: TextureViewDimension,
        sample_type: TextureSampleType,
    },
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct VertexBufferLayout {
    pub array_stride: u64,
    pub attributes: Vec<VertexAttribute>,
}

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub struct VertexAttribute {
    pub offset: u64,
    pub shader_location: u32,
    pub format: VertexFormat,
}
