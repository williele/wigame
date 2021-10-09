mod render_assets;
mod renderer;
mod surface;

use app::Plugin;

use renderer::Renderer;

use crate::wgpu_backend::renderer::{
    handle_window_closed_sys, handle_window_created_sys, handle_window_resized_sys,
    update_renderer_sys,
};

use self::render_assets::{sync_render_pipeline_assets_sys, sync_shader_assets_sys, RenderAssets};

pub const DEFAULT_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;
pub const DEFAULT_COLOR_TARGET: wgpu::ColorTargetState = wgpu::ColorTargetState {
    format: DEFAULT_TEXTURE_FORMAT,
    blend: Some(wgpu::BlendState {
        color: wgpu::BlendComponent::REPLACE,
        alpha: wgpu::BlendComponent::REPLACE,
    }),
    write_mask: wgpu::ColorWrites::all(),
};

#[derive(Default)]
pub struct WgpuPlugin {}

impl Plugin for WgpuPlugin {
    fn build(&mut self, app: &mut app::App) {
        let renderer = util::pollster::block_on(Renderer::new());
        let render_assets = RenderAssets::from(&renderer);

        app.add_resource(renderer)
            .add_resource(render_assets)
            .add_system(update_renderer_sys())
            .add_system(handle_window_created_sys())
            .add_system(handle_window_closed_sys())
            .add_system(handle_window_resized_sys())
            .add_system(sync_shader_assets_sys())
            .add_system(sync_render_pipeline_assets_sys());
    }
}
