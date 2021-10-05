use util::cgmath;
use wgpu::util::DeviceExt;
use window_plugin::winit::window::Window;

use crate::{
    camera::{camera_projection_layout, Camera, PerspectiveProjection},
    renderable::Renderable,
};

pub struct SurfaceInfo {
    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,
    // camera: Camera,
    // camera_buffer: wgpu::Buffer,
    proj: PerspectiveProjection,
    proj_buffer: wgpu::Buffer,
    camera_proj_bind_group: wgpu::BindGroup,
}

impl SurfaceInfo {
    pub fn new(window: &Window, instance: &wgpu::Instance, device: &wgpu::Device) -> Self {
        let size = window.inner_size();

        let surface = unsafe { instance.create_surface(window) };
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Immediate,
        };
        surface.configure(&device, &config);

        let camera = Camera {
            eye: (0., 0., 2.).into(),
            target: (0., 0., 0.).into(),
            up: cgmath::Vector3::unit_y(),
        };

        let proj = PerspectiveProjection {
            aspect: size.width as f32 / size.height as f32,
            fovy: 45.,
            znear: 0.1,
            zfar: 100.,
        };

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[camera.uniform()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let proj_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[proj.uniform()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_proj_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &camera_projection_layout(device),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: proj_buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            surface,
            config,
            // camera,
            // camera_buffer,
            proj,
            proj_buffer,
            camera_proj_bind_group,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(device, &self.config);

        self.proj.aspect = width as f32 / height as f32;
        queue.write_buffer(
            &self.proj_buffer,
            0,
            bytemuck::cast_slice(&[self.proj.uniform()]),
        )
    }

    pub fn render(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        renderable: &Renderable,
    ) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_frame()?.output;
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

            renderable.render(&mut render_pass, &self.camera_proj_bind_group);
        }

        queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }
}
