pub mod renderer;

use app::{AppStage, Plugin, Stage, StageLabel};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderStage {
    Init,
    PreRender,
    Render,
    PostRender,
}
impl StageLabel for RenderStage {
    fn dyn_clone(&self) -> Box<dyn StageLabel> {
        match self {
            RenderStage::Init => Box::new("Render:Init"),
            RenderStage::PreRender => Box::new("Render:PreRender"),
            RenderStage::Render => Box::new("Render:Render"),
            RenderStage::PostRender => Box::new("Render:PostRender"),
        }
    }
}

pub struct RenderPlugin;

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
            );
    }
}
