use std::collections::HashMap;

use app::{EventReader, Events, ParRunnable, SystemBuilder};
use wgpu::{Surface, SurfaceConfiguration};
use window_plugin::{
    winit::window::{Window, WindowId},
    WindowClosed, WindowCreated, WindowManager, WindowResized,
};

use crate::RenderStage;

pub struct Renderer {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub initialized: bool,
    surfaces: HashMap<WindowId, (Surface, SurfaceConfiguration)>,
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

        Renderer {
            instance,
            adapter,
            device,
            queue,
            initialized: false,
            surfaces: Default::default(),
        }
    }

    pub fn create_surface(&mut self, window: &Window) {
        let size = window.inner_size();
        let window_id = window.id();

        if self.surfaces.contains_key(&window_id) {
            panic!("Duplicated create surface for a window",);
        }

        let surface = unsafe { self.instance.create_surface(window) };
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&self.adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Immediate,
        };
        surface.configure(&self.device, &config);
        self.surfaces.insert(window_id, (surface, config));
    }

    pub fn remove_surface(&mut self, window_id: &WindowId) {
        self.surfaces.remove(window_id);
    }

    pub fn resize(&mut self, window_id: &WindowId, width: u32, height: u32) {
        if width > 0 && height > 0 {
            if let Some((surface, config)) = self.surfaces.get_mut(window_id) {
                config.width = width;
                config.height = height;
                surface.configure(&self.device, &config);
            }
        }
    }

    pub fn update(&mut self) {
        for (_, (surface, config)) in self.surfaces.iter_mut() {
            match render_surface(surface, &mut self.device, &mut self.queue) {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => surface.configure(&self.device, &config),
                Err(wgpu::SurfaceError::OutOfMemory) => println!("Out of memory"),
                Err(err) => eprintln!("Render error: {:?}", err),
            }
        }
    }
}

fn render_surface(
    surface: &mut Surface,
    device: &mut wgpu::Device,
    queue: &mut wgpu::Queue,
) -> Result<(), wgpu::SurfaceError> {
    let output = surface.get_current_frame()?.output;
    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
    }
    queue.submit(std::iter::once(encoder.finish()));
    Ok(())
}

pub(crate) fn update_renderer_sys() -> impl ParRunnable {
    SystemBuilder::new()
        .on_stage(RenderStage::Render)
        .write_resource::<Renderer>()
        .build(|_, _, renderer, _| renderer.update())
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
