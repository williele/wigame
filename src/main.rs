use app::App;
use render::RenderPlugin;

fn main() {
    App::new()
        .add_plugin(RenderPlugin)
        .add_plugin(window_plugin::WindowPlugin)
        .run();
}
