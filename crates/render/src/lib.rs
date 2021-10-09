pub mod base;
mod wgpu_backend;

// mod buffer;
// mod camera;
// mod material;
// mod renderable;
// pub mod renderer;
// mod surface;
// mod texture;
// pub use renderer::*;
// pub use wgpu;

use app::{AppStage, Plugin, Stage, StageLabel};

pub use base::*;

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
            .add_asssets::<Shader>()
            .add_asssets::<Pipeline>()
            .add_plugin(wgpu_backend::WgpuPlugin::default());
    }
}
