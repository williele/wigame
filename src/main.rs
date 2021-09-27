use app::{App, AppStage, EventReader, Events, ParRunnable, SystemBuilder};
use render_plugin::RenderPlugin;
use window_plugin::{
    winit::event::{ElementState, VirtualKeyCode},
    WindowCreateRequest, WindowDescriptor, WindowKeyboardInput, WindowPlugin,
};

fn create_window() -> impl ParRunnable {
    let mut event_reader = EventReader::<WindowKeyboardInput>::default();

    SystemBuilder::new()
        .on_stage(AppStage::PreUpdate)
        .read_resource::<Events<WindowKeyboardInput>>()
        .write_resource::<Events<WindowCreateRequest>>()
        .build(
            move |_, _, (keyboard_input_events, window_create_events), _| {
                for event in event_reader.iter(keyboard_input_events) {
                    if event.key_code == Some(VirtualKeyCode::A)
                        && event.state == ElementState::Released
                    {
                        window_create_events.send(WindowCreateRequest {
                            descriptor: WindowDescriptor {
                                width: 550,
                                height: 400,
                                title: "Another window".to_string(),
                            },
                        })
                    }
                }
            },
        )
}

fn main() {
    App::new()
        .add_plugin(WindowPlugin::with_initial(WindowDescriptor {
            width: 800,
            height: 600,
            title: "WiGame".to_string(),
        }))
        .add_plugin(RenderPlugin::default())
        .add_system(create_window())
        .run();
}
