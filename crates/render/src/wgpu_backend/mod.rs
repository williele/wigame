mod renderer;
mod surface;

use app::Plugin;

use renderer::Renderer;

use crate::wgpu_backend::renderer::{
    handle_window_closed_sys, handle_window_created_sys, handle_window_resized_sys,
    update_renderer_sys,
};

#[derive(Default)]
pub struct WgpuPlugin {}

impl Plugin for WgpuPlugin {
    fn build(&mut self, app: &mut app::App) {
        let renderer = util::pollster::block_on(Renderer::new());

        app.add_resource(renderer)
            .add_system(update_renderer_sys())
            .add_system(handle_window_created_sys())
            .add_system(handle_window_closed_sys())
            .add_system(handle_window_resized_sys());
    }
}
