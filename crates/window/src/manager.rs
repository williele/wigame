use std::collections::HashMap;

use winit::{
    dpi::PhysicalSize,
    event_loop::EventLoopWindowTarget,
    window::{Window, WindowBuilder, WindowId},
};

use crate::WindowDescriptor;

#[derive(Default)]
pub struct WindowManager {
    windows: HashMap<WindowId, Window>,
}

impl WindowManager {
    #[inline]
    pub fn len(&self) -> usize {
        self.windows.len()
    }

    pub fn create(
        &mut self,
        event_loop: &EventLoopWindowTarget<()>,
        descriptor: WindowDescriptor,
    ) -> WindowId {
        let builder = WindowBuilder::new()
            .with_inner_size(PhysicalSize::new(descriptor.width, descriptor.height))
            .with_title(descriptor.title);

        let window = builder.build(event_loop).unwrap();
        let window_id = window.id();
        self.windows.insert(window_id, window);
        window_id
    }

    pub fn get(&self, id: &WindowId) -> Option<&Window> {
        self.windows.get(id)
    }

    pub fn remove(&mut self, id: &WindowId) -> Option<Window> {
        self.windows.remove(id)
    }
}
