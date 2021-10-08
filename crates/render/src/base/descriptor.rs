use util::bitflags::bitflags;

bitflags! {
    pub struct ShaderStages: u16 {
        const VERTEX = 0b0001;
        const FRAGMENT = 0b0010;
        const COMPUTE = 0b0100;
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum PrimitiveTopology {
    Points = 0,
    Lines = 1,
    LineStrip = 2,
    Triangles = 3,
    TriangleStrip = 4,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum FrontFace {
    Ccw = 0,
    Cw = 1,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Face {
    Front = 0,
    Back = 1,
}

#[derive(Debug, Clone)]
pub struct MultisampleState {
    pub count: u32,
    pub mask: u64,
    pub alpha_to_coverage_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct PrimitiveState {
    pub topology: PrimitiveTopology,
    pub front_face: FrontFace,
    pub clamp_depth: bool,
    pub cull_mode: Option<Face>,
    pub conservative: bool,
}
