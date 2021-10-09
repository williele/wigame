// Vertex shader
struct GeometryInput {
    [[location(0)]] position: vec3<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
};

[[stage(vertex)]]
fn main(
    geometry: GeometryInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(geometry.position, 1.0);
    return out;
}

// Fragment shader

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return vec4<f32>(0.3, 0.2, 0.1, 1.0);
}
