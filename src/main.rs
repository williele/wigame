use app::App;
use render_plugin::RenderPlugin;
use window_plugin::{WindowDescriptor, WindowPlugin};

fn main() {
    App::new()
        .add_plugin(WindowPlugin::with_initial(WindowDescriptor {
            width: 800,
            height: 600,
            title: "WiGame".to_string(),
        }))
        .add_plugin(RenderPlugin::default())
        .run();
}
