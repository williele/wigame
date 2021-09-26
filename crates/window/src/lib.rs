mod events;
mod manager;
mod runner;

use app::Plugin;
use manager::WindowManager;
use runner::window_runner;

pub use events::*;
pub use manager::*;

#[derive(Debug, Clone)]
pub struct WindowDescriptor {
    pub width: f32,
    pub height: f32,
    pub title: String,
}

#[derive(Debug, Default)]
pub struct WindowPlugin;
impl Plugin for WindowPlugin {
    fn build(&mut self, app: &mut app::App) {
        app.add_resource(WindowManager::default())
            .add_event::<CreateWindow>()
            .add_event::<WindowLaunched>()
            .add_event::<WindowResized>()
            .add_event::<WindowCloseRequested>()
            .set_runner(window_runner);
    }
}
