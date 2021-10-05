use crate::{buffer::Vertex, material::Material};
use wgpu::util::DeviceExt;

pub struct Renderable {
    material: Material,
    vertex_buffer: wgpu::Buffer,
    indices_buffer: wgpu::Buffer,
}

impl Renderable {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
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

        let material = Material::new(&device, &queue);

        Self {
            material,
            vertex_buffer,
            indices_buffer,
        }
    }

    pub fn render<'a, 'b>(
        &'a self,
        pass: &'b mut wgpu::RenderPass<'a>,
        ctx_bind_group: &'a wgpu::BindGroup,
    ) {
        self.material.binding(pass, ctx_bind_group);

        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_index_buffer(self.indices_buffer.slice(..), wgpu::IndexFormat::Uint16);

        pass.draw_indexed(0..6, 0, 0..1);
    }
}
