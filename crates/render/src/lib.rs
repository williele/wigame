pub mod renderer;

use app::{AppStage, Plugin, Stage, StageLabel};
use util::pollster;

pub use renderer::*;
pub use wgpu;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderStage {
    PreRender,
    Render,
    PostRender,
}
impl StageLabel for RenderStage {
    fn dyn_clone(&self) -> Box<dyn StageLabel> {
        match self {
            RenderStage::PreRender => Box::new("Render:PreRender"),
            RenderStage::Render => Box::new("Render:Render"),
            RenderStage::PostRender => Box::new("Render:PostRender"),
        }
    }
}

#[derive(Debug, Default)]
pub struct RenderPlugin {}

impl Plugin for RenderPlugin {
    fn build(&mut self, app: &mut app::App) {
        let renderer = pollster::block_on(Renderer::new());

        app.add_stage_before(AppStage::End, RenderStage::Render, Stage::sequence())
            .add_stage_before(
                RenderStage::Render,
                RenderStage::PreRender,
                Stage::sequence(),
            )
            .add_stage_after(
                RenderStage::Render,
                RenderStage::PostRender,
                Stage::sequence(),
            )
            .add_resource(renderer)
            .add_system(handle_window_created_sys())
            .add_system(handle_window_resized_sys())
            .add_system(handle_window_closed_sys())
            .add_system(update_renderer_sys());
    }
}
