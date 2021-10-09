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

// #[derive(Debug, Clone)]
// pub struct PrimitiveState {
//     pub topology: PrimitiveTopology,
//     pub front_face: FrontFace,
//     pub clamp_depth: bool,
//     pub cull_mode: Option<Face>,
//     pub conservative: bool,
// }

#[repr(C)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum VertexFormat {
    Uint8x2 = 0,
    Uint8x4 = 1,
    Sint8x2 = 2,
    Sint8x4 = 3,
    Unorm8x2 = 4,
    Unorm8x4 = 5,
    Snorm8x2 = 6,
    Snorm8x4 = 7,
    Uint16x2 = 8,
    Uint16x4 = 9,
    Sint16x2 = 10,
    Sint16x4 = 11,
    Unorm16x2 = 12,
    Unorm16x4 = 13,
    Snorm16x2 = 14,
    Snorm16x4 = 15,
    Float16x2 = 16,
    Float16x4 = 17,
    Float32 = 18,
    Float32x2 = 19,
    Float32x3 = 20,
    Float32x4 = 21,
    Uint32 = 22,
    Uint32x2 = 23,
    Uint32x3 = 24,
    Uint32x4 = 25,
    Sint32 = 26,
    Sint32x2 = 27,
    Sint32x3 = 28,
    Sint32x4 = 29,
    Float64 = 30,
    Float64x2 = 31,
    Float64x3 = 32,
    Float64x4 = 33,
}
