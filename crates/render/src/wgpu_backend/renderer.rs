use std::collections::HashMap;

use app::{EventReader, Events, ParRunnable, SystemBuilder};
use window_plugin::{
    winit::window::{Window, WindowId},
    WindowClosed, WindowCreated, WindowManager, WindowResized,
};

use crate::RenderStage;

use super::surface::SurfaceData;

pub struct Renderer {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub initialized: bool,
    pub surface_data: HashMap<WindowId, SurfaceData>,
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
            surface_data: Default::default(),
        }
    }

    pub fn create_surface(&mut self, window: &Window) {
        let window_id = window.id();
        if self.surface_data.contains_key(&window_id) {
            panic!("Duplicated create surface for a window",);
        }

        let surface_info = SurfaceData::new(window, &self.instance, &self.device);
        self.surface_data.insert(window_id, surface_info);
    }

    pub fn remove_surface(&mut self, window_id: &WindowId) {
        self.surface_data.remove(window_id);
    }

    pub fn resize(&mut self, window_id: &WindowId, width: u32, height: u32) {
        if width > 0 && height > 0 {
            if let Some(surface_info) = self.surface_data.get_mut(window_id) {
                surface_info.resize(&self.device, &self.queue, width, height);
            }
        }
    }

    pub fn render(&mut self) {
        for (_, surface_info) in self.surface_data.iter() {
            match surface_info.render(&self.device, &self.queue) {
                Ok(_) => {}
                Err(wgpu::SurfaceError::OutOfMemory) => println!("Out of memory"),
                Err(err) => eprintln!("Render error: {:?}", err),
            }
        }
    }
}

pub(crate) fn update_renderer_sys() -> impl ParRunnable {
    SystemBuilder::new()
        .on_stage(RenderStage::Render)
        .write_resource::<Renderer>()
        .build(|_, _, renderer, _| {
            renderer.render();
        })
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
