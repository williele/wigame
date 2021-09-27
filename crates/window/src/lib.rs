pub mod events;
pub mod manager;
mod runner;

use app::{Events, Plugin};
use runner::{handle_window_event_sys, window_runner};

pub use events::*;
pub use manager::*;
pub use winit;

#[derive(Debug, Clone)]
pub struct WindowDescriptor {
    pub width: u32,
    pub height: u32,
    pub title: String,
}

#[derive(Debug, Default)]
pub struct WindowPlugin {
    initial_window: Option<WindowDescriptor>,
}

impl WindowPlugin {
    pub fn with_initial(descriptor: WindowDescriptor) -> Self {
        WindowPlugin {
            initial_window: Some(descriptor),
        }
    }
}

impl Plugin for WindowPlugin {
    fn build(&mut self, app: &mut app::App) {
        app.add_resource(WindowManager::default())
            .add_event::<WindowCreateRequest>()
            .add_event::<WindowCreated>()
            .add_event::<WindowCloseRequest>()
            .add_event::<WindowClosed>()
            .add_event::<WindowResized>()
            .add_system(handle_window_event_sys())
            .set_runner(window_runner);

        if let Some(descriptor) = self.initial_window.take() {
            app.resources
                .get_mut::<Events<WindowCreateRequest>>()
                .unwrap()
                .send(WindowCreateRequest { descriptor })
        }
    }
}
